pub use crate as pallet_protos;
use crate::*;
use frame_support::{
	parameter_types,
	traits::{ConstU32, ConstU64},
};
use frame_system as system;
use sp_core::{ed25519::Signature, H256};
use sp_runtime::{
	testing::{Header, TestXt},
	traits::{BlakeTwo256, Extrinsic as ExtrinsicT, IdentifyAccount, IdentityLookup, Verify},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
pub const DATA: &str = "0x0155a0e40220";
pub const PROTO_HASH: Hash256 = [
	30, 138, 136, 186, 232, 46, 112, 65, 122, 54, 110, 89, 123, 195, 7, 150, 12, 134, 10, 179, 245,
	51, 83, 227, 72, 251, 5, 148, 207, 251, 119, 59,
];
pub const PUBLIC: [u8; 33] = [
	3, 137, 65, 23, 149, 81, 74, 241, 98, 119, 101, 236, 239, 252, 189, 0, 39, 25, 240, 49, 96, 79,
	173, 215, 209, 136, 226, 220, 88, 91, 78, 26, 251,
];
pub const PUBLIC1: [u8; 32] = [
	137, 65, 23, 149, 81, 74, 241, 98, 119, 101, 236, 239, 252, 189, 0, 39, 25, 240, 49, 96, 79,
	173, 215, 209, 136, 226, 220, 88, 91, 78, 26, 251,
];

/// Construct a mock runtime environment.
frame_support::construct_runtime!(
	// The **configuration type `Test`** is defined as a **Rust enum** with **implementations**
	// for **each of the pallet configuration trait** that are **used in the mock runtime**. (https://docs.substrate.io/v3/runtime/testing/)
	//
	// Basically the **enum `Test`** is mock-up of **`Runtime` in pallet-protos (i.e in `pallet/protos/src/lib.rs`)
	// NOTE: The aforementioned `T` is bound by **trait `pallet:Config`**, if you didn't know
	pub enum Test where
		Block = Block, //  Block is the block type that is used in the runtime
		NodeBlock = Block, // NodeBlock is the block type that is used in the node
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		ProtosPallet: pallet_protos::{Pallet, Call, Storage, Event<T>},
		DetachPallet: pallet_detach::{Pallet, Call, Storage, Event<T>},
		CollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Tickets: pallet_tickets::{Pallet, Call, Storage, Event<T>},
	}
);

/// When to use:
///
/// To declare parameter types for a pallet's relevant associated types during runtime construction.
///
/// What it does:
///
/// The macro replaces each parameter specified into a struct type with a get() function returning
/// its specified value. Each parameter struct type also implements the
/// frame_support::traits::Get<I> trait to convert the type to its specified value.
///
/// Source: https://docs.substrate.io/v3/runtime/macros/
parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
	pub StorageBytesMultiplier: u64 = 10;
}

impl system::Config for Test {
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
	type AccountData = pallet_balances::AccountData<u64>;
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
	type Signature = Signature;
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

/// If `Test` implements `pallet_balances::Config`, the assignment might use `u64` for the `Balance` type. (https://docs.substrate.io/v3/runtime/testing/)
///
/// By assigning `pallet_balances::Balance` and `frame_system::AccountId` (see implementation block
/// `impl system::Config for Test` above) to `u64`, mock runtimes ease the mental overhead of
/// comprehensive, conscientious testers. Reasoning about accounts and balances only requires tracking a `(AccountId: u64, Balance: u64)` mapping. (https://docs.substrate.io/v3/runtime/testing/)
impl pallet_balances::Config for Test {
	type Balance = u64;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
}

impl pallet_tickets::Config for Test {
	type Event = Event;
	type WeightInfo = ();
	type EthChainId = ConstU64<5>; // goerli
	type EthFragContract = ();
	type EthConfirmations = ConstU64<1>;
	type Threshold = ConstU64<1>;
	type AuthorityId = pallet_tickets::crypto::FragAuthId;
}

impl Config for Test {
	type Event = Event;
	type WeightInfo = ();
	type StorageBytesMultiplier = StorageBytesMultiplier;
	type StakeLockupPeriod = ConstU64<100800>; // one week
}

impl pallet_detach::Config for Test {
	type Event = Event;
	type WeightInfo = ();
	type AuthorityId = pallet_detach::crypto::DetachAuthId;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	t.into()
}
