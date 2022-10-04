//! The Runtime of the Clamor Node.
//!
//! The runtime for a Substrate node contains all of the business logic
//! for executing transactions, saving state transitions, and interacting with the outer node.

// Some of the Substrate Macros in this file throw missing_docs warnings.
// That's why we allow this file to have missing_docs.
#![allow(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

/// This will include the generated WASM binary as two constants WASM_BINARY and WASM_BINARY_BLOATY. The former is a compact WASM binary and the latter is not compacted.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use frame_support::{
	traits::{ConstU128, ConstU16, ConstU32, ConstU64},
	weights::DispatchClass,
};
use frame_system::{
	limits::{BlockLength, BlockWeights},
	EnsureRoot,
};
use pallet_grandpa::{
	fg_primitives, AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList,
};
use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{
		BlakeTwo256, Block as BlockT, Extrinsic as ExtrinsicT, IdentifyAccount, NumberFor, Verify,
	},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, MultiSignature,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

// A few exports that help ease life for downstream crates.
pub use frame_support::{
	construct_runtime, parameter_types,
	traits::{Contains, KeyOwnerProofSystem, Randomness, StorageInfo},
	weights::{
		constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
		IdentityFee, Weight,
	},
	StorageValue,
};
pub use pallet_balances::Call as BalancesCall;
pub use pallet_protos::Call as ProtosCall;
pub use pallet_timestamp::Call as TimestampCall;
use pallet_transaction_payment::CurrencyAdapter;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{Perbill, Permill};

use scale_info::prelude::string::String;

use codec::Encode;
use sp_runtime::traits::{SaturatedConversion, StaticLookup};

use pallet_fragments::{GetDefinitionsParams, GetInstancesParams};
use pallet_protos::GetProtosParams;

pub use pallet_contracts::Schedule;

/// Prints debug output of the `contracts` pallet to stdout if the node is
/// started with `-lruntime::contracts=debug`.
pub const CONTRACTS_DEBUG_OUTPUT: bool = true;

/// An index to a block.
pub type BlockNumber = u64;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;

/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<Call, SignedExtra>;

/// Related to Index pallet
pub type AccountIndex = u64;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
	use super::*;

	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;

	/// Implement OpaqueKeys for a described struct.
	/// Every field type must implement BoundToRuntimeAppPublic. KeyTypeIdProviders is set to the types given as fields.
	impl_opaque_keys! {
		/// TODO: Documentation
		pub struct SessionKeys {
			/// TODO: Documentation
			pub aura: Aura,
			/// TODO: Documentation
			pub grandpa: Grandpa,
		}
	}
}

/// To learn more about runtime versioning and what each of the following value means:
///   https://docs.substrate.io/v3/runtime/upgrades#runtime-versioning
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("fragnova-testnet"),
	impl_name: create_runtime_str!("fragnova-color"),
	authoring_version: 1,
	// The version of the runtime specification. A full node will not attempt to use its native
	//   runtime in substitute for the on-chain Wasm runtime unless all of `spec_name`,
	//   `spec_version`, and `authoring_version` are the same between Wasm and native.
	// This value is set to 100 to notify Polkadot-JS App (https://polkadot.js.org/apps) to use
	//   the compatible custom types.
	spec_version: 2,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 1,
};

/// This determines the average expected block time that we are targeting.
/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
/// up by `pallet_aura` to implement `fn slot_duration()`.
///
/// Change this to adjust the block time.
pub const MILLISECS_PER_BLOCK: u64 = 6000;
/// TODO: Documentation
pub const MILLICENTS: Balance = 1_000_000_000;
/// TODO: Documentation
pub const CENTS: Balance = 1_000 * MILLICENTS; // assume this is worth about a cent.
/// TODO: Documentation
pub const DOLLARS: Balance = 100 * CENTS;
/// The amount of balance a caller has to pay for calling an extrinsic with `bytes` bytes and .
pub const fn deposit(items: u32, bytes: u32) -> Balance {
	items as Balance * 15 * CENTS + (bytes as Balance) * 6 * CENTS
}

/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
///
/// Note: Currently it is not possible to change the slot duration after the chain has started.
///       Attempting to do so will brick block production.
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// Time is measured by number of blocks.
/// Number of blocks that would be added to the Blockchain on average, when one minute passes
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
/// Number of blocks that would be added to the Blockchain on average, when one hour passes
pub const HOURS: BlockNumber = MINUTES * 60;
/// Number of blocks that would be added to the Blockchain on average, when one day passes
pub const DAYS: BlockNumber = HOURS * 24;

/// The version information is used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

/// We assume that ~10% of the block weight is consumed by `on_initialize` handlers.
/// This is used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used
/// by  Operational  extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for 2 seconds of compute with a 6 second average block time.
const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_SECOND.saturating_mul(2);

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
	/// TODO: Documentation
	pub const Version: RuntimeVersion = VERSION;
	/// TODO: Documentation
	pub const BlockHashCount: BlockNumber = 2400;
	/// TODO: Documentation
	pub const SS58Prefix: u8 = 93;

	/// We allow for 2 seconds of compute with a 6 second average block time.
	pub RuntimeBlockLength: BlockLength = BlockLength
		::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);

	pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have some extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Config for Runtime {
	/// The basic call filter to use in dispatchable.
	type BaseCallFilter = frame_support::traits::Everything;
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = RuntimeBlockWeights;
	/// The maximum length of a block (in bytes).
	type BlockLength = RuntimeBlockLength;
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The aggregated dispatch type that is available for extrinsics.
	type Call = Call;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = Indices;
	/// The index type for storing how many extrinsics an account has signed.
	type Index = Index;
	/// The index type for blocks.
	type BlockNumber = BlockNumber;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The header type.
	type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// The ubiquitous event type.
	type Event = Event;
	/// The ubiquitous origin type.
	type Origin = Origin;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = RocksDbWeight;
	/// Version of the runtime.
	type Version = Version;
	/// Converts a module to the index of the module in `construct_runtime!`.
	///
	/// This type is being generated by `construct_runtime!`.
	type PalletInfo = PalletInfo;
	/// What to do if a new account is created.
	type OnNewAccount = ();
	/// What to do if an account is fully reaped from the system.
	type OnKilledAccount = ();
	/// The data to be stored in an account.
	type AccountData = pallet_balances::AccountData<Balance>;
	/// Weight information for the extrinsics of this pallet.
	type SystemWeightInfo = ();
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = SS58Prefix;
	/// The set code logic, just the default since we're not a parachain.
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_randomness_collective_flip::Config for Runtime {}

parameter_types! {
	/// The maximum number of authorities that `pallet_aura` can hold.
	pub const MaxAuthorities: u32 = 32;
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = MaxAuthorities;
}

impl pallet_grandpa::Config for Runtime {
	type Event = Event;
	type Call = Call;

	type KeyOwnerProofSystem = ();

	type KeyOwnerProof =
		<Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;

	type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
		KeyTypeId,
		GrandpaId,
	)>>::IdentificationTuple;

	type HandleEquivocation = ();

	type WeightInfo = ();
	type MaxAuthorities = MaxAuthorities;
}

parameter_types! {
	/// TODO: Documentation
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = Aura;
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	/// The minimum amount required to keep an account open.
	pub const ExistentialDeposit: u128 = 500;
	/// The maximum number of locks that should exist on an account.
	/// Not strictly enforced, but used for weight estimation.
	pub const MaxLocks: u32 = 50;
	pub const IsTransferable: bool = false;
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = MaxLocks;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
	type IsTransferable = IsTransferable;
}

/// Parameters related to calculating the Weight fee.
parameter_types! {
	/// The amount of balance a caller (here "caller" refers to a "smart-contract account") has to pay for each storage item.
	///
	/// Note: Changing this value for an existing chain might need a storage migration.
	///
	/// # Definition of a "smart-contract account"
	///
	/// “smart-contract accounts” have the ability to instantiate smart-contracts and make calls to other contract and non-contract accounts.
	/// When a smart-contract is called,
	/// its associated code is retrieved via the code hash and gets executed.
	/// This call can alter the storage entries of the smart-contract account, instantiate new smart-contracts, or call other smart-contracts.
	///
	/// See for more information: https://paritytech.github.io/substrate/master/pallet_contracts/index.html
	pub const DepositPerItem: Balance = deposit(1, 0);
	/// The amount of balance a caller (here "caller" refers to a "smart-contract account") has to pay for each byte of storage.
	///
	/// Note: Changing this value for an existing chain might need a storage migration.
	///
	/// # Definition of  a "smart-contract account"
	///
	/// “smart-contract accounts” have the ability to instantiate smart-contracts and make calls to other contract and non-contract accounts.
	/// When a smart-contract is called,
	/// its associated code is retrieved via the code hash and gets executed.
	/// This call can alter the storage entries of the smart-contract account, instantiate new smart-contracts, or call other smart-contracts.
	///
	/// See for mor information: https://paritytech.github.io/substrate/master/pallet_contracts/index.html
	pub const DepositPerByte: Balance = deposit(0, 1);
	// pub const MaxValueSize: u32 = 16_384;
	/// The maximum number of contracts that can be pending for deletion.
	pub const DeletionQueueDepth: u32 = 1024;
	/// The maximum amount of weight that can be consumed per block for lazy trie removal.
	pub const DeletionWeightLimit: Weight = Weight::from_ref_time(500_000_000_000);
	// pub const MaxCodeSize: u32 = 2 * 1024;
	/// Cost schedule and limits.
	pub MySchedule: Schedule<Runtime> = <Schedule<Runtime>>::default();
	/// A fee mulitplier for `Operational` extrinsics to compute "virtual tip" to boost their
	/// `priority`
	pub OperationalFeeMultiplier: u8 = 5;
	/// Weight for adding a a byte worth of storage in certain extrinsics such as `upload()`.
	pub StorageBytesMultiplier: u64 = 10;
}

impl pallet_transaction_payment::Config for Runtime {
	type Event = Event;
	type OnChargeTransaction = CurrencyAdapter<Balances, ()>;
	type OperationalFeeMultiplier = OperationalFeeMultiplier;
	type WeightToFee = IdentityFee<Balance>;
	type LengthToFee = IdentityFee<Balance>;
	type FeeMultiplierUpdate = ();
}

impl pallet_sudo::Config for Runtime {
	type Event = Event;
	type Call = Call;
}

impl pallet_fragments::Config for Runtime {
	type Event = Event;
	type WeightInfo = ();
}

impl pallet_accounts::EthFragContract for Runtime {
	fn get_partner_contracts() -> Vec<String> {
		vec![String::from("0x34670f29e28b5dc0c47a8cc22d221bf26929f9ac")]
	}
}

parameter_types! {
	pub const TicketsAssetId: u64 = 1337;
}

impl pallet_accounts::Config for Runtime {
	type Event = Event;
	type WeightInfo = ();
	type EthChainId = ConstU64<5>; // goerli
	type EthFragContract = Runtime;
	type EthConfirmations = ConstU64<1>;
	type Threshold = ConstU64<1>;
	type AuthorityId = pallet_accounts::crypto::FragAuthId;
	type TicketsAssetId = TicketsAssetId;
	type InitialPercentageTickets = ConstU64<80>;
	type InitialPercentageNova = ConstU64<20>;
	type USDEquivalentAmount = ConstU64<100>;
}

impl pallet_protos::Config for Runtime {
	type Event = Event;
	type WeightInfo = ();
	type StorageBytesMultiplier = StorageBytesMultiplier;
	type CurationExpiration = ConstU64<100800>; // one week
	type TicketsAssetId = TicketsAssetId;
}

impl pallet_detach::Config for Runtime {
	type Event = Event;
	type WeightInfo = ();
	type AuthorityId = pallet_detach::crypto::DetachAuthId;
}

impl pallet_multisig::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type Currency = Balances;
	type DepositBase = ConstU128<1>;
	type DepositFactor = ConstU128<1>;
	type MaxSignatories = ConstU16<3>;
	type WeightInfo = ();
}

impl pallet_proxy::Config for Runtime {
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

parameter_types! {
	/// Maximum number of additional fields that may be stored in an ID. Needed to bound the I/O
	/// required to access an identity, but can be pretty high.
	pub const MaxAdditionalFields: u32 = 2;
	/// Maxmimum number of registrars allowed in the system. Needed to bound the complexity
	/// of, e.g., updating judgements.
	pub const MaxRegistrars: u32 = 20;
}

impl pallet_identity::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type Slashed = ();
	type BasicDeposit = ConstU128<10>;
	type FieldDeposit = ConstU128<10>;
	type SubAccountDeposit = ConstU128<10>;
	type MaxSubAccounts = ConstU32<2>;
	type MaxAdditionalFields = MaxAdditionalFields;
	type MaxRegistrars = MaxRegistrars;
	type RegistrarOrigin = EnsureRoot<AccountId>;
	type ForceOrigin = EnsureRoot<AccountId>;
	type WeightInfo = ();
}

impl pallet_utility::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = ();
}

/// Implement SigningTypes and SendTransactionTypes in the runtime to support submitting transactions by an off-chain worker,
/// whether they are signed or unsigned.
///
/// Source: https://docs.substrate.io/how-to-guides/v3/ocw/transactions/
impl frame_system::offchain::SigningTypes for Runtime {
	type Public = <Signature as Verify>::Signer;
	type Signature = Signature;
}

/// Implement SigningTypes and SendTransactionTypes in the runtime to support submitting transactions by an off-chain worker,
/// whether they are signed or unsigned.
///
/// Source: https://docs.substrate.io/how-to-guides/v3/ocw/transactions/
impl<LocalCall> frame_system::offchain::SendTransactionTypes<LocalCall> for Runtime
where
	Call: From<LocalCall>,
{
	type OverarchingCall = Call;
	type Extrinsic = UncheckedExtrinsic;
}

/// Because you configured the Config trait for detach pallet and frag pallet
/// to implement the CreateSignedTransaction trait, you also need to implement that trait for the runtime.
impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
	Call: From<LocalCall>,
{
	/// The code seems long, but what it tries to do is really:
	/// 	- Create and prepare extra of SignedExtra type, and put various checkers in-place.
	/// 	- Create a raw payload based on the passed in call and extra.
	/// 	- Sign the raw payload with the account public key.
	/// 	- Finally, bundle all data up and return a tuple of the call, the caller, its signature,
	/// 	  and any signed extension data.
	///
	/// Source: https://docs.substrate.io/how-to-guides/v3/ocw/transactions/
	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
		call: Call,
		public: <Signature as Verify>::Signer,
		account: AccountId,
		nonce: Index,
	) -> Option<(Call, <UncheckedExtrinsic as ExtrinsicT>::SignaturePayload)> {
		let tip = 0;
		// take the biggest period possible.
		let period =
			BlockHashCount::get().checked_next_power_of_two().map(|c| c / 2).unwrap_or(2) as u64;
		let current_block = System::block_number()
			.saturated_into::<u64>()
			// The `System::block_number` is initialized with `n+1`,
			// so the actual block number is `n`.
			.saturating_sub(1);
		let era = generic::Era::mortal(period, current_block);
		let extra = (
			frame_system::CheckSpecVersion::<Runtime>::new(),
			frame_system::CheckTxVersion::<Runtime>::new(),
			frame_system::CheckGenesis::<Runtime>::new(),
			frame_system::CheckEra::<Runtime>::from(era),
			frame_system::CheckNonce::<Runtime>::from(nonce),
			frame_system::CheckWeight::<Runtime>::new(),
			pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
		);
		let raw_payload = SignedPayload::new(call, extra)
			.map_err(|_e| {
				// log::warn!("Unable to create signed payload: {:?}", e);
			})
			.ok()?;
		let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
		let address = Indices::unlookup(account);
		let (call, extra, _) = raw_payload.deconstruct();
		Some((call, (address, signature.into(), extra)))
	}
}

impl pallet_contracts::Config for Runtime {
	type Time = Timestamp;
	type Randomness = RandomnessCollectiveFlip;
	type Currency = Balances;
	type Event = Event;
	type Call = Call;
	/// The safest default is to allow no calls at all.
	///
	/// Runtimes should whitelist dispatchables that are allowed to be called from contracts
	/// and make sure they are stable. Dispatchables exposed to contracts are not allowed to
	/// change because that would break already deployed contracts. The `Call` structure itself
	/// is not allowed to change the indices of existing pallets, too.
	type CallFilter = frame_support::traits::Nothing;
	type DepositPerItem = DepositPerItem;
	type DepositPerByte = DepositPerByte;
	type CallStack = [pallet_contracts::Frame<Self>; 31];
	type WeightPrice = pallet_transaction_payment::Pallet<Self>;
	type WeightInfo = ();
	type ChainExtension = ();
	type DeletionQueueDepth = DeletionQueueDepth;
	type DeletionWeightLimit = DeletionWeightLimit;
	type Schedule = MySchedule;
	type AddressGenerator = pallet_contracts::DefaultAddressGenerator;
	type ContractAccessWeight = pallet_contracts::DefaultContractAccessWeight<RuntimeBlockWeights>;
	type MaxCodeLen = ConstU32<{ 128 * 1024 }>;
	type RelaxedMaxCodeLen = ConstU32<{ 256 * 1024 }>;
	type MaxStorageKeyLen = ConstU32<128>;
}

parameter_types! {
	/// TODO: Documentation
	pub const IndexDeposit: Balance = 500;
}

impl pallet_indices::Config for Runtime {
	type AccountIndex = AccountIndex;
	type Currency = Balances;
	type Deposit = IndexDeposit;
	type Event = Event;
	type WeightInfo = pallet_indices::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	/// The basic amount of funds that must be reserved for an asset.
	pub const AssetDeposit: Balance = 100 * DOLLARS;
	/// The amount of funds that must be reserved when creating a new approval.
	pub const ApprovalDeposit: Balance = 1 * DOLLARS;
	/// The maximum length of a name or symbol of an asset stored on-chain.
	pub const StringLimit: u32 = 50;
	/// The basic amount of funds that must be reserved when adding metadata to your asset.
	pub const MetadataDepositBase: Balance = 10 * DOLLARS;
	/// The additional funds that must be reserved for the number of bytes you store in your
	/// asset's metadata.
	pub const MetadataDepositPerByte: Balance = 1 * DOLLARS;
}

impl pallet_assets::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type AssetId = u64;
	type Currency = Balances;
	type ForceOrigin = EnsureRoot<AccountId>;
	type AssetDeposit = AssetDeposit;
	type AssetAccountDeposit = ConstU128<DOLLARS>;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = StringLimit;
	type Freezer = ();
	type Extra = ();
	type WeightInfo = pallet_assets::weights::SubstrateWeight<Runtime>;
}

// Construct the Substrate runtime and integrates various pallets into the aforementioned runtime.
//
// The parameters here are specific types for `Block`, `NodeBlock`, and `UncheckedExtrinsic` and the pallets that are used by the runtime.
//
// Each pallet is declared like **"<Identifier>: <path::to::pallet>[<::{Part1, Part<T>, ..}>]"**, where:
//
// - `Identifier`: name given to the pallet that uniquely identifies it.
// - `:`: colon separator
// - `path::to::pallet`: identifiers separated by colons which declare the path to a pallet definition.
// - `::{ Part1, Part2<T>, .. }` (optional if the pallet was declared with a `frame_support::pallet:` macro): **Comma separated parts declared with their generic**.

// 	**If** a **pallet is **declared with `frame_support::pallet` macro** then the **parts can be automatically derived if not explicitly provided**.
//  We provide support for the following module parts in a pallet:
//
// 	- `Pallet` - Required for all pallets
// 	- `Call` - If the pallet has callable functions
// 	- `Storage` - If the pallet uses storage
// 	- `Event` or `Event<T>` (if the event is generic) - If the pallet emits events
// 	- `Origin` or `Origin<T>` (if the origin is generic) - If the pallet has instanciable origins
// 	- `Config` or `Config<T>` (if the config is generic) - If the pallet builds the genesis storage with GenesisConfig
// 	- `Inherent` - If the pallet provides/can check inherents.
// 	- `ValidateUnsigned` - If the pallet validates unsigned extrinsics.
//
//
// IMP NOTE 1: The macro generates a type alias for each pallet to their `Pallet`. E.g. `type System = frame_system::Pallet<Runtime>`
//
// IMP NOTE 2: The population of the genesis storage depends on the order of pallets.
// So, if one of your pallets depends on another pallet, the pallet that is depended upon needs to come before the pallet depending on it.
//
// V IMP NOTE 3: The order that the pallets appear in this macro determines its pallet index
construct_runtime!(
	pub enum Runtime where
		Block = Block, //  Block is the block type that is used in the runtime
		NodeBlock = opaque::Block, // NodeBlock is the block type that is used in the node
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		// The System pallet is responsible for accumulating the weight of each block as it gets executed and making sure that it does not exceed the limit.
		System: frame_system,
		RandomnessCollectiveFlip: pallet_randomness_collective_flip,
		Timestamp: pallet_timestamp,
		Aura: pallet_aura,
		Grandpa: pallet_grandpa,
		Balances: pallet_balances,
		TransactionPayment: pallet_transaction_payment,
		Sudo: pallet_sudo,
		Assets: pallet_assets,
		// Our additions
		Indices: pallet_indices,
		Contracts: pallet_contracts,
		// Since this is the 11th pallet that's defined in this macro, its pallet index is "11"
		Protos: pallet_protos,
		Fragments: pallet_fragments,
		Detach: pallet_detach,
		Multisig: pallet_multisig,
		Proxy: pallet_proxy,
		Identity: pallet_identity,
		Utility: pallet_utility,
		Accounts: pallet_accounts,
	}
);

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, AccountIndex>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
>;

/// Marks the given trait implementations as runtime apis.
///
/// For more information, read: https://paritytech.github.io/substrate/master/sp_api/macro.impl_runtime_apis.html
impl_runtime_apis! {

	/// TODO: Documentation
	impl sp_api::Core<Block> for Runtime {
		/// TODO: Documentation
		fn version() -> RuntimeVersion {
			VERSION
		}

		/// TODO: Documentation
		fn execute_block(block: Block) {
			Executive::execute_block(block);
		}

		/// TODO: Documentation
		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	/// TODO: Documentation
	impl sp_api::Metadata<Block> for Runtime {
		/// TODO: Documentation
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}
	}

	/// TODO: Documentation
	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		/// TODO: Documentation
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		/// TODO: Documentation
		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		/// TODO: Documentation
		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		/// TODO: Documentation
		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	/// TODO: Documentation
	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		/// TODO: Documentation
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			match tx.function {
				// We want to prevent polluting blocks with a lot of useless invalid data.
				// TODO perform quick and preliminary data validation
				#[allow(unused_variables)]
				Call::Protos(ProtosCall::upload{ref data, ref category, ref tags, ..}) => {
					// TODO
				},
				#[allow(unused_variables)]
				Call::Protos(ProtosCall::patch{ref data, ..}) |
				Call::Protos(ProtosCall::set_metadata{ref data, ..}) => {
					// TODO
					// if let Err(_) = <pallet_protos::Pallet<Runtime>>::ensure_valid_auth(auth) {
					// 	return InvalidTransaction::BadProof.into();
					// }
				},
				_ => {},
			}
			// Always run normally anyways
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	/// TODO: Documentation
	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		/// TODO: Documentation
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	/// TODO: Documentation
	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		/// TODO: Documentation
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		/// TODO: Documentation
		fn authorities() -> Vec<AuraId> {
			Aura::authorities().into_inner()
		}
	}

	/// TODO: Documentation
	impl sp_session::SessionKeys<Block> for Runtime {
		/// TODO: Documentation
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			opaque::SessionKeys::generate(seed)
		}

		/// TODO: Documentation
		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	/// TODO: Documentation
	impl fg_primitives::GrandpaApi<Block> for Runtime {
		/// TODO: Documentation
		fn grandpa_authorities() -> GrandpaAuthorityList {
			Grandpa::grandpa_authorities()
		}

		/// TODO: Documentation
		fn current_set_id() -> fg_primitives::SetId {
			Grandpa::current_set_id()
		}

		/// TODO: Documentation
		fn submit_report_equivocation_unsigned_extrinsic(
			_equivocation_proof: fg_primitives::EquivocationProof<
				<Block as BlockT>::Hash,
				NumberFor<Block>,
			>,
			_key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			None
		}

		/// TODO: Documentation
		fn generate_key_ownership_proof(
			_set_id: fg_primitives::SetId,
			_authority_id: GrandpaId,
		) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
			// NOTE: this is the only implementation possible since we've
			// defined our key owner proof type as a bottom type (i.e. a type
			// with no values).
			None
		}
	}

	/// TODO: Documentation
	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
		/// TODO: Documentation
		fn account_nonce(account: AccountId) -> Index {
			System::account_nonce(account)
		}
	}

	/// TODO: Documentation
	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		/// TODO: Documentation
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}

		/// TODO: Documentation
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
	}

	/// TODO: Documentation
		impl pallet_contracts_rpc_runtime_api::ContractsApi<Block, AccountId, Balance, BlockNumber, Hash>
		for Runtime
	{
		/// TODO: Documentation
		fn call(
			origin: AccountId,
			dest: AccountId,
			value: Balance,
			gas_limit: u64,
			storage_deposit_limit: Option<Balance>,
			input_data: Vec<u8>,
		) -> pallet_contracts_primitives::ContractExecResult<Balance> {
			Contracts::bare_call(origin, dest, value, Weight::from_ref_time(gas_limit), storage_deposit_limit, input_data, true)
		}

		/// TODO: Documentation
		fn instantiate(
			origin: AccountId,
			value: Balance,
			gas_limit: u64,
			storage_deposit_limit: Option<Balance>,
			code: pallet_contracts_primitives::Code<Hash>,
			data: Vec<u8>,
			salt: Vec<u8>,
		) -> pallet_contracts_primitives::ContractInstantiateResult<AccountId, Balance>
		{
			Contracts::bare_instantiate(origin, value, Weight::from_ref_time(gas_limit), storage_deposit_limit, code, data, salt, true)
		}

		/// TODO: Documentation
		fn upload_code(
			origin: AccountId,
			code: Vec<u8>,
			storage_deposit_limit: Option<Balance>,
		) -> pallet_contracts_primitives::CodeUploadResult<Hash, Balance>
		{
			Contracts::bare_upload_code(origin, code, storage_deposit_limit)
		}

		/// TODO: Documentation
		fn get_storage(
			address: AccountId,
			key: Vec<u8>,
		) -> pallet_contracts_primitives::GetStorageResult {
			Contracts::get_storage(address, key)
		}
	}

	/// TODO: Documentation
	impl pallet_protos_rpc_runtime_api::ProtosApi<Block, AccountId> for Runtime {
		/// TODO: Documentation
		fn get_protos(params: GetProtosParams<AccountId, Vec<u8>>) -> Result<Vec<u8>, Vec<u8>> {
			Protos::get_protos(params)
		}
	}

	/// TODO: Documentation
	impl pallet_fragments_rpc_runtime_api::FragmentsRuntimeApi<Block, AccountId> for Runtime {
		fn get_definitions(params: GetDefinitionsParams<AccountId, Vec<u8>>) -> Result<Vec<u8>, Vec<u8>> {
			Fragments::get_definitions(params)
		}
		fn get_instances(params: GetInstancesParams<AccountId, Vec<u8>>) -> Result<Vec<u8>, Vec<u8>> {
			Fragments::get_instances(params)
		}
	}

	/// TODO: Documentation
	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		/// TODO: Documentation
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{list_benchmark, baseline, Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;
			use frame_system_benchmarking::Pallet as SystemBench;
			use baseline::Pallet as BaselineBench;

			let mut list = Vec::<BenchmarkList>::new();

			list_benchmark!(list, extra, frame_benchmarking, BaselineBench::<Runtime>);
			list_benchmark!(list, extra, frame_system, SystemBench::<Runtime>);
			list_benchmark!(list, extra, pallet_balances, Balances);
			list_benchmark!(list, extra, pallet_timestamp, Timestamp);
			// list_benchmark!(list, extra, pallet_accounts, Accounts);
			list_benchmark!(list, extra, pallet_protos, Protos);
			list_benchmark!(list, extra, pallet_assets, Assets);
			list_benchmark!(list, extra, pallet_fragments, Fragments);
			list_benchmark!(list, extra, pallet_detach, Detach);
			list_benchmark!(list, extra, pallet_multisig, Multisig);
			list_benchmark!(list, extra, pallet_proxy, Proxy);
			list_benchmark!(list, extra, pallet_identity, Identity);
			list_benchmark!(list, extra, pallet_utility, Utility);

			let storage_info = AllPalletsWithSystem::storage_info();

			return (list, storage_info)
		}

		/// TODO: Documentation
		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{baseline, Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};

			use frame_system_benchmarking::Pallet as SystemBench;
			use baseline::Pallet as BaselineBench;

			impl frame_system_benchmarking::Config for Runtime {}
			impl baseline::Config for Runtime {}

			let whitelist: Vec<TrackedStorageKey> = vec![
				// Block Number
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
				// Total Issuance
				hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
				// Execution Phase
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
				// Event Count
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
				// System Events
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
			];

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);

			add_benchmark!(params, batches, frame_benchmarking, BaselineBench::<Runtime>);
			add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
			add_benchmark!(params, batches, pallet_balances, Balances);
			add_benchmark!(params, batches, pallet_timestamp, Timestamp);
			// add_benchmark!(params, batches, pallet_accounts, Accounts);
			add_benchmark!(params, batches, pallet_protos, Protos);
			add_benchmark!(params, batches, pallet_assets, Assets);
			add_benchmark!(params, batches, pallet_fragments, Fragments);
			add_benchmark!(params, batches, pallet_detach, Detach);
			add_benchmark!(params, batches, pallet_multisig, Multisig);
			add_benchmark!(params, batches, pallet_proxy, Proxy);
			add_benchmark!(params, batches, pallet_identity, Identity);
			add_benchmark!(params, batches, pallet_utility, Utility);

			Ok(batches)
		}
	}
}
