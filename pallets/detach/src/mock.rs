#![cfg(test)]

pub use crate as pallet_detach;
use crate::*;

use frame_system;
use frame_support::{parameter_types, traits::ConstU32};
use sp_core::{
	offchain::{
		testing::{self, OffchainState, PoolState, TestOffchainExt},
		OffchainDbExt, OffchainWorkerExt, TransactionPoolExt,
	},
	// ed25519::Signature,
	H256,
};
use sp_runtime::{
	testing::{Header, TestXt},
	traits::{BlakeTwo256, Extrinsic as ExtrinsicT, IdentifyAccount, IdentityLookup, Verify},
	MultiSignature, RuntimeAppPublic,
};

use parking_lot::RwLock;
use sp_keystore::{testing::KeyStore, KeystoreExt, SyncCryptoStore};
use std::sync::Arc;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// We need to make the `Signature` is `MultiSignature` since `<pallet_detach::pallet::Pallet<T> as ValidateUnsigned>::validate_unsigned()` expects the signer of the payload to be a `MultiSigner` (see `<pallet_detach::pallet::Pallet<T> as ValidateUnsigned>::validate_unsigned()` to understand more)
pub type Signature = MultiSignature;
type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		CollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage},

		DetachPallet: pallet_detach::{Pallet, Call, Storage, Event<T>},
	}
);

// When to use:
//
// To declare parameter types for a pallet's relevant associated types during runtime construction.
//
// What it does:
//
// The macro replaces each parameter specified into a struct type with a get() function returning its specified value.
// Each parameter struct type also implements the frame_support::traits::Get<I> trait to convert the type to its specified value.
//
// Source: https://docs.substrate.io/v3/runtime/macros/
parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}
impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId; // type AccountId = sp_core::ed25519::Public;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = ConstU32<2>;
}

impl frame_system::offchain::SigningTypes for Test {
	type Public = <Signature as Verify>::Signer;
	type Signature = Signature;
}

pub type Extrinsic = TestXt<Call, ()>;
impl<LocalCall> frame_system::offchain::SendTransactionTypes<LocalCall> for Test
where
	Call: From<LocalCall>,
{
	type OverarchingCall = Call;
	type Extrinsic = Extrinsic;
}
impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Test
where
	Call: From<LocalCall>,
{
	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
		call: Call,
		_public: <Signature as Verify>::Signer,
		_account: AccountId,
		nonce: u64,
	) -> Option<(Call, <Extrinsic as ExtrinsicT>::SignaturePayload)> {
		Some((call, (nonce, ())))
	}
}

impl pallet_randomness_collective_flip::Config for Test {}

impl Config for Test {
	type Event = Event;
	type WeightInfo = ();
	type AuthorityId = pallet_detach::crypto::DetachAuthId;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	let mut ext = sp_io::TestExternalities::new(t);

	ext.execute_with(|| System::set_block_number(1)); // if we don't execute this line, Events are not emitted from extrinsics (I don't know why this is the case though)

	ext
}

pub fn new_test_ext_with_ocw() -> (
	sp_io::TestExternalities,
	Arc<RwLock<PoolState>>,
	Arc<RwLock<OffchainState>>,
	sp_core::ed25519::Public,
) {
	const PHRASE: &str =
		"news slush supreme milk chapter athlete soap sausage put clutch what kitten";

	let (offchain, offchain_state) = testing::TestOffchainExt::new();
	let (pool, pool_state) = testing::TestTransactionPoolExt::new();

	let keystore = KeyStore::new();

	SyncCryptoStore::ed25519_generate_new(&keystore, KEY_TYPE, Some(&format!("{}", PHRASE)))
		.unwrap();

	// Since the struct `KeyStore` stores cryptographic keys as bytes, it doesn't know whether it stored it stored an Ed25519 key or a ECDSA key or a Sr25519 key.
	// That's why all these print statements will not be empty, even though we only called `SyncCryptoStore::ed25519_generate_new()` above.
	// println!("ed25519 keys are: {:?}", SyncCryptoStore::ed25519_public_keys(&keystore, KEY_TYPE));
	// println!("ecdsa keys are: {:?}", SyncCryptoStore::ecdsa_public_keys(&keystore, KEY_TYPE));
	// println!("sr25519 keys are: {:?}", SyncCryptoStore::sr25519_public_keys(&keystore, KEY_TYPE));

	let ed25519_public_key =
		SyncCryptoStore::ed25519_public_keys(&keystore, crate::crypto::Public::ID)
			.get(0)
			.unwrap()
			.clone();

	let mut t = sp_io::TestExternalities::default();
	t.register_extension(OffchainDbExt::new(offchain.clone()));
	t.register_extension(OffchainWorkerExt::new(offchain));
	t.register_extension(TransactionPoolExt::new(pool));
	t.register_extension(KeystoreExt(Arc::new(keystore)));

	t.execute_with(|| System::set_block_number(1)); // if we don't execute this line, Events are not emitted from extrinsics (I don't know why this is the case though)

	(t, pool_state, offchain_state, ed25519_public_key) // copied from https://github.com/JoshOrndorff/recipes/blob/master/pallets/ocw-demo/src/tests.rs
}

// Copied from "frame/merkle-mountain-range/src/tests.rs"
pub fn register_offchain_ext(ext: &mut sp_io::TestExternalities) {
	let (offchain, _offchain_state) = TestOffchainExt::with_offchain_db(ext.offchain_db());
	ext.register_extension(OffchainDbExt::new(offchain.clone()));
	ext.register_extension(OffchainWorkerExt::new(offchain));
}
