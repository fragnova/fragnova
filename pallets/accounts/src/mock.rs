use crate as pallet_accounts;
use crate::*;

use frame_support::{
	parameter_types,
	traits::{ConstU128, ConstU32, ConstU64},
};

use parking_lot::RwLock;
use sp_keystore::{testing::KeyStore, KeystoreExt, SyncCryptoStore};
use std::sync::Arc;

use sp_core::{
	ed25519::Signature,
	offchain::{
		testing::{self, OffchainState, PoolState},
		OffchainDbExt, OffchainWorkerExt, TransactionPoolExt,
	},
	H256,
};

use frame_support::traits::{AsEnsureOriginWithArg, GenesisBuild};
use frame_system::EnsureSigned;

use pallet_oracle::{OracleContract, OracleProvider};
use sp_runtime::{
	testing::{Header, TestXt},
	traits::{
		BlakeTwo256, ConstU8, Extrinsic as ExtrinsicT, IdentifyAccount, IdentityLookup, Verify,
	},
	RuntimeAppPublic,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

/// Balance of an account.
pub type Balance = u128;

pub const MILLICENTS: Balance = 1_000_000_000;
pub const CENTS: Balance = 1_000 * MILLICENTS; // assume this is worth about a cent.
pub const DOLLARS: Balance = 100 * CENTS;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Accounts: pallet_accounts::{Pallet, Call, Storage, Event<T>, ValidateUnsigned},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
		Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		Oracle: pallet_oracle::{Pallet, Call, Storage, Event<T>, ValidateUnsigned},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
	pub StorageBytesMultiplier: u64 = 10;
	pub const IsTransferable: bool = false;
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
	type AccountId = sp_core::ed25519::Public;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u128>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = ConstU32<2>;
}

pub type Extrinsic = TestXt<Call, ()>;
type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

impl frame_system::offchain::SigningTypes for Test {
	type Public = <Signature as Verify>::Signer;
	type Signature = Signature; // The RHS is `sp_core::ed25519::Signature` (which you also find at the top as an import)
}

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

impl pallet_balances::Config for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	/// The minimum amount required to keep an account open.
	type ExistentialDeposit = ConstU128<500>;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type IsTransferable = IsTransferable;
}

parameter_types! {
	pub const AssetDeposit: Balance = 100 * DOLLARS;
	pub const ApprovalDeposit: Balance = 1 * DOLLARS;
	pub const StringLimit: u32 = 50;
	pub const MetadataDepositBase: Balance = 10 * DOLLARS;
	pub const MetadataDepositPerByte: Balance = 1 * DOLLARS;
}
impl pallet_assets::Config for Test {
	type Event = Event;
	type Balance = Balance;
	type AssetId = u64;
	type Currency = Balances;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type AssetDeposit = AssetDeposit;
	type AssetAccountDeposit = ConstU128<DOLLARS>;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = StringLimit;
	type Freezer = ();
	type WeightInfo = ();
	type Extra = ();
}

impl pallet_accounts::EthFragContract for Test {
	fn get_partner_contracts() -> Vec<String> {
		vec![String::from("0x8a819F380ff18240B5c11010285dF63419bdb2d5")]
	}
}

impl pallet_accounts::Config for Test {
	type Event = Event;
	type WeightInfo = ();
	type EthChainId = ConstU64<5>; // goerli
	type EthConfirmations = ConstU64<1>;
	type EthFragContract = Test;
	type Threshold = ConstU64<1>;
	type AuthorityId = pallet_accounts::crypto::FragAuthId;
	type InitialPercentageNova = ConstU8<20>;
	type USDEquivalentAmount = ConstU128<100>;
}

impl pallet_proxy::Config for Test {
	type Event = Event;
	type Call = Call;
	type Currency = Balances;
	type ProxyType = ();
	type ProxyDepositBase = ConstU128<1>;
	type ProxyDepositFactor = ConstU128<1>;
	type MaxProxies = ConstU32<4>;
	type WeightInfo = ();
	type MaxPending = ConstU32<2>;
	type CallHasher = BlakeTwo256;
	type AnnouncementDepositBase = ConstU128<1>;
	type AnnouncementDepositFactor = ConstU128<1>;
}

impl pallet_timestamp::Config for Test {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = ();
	type WeightInfo = ();
}

impl OracleContract for Test {
	/// get the default oracle provider
	fn get_provider() -> OracleProvider {
		OracleProvider::Uniswap("can-be-whatever-here".encode()) // never used
	}
}

impl pallet_oracle::Config for Test {
	type AuthorityId = pallet_oracle::crypto::FragAuthId;
	type Event = Event;
	type OracleProvider = Test;
	type Threshold = ConstU64<1>;
}

fn create_public_key(keystore: &KeyStore) -> sp_core::ed25519::Public {
	const PHRASE: &str =
		"news slush supreme milk chapter athlete soap sausage put clutch what kitten";
	SyncCryptoStore::ed25519_generate_new(
		keystore,
		<crate::crypto::Public as RuntimeAppPublic>::ID,
		Some(&format!("{}", PHRASE)),
	)
	.unwrap();
	keystore.ed25519_public_keys(crate::crypto::Public::ID).get(0).unwrap().clone()
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
	let (offchain, offchain_state) = testing::TestOffchainExt::new();
	let (pool, pool_state) = testing::TestTransactionPoolExt::new();

	let keystore = KeyStore::new();
	let ed25519_public_key = create_public_key(&keystore);

	let mut t = sp_io::TestExternalities::default();
	t.register_extension(OffchainDbExt::new(offchain.clone()));
	t.register_extension(OffchainWorkerExt::new(offchain));
	t.register_extension(TransactionPoolExt::new(pool));
	t.register_extension(KeystoreExt(Arc::new(keystore)));

	t.execute_with(|| System::set_block_number(1)); // if we don't execute this line, Events are not emitted from extrinsics (I don't know why this is the case though)

	(t, pool_state, offchain_state, ed25519_public_key) // copied from https://github.com/JoshOrndorff/recipes/blob/master/pallets/ocw-demo/src/tests.rs
}
