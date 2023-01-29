//! The Runtime of the Clamor Node.
//!
//! The runtime for a Substrate node contains all of the business logic
//! for executing transactions, saving state transitions, and interacting with the outer node.
//!
//! # Footnotes
//!
//! ## Weights
//!
//! In Substrate, **weight** is a **unit of measurement** to measure **computation time**.
//!
//! "Built into FRAME, it is hardcoded that 10**12 weight is equivalent to 1 second of computation time [on the reference hardware]" - https://www.youtube.com/watch?v=i3zW4wGexAc&t=138s
//!
//! Therefore, one unit of weight is one picosecond of computation time on the reference hardware. Source: https://docs.substrate.io/reference/glossary/#weight
//!
//! ### What does "Maximum Block Weight" mean?
//!
//! The **maximum block weight** is the **maximum amount of computation time** that a Node is **allowed to spend in constructing a block**.
//!
//! In Substrate, the **maximum block weight** should be equivalent to **one-third of the target block time** with an allocation of:
//! - One third for block construction
//! - One third for network propagation
//! - One third for import and verification
//!
//! Source: https://docs.substrate.io/reference/glossary/#weight
//!
//! ## Transaction lifecycle (i.e the sequence of events that happens when a transaction/extrinsic is sent to a Substrate Blockchain):
//!
//! 1. The transaction (i.e the payload) is sent to a **single Substrate Node** using an RPC
//! 2. The transaction enters the transaction pool of this Substrate Node
//! 2a. In the transaction pool, the payload is checked to be valid in `validate_transaction()`
//!
//! 3. The Substrate Node gossips this transaction to peer nodes using libp2p
//! 3a. The peer nodes also check if this transaction is valid by running `validate_transaction()`
//!
//! 4. One of the nodes creates a block which will include the transaction.
//! 5. This "block author node" will gossip the block to peer nodes using libp2p
//!
//! 6. The gossiped block will be imported by the peer nodes (and surprisingly by the "block author node" also).
//! 6a. All the transactions of the imported block will be executed. (This includes the transaction-in-question obviously)
//!
//! Source: https://www.youtube.com/watch?v=3pfM0GOp02c&ab_channel=ParityTech
//!
//! TODO Review - Notice that once the block is imported (i.e step 6a) - all its transactions/extrinsics get executed right-away (if we take what the presenter said in the YouTube video completely literally). It does not validate the transactions (i.e perform `validate_transaction` on them) in the block before executing them!
//!
//! # Transaction Format
//!
//! Please see the types `UncheckedExtrinsic` and `SignedExtra` below to know what the transaction format should be when submitting a transaction to a **Node of this Blockchain** via RPC

// Some of the Substrate Macros in this file throw missing_docs warnings.
// That's why we allow this file to have missing_docs.
#![allow(missing_docs)]
// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// This will include the generated WASM binary as two constants WASM_BINARY and WASM_BINARY_BLOATY. The former is a compact WASM binary and the latter is not compacted.
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
	transaction_validity::{
		InvalidTransaction, TransactionSource, TransactionValidity, TransactionValidityError,
	},
	ApplyExtrinsicResult, MultiSignature,
};
use sp_std::{prelude::*, str};
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
pub use pallet_fragments::Call as FragmentsCall;
pub use pallet_protos::Call as ProtosCall;
pub use pallet_timestamp::Call as TimestampCall;
use pallet_transaction_payment::CurrencyAdapter;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{Perbill, Permill};

use scale_info::prelude::string::String;

use codec::{Decode, Encode};
use sp_runtime::traits::{ConstU8, SaturatedConversion, StaticLookup};

use pallet_fragments::{GetDefinitionsParams, GetInstanceOwnerParams, GetInstancesParams};
use pallet_protos::{GetGenealogyParams, GetProtosParams};

pub use pallet_contracts::Schedule;
use pallet_oracle::OracleProvider;

// IMPORTS BELOW ARE USED IN the module `validation_logic`
use protos::categories::{
	AudioCategories,
	BinaryCategories,
	Categories,
	ModelCategories,
	ShardsFormat,
	// ShardsScriptInfo,
	// ShardsTrait,
	TextCategories,
	TextureCategories,
	VectorCategories,
	VideoCategories,
};
use protos::traits::Trait;
use sp_fragnova::Hash256;

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

	// Implement OpaqueKeys for a described struct.
	// Every field type must implement BoundToRuntimeAppPublic. KeyTypeIdProviders is set to the types given as fields.
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
	spec_version: 3,
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
/// We allow `Normal` extrinsics to fill up the block up to 75% (i.e up to 75% of the block length and block weight of a block can be filled up by Normal extrinsics).
/// The rest can be used by Operational extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// TODO Documentation - It is actually the "target block weight", not "maximum block weight".
/// The **maximum block weight** is the **maximum amount of computation time** (assuming no extrinsic class uses its `reserved` space - please see the type `RuntimeBlockWeights` below to understand what `reserved` is)
/// that is **allowed to be spent in constructing a block** by a Node.
///
/// Here, we set this to 2 seconds because we want a 6 second average block time. (since in Substrate, the **maximum block weight** should be equivalent to **one-third of the target block time** - see the crate documentation above for more information)
const MAXIMUM_BLOCK_WEIGHT: Weight = 2 * WEIGHT_PER_SECOND;

/// The maximum possible length (in bytes) that a Clamor Block can be
pub const MAXIMUM_BLOCK_LENGTH: u32 = 5 * 1024 * 1024;

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
	/// Version of the runtime.
	pub const Version: RuntimeVersion = VERSION;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	pub const BlockHashCount: BlockNumber = 2400;
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	pub const SS58Prefix: u8 = 93;

	/// `max_with_normal_ratio(max: u32, normal: Perbill)` creates a new `BlockLength` with `max` for `Operational` & `Mandatory` and `normal * max` for `Normal`.
	///
	/// Note: `BlockLength` is a struct that specifies the maximum total length (in bytes) each extrinsic class can have in a block.
	/// The maximum possible length that a block can be is the maximum of these maximums - i.e `MAX(BlockLength::max)`.
	///
	/// Source: https://paritytech.github.io/substrate/master/frame_system/limits/struct.BlockLength.html#
	pub RuntimeBlockLength: BlockLength = BlockLength
		::max_with_normal_ratio(MAXIMUM_BLOCK_LENGTH, NORMAL_DISPATCH_RATIO);

	/// Set the "target block weight" for the Clamor Blockchain.
	///
	/// # Footnotes
	///
	/// ## `BlockWeights::builder()`
	///
	/// `BlockWeights::builder()` is called to start constructing a new `BlockWeights` object. By default all kinds except of Mandatory extrinsics are disallowed.
	///
	/// ## `BlockWeights` struct
	///
	/// A `BlockWeights` struct contains the following information for each extrinsic class:
	/// 1. **Base weight of any extrinsic** of the **extrinsic class** (`WeightsPerClass::base_extrinsic`): The "overhead cost" (i.e overhead computation time) when running any extrinsic of the extrinsic class.
	/// 2. **Maximum possible weight** that can be **consumed by a single extrinsic** of the **extrinsic class**: `WeightsPerClass::max_extrinsic`.
	/// 3. **Maximum possible weight in a block** that can be **consumed by the extrinsic class** (to compute extrinsics): `WeightsPerClass::max_total`.
	///    - If this is `None`, an unlimited amount of weight can be consumed by the extrinsic class. This is generally highly recommended for `Mandatory` dispatch class, while it can be dangerous for `Normal` class and should only be done with extra care and consideration.
	///    - In the worst case, the total weight consumed by the extrinsic class is going to be: `MAX(max_total) + MAX(reserved)`. Source: https://paritytech.github.io/substrate/master/frame_system/limits/struct.WeightsPerClass.html#structfield.max_total
	/// 4. **Maximum additional amount of weight that can always be consumed by an extrinsic class**, **once a block's target block weight (`BlockWeights::max_block`)
	///	   has been reached** (`WeightsPerClass::reserved`): "Often it's desirable for some class of transactions to be added to the block despite it being full.
	///    For instance one might want to prevent high-priority `Normal` transactions from pushing out lower-priority `Operational` transactions.
	///    In such cases you might add a `reserved` capacity for given class.".
	///    Source: https://paritytech.github.io/substrate/master/frame_system/limits/struct.BlockWeights.html# (click the "src" button - since the diagram gets cut off on the webpage)
	///    - Note that the `max_total` limit applies to `reserved` space as well.
	///		 For example, in the diagram (https://paritytech.github.io/substrate/master/frame_system/limits/struct.BlockWeights.html# - click the "src" button - since the diagram gets cut off on the
	///		 webpage), "the sum of weights of `Ext7` & `Ext8` (which are `Operational` extrinsics) mustn't exceed the `max_total` (of the `Operational` extrinsic)".
	///      Please do see the diagram for complete understanding.
	///    - Setting `reserved` to `Some(x)`, guarantees that at least `x` weight can be consumed by the extrinsic class in every block.
	///    	  - Note: `x` can be zero obviously. It doesn't have to only be non-zero integers (obviously!).
	///    - Setting `reserved` to `None` guarantees that at least `max_total` weight can be consumed by the extrinsic class in every block.
	///    	  - If `max_total` is set to `None` as well, all extrinsics of the extrisnic class will always end up in the block (recommended for the `Mandatory` extrinsic class).
	/// Furthermore, a `BlockWeights` struct also contains information about the block:
	/// 1. **Maximum total amount of weight that can be consumed by all kinds of extrinsics in a block (assuming no extrinsic class uses its `reserved` space)** (`BlockWeights::max_block`)
	/// 2. **Base weight of a block (i.e the computation time to execute an empty block)** (`BlockWeights::base_block`).
	///
	///
	/// Note: Even though each extrinsic class has its own separate limit `max_total`,
	/// in an actual block the total weight consumed by each extrinsic class cannot not exceed `max_block` (except for `reserved`).
	///
	/// Note 2: As a consequence of `reserved` space, total consumed block weight might exceed `max_block` value,
	/// so this parameter (i.e `max_block`) should rather be thought of as "target block weight" than a hard limit.
	///
	/// Note 3: Each block starts with `BlockWeights::base_block` weight being consumed right away **as part of the Mandatory extrinsic class**. Source: https://paritytech.github.io/substrate/master/frame_system/limits/struct.BlockWeights.html#
	///
	/// Source: https://paritytech.github.io/substrate/master/frame_system/limits/struct.BlockWeights.html# (click the "src" button - since the diagram gets cut off on the webpage)
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
		// `BlockWeightsBuilder::base_block()` sets the base weight of the block (i.e `BlockWeights::base_block`) to
		// `frame_support::weights::constants::BlockExecutionWeight` which is defined as the "Time to execute an empty block" (https://paritytech.github.io/substrate/master/frame_support/weights/constants/struct.BlockExecutionWeight.html#).
		.base_block(BlockExecutionWeight::get())
		// Set the base weight (i.e `WeightsPerClass::base_extrinsic`) for each extrinsic class to `ExtrinsicBaseWeight::get()`.
		//
		// Note: `frame_support::weights::constants::ExtrinsicBaseWeight` is the "Time to execute a NO-OP extrinsic, for example `System::remark`" (https://paritytech.github.io/substrate/master/frame_support/weights/constants/struct.ExtrinsicBaseWeight.html#).
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		// Set the maximum block weight for the `Normal` extrinsic class to `NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT`.
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		// Set the maximum block weight and the reserved weight for the `Operational` extrinsic class to `MAXIMUM_BLOCK_WEIGHT` and `MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT` respectively.
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

///  **Maximum permitted depth level** that a **descendant call** of the **outermost call of an extrinsic** can be
pub const MAXIMUM_NESTED_CALL_DEPTH_LEVEL: u8 = 4;
/// Maximum length (in bytes) that the metadata data of a Proto-Fragment / Fragment Definition / Fragment Instance can be
pub const MAXIMUM_METADATA_DATA_LENGTH: usize = 1 * 1024 * 1024;

mod validation_logic {

	use super::*;

	/// Does the call `c` use `transaction_index::index`.
	fn does_call_index_the_transaction(c: &Call) -> bool {
		matches!(
			c,
			Call::Protos(pallet_protos::Call::upload { .. }) | // https://fragcolor-xyz.github.io/clamor/doc/pallet_protos/pallet/enum.Call.html#
		Call::Protos(pallet_protos::Call::patch { .. }) |
		Call::Protos(pallet_protos::Call::set_metadata { .. }) |
		Call::Fragments(pallet_fragments::Call::set_definition_metadata { .. }) | // https://fragcolor-xyz.github.io/clamor/doc/pallet_fragments/pallet/enum.Call.html#
		Call::Fragments(pallet_fragments::Call::set_instance_metadata { .. })
		)
	}

	fn is_valid(category: &Categories, data: &Vec<u8>, proto_references: &Vec<Hash256>) -> bool {
		match category {
			Categories::Text(sub_categories) => match sub_categories {
				TextCategories::Plain | TextCategories::Wgsl => str::from_utf8(data).is_ok(),
				// REVIEW - does a Json have to be a `serde_json::Map` or can it `serde_json::Value`?
				TextCategories::Json =>
					serde_json::from_slice::<serde_json::Map<String, serde_json::Value>>(&data[..])
						.is_ok(),
			},
			Categories::Trait(trait_hash) => match trait_hash {
				Some(_) => false,
				None => {
					let Ok(trait_struct) = Trait::decode(&mut &data[..]) else { // TODO Review - is `&mut *data` safe?
						return false;
					};

					if trait_struct.name.len() == 0 {
						return false
					}

					trait_struct.records.windows(2).all(|window| {
						let (record_1, record_2) = (&window[0], &window[1]);
						let (Ok(a1), Ok(a2)) = (get_utf8_string(&record_1.0), get_utf8_string(&record_2.0)) else { // `a1` is short for `attribute_1`, `a2` is short for `attribute_2`
							return false;
						};

						let (Some(first_char_a1), Some(first_char_a2)) = (a1.chars().next(), a2.chars().next()) else {
							return false;
						};

						(first_char_a1.is_alphabetic() && first_char_a2.is_alphabetic()) // ensure first character is an alphabet
							&&
							(a1 == a1.to_lowercase() && a2 == a2.to_lowercase()) // ensure lowercase
							&&
							a1 <= a2 // ensure sorted. Note: "Strings are ordered lexicographically by their byte values ... This is not necessarily the same as “alphabetical” order, which varies by language and locale". Source: https://doc.rust-lang.org/std/primitive.str.html#impl-Ord-for-str
							&&
							a1 != a2 // ensure no duplicates
					})
				},
			},
			Categories::Shards(shards_script_info_struct) => {
				let format = shards_script_info_struct.format;
				let requiring = &shards_script_info_struct.requiring;
				let implementing = &shards_script_info_struct.implementing;

				let all_required_trait_impls_found = requiring.iter().all(|shards_trait| {
					proto_references.iter().any(|proto| {
						if let Some(trait_impls) =
							pallet_protos::TraitImplsByShard::<Runtime>::get(proto)
						{
							trait_impls.contains(shards_trait)
						} else {
							false
						}
					})
				});

				let all_traits_implemented_in_this_shards =
					implementing.iter().all(|_shards_trait| match format {
						ShardsFormat::Edn => false,
						ShardsFormat::Binary => false,
					});

				all_required_trait_impls_found && all_traits_implemented_in_this_shards
			},
			Categories::Audio(sub_categories) => match sub_categories {
				AudioCategories::OggFile => infer::is(data, "ogg"), // TODO Review - We are not checking for other OGG file extensions https://en.wikipedia.org/wiki/Ogg
				AudioCategories::Mp3File => infer::is(data, "mp3"),
			},
			Categories::Texture(sub_categories) => match sub_categories {
				TextureCategories::PngFile => infer::is(data, "png"), // png_decoder::decode(&data[..]).is_ok(),
				TextureCategories::JpgFile => infer::is(data, "jpg"), // TODO Review - we do not include ".jpeg" images, only ".jpg" images
			},
			Categories::Vector(sub_categories) => match sub_categories {
				VectorCategories::SvgFile => false,
				VectorCategories::TtfFile => infer::is(data, "ttf"), // ttf_parser::Face::parse(&data[..], 0).is_ok(),
			},
			Categories::Video(sub_categories) => match sub_categories {
				VideoCategories::MkvFile => infer::is(data, "mkv"),
				VideoCategories::Mp4File => infer::is(data, "mp4"),
			},
			Categories::Model(sub_categories) => match sub_categories {
				ModelCategories::GltfFile => false,
				ModelCategories::Sdf => false,
				ModelCategories::PhysicsCollider => false, // Note: "This is a Fragnova/Fragcolor data type"
			},
			Categories::Binary(sub_categories) => match sub_categories {
				BinaryCategories::WasmProgram => infer::is(data, "wasm"), // wasmparser_nostd::Parser::new(0).parse_all(data).all(|payload| payload.is_ok()), // REVIEW - shouldn't I check if the last `payload` is `Payload::End`?
				BinaryCategories::WasmReactor => infer::is(data, "wasm"),
				BinaryCategories::BlendFile => false,
				BinaryCategories::OnnxModel => false,
				BinaryCategories::SafeTensors => false,
			},
		}
	}

	/// Is the call `c` valid?
	///
	/// Note: This function does not check whether the child/descendant calls of `c` (if it has any) are valid.
	pub fn is_the_immediate_call_valid(c: &Call) -> bool {
		match c {
			Call::Protos(ProtosCall::upload{ref data, ref category, ref references, ..}) => {
				// `Categories::Shards`, `Categories::Traits` and `Categories::Text`
				// must have `data` that is of the enum variant type `ProtoData::Local`
				match category {
					Categories::Shards(_) | Categories::Trait(_) | Categories::Text(_) => match data {
						pallet_protos::ProtoData::Local(_) => (),
						_ => return false,
					},
					_ => (),
				};
				match data {
					pallet_protos::ProtoData::Local(ref data) => is_valid(category, data, references),
					_ => true,
				}
			},
			Call::Protos(ProtosCall::patch{ref proto_hash, ref data, ref new_references, ..}) => {
				let Some(proto_struct) = pallet_protos::Protos::<Runtime>::get(proto_hash) else {
					return false;
				};
				match data {
					None => true,
					Some(pallet_protos::ProtoData::Local(ref data)) => is_valid(&proto_struct.category, data, new_references),
					_ => true,
				}
			},
			Call::Protos(ProtosCall::set_metadata{ref data, ref metadata_key, ..}) |
			Call::Fragments(FragmentsCall::set_definition_metadata{ref data, ref metadata_key, ..}) |
			Call::Fragments(FragmentsCall::set_instance_metadata{ref data, ref metadata_key, ..}) => {
				if data.len() > MAXIMUM_METADATA_DATA_LENGTH {
					return false;
				}
				match &metadata_key[..] {
					b"title" => is_valid(&Categories::Text(TextCategories::Plain), data, &vec![]),
					b"json_description" => is_valid(&Categories::Text(TextCategories::Json), data, &vec![]),
					b"image" => is_valid(&Categories::Texture(TextureCategories::PngFile), data, &vec![]) || is_valid(&Categories::Texture(TextureCategories::JpgFile), data, &vec![]),
					_ => false,
				}
			},
			// Prevent batch calls from containing any call that uses `transaction_index::index`. The reason we do this is because "any e̶x̶t̶r̶i̶n̶s̶i̶c̶ call using `transaction_index::index` will not work properly if used within a `pallet_utility` batch call as it depends on extrinsic index and during a batch there is only one index." (https://github.com/paritytech/substrate/issues/12835)
			Call::Utility(pallet_utility::Call::batch { calls }) | // https://paritytech.github.io/substrate/master/pallet_utility/pallet/enum.Call.html
			Call::Utility(pallet_utility::Call::batch_all { calls }) |
			Call::Utility(pallet_utility::Call::force_batch { calls }) => {
				calls.iter().all(|call| !does_call_index_the_transaction(call))
			}
			_ => true,
		}
	}

	// We have decided not to restrict an Extrinsic's Call Depth Level, so therefore these functions below have been commented out (since they will not be used):
	// /// Returns true if the **call `c` is a Nesting Call whose Depth Level is greater than `MAXIMUM_NESTED_CALL_DEPTH_LEVEL`**
	// /// or if the **call `c` or one of `c`'s descendants is invalid**.
	// pub fn does_extrinsic_contain_invalid_call_or_exceed_maximum_depth_level(xt: &<Block as BlockT>::Extrinsic) -> bool {
	//
	// 	let c = &xt.function;
	//
	// 	let mut stack  = Vec::<(&Call, u8)>::new();
	// 	stack.push((c, 0));
	//
	// 	while let Some((call, depth_level)) = stack.pop() {
	// 		if !is_the_immediate_call_valid(call) {
	// 			return true;
	// 		}
	// 		let child_calls = get_child_calls(call);
	// 		if child_calls.len() > 0 && depth_level + 1 > MAXIMUM_NESTED_CALL_DEPTH_LEVEL {
	// 			return true;
	// 		}
	// 		child_calls.iter().for_each(|call| stack.push((call, depth_level + 1)));
	// 	}
	//
	// 	false
	// }
	//
	// /// Return the list of `Call` that will be directly called by the call `c`, if any.
	// fn get_child_calls(c: &Call) -> &[Call] {
	// 	match c {
	// 		// TODO - Add `multisig.***` and `utility.***`
	// 		Call::Utility(pallet_utility::Call::batch { calls }) | // https://paritytech.github.io/substrate/master/pallet_utility/pallet/enum.Call.html#
	// 		Call::Utility(pallet_utility::Call::batch_all { calls }) |
	// 		Call::Utility(pallet_utility::Call::force_batch { calls }) => calls,
	// 		Call::Utility(pallet_utility::Call::as_derivative { call, .. }) | // https://paritytech.github.io/substrate/master/pallet_utility/pallet/enum.Call.html#
	// 		Call::Utility(pallet_utility::Call::dispatch_as { call, .. }) |
	// 		Call::Utility(pallet_utility::Call::with_weight { call, .. }) |
	// 		Call::Proxy(pallet_proxy::Call::proxy { call, .. }) | // https://paritytech.github.io/substrate/master/pallet_proxy/pallet/enum.Call.html#
	// 		Call::Proxy(pallet_proxy::Call::proxy_announced { call, .. }) |
	// 		Call::Multisig(pallet_multisig::Call::as_multi_threshold_1 { call, .. }) | // https://paritytech.github.io/substrate/master/pallet_multisig/pallet/enum.Call.html#
	// 		Call::Multisig(pallet_multisig::Call::as_multi { call, .. }) => sp_std::slice::from_ref(call), // `sp_std::slice::from_ref()` converts a reference to T into a slice of length 1 (without copying). Source: https://paritytech.github.io/substrate/master/sp_std/slice/fn.from_ref.html#
	// 		_ => &[]
	// 	}
	// }

	#[cfg(test)]
	mod tests {
		use super::*;

		#[test]
		fn is_the_immediate_call_valid_should_not_work_if_proto_category_is_invalid() {
			for (category, (valid_data, invalid_data)) in [
				(
					Categories::Text(TextCategories::Plain),
					(b"I am valid UTF-8 text!".to_vec(), vec![0xF0, 0x9F, 0x98]),
				),
				(
					Categories::Text(TextCategories::Json),
					(b"{\"key\": \"value\"}".to_vec(), b"I am not JSON text!".to_vec()),
				),
				(
					Categories::Texture(TextureCategories::PngFile),
					(vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A], vec![7u8; 10]),
				),
				(
					Categories::Texture(TextureCategories::JpgFile),
					(vec![0xFF, 0xD8, 0xFF, 0xE0], vec![7u8; 10]),
				),
			] {
				assert_eq!(
					is_the_immediate_call_valid(&Call::Protos(pallet_protos::Call::upload {
						// https://fragcolor-xyz.github.io/clamor/doc/pallet_protos/pallet/enum.Call.html#
						references: vec![],
						category: category.clone(),
						tags: vec![].try_into().unwrap(),
						linked_asset: None,
						license: pallet_protos::UsageLicense::Closed,
						cluster: None,
						data: pallet_protos::ProtoData::Local(valid_data)
					})),
					true
				);
				assert_eq!(
					is_the_immediate_call_valid(&Call::Protos(pallet_protos::Call::upload {
						references: vec![],
						category: category.clone(),
						tags: vec![].try_into().unwrap(),
						linked_asset: None,
						license: pallet_protos::UsageLicense::Closed,
						cluster: None,
						data: pallet_protos::ProtoData::Local(invalid_data)
					}),),
					false
				);
			}
		}

		#[test]
		fn is_the_immediate_call_valid_should_not_work_if_metadata_key_is_invalid() {
			for (metadata_key, data) in [
				(b"title".to_vec(), b"I am valid UTF-8 text!".to_vec()),
				(b"json_description".to_vec(), b"{\"key\": \"value\"}".to_vec()),
				(b"image".to_vec(), vec![0xFF, 0xD8, 0xFF, 0xE0]),
			] {
				assert_eq!(
					is_the_immediate_call_valid(&Call::Protos(pallet_protos::Call::set_metadata {
						// https://fragcolor-xyz.github.io/clamor/doc/pallet_protos/pallet/enum.Call.html#
						proto_hash: [7u8; 32],
						metadata_key: metadata_key.clone().try_into().unwrap(),
						data: data.clone()
					})),
					true
				);
				assert_eq!(
					is_the_immediate_call_valid(&Call::Protos(pallet_protos::Call::set_metadata {
						proto_hash: [7u8; 32],
						metadata_key: b"invalid_key".to_vec().try_into().unwrap(),
						data: data.clone()
					})),
					false
				);

				assert_eq!(
					is_the_immediate_call_valid(&Call::Fragments(
						pallet_fragments::Call::set_definition_metadata {
							// https://fragcolor-xyz.github.io/clamor/doc/pallet_fragments/pallet/enum.Call.html#
							definition_hash: [7u8; 16],
							metadata_key: metadata_key.clone().try_into().unwrap(),
							data: data.clone()
						}
					)),
					true
				);
				assert_eq!(
					is_the_immediate_call_valid(&Call::Fragments(
						pallet_fragments::Call::set_definition_metadata {
							definition_hash: [7u8; 16],
							metadata_key: b"invalid_key".to_vec().try_into().unwrap(),
							data: data.clone()
						}
					)),
					false
				);

				assert_eq!(
					is_the_immediate_call_valid(&Call::Fragments(
						pallet_fragments::Call::set_instance_metadata {
							definition_hash: [7u8; 16],
							edition_id: 1,
							copy_id: 1,
							metadata_key: metadata_key.clone().try_into().unwrap(),
							data: data.clone()
						}
					)),
					true
				);
				assert_eq!(
					is_the_immediate_call_valid(&Call::Fragments(
						pallet_fragments::Call::set_instance_metadata {
							definition_hash: [7u8; 16],
							edition_id: 1,
							copy_id: 1,
							metadata_key: b"invalid_key".to_vec().try_into().unwrap(),
							data: data.clone()
						}
					)),
					false
				);
			}
		}

		#[test]
		fn is_the_immediate_call_valid_should_not_work_if_metadata_data_is_invalid() {
			for (metadata_key, data) in [
				(b"title".to_vec(), b"I am valid UTF-8 text!".to_vec()),
				(b"json_description".to_vec(), b"{\"key\": \"value\"}".to_vec()),
				(b"image".to_vec(), vec![0xFF, 0xD8, 0xFF, 0xE0]),
			] {
				assert_eq!(
					is_the_immediate_call_valid(&Call::Protos(pallet_protos::Call::set_metadata {
						// https://fragcolor-xyz.github.io/clamor/doc/pallet_protos/pallet/enum.Call.html#
						proto_hash: [7u8; 32],
						metadata_key: metadata_key.clone().try_into().unwrap(),
						data: data.clone()
					})),
					true
				);
				assert_eq!(
					is_the_immediate_call_valid(&Call::Protos(pallet_protos::Call::set_metadata {
						proto_hash: [7u8; 32],
						metadata_key: metadata_key.clone().try_into().unwrap(),
						data: vec![0xF0, 0x9F, 0x98] // Invalid UTF-8 Text
					})),
					false
				);

				assert_eq!(
					is_the_immediate_call_valid(&Call::Fragments(
						pallet_fragments::Call::set_definition_metadata {
							// https://fragcolor-xyz.github.io/clamor/doc/pallet_fragments/pallet/enum.Call.html#
							definition_hash: [7u8; 16],
							metadata_key: metadata_key.clone().try_into().unwrap(),
							data: data.clone() // Invalid UTF-8 Text
						}
					)),
					true
				);
				assert_eq!(
					is_the_immediate_call_valid(&Call::Fragments(
						pallet_fragments::Call::set_definition_metadata {
							definition_hash: [7u8; 16],
							metadata_key: metadata_key.clone().try_into().unwrap(),
							data: vec![0xF0, 0x9F, 0x98] // Invalid UTF-8 Text
						}
					)),
					false
				);

				assert_eq!(
					is_the_immediate_call_valid(&Call::Fragments(
						pallet_fragments::Call::set_instance_metadata {
							definition_hash: [7u8; 16],
							edition_id: 1,
							copy_id: 1,
							metadata_key: metadata_key.clone().try_into().unwrap(),
							data: data.clone()
						}
					)),
					true
				);
				assert_eq!(
					is_the_immediate_call_valid(&Call::Fragments(
						pallet_fragments::Call::set_instance_metadata {
							definition_hash: [7u8; 16],
							edition_id: 1,
							copy_id: 1,
							metadata_key: metadata_key.try_into().unwrap(),
							data: vec![0xF0, 0x9F, 0x98] // Invalid UTF-8 Text
						}
					)),
					false
				);
			}
		}

		#[test]
		fn is_the_immediate_call_valid_should_not_work_if_a_batch_call_contains_a_call_that_indexes_the_transaction(
		) {
			assert_eq!(
				is_the_immediate_call_valid(&Call::Utility(pallet_utility::Call::batch {
					calls: vec![Call::Protos(pallet_protos::Call::ban {
						// https://fragcolor-xyz.github.io/clamor/doc/pallet_protos/pallet/enum.Call.html#
						proto_hash: [7u8; 32],
					})]
				}),),
				true
			);

			assert_eq!(
				is_the_immediate_call_valid(&Call::Utility(pallet_utility::Call::batch {
					calls: vec![Call::Protos(pallet_protos::Call::upload {
						references: vec![],
						category: Categories::Text(TextCategories::Plain),
						tags: vec![].try_into().unwrap(),
						linked_asset: None,
						license: pallet_protos::UsageLicense::Closed,
						cluster: None,
						data: pallet_protos::ProtoData::Local(b"Bonjour".to_vec())
					})]
				}),),
				false
			);
		}
	}
}

// Configure FRAME pallets to include in runtime.
pub struct BaseCallFilter;
impl Contains<Call> for BaseCallFilter {
	fn contains(c: &Call) -> bool {
		// log::info!("The call {:?} is {}", c, validation_logic::is_the_immediate_call_valid(c));
		validation_logic::is_the_immediate_call_valid(c)
	}
}
impl frame_system::Config for Runtime {
	/// The basic call filter to use in dispatchable.
	type BaseCallFilter = BaseCallFilter;
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
	/// TODO: Documentation
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

// Parameters related to calculating the Weight fee.
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
	pub const DeletionWeightLimit: Weight = 500_000_000_000;
	// pub const MaxCodeSize: u32 = 2 * 1024;
	/// Cost schedule and limits.
	pub MySchedule: Schedule<Runtime> = <Schedule<Runtime>>::default();
	/// A fee mulitplier for `Operational` extrinsics to compute "virtual tip" to boost their
	/// `priority`
	pub OperationalFeeMultiplier: u8 = 5;
	/// Weight for adding a a byte worth of storage in certain extrinsics such as `upload()`.
	pub StorageBytesMultiplier: u64 = 10;
}

/// This pallet provides the basic logic needed to pay the absolute minimum amount needed for a
/// transaction to be included. This includes:
///   - _base fee_: This is the minimum amount a user pays for a transaction. It is declared
/// 	as a base _weight_ in the runtime and converted to a fee using `WeightToFee`.
///   - _weight fee_: A fee proportional to amount of weight a transaction consumes.
///   - _length fee_: A fee proportional to the encoded length of the transaction.
///   - _tip_: An optional tip. Tip increases the priority of the transaction, giving it a higher
///     chance to be included by the transaction queue.
///
/// The base fee and adjusted weight and length fees constitute the _inclusion fee_, which is
/// the minimum fee for a transaction to be included in a block.
///
/// The formula of final fee:
///   ```ignore
///   inclusion_fee = base_fee + length_fee + [targeted_fee_adjustment * weight_fee];
///   final_fee = inclusion_fee + tip;
///   ```
impl pallet_transaction_payment::Config for Runtime {
	type Event = Event;
	type OnChargeTransaction = CurrencyAdapter<Balances, ()>;
	type OperationalFeeMultiplier = OperationalFeeMultiplier;
	/// Convert a weight value into a deductible fee based on the currency type.
	///
	/// Footnote: The Struct `IdentityFee<T>` is an Implementor of trait `WeightToFee` that maps one unit of weight to one unit of fee. Source: https://paritytech.github.io/substrate/master/frame_support/weights/struct.IdentityFee.html
	type WeightToFee = IdentityFee<Balance>;
	/// Convert a length value into a deductible fee based on the currency type.
	///
	/// Footnote: The Struct `IdentityFee<T>` is an Implementor of trait `WeightToFee` that maps one unit of weight to one unit of fee. Source: https://paritytech.github.io/substrate/master/frame_support/weights/struct.IdentityFee.html
	type LengthToFee = IdentityFee<Balance>;
	/// Update the multiplier of the next block, based on the previous block's weight.
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
		vec![String::from("0x8a819F380ff18240B5c11010285dF63419bdb2d5")]
	}
}

impl pallet_accounts::Config for Runtime {
	type Event = Event;
	type WeightInfo = ();
	type EthChainId = ConstU64<5>; // goerli
	type EthFragContract = Runtime;
	type EthConfirmations = ConstU64<1>;
	type Threshold = ConstU64<1>;
	type AuthorityId = pallet_accounts::crypto::FragAuthId;
	type InitialPercentageNova = ConstU8<20>;
	type USDEquivalentAmount = ConstU128<100>;
}

impl pallet_oracle::OracleContract for Runtime {
	fn get_provider() -> pallet_oracle::OracleProvider {
		/* https://docs.uniswap.org/contracts/v3/reference/deployments
		 The contract of the Quoter smart contract on Ethereum mainnet that provides quotes for swaps.
		 It allows getting the expected amount out or amount in for a given swap by optimistically executing the swap
		 and checking the amounts in the callback.
		*/
		OracleProvider::Uniswap("0xb27308f9F90D607463bb33eA1BeBb41C27CE5AB6".encode())

		/*
		OracleProvider::Chainlink("0x547a514d5e3769680Ce22B2361c10Ea13619e8a9".encode())
		// https://docs.chain.link/docs/data-feeds/price-feeds/addresses/
		"0x547a514d5e3769680Ce22B2361c10Ea13619e8a9" // the address of the price feed contract of AAVE/USD on Ethereum mainnet.
		TODO to change when FRAG pool will be known
		*/
	}
}

impl pallet_oracle::Config for Runtime {
	type AuthorityId = pallet_oracle::crypto::FragAuthId;
	type Event = Event;
	type OracleProvider = Runtime; // the contract address determines the network to connect (mainnet, goerli, etc.)
	type Threshold = ConstU64<1>;
}

impl pallet_protos::Config for Runtime {
	type Event = Event;
	type WeightInfo = ();
	type StringLimit = StringLimit;
	type DetachAccountLimit = ConstU32<20>; // An ethereum public account address has a length of 20.
	type MaxTags = ConstU32<10>;
	type StorageBytesMultiplier = StorageBytesMultiplier;
}

impl pallet_detach::Config for Runtime {
	type Event = Event;
	type WeightInfo = ();
	type AuthorityId = pallet_detach::crypto::DetachAuthId;
}

impl pallet_clusters::Config for Runtime {
	type Event = Event;
	type NameLimit = ConstU32<20>;
	type DataLimit = ConstU32<300>;
	type MembersLimit = ConstU32<20>;
	type RoleSettingsLimit = ConstU32<20>;
}

parameter_types! {
	pub RootNamespace: Vec<u8> = b"Frag".to_vec();
}

impl pallet_aliases::Config for Runtime {
	type Event = Event;
	type NamespacePrice = ConstU128<100>;
	type NameLimit = ConstU32<20>;
	type RootNamespace = RootNamespace;
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
/// to implement the `CreateSignedTransaction` trait, you also need to implement that trait for the runtime.
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
//
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
// V IMP NOTE 3: The order that the pallets appear in this macro determines its pallet index. Thus, new pallets should be added at the end to avoid breaking changes.
construct_runtime!(
	pub enum Runtime where
		Block = Block, //  Block is the block type that is used in the runtime
		NodeBlock = opaque::Block, // NodeBlock is the block type that is used in the node
		UncheckedExtrinsic = UncheckedExtrinsic // UncheckedExtrinsic is the format in which the "outside world" must send an extrinsic to the node
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
		Clusters: pallet_clusters,
		Oracle: pallet_oracle,
		Aliases: pallet_aliases,
	}
);

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, AccountIndex>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// The SignedExtension to the basic transaction logic.
///
/// ## What is a Signed Extension?
///
/// Substrate provides the concept of **signed extensions** to extend an extrinsic with additional data, provided by the `SignedExtension` trait.
///
/// The transaction queue regularly calls signed extensions to keep checking that a transaction is valid before it gets put in the ready queue.
/// This is a useful safeguard for verifying that transactions won't fail in a block.
/// They are commonly used to enforce validation logic to protect the transaction pool from spam and replay attacks.
///
/// Source: https://docs.substrate.io/reference/transaction-format/
///
/// # Footnote
///
/// 1. Each element in the tuple implements the trait `SignedExtension`.
/// 2. This tuple implements the trait `SignedExtension`. See: https://paritytech.github.io/substrate/master/sp_runtime/traits/trait.SignedExtension.html#impl-SignedExtension-for-(TupleElement0%2C%20TupleElement1%2C%20TupleElement2%2C%20TupleElement3%2C%20TupleElement4%2C%20TupleElement5%2C%20TupleElement6)
///
/// # Example
///
/// Notice that in any signed transaction/extrinsic that is sent to Clamor, it will the extra data🥕 "era", "nonce" and "tip": https://polkadot.js.org/apps/#/extrinsics/decode/0xf5018400d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d0150cc530a8f70343680c46687ade61e1e4cdc0dfc6d916c3143828dc588938c1934030b530d9117001260426798d380306ea3a9d04fe7b525a33053a1c31bee86750200000b000000000000003448656c6c6f2c20576f726c6421
///
/// The reason we see this additional data in the encoded extrinsic is because we have defined them here.
///
pub type SignedExtra = (
	// Since the `SignedExtension::AdditionalSigned` of this `SignedExtension` object is `u32` (and not `()`),
	// the signing payload (i.e the payload that will be signed to generate the signature🥖) must contain the runtime version number.
	frame_system::CheckSpecVersion<Runtime>,
	// Since the `SignedExtension::AdditionalSigned` of this `SignedExtension` object is `u32` (and not `()`),
	// the signing payload must contain the transaction version number.
	frame_system::CheckTxVersion<Runtime>,
	// Since the `SignedExtension::AdditionalSigned` of this `SignedExtension` object is `Runtime::Hash` (and not `()`),
	// the signing payload must contain the Genesis Block's Block Hash.
	frame_system::CheckGenesis<Runtime>,
	// Since the `SignedExtension::AdditionalSigned` of this `SignedExtension` object is `Runtime::Hash` (and not `()`),
	// the signing payload must contain <TODO>.
	//
	// Furthermore - since this `SignedExtension` object is a tuple struct that has a "non-`PhontomData` element" (i.e the enum `Era`),
	// any encoded extrinsic sent to this blockchain must have the enum `Era` as part of its extra data🥕
	frame_system::CheckEra<Runtime>,
	// Since this `SignedExtension` object is a tuple struct that has a "non-`PhontomData` element" (i.e the `#[codec(compact)] pub Runtime::Index`),
	// any encoded extrinsic sent to this blockchain must have the sender account's nonce (encoded as a `Comapct<Runtime::Index>`) as part of its extra data🥕
	frame_system::CheckNonce<Runtime>,
	// Since the `SigningExtension` object is a tuple struct with only `PhantomData`-typed element(s) AND since its `SignedExtension::AdditionalSigned` is `()`,
	// this `SignedExtension` object will not have any impact on the signing payload and thus the signature🥖.
	frame_system::CheckWeight<Runtime>,
	// Since this `SignedExtension` object is a tuple struct that has a "non-`PhontomData` element" of type `#[codec(compact)] pub BalanceOf<Runtime>`,
	// any encoded extrinsic sent to this blockchain must have a tip (encoded as a `Compact<BalanceOf<Runtime>>`) as part of its extra data🥕
	//
	// Footnote: `type BalanceOf<T> = <<T as Config>::OnChargeTransaction as OnChargeTransaction<T>>::Balance`
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// The **type** (i.e "format") that an **unchecked extrinsic** must have.
///
/// # Footnote:
///
/// ## Definition of "Unchecked Extrinsic"
///
/// An Unchecked Extrinsic is a **signed transaction** that requires some validation check before they can be accepted in the transaction pool.
/// Any unchecked extrinsic contains the signature for the data being sent plus some extra data.
///
/// Source: https://docs.substrate.io/reference/transaction-format/
///
/// ## How (signed) transactions are constructed
///
/// Substrate defines its transaction formats generically to allow developers to implement custom ways to define valid transactions.
/// In a runtime built with FRAME however (assuming transaction version 4), a transaction must be constructed by submitting the following encoded data:
///
/// `<signing account ID> + <signature>🥖 + <additional data>🥕🥦`
///
/// When submitting a signed transaction, the signature🥖is constructed by signing:
/// - The actual call, composed of:
///   - The index of the pallet.
///   - The index of the function call in the pallet.
///   - The parameters required by the function call being targeted.
/// - Some extra information🥕, verified by the signed extensions of the transaction:
///   - What's the era for this transaction, i.e. how long should this call last in the transaction pool before it gets discarded?
///	  - The nonce, i.e. how many prior transactions have occurred from this account? This helps protect against replay attacks or accidental double-submissions.
///   - The tip amount paid to the block producer to help incentive it to include this transaction in the block.
///
/// Then, some additional data that's not part of what gets s̶i̶g̶n̶e̶d̶ sent (**Note from Karan: I think they meant to say "sent" here, not "signed"**),  which includes:
/// - The spec version and the transaction version. This ensures the transaction is being submitted to a compatible runtime.
/// - The genesis hash. This ensures that the transaction is valid for the correct chain.
/// - The block hash. This corresponds to the hash of the checkpoint block, which enables the signature to verify that the transaction doesn't execute on the wrong fork, by checking against the block number provided by the era information.
///
/// **The SCALE encoded data is then signed (i.e. (`call`, `extra`, `additional`)) and the signature🥖, extra data🥕 and call data🥦
/// is attached in the correct order and SCALE encoded, ready to send off to a node that will verify the signed payload.**
/// If the payload to be signed is longer than 256 bytes, it is hashed just prior to being signed,
/// to ensure that the size of the signed data does not grow beyond a certain size.
///
/// This process can be broken down into the following steps:
///
/// 1. Construct the unsigned payload.
/// 2. Create a signing payload.
/// 3. Sign the payload.
/// 4. Serialize the signed payload.
/// 5. Submit the serialized transaction.
///
/// An extrinsic is encoded into the following sequence of bytes just prior to being hex encoded:
//
/// `[ 1 ] + [ 2 ] + [ 3 ] + [ 4 ]`
///
/// where:
///
/// `[1]` contains the compact encoded length in bytes of all of the following data. (Learn how compact encoding works using SCALE here: https://docs.substrate.io/reference/scale-codec/)
/// `[2]` is a `u8` containing 1 byte to indicate whether the transaction is signed or unsigned (1 bit), and the encoded transaction version ID (7 bits).
/// `[3]`️ if a signature🥖️ is present, this field contains an account ID, an SR25519 signature🥖 and some extra data🥕. If unsigned this field contains 0 bytes.
/// `[4]` is the encoded call data🥦. This comprises of 1 byte denoting the pallet to call into, 1 byte denoting the call to make in that pallet, and then as many bytes as needed to encode the arguments expected by that call.
///
/// Source: https://docs.substrate.io/reference/transaction-format/
///
/// # Example
///
/// For example - notice that the encoded transaction for `protos.upload()` has the order of first "signer", "signature"🥖, extra data🥕 (i.e "era", "nonce" and "tip") and finally the encoded call data🥦 (i.e "callIndex" and the arguments of `protos.upload()`): https://polkadot.js.org/apps/#/extrinsics/decode/0xf5018400d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d0150cc530a8f70343680c46687ade61e1e4cdc0dfc6d916c3143828dc588938c1934030b530d9117001260426798d380306ea3a9d04fe7b525a33053a1c31bee86750200000b000000000000003448656c6c6f2c20576f726c6421
///
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// The payload being signed in transactions.
///
/// Note: This type is only needed if you want to enable an off-chain worker for the runtime,
/// since it is only used when implementing the trait `frame_system::offchain::CreateSignedTransaction` for `Runtime`.
pub type SignedPayload = generic::SignedPayload<Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
>;

#[cfg(feature = "std")]
fn get_utf8_string(string: &str) -> Result<&str, &str> {
	Ok(string)
}
#[cfg(not(feature = "std"))]
fn get_utf8_string(string: &Vec<u8>) -> Result<&str, &str> {
	str::from_utf8(&string[..]).map_err(|_| "Lo siento")
}

// Marks the given trait implementations as runtime apis.
//
// For more information, read: https://paritytech.github.io/substrate/master/sp_api/macro.impl_runtime_apis.html
impl_runtime_apis! {

	/// The `Core` runtime api that every Substrate runtime needs to implement.
	///
	/// See: https://paritytech.github.io/substrate/master/sp_api/trait.Core.html
	impl sp_api::Core<Block> for Runtime {
		/// Returns the version of the runtime.
		fn version() -> RuntimeVersion {
			VERSION
		}

		/// Execute the given block.
		fn execute_block(block: Block) {
			Executive::execute_block(block);
		}

		/// Initialize a block with the given header.
		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	/// The `Metadata` runtime api that returns the metadata of a runtime.
	///
	/// See: https://paritytech.github.io/substrate/master/sp_api/trait.Metadata.html
	impl sp_api::Metadata<Block> for Runtime {
		/// Returns the metadata of a runtime.
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}
	}

	/// The `BlockBuilder` runtime api trait that provides the required functionality for building a block.
	///
	/// See: https://paritytech.github.io/substrate/master/sp_block_builder/trait.BlockBuilder.html#
	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		/// Apply the given extrinsic.
		///
		/// Returns an inclusion outcome which specifies if this extrinsic is included in
		/// this block or not.
		///
		/// # Footnote from Karan:
		///
		/// `ApplyExtrinsicResult` is defined as `Result<DispatchOutcome, transaction_validity::TransactionValidityError>` (https://paritytech.github.io/substrate/master/sp_runtime/type.ApplyExtrinsicResult.html#),
		/// where `DispatchOutcome` is defined as `Result<(), DispatchError>` (https://paritytech.github.io/substrate/master/sp_runtime/type.DispatchOutcome.html#).
		///
		/// Here the error `DispatchError` refers to types of errors (represented as a enum) thrown **while executing the extrinsic**,
		/// while the error `transaction_validity::TransactionValidityError` refers to the types of errors (represented as a enum)
		/// that are thrown **when the extrinsic is being verified (which obviously always happens before the extrinsic is executed)**
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		/// Finish the current block.
		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		/// Generate inherent extrinsics. The inherent data will vary from chain to chain.
		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		/// Check that the inherents are valid. The inherent data will vary from chain to chain.
		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	/// The `TaggedTransactionQueue` runtime api trait for interfering with the transaction queue.
	///
	/// See: https://paritytech.github.io/substrate/master/sp_transaction_pool/runtime_api/trait.TaggedTransactionQueue.html#
	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		/// Validate the transaction.
		///
		/// This method is invoked by the transaction pool to learn details about given transaction.
		/// The implementation should make sure to verify the correctness of the transaction
		/// against current state. The given `block_hash` corresponds to the hash of the block
		/// that is used as current state.
		///
		/// Note that this call may be performed by the pool multiple times and transactions
		/// might be verified in any possible order.
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			// We want to prevent nodes from gossiping extrinsics that have invalid calls.
			// log::info!("The call to gossip {:?} is {}", &tx.function, validation_logic::is_the_immediate_call_valid(&tx.function));
			if !validation_logic::is_the_immediate_call_valid(&tx.function) {
				return Err(TransactionValidityError::Invalid(InvalidTransaction::Call)); // TODO Review - Maybe change `InvalidTransaction::Call` to `InvalidTransaction::Custom(u8)`
			}
			// Always run normally anyways
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	/// The Offchain Worker Runtime API
	///
	/// See: https://paritytech.github.io/substrate/master/sp_offchain/trait.OffchainWorkerApi.html#
	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		/// Starts the off-chain task for given block header.
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	/// Runtime API necessary for block authorship with aura.
	///
	/// See: https://paritytech.github.io/substrate/master/sp_consensus_aura/trait.AuraApi.html#
	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		/// Returns the slot duration for Aura.
		///
		/// Currently, only the value provided by this type at genesis will be used.
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		/// Return the current set of authorities.
		fn authorities() -> Vec<AuraId> {
			Aura::authorities().into_inner()
		}
	}

	/// Session keys runtime api.
	///
	/// See: https://paritytech.github.io/substrate/master/sp_session/trait.SessionKeys.html#
	impl sp_session::SessionKeys<Block> for Runtime {
		/// Generate a set of session keys with optionally using the given seed.
		/// The keys should be stored within the keystore exposed via runtime
		/// externalities.
		///
		/// The seed needs to be a valid `utf8` string.
		///
		/// Returns the concatenated SCALE encoded public keys.
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			opaque::SessionKeys::generate(seed)
		}

		/// Decode the given public session keys.
		///
		/// Returns the list of public raw public keys + key type.
		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	/// Runtime APIs for integrating the GRANDPA finality gadget into runtimes.
	/// This should be implemented on the runtime side.
	///
	/// This is primarily used for negotiating authority-set changes for the
	/// gadget. GRANDPA uses a signaling model of changing authority sets:
	/// changes should be signaled with a delay of N blocks, and then automatically
	/// applied in the runtime after those N blocks have passed.
	///
	/// The consensus protocol will coordinate the handoff externally.
	///
	/// See: https://paritytech.github.io/substrate/master/sp_finality_grandpa/trait.GrandpaApi.html#
	impl fg_primitives::GrandpaApi<Block> for Runtime {
		/// Get the current GRANDPA authorities and weights. This should not change except
		/// for when changes are scheduled and the corresponding delay has passed.
		///
		/// When called at block B, it will return the set of authorities that should be
		/// used to finalize descendants of this block (B+1, B+2, ...). The block B itself
		/// is finalized by the authorities from block B-1.
		fn grandpa_authorities() -> GrandpaAuthorityList {
			Grandpa::grandpa_authorities()
		}

		/// Get current GRANDPA authority set id.
		fn current_set_id() -> fg_primitives::SetId {
			Grandpa::current_set_id()
		}

		/// Submits an unsigned extrinsic to report an equivocation. The caller
		/// must provide the equivocation proof and a key ownership proof
		/// (should be obtained using `generate_key_ownership_proof`). The
		/// extrinsic will be unsigned and should only be accepted for local
		/// authorship (not to be broadcast to the network). This method returns
		/// `None` when creation of the extrinsic fails, e.g. if equivocation
		/// reporting is disabled for the given runtime (i.e. this method is
		/// hardcoded to return `None`). Only useful in an offchain context.
		fn submit_report_equivocation_unsigned_extrinsic(
			_equivocation_proof: fg_primitives::EquivocationProof<
				<Block as BlockT>::Hash,
				NumberFor<Block>,
			>,
			_key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			None
		}

		/// Generates a proof of key ownership for the given authority in the
		/// given set. An example usage of this module is coupled with the
		/// session historical module to prove that a given authority key is
		/// tied to a given staking identity during a specific session. Proofs
		/// of key ownership are necessary for submitting equivocation reports.
		/// NOTE: even though the API takes a `set_id` as parameter the current
		/// implementations ignore this parameter and instead rely on this
		/// method being called at the correct block height, i.e. any point at
		/// which the given set id is live on-chain. Future implementations will
		/// instead use indexed data through an offchain worker, not requiring
		/// older states to be available.
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

	/// The Runtime API to query account nonce (aka transaction index).
	///
	/// See: https://paritytech.github.io/substrate/master/frame_system_rpc_runtime_api/trait.AccountNonceApi.html#
	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
		/// Get current account nonce of given `AccountId`.
		fn account_nonce(account: AccountId) -> Index {
			System::account_nonce(account)
		}
	}

	/// Runtime API for transaction payment pallet.
	///
	/// See: https://paritytech.github.io/substrate/master/pallet_transaction_payment_rpc_runtime_api/trait.TransactionPaymentApi.html#
	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		/// Query the data that we know about the fee of a given `call`.
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}

		/// Query the detailed fee of a given `call`.
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
	}

	/// The Runtime API used to dry-run contract interactions.
	///
	/// See: https://paritytech.github.io/substrate/master/pallet_contracts/trait.ContractsApi.html#
	impl pallet_contracts_rpc_runtime_api::ContractsApi<Block, AccountId, Balance, BlockNumber, Hash> for Runtime {
		/// Perform a call from a specified account to a given contract.
		///
		/// See [`crate::Pallet::bare_call`].
		fn call(
			origin: AccountId,
			dest: AccountId,
			value: Balance,
			gas_limit: u64,
			storage_deposit_limit: Option<Balance>,
			input_data: Vec<u8>,
		) -> pallet_contracts_primitives::ContractExecResult<Balance> {
			Contracts::bare_call(origin, dest, value, gas_limit, storage_deposit_limit, input_data, true)
		}

		/// Instantiate a new contract.
		///
		/// See `[crate::Pallet::bare_instantiate]`.
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
			Contracts::bare_instantiate(origin, value, gas_limit, storage_deposit_limit, code, data, salt, true)
		}

		/// Upload new code without instantiating a contract from it.
		///
		/// See [`crate::Pallet::bare_upload_code`].
		fn upload_code(
			origin: AccountId,
			code: Vec<u8>,
			storage_deposit_limit: Option<Balance>,
		) -> pallet_contracts_primitives::CodeUploadResult<Hash, Balance>
		{
			Contracts::bare_upload_code(origin, code, storage_deposit_limit)
		}

		/// Query a given storage key in a given contract.
		///
		/// Returns `Ok(Some(Vec<u8>))` if the storage value exists under the given key in the
		/// specified account and `Ok(None)` if it doesn't. If the account specified by the address
		/// doesn't exist, or doesn't have a contract then `Err` is returned.
		fn get_storage(
			address: AccountId,
			key: Vec<u8>,
		) -> pallet_contracts_primitives::GetStorageResult {
			Contracts::get_storage(address, key)
		}
	}

	/// Runtime API that allows the Outer Node to communicate with the Runtime's Pallet-Protos
	impl pallet_protos_rpc_runtime_api::ProtosRuntimeApi<Block, AccountId> for Runtime {
		/// **Query** and **Return** **Proto-Fragment(s)** based on **`params`**
		fn get_protos(params: GetProtosParams<AccountId, Vec<u8>>) -> Result<Vec<u8>, Vec<u8>> {
			Protos::get_protos(params)
		}
		/// **Query** the Genealogy of a Proto-Fragment based on **`params`**
		fn get_genealogy(params: GetGenealogyParams<Vec<u8>>) -> Result<Vec<u8>, Vec<u8>> {
			Protos::get_genealogy(params)
		}
	}

	/// Runtime API that allows the Outer Node to communicate with the Runtime's Pallet-Fragments
	impl pallet_fragments_rpc_runtime_api::FragmentsRuntimeApi<Block, AccountId> for Runtime {
		/// **Query** and **Return** **Fragment Definition(s)** based on **`params`**
		fn get_definitions(params: GetDefinitionsParams<AccountId, Vec<u8>>) -> Result<Vec<u8>, Vec<u8>> {
			Fragments::get_definitions(params)
		}
		/// **Query** and **Return** **Fragment Instance(s)** based on **`params`**
		fn get_instances(params: GetInstancesParams<AccountId, Vec<u8>>) -> Result<Vec<u8>, Vec<u8>> {
			Fragments::get_instances(params)
		}
		/// Query the owner of a Fragment Instance. The return type is a String
		fn get_instance_owner(params: GetInstanceOwnerParams<Vec<u8>>) -> Result<Vec<u8>, Vec<u8>> {
			Fragments::get_instance_owner(params)
		}
	}

	/// Runtime api for benchmarking a FRAME runtime.
	///
	/// See: https://paritytech.github.io/substrate/master/frame_benchmarking/trait.Benchmark.html#
	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		/// Get the benchmark metadata available for this runtime.
		///
		/// Parameters
		/// - `extra`: Also list benchmarks marked "extra" which would otherwise not be
		///            needed for weight calculation.
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{list_benchmark, baseline, Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;
			use baseline::Pallet as BaselineBench;

			let mut list = Vec::<BenchmarkList>::new();

			list_benchmark!(list, extra, frame_benchmarking, BaselineBench::<Runtime>);
			list_benchmark!(list, extra, pallet_balances, Balances);
			list_benchmark!(list, extra, pallet_timestamp, Timestamp);
			list_benchmark!(list, extra, pallet_assets, Assets);
			list_benchmark!(list, extra, pallet_multisig, Multisig);
			list_benchmark!(list, extra, pallet_proxy, Proxy);
			list_benchmark!(list, extra, pallet_identity, Identity);
			list_benchmark!(list, extra, pallet_utility, Utility);

			list_benchmark!(list, extra, pallet_accounts, Accounts);
			list_benchmark!(list, extra, pallet_detach, Detach);
			list_benchmark!(list, extra, pallet_fragments, Fragments);
			list_benchmark!(list, extra, pallet_protos, Protos);

			let storage_info = AllPalletsWithSystem::storage_info();

			return (list, storage_info)
		}

		/// Dispatch the given benchmark.
		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{baseline, Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};

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
			add_benchmark!(params, batches, pallet_timestamp, Timestamp);
			add_benchmark!(params, batches, pallet_assets, Assets);
			add_benchmark!(params, batches, pallet_multisig, Multisig);
			add_benchmark!(params, batches, pallet_proxy, Proxy);
			add_benchmark!(params, batches, pallet_identity, Identity);
			add_benchmark!(params, batches, pallet_utility, Utility);

			add_benchmark!(params, batches, pallet_accounts, Accounts);
			add_benchmark!(params, batches, pallet_detach, Detach);
			add_benchmark!(params, batches, pallet_fragments, Fragments);
			add_benchmark!(params, batches, pallet_protos, Protos);

			Ok(batches)
		}
	}
}
