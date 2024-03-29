#![cfg(test)]

use crate as pallet_aliases;
use crate::*;
use codec::Encode;
use frame_support::{parameter_types, traits::AsEnsureOriginWithArg, weights::Weight};
use frame_system;
use pallet_oracle::{OracleContract, OracleProvider};
use sp_core::{ed25519::Signature, H256};
use sp_runtime::{
	testing::{Header, TestXt},
	traits::{
		BlakeTwo256, ConstBool, ConstU128, ConstU32, ConstU64, Extrinsic as ExtrinsicT,
		IdentifyAccount, IdentityLookup, Verify,
	},
};

/// Balance of an account.
pub type Balance = u128;

pub const MILLICENTS: Balance = 1_000_000_000;
pub const CENTS: Balance = 1_000 * MILLICENTS;
pub const DOLLARS: Balance = 100 * CENTS;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		AliasesPallet: pallet_aliases::{Pallet, Call, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Protos: pallet_protos::{Pallet, Call, Storage, Event<T>},
		FragmentsPallet: pallet_fragments::{Pallet, Call, Storage, Event<T>},
		Contracts: pallet_contracts::{Pallet, Call, Storage, Event<T>},
		Oracle: pallet_oracle::{Pallet, Call, Storage, Event<T>},
		Clusters: pallet_clusters::{Pallet, Call, Storage, Event<T>},
		Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
		Accounts: pallet_accounts::{Pallet, Call, Storage, Event<T>},
		Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		Detach: pallet_detach::{Pallet, Call, Storage, Event<T>},
		CollectiveFlip: pallet_insecure_randomness_collective_flip::{Pallet, Storage},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
	pub const IsTransferable: bool = false;
}

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
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
	type BlockHashCount = ConstU64<250>;
	type DbWeight = ();
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

impl pallet_detach::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type AuthorityId = pallet_detach::crypto::DetachAuthId;
}

impl pallet_insecure_randomness_collective_flip::Config for Test {}

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

impl pallet_proxy::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
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

impl pallet_oracle::Config for Test {
	type AuthorityId = pallet_oracle::crypto::FragAuthId;
	type RuntimeEvent = RuntimeEvent;
	type OracleProvider = Test;
	type Threshold = ConstU64<1>;
}

impl OracleContract for Test {
	/// get the default oracle provider
	fn get_provider() -> pallet_oracle::OracleProvider {
		OracleProvider::Uniswap("can-be-whatever-here".encode()) // never used
	}
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
	type ChainExtension = ();
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

impl pallet_clusters::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type NameLimit = ConstU32<10>;
	type DataLimit = ConstU32<100>;
	type MembersLimit = ConstU32<20>;
	type RoleSettingsLimit = ConstU32<20>;
}

parameter_types! {
	pub RootNamespace: Vec<u8> = b"Frag".to_vec();
}

impl pallet_aliases::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type NamespacePrice = ConstU128<100>;
	type NameLimit = ConstU32<20>;
	type RootNamespace = RootNamespace;
}

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

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	let mut ext = sp_io::TestExternalities::new(t);

	ext.execute_with(|| System::set_extrinsic_index(2));
	ext.execute_with(|| System::set_block_number(1)); // if we don't execute this line, Events are not emitted from extrinsics (I don't know why this is the case though)

	ext
}
