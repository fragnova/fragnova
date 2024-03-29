#![cfg(test)]

use crate::chain_extension;

use frame_support::{
	parameter_types,
	traits::{AsEnsureOriginWithArg, ConstBool, ConstU32, ConstU64},
	weights::Weight,
};
use frame_system;

use codec::Encode;
use sp_core::{ed25519::Signature, H256};
use sp_runtime::{
	testing::{Header, TestXt},
	traits::{
		BlakeTwo256, ConstU128, Extrinsic as ExtrinsicT, IdentifyAccount, IdentityLookup, Verify,
	},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

/// Balance of an account.
pub type Balance = u128;

pub const MILLICENTS: Balance = 1_000_000_000;
pub const CENTS: Balance = 1_000 * MILLICENTS; // assume this is worth about a cent.
pub const DOLLARS: Balance = 100 * CENTS;

// Construct a mock runtime environment.
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
		CollectiveFlip: pallet_insecure_randomness_collective_flip::{Pallet, Storage},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
		Contracts: pallet_contracts::{Pallet, Call, Storage, Event<T>},
		Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},

		Protos: pallet_protos::{Pallet, Call, Storage, Event<T>},
		Fragments: pallet_fragments::{Pallet, Call, Storage, Event<T>},
		Detach: pallet_detach::{Pallet, Call, Storage, Event<T>},
		Accounts: pallet_accounts::{Pallet, Call, Storage, Event<T>},
		Oracle: pallet_oracle::{Pallet, Call, Storage, Event<T>},
		Clusters: pallet_clusters::{Pallet, Call, Storage, Event<T>},
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
	pub const IsTransferable: bool = false;
}
impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = sp_core::ed25519::Public;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
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

pub type Extrinsic = TestXt<RuntimeCall, ()>;
type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

impl frame_system::offchain::SigningTypes for Test {
	type Public = <Signature as Verify>::Signer;
	type Signature = Signature;
}

impl<LocalCall> frame_system::offchain::SendTransactionTypes<LocalCall> for Test
where
	RuntimeCall: From<LocalCall>,
{
	type OverarchingCall = RuntimeCall;
	type Extrinsic = Extrinsic;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Test
where
	RuntimeCall: From<LocalCall>,
{
	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
		call: RuntimeCall,
		_public: <Signature as Verify>::Signer,
		_account: AccountId,
		nonce: u64,
	) -> Option<(RuntimeCall, <Extrinsic as ExtrinsicT>::SignaturePayload)> {
		Some((call, (nonce, ())))
	}
}

impl pallet_insecure_randomness_collective_flip::Config for Test {}

/// If `Test` implements `pallet_balances::Config`, the assignment might use `u64` for the `Balance` type. (https://docs.substrate.io/v3/runtime/testing/)
///
/// By assigning `pallet_balances::Balance` and `frame_system::AccountId` (see implementation block `impl system::Config for Test` above) to `u64`,
/// mock runtimes ease the mental overhead of comprehensive, conscientious testers.
/// Reasoning about accounts and balances only requires tracking a `(AccountId: u64, Balance: u64)` mapping. (https://docs.substrate.io/v3/runtime/testing/)
impl pallet_balances::Config for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
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
	pub const StringLimit: u32 = 75;
	pub const MetadataDepositBase: Balance = 10 * DOLLARS;
	pub const MetadataDepositPerByte: Balance = 1 * DOLLARS;
}
impl pallet_assets::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type RemoveItemsLimit = ConstU32<1000>;
	type AssetId = u64;
	type AssetIdParameter = u64;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<AccountId>>;
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
	type CallbackHandle = ();
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

parameter_types! {
	pub const DeletionWeightLimit: Weight = Weight::from_ref_time(500_000_000_000);
	pub MySchedule: pallet_contracts::Schedule<Test> = {
		let mut schedule = <pallet_contracts::Schedule<Test>>::default();
		schedule.instruction_weights.fallback = 1;
		schedule
	};
	pub static DepositPerByte: u64 = 1;
	pub const DepositPerItem: u64 = 2;
}

impl pallet_contracts::Config for Test {
	type Time = Timestamp;
	type Randomness = CollectiveFlip;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type CallFilter = frame_support::traits::Nothing;
	type CallStack = [pallet_contracts::Frame<Self>; 5];
	type WeightPrice = ();
	type WeightInfo = ();
	type ChainExtension = chain_extension::MyExtension; // This is what we are testing!
	type DeletionQueueDepth = ConstU32<1024>;
	type DeletionWeightLimit = DeletionWeightLimit;
	type Schedule = MySchedule;
	type DepositPerByte = DepositPerByte;
	type DepositPerItem = DepositPerItem;
	type AddressGenerator = pallet_contracts::DefaultAddressGenerator;
	type MaxCodeLen = ConstU32<{ 123 * 1024 }>;
	type MaxStorageKeyLen = ConstU32<128>;
	type UnsafeUnstableInterface = ConstBool<false>;
	type MaxDebugBufferLen = ConstU32<{ 2 * 1024 * 1024 }>;
	type IsTransferable = ConstBool<false>;
}

impl pallet_timestamp::Config for Test {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = ();
	type WeightInfo = ();
}

impl pallet_proxy::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = ();
	type ProxyType = ();
	type ProxyDepositBase = ConstU32<1>;
	type ProxyDepositFactor = ConstU32<1>;
	type MaxProxies = ConstU32<4>;
	type WeightInfo = ();
	type MaxPending = ConstU32<2>;
	type CallHasher = BlakeTwo256;
	type AnnouncementDepositBase = ConstU32<1>;
	type AnnouncementDepositFactor = ConstU32<1>;
}

impl pallet_detach::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type AuthorityId = pallet_detach::crypto::DetachAuthId;
}

impl pallet_clusters::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type NameLimit = ConstU32<10>;
	type DataLimit = ConstU32<100>;
	type MembersLimit = ConstU32<20>;
	type RoleSettingsLimit = ConstU32<20>;
}

impl pallet_oracle::OracleContract for Test {
	/// get the default oracle provider
	fn get_provider() -> pallet_oracle::OracleProvider {
		pallet_oracle::OracleProvider::Uniswap("can-be-whatever-here".encode()) // never used
	}
}
impl pallet_oracle::Config for Test {
	type AuthorityId = pallet_oracle::crypto::FragAuthId;
	type RuntimeEvent = RuntimeEvent;
	type OracleProvider = Test;
	type Threshold = ConstU64<1>;
}

impl pallet_accounts::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type EthChainId = ConstU64<5>; // goerli
	type EthFragContract = ();
	type EthConfirmations = ConstU64<1>;
	type Threshold = ConstU64<1>;
	type AuthorityId = pallet_accounts::crypto::FragAuthId;
	type InitialPercentageNova = sp_runtime::traits::ConstU8<20>;
	type USDEquivalentAmount = ConstU128<100>;
}

impl pallet_protos::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type StringLimit = StringLimit;
	type DetachAccountLimit = ConstU32<20>;
	type MaxTags = ConstU32<10>;
}

impl pallet_fragments::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	let mut ext = sp_io::TestExternalities::new(t);

	ext.execute_with(|| System::set_block_number(1)); // if we don't execute this line, Events are not emitted from extrinsics (I don't know why this is the case though)

	ext
}
