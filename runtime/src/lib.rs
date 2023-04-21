//! The Runtime of the Fragnova Node.
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

mod chain_extension;

/// Constant values used within the runtime.
pub mod constants;
use constants::{currency::*, time::*, block::*, validation_logic::*, CONTRACTS_DEBUG_OUTPUT};

/// Generated voter bag information.
mod voter_bags;

use frame_support::{
	dispatch::DispatchClass,
	traits::{ConstBool, ConstU128, ConstU32, ConstU64, EitherOfDiverse, U128CurrencyToVote},
	PalletId,
};
use frame_system::{EnsureRoot, EnsureSigned, EnsureWithSuccess};
use pallet_grandpa::{
	fg_primitives, AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList,
};
pub use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
pub use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_session::historical::{self as pallet_session_historical};
use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
	create_runtime_str,
	curve::PiecewiseLinear,
	generic,
	impl_opaque_keys,
	traits::{
		BlakeTwo256, Block as BlockT, Extrinsic as ExtrinsicT, NumberFor, OpaqueKeys, Verify,
	},
	transaction_validity::{
		InvalidTransaction, TransactionPriority, TransactionSource, TransactionValidity, TransactionValidityError,
	},
	ApplyExtrinsicResult,
	FixedU128,
};
use sp_std::{prelude::*, str};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

// A few exports that help ease life for downstream crates.
pub use frame_support::{
	construct_runtime,
	pallet_prelude::Get,
	parameter_types,
	traits::{Contains, KeyOwnerProofSystem, Randomness, StorageInfo},
	weights::{
		constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight},
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
pub use sp_runtime::{Perbill, Permill, Percent};

use scale_info::prelude::string::String;

#[cfg(any(feature = "std", test))]
pub use pallet_staking::StakerStatus;

use codec::{Decode, Encode};
use frame_election_provider_support::{
	onchain, BalancingConfig, ElectionDataProvider, SequentialPhragmen,
};
use frame_support::traits::AsEnsureOriginWithArg;
use sp_runtime::traits::{ConstU8, SaturatedConversion, StaticLookup};

use pallet_fragments::{GetDefinitionsParams, GetInstanceOwnerParams, GetInstancesParams};
use pallet_protos::{GetGenealogyParams, GetProtosParams};

use pallet_oracle::OracleProvider;

// IMPORTS BELOW ARE USED IN the module `validation_logic`
use protos::{
	categories::{
		AudioCategories, BinaryCategories, Categories, ModelCategories, TextCategories,
		TextureCategories, VectorCategories, VideoCategories,
	},
	traits::Trait,
};

pub use sp_fragnova::{
	BlockNumber,
	Signature,
	AccountId,
	Balance,
	Index,
	Hash,
	AccountIndex,
	Moment,
};

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
		/// A session key is a concatenation of four public keys. They are used by validators for signing consensus-related messages
		///
		/// Source: https://www.youtube.com/watch?v=-PSPfbAcpiQ&t=1120s
		pub struct SessionKeys {
			pub aura: Aura,
			pub grandpa: Grandpa,
			pub im_online: ImOnline,
			pub authority_discovery: AuthorityDiscovery,
		}
	}
}

/// To learn more about runtime versioning and what each of the following value means:
///   https://docs.substrate.io/v3/runtime/upgrades#runtime-versioning
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("fragnova-testnet"),
	impl_name: create_runtime_str!("fragnova-hstn"),
	authoring_version: 1,
	// The version of the runtime specification. A full node will not attempt to use its native
	//   runtime in substitute for the on-chain Wasm runtime unless all of `spec_name`,
	//   `spec_version`, and `authoring_version` are the same between Wasm and native.
	// This value is set to 100 to notify Polkadot-JS App (https://polkadot.js.org/apps) to use
	//   the compatible custom types.
	spec_version: 1,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 1,
};

/// The version information is used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

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
	pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
		::max_with_normal_ratio(MAXIMUM_BLOCK_LENGTH, NORMAL_DISPATCH_RATIO);

	// TODO Review - The same code written here can now be done in a single line with the function `BlockWeights::with_sensible_defaults()`. See this example here: https://github.com/paritytech/substrate/blob/polkadot-v0.9.37/bin/node-template/runtime/src/lib.rs#L145-L148
	/// Set the "target block weight" for the Fragnova Blockchain.
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
	///    	  - If `max_total` is set to `None` as well, all extrinsics of the extrinsic class will always end up in the block (recommended for the `Mandatory` extrinsic class).
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
	pub BlockWeights: frame_system::limits::BlockWeights = frame_system::limits::BlockWeights::builder()
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

mod validation_logic {
	use protos::categories::ShardsFormat;

	use super::*;

	fn match_gltf(buf: &[u8]) -> bool {
		// gltf v2
		return buf.len() >= 5 &&
			buf[0] == 0x67 &&
			buf[1] == 0x6C &&
			buf[2] == 0x54 &&
			buf[3] == 0x46 &&
			buf[4] == 0x02
	}

	fn match_rared(buf: &[u8]) -> bool {
		// rared v1
		return buf.len() >= 6 &&
			buf[0] == 0x72 &&
			buf[1] == 0x61 &&
			buf[2] == 0x72 &&
			buf[3] == 0x65 &&
			buf[4] == 0x64 &&
			buf[5] == 0x01
	}

	fn match_blender(buf: &[u8]) -> bool {
		// blender BLENDER-
		return buf.len() >= 8 &&
			buf[0] == 0x42 &&
			buf[1] == 0x4C &&
			buf[2] == 0x45 &&
			buf[3] == 0x4E &&
			buf[4] == 0x44 &&
			buf[5] == 0x45 &&
			buf[6] == 0x52 &&
			buf[7] == 0x2D
	}

	fn match_safetensor(buf: &[u8]) -> bool {
		// safetensor v1
		return buf.len() >= 24 &&
			// skip 8 bytes
			// find {"__metadata__"
			// not the best but might work most of times!
			buf[8] == 0x7B &&
			buf[9] == 0x22 &&
			buf[10] == 0x5F &&
			buf[11] == 0x5F &&
			buf[12] == 0x6D &&
			buf[13] == 0x65 &&
			buf[14] == 0x74 &&
			buf[15] == 0x61 &&
			buf[16] == 0x64 &&
			buf[17] == 0x61 &&
			buf[18] == 0x74 &&
			buf[19] == 0x61 &&
			buf[20] == 0x5F &&
			buf[21] == 0x5F &&
			buf[22] == 0x22 &&
			buf[23] == 0x3A
	}

	/// Does the call `c` use `transaction_index::index`.
	fn does_call_index_the_transaction(c: &RuntimeCall) -> bool {
		matches!(
			c,
			RuntimeCall::Protos(pallet_protos::Call::upload { .. }) | // https://fragcolor-xyz.github.io/fragnova/doc/pallet_protos/pallet/enum.Call.html#
		RuntimeCall::Protos(pallet_protos::Call::patch { .. }) |
		RuntimeCall::Protos(pallet_protos::Call::set_metadata { .. }) |
		RuntimeCall::Fragments(pallet_fragments::Call::set_definition_metadata { .. }) | // https://fragcolor-xyz.github.io/fragnova/doc/pallet_fragments/pallet/enum.Call.html#
		RuntimeCall::Fragments(pallet_fragments::Call::set_instance_metadata { .. })
		)
	}

	fn is_valid(category: &Categories, data: &Vec<u8>) -> bool {
		match category {
			Categories::Text(sub_categories) => match sub_categories {
				TextCategories::Plain | TextCategories::Wgsl | TextCategories::Markdown =>
					str::from_utf8(data).is_ok(),
				TextCategories::Json =>
					serde_json::from_slice::<serde_json::Value>(&data[..]).is_ok(),
			},
			Categories::Trait(trait_hash) => match trait_hash {
				Some(_) => false,
				None => {
					let Ok(trait_struct) = Trait::decode(&mut &data[..]) else {
						return false;
					};

					if trait_struct.name.len() == 0 {
						return false
					}

					if trait_struct.records.len() == 0 {
						return false
					}

					trait_struct.records.windows(2).all(|window| {
						let (record_1, record_2) = (&window[0], &window[1]);
						let (Ok(a1), Ok(a2)) = (get_utf8_string(&record_1.name), get_utf8_string(&record_2.name)) else { // `a1` is short for `attribute_1`, `a2` is short for `attribute_2`
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
			Categories::Shards(shards_script_info_struct) =>
				shards_script_info_struct.format == ShardsFormat::Edn &&
					str::from_utf8(data).is_ok(),
			Categories::Audio(sub_categories) => match sub_categories {
				AudioCategories::OggFile => infer::is(data, "ogg"),
				AudioCategories::Mp3File => infer::is(data, "mp3"),
			},
			Categories::Texture(sub_categories) => match sub_categories {
				TextureCategories::PngFile => infer::is(data, "png"), // png_decoder::decode(&data[..]).is_ok(),
				TextureCategories::JpgFile => infer::is(data, "jpg"),
			},
			Categories::Vector(sub_categories) => match sub_categories {
				VectorCategories::SvgFile => false,
				VectorCategories::TtfFile => infer::is(data, "ttf"), // ttf_parser::Face::parse(&data[..], 0).is_ok(),
				VectorCategories::OtfFile => infer::is(data, "otf"),
			},
			Categories::Video(sub_categories) => match sub_categories {
				VideoCategories::MkvFile => infer::is(data, "mkv"),
				VideoCategories::Mp4File => infer::is(data, "mp4"),
			},
			Categories::Model(sub_categories) => match sub_categories {
				ModelCategories::GltfFile => match_gltf(data),
				ModelCategories::Sdf => false, // Note: "This is a Fragnova/Fragcolor data type"
				ModelCategories::PhysicsCollider => false, // Note: "This is a Fragnova/Fragcolor data type"
			},
			Categories::Binary(sub_categories) => match sub_categories {
				BinaryCategories::WasmProgram => infer::is(data, "wasm"), // wasmparser_nostd::Parser::new(0).parse_all(data).all(|payload| payload.is_ok()), // REVIEW - shouldn't I check if the last `payload` is `Payload::End`?
				BinaryCategories::WasmReactor => infer::is(data, "wasm"),
				BinaryCategories::BlendFile => match_blender(data),
				BinaryCategories::OnnxModel => false,
				BinaryCategories::SafeTensors => match_safetensor(data),
				BinaryCategories::RareDomain => match_rared(data),
			},
			Categories::Bundle => data.is_empty(),
		}
	}

	/// Is the call `c` valid?
	///
	/// Note: This function does not check whether the child/descendant calls of `c` (if it has any) are valid.
	pub fn is_the_immediate_call_valid(c: &RuntimeCall) -> bool {
		match c {
			RuntimeCall::Protos(ProtosCall::upload{ref data, ref category, ..}) => {
				// `Categories::Shards`, `Categories::Traits` and `Categories::Text`
				// must have `data` that is of the enum variant type `ProtoData::Local`
				match category {
					Categories::Shards(_) | Categories::Trait(_) | Categories::Text(_) | Categories::Bundle => match data {
						pallet_protos::ProtoData::Local(_) => (),
						_ => return false,
					},
					_ => (),
				};
				match data {
					pallet_protos::ProtoData::Local(ref data) => is_valid(category, data),
					_ => true,
				}
			},
			RuntimeCall::Protos(ProtosCall::patch{ref proto_hash, ref data, ..}) => {
				let Some(proto_struct) = pallet_protos::Protos::<Runtime>::get(proto_hash) else {
					return false;
				};
				match data {
					None => true,
					Some(pallet_protos::ProtoData::Local(ref data)) => is_valid(&proto_struct.category, data),
					_ => true,
				}
			},
			RuntimeCall::Protos(ProtosCall::set_metadata{ref data, ref metadata_key, ..}) |
			RuntimeCall::Fragments(FragmentsCall::set_definition_metadata{ref data, ref metadata_key, ..}) |
			RuntimeCall::Fragments(FragmentsCall::set_instance_metadata{ref data, ref metadata_key, ..}) => {
				if data.len() > MAXIMUM_METADATA_DATA_LENGTH {
					return false;
				}
				let _metadata_key = metadata_key;
				// match &metadata_key[..] {
				// 	b"title" => is_valid(&Categories::Text(TextCategories::Plain), data, &vec![]),
				// 	b"json_description" => is_valid(&Categories::Text(TextCategories::Json), data, &vec![]),
				// 	b"image" => is_valid(&Categories::Texture(TextureCategories::PngFile), data, &vec![]) || is_valid(&Categories::Texture(TextureCategories::JpgFile), data, &vec![]),
				// 	_ => false,
				// }
				true
			},
			// Prevent batch calls from containing any call that uses `transaction_index::index`. The reason we do this is because "any e̶x̶t̶r̶i̶n̶s̶i̶c̶ call using `transaction_index::index` will not work properly if used within a `pallet_utility` batch call as it depends on extrinsic index and during a batch there is only one index." (https://github.com/paritytech/substrate/issues/12835)
			RuntimeCall::Utility(pallet_utility::Call::batch { calls }) | // https://paritytech.github.io/substrate/master/pallet_utility/pallet/enum.Call.html
			RuntimeCall::Utility(pallet_utility::Call::batch_all { calls }) |
			RuntimeCall::Utility(pallet_utility::Call::force_batch { calls }) => {
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
					is_the_immediate_call_valid(&RuntimeCall::Protos(
						pallet_protos::Call::upload {
							// https://fragcolor-xyz.github.io/fragnova/doc/pallet_protos/pallet/enum.Call.html#
							references: vec![],
							category: category.clone(),
							tags: vec![].try_into().unwrap(),
							linked_asset: None,
							license: pallet_protos::UsageLicense::Closed,
							cluster: None,
							data: pallet_protos::ProtoData::Local(valid_data)
						}
					)),
					true
				);
				assert_eq!(
					is_the_immediate_call_valid(&RuntimeCall::Protos(
						pallet_protos::Call::upload {
							references: vec![],
							category: category.clone(),
							tags: vec![].try_into().unwrap(),
							linked_asset: None,
							license: pallet_protos::UsageLicense::Closed,
							cluster: None,
							data: pallet_protos::ProtoData::Local(invalid_data)
						}
					),),
					false
				);
			}
		}

		// For now we allow all keys!
		// #[test]
		// fn is_the_immediate_call_valid_should_not_work_if_metadata_key_is_invalid() {
		// 	for (metadata_key, data) in [
		// 		(b"title".to_vec(), b"I am valid UTF-8 text!".to_vec()),
		// 		(b"json_description".to_vec(), b"{\"key\": \"value\"}".to_vec()),
		// 		(b"image".to_vec(), vec![0xFF, 0xD8, 0xFF, 0xE0]),
		// 	] {
		// 		assert_eq!(
		// 			is_the_immediate_call_valid(&RuntimeCall::Protos(
		// 				pallet_protos::Call::set_metadata {
		// 					// https://fragcolor-xyz.github.io/fragnova/doc/pallet_protos/pallet/enum.Call.html#
		// 					proto_hash: [7u8; 32],
		// 					metadata_key: metadata_key.clone().try_into().unwrap(),
		// 					data: data.clone()
		// 				}
		// 			)),
		// 			true
		// 		);
		// 		assert_eq!(
		// 			is_the_immediate_call_valid(&RuntimeCall::Protos(
		// 				pallet_protos::Call::set_metadata {
		// 					proto_hash: [7u8; 32],
		// 					metadata_key: b"invalid_key".to_vec().try_into().unwrap(),
		// 					data: data.clone()
		// 				}
		// 			)),
		// 			false
		// 		);

		// 		assert_eq!(
		// 			is_the_immediate_call_valid(&RuntimeCall::Fragments(
		// 				pallet_fragments::Call::set_definition_metadata {
		// 					// https://fragcolor-xyz.github.io/fragnova/doc/pallet_fragments/pallet/enum.Call.html#
		// 					definition_hash: [7u8; 16],
		// 					metadata_key: metadata_key.clone().try_into().unwrap(),
		// 					data: data.clone()
		// 				}
		// 			)),
		// 			true
		// 		);
		// 		assert_eq!(
		// 			is_the_immediate_call_valid(&RuntimeCall::Fragments(
		// 				pallet_fragments::Call::set_definition_metadata {
		// 					definition_hash: [7u8; 16],
		// 					metadata_key: b"invalid_key".to_vec().try_into().unwrap(),
		// 					data: data.clone()
		// 				}
		// 			)),
		// 			false
		// 		);

		// 		assert_eq!(
		// 			is_the_immediate_call_valid(&RuntimeCall::Fragments(
		// 				pallet_fragments::Call::set_instance_metadata {
		// 					definition_hash: [7u8; 16],
		// 					edition_id: 1,
		// 					copy_id: 1,
		// 					metadata_key: metadata_key.clone().try_into().unwrap(),
		// 					data: data.clone()
		// 				}
		// 			)),
		// 			true
		// 		);
		// 		assert_eq!(
		// 			is_the_immediate_call_valid(&RuntimeCall::Fragments(
		// 				pallet_fragments::Call::set_instance_metadata {
		// 					definition_hash: [7u8; 16],
		// 					edition_id: 1,
		// 					copy_id: 1,
		// 					metadata_key: b"invalid_key".to_vec().try_into().unwrap(),
		// 					data: data.clone()
		// 				}
		// 			)),
		// 			false
		// 		);
		// 	}
		// }

		// For now only size check
		// #[test]
		// fn is_the_immediate_call_valid_should_not_work_if_metadata_data_is_invalid() {
		// 	for (metadata_key, data) in [
		// 		(b"title".to_vec(), b"I am valid UTF-8 text!".to_vec()),
		// 		(b"json_description".to_vec(), b"{\"key\": \"value\"}".to_vec()),
		// 		(b"image".to_vec(), vec![0xFF, 0xD8, 0xFF, 0xE0]),
		// 	] {
		// 		assert_eq!(
		// 			is_the_immediate_call_valid(&RuntimeCall::Protos(
		// 				pallet_protos::Call::set_metadata {
		// 					// https://fragcolor-xyz.github.io/fragnova/doc/pallet_protos/pallet/enum.Call.html#
		// 					proto_hash: [7u8; 32],
		// 					metadata_key: metadata_key.clone().try_into().unwrap(),
		// 					data: data.clone()
		// 				}
		// 			)),
		// 			true
		// 		);
		// 		assert_eq!(
		// 			is_the_immediate_call_valid(&RuntimeCall::Protos(
		// 				pallet_protos::Call::set_metadata {
		// 					proto_hash: [7u8; 32],
		// 					metadata_key: metadata_key.clone().try_into().unwrap(),
		// 					data: vec![0xF0, 0x9F, 0x98] // Invalid UTF-8 Text
		// 				}
		// 			)),
		// 			false
		// 		);

		// 		assert_eq!(
		// 			is_the_immediate_call_valid(&RuntimeCall::Fragments(
		// 				pallet_fragments::Call::set_definition_metadata {
		// 					// https://fragcolor-xyz.github.io/fragnova/doc/pallet_fragments/pallet/enum.Call.html#
		// 					definition_hash: [7u8; 16],
		// 					metadata_key: metadata_key.clone().try_into().unwrap(),
		// 					data: data.clone() // Invalid UTF-8 Text
		// 				}
		// 			)),
		// 			true
		// 		);
		// 		assert_eq!(
		// 			is_the_immediate_call_valid(&RuntimeCall::Fragments(
		// 				pallet_fragments::Call::set_definition_metadata {
		// 					definition_hash: [7u8; 16],
		// 					metadata_key: metadata_key.clone().try_into().unwrap(),
		// 					data: vec![0xF0, 0x9F, 0x98] // Invalid UTF-8 Text
		// 				}
		// 			)),
		// 			false
		// 		);

		// 		assert_eq!(
		// 			is_the_immediate_call_valid(&RuntimeCall::Fragments(
		// 				pallet_fragments::Call::set_instance_metadata {
		// 					definition_hash: [7u8; 16],
		// 					edition_id: 1,
		// 					copy_id: 1,
		// 					metadata_key: metadata_key.clone().try_into().unwrap(),
		// 					data: data.clone()
		// 				}
		// 			)),
		// 			true
		// 		);
		// 		assert_eq!(
		// 			is_the_immediate_call_valid(&RuntimeCall::Fragments(
		// 				pallet_fragments::Call::set_instance_metadata {
		// 					definition_hash: [7u8; 16],
		// 					edition_id: 1,
		// 					copy_id: 1,
		// 					metadata_key: metadata_key.try_into().unwrap(),
		// 					data: vec![0xF0, 0x9F, 0x98] // Invalid UTF-8 Text
		// 				}
		// 			)),
		// 			false
		// 		);
		// 	}
		// }

		#[test]
		fn is_the_immediate_call_valid_should_not_work_if_a_batch_call_contains_a_call_that_indexes_the_transaction(
		) {
			assert_eq!(
				is_the_immediate_call_valid(&RuntimeCall::Utility(pallet_utility::Call::batch {
					calls: vec![RuntimeCall::Protos(pallet_protos::Call::ban {
						// https://fragcolor-xyz.github.io/fragnova/doc/pallet_protos/pallet/enum.Call.html#
						proto_hash: [7u8; 32],
					})]
				}),),
				true
			);

			assert_eq!(
				is_the_immediate_call_valid(&RuntimeCall::Utility(pallet_utility::Call::batch {
					calls: vec![RuntimeCall::Protos(pallet_protos::Call::upload {
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
impl Contains<RuntimeCall> for BaseCallFilter {
	fn contains(c: &RuntimeCall) -> bool {
		// log::info!("The call {:?} is {}", c, validation_logic::is_the_immediate_call_valid(c));
		validation_logic::is_the_immediate_call_valid(c)
	}
}
impl frame_system::Config for Runtime {
	/// The basic call filter to use in dispatchable.
	type BaseCallFilter = BaseCallFilter;
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = BlockWeights;
	/// The maximum length of a block (in bytes).
	type BlockLength = BlockLength;
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The aggregated dispatch type that is available for extrinsics.
	type RuntimeCall = RuntimeCall;
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
	type RuntimeEvent = RuntimeEvent;
	/// The ubiquitous origin type.
	type RuntimeOrigin = RuntimeOrigin;
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

impl pallet_insecure_randomness_collective_flip::Config for Runtime {}

parameter_types! {
	pub const ImOnlineUnsignedPriority: TransactionPriority = TransactionPriority::max_value();
	/// We prioritize im-online heartbeats over election solution submission.
	pub const StakingUnsignedPriority: TransactionPriority = TransactionPriority::max_value() / 2;
	/// The maximum number of authorities that `pallet_aura` can hold.
	pub const MaxAuthorities: u32 = 100;
	pub const MaxKeys: u32 = 10_000;
	pub const MaxPeerInHeartbeats: u32 = 10_000;
	pub const MaxPeerDataEncodingSize: u32 = 1_000;
	// NOTE: Currently it is not possible to change the epoch duration after the chain has started.
	//       Attempting to do so will brick block production.
	pub const EpochDuration: u64 = EPOCH_DURATION_IN_SLOTS;
	pub const ReportLongevity: u64 =
		BondingDuration::get() as u64 * SessionsPerEra::get() as u64 * EpochDuration::get();
	pub const MaxSetIdSessionEntries: u32 = BondingDuration::get() * SessionsPerEra::get();
}

/// The Aura module extends Aura consensus by managing offline reporting.
///
/// ## Interface
///
/// ### Public Functions
///
/// - `slot_duration` - Determine the Aura slot-duration based on the Timestamp module
///   configuration.
///
/// ## Related Modules
///
/// - [Timestamp](https://paritytech.github.io/substrate/master/pallet_timestamp/index.html): The Timestamp module is used in Aura to track
/// consensus rounds (via `slots`).
///
/// Source: https://paritytech.github.io/substrate/master/pallet_aura/index.html#
impl pallet_aura::Config for Runtime {
	/// The identifier type for an authority.
	type AuthorityId = AuraId;
	/// A way to check whether a given validator is disabled and should not be authoring blocks.
	/// Blocks authored by a disabled validator will lead to a panic as part of this module's
	/// initialization.
	type DisabledValidators = ();
	/// The maximum number of authorities that the pallet can hold.
	type MaxAuthorities = MaxAuthorities;
}

/// GRANDPA Consensus module for runtime.
///
/// This manages the GRANDPA authority set ready for the native code.
/// These authorities are only for GRANDPA finality, not for consensus overall.
///
/// In the future, it will also handle misbehavior reports, and on-chain
/// finality notifications.
///
/// For full integration with GRANDPA, the `GrandpaApi` should be implemented.
/// The necessary items are re-exported via the `fg_primitives` crate.
///
/// Source: https://paritytech.github.io/substrate/master/pallet_grandpa/index.html#
///
/// ## Terminology
///
/// - Equivocation Report: An equivocation report is a signaling that someone detected a Aura/Babe authority to have build two blocks on the same height.
///   So, if someone detects a Aura/Babe authority building two blocks at the same height, it will issue such an equivocation report.
///   It is up to chain how it wants to handle this. Polkadot for example will slash the offending authority." -
///
/// Source: https://stackoverflow.com/questions/71004838/in-substrate-what-is-the-equivocation-handling-system-and-keyownerproof-identif/71016144#71016144
impl pallet_grandpa::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	/// A system for proving ownership of keys, i.e. that a given key was part
	/// of a validator set, needed for validating equivocation reports.
	type KeyOwnerProofSystem = Historical;
	/// The proof of key ownership, used for validating equivocation reports
	/// The proof must include the session index and validator count of the
	/// session at which the equivocation occurred.
	type KeyOwnerProof =
	<Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;
	/// The identification of a key owner, used when reporting equivocations.
	type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
		KeyTypeId,
		GrandpaId,
	)>>::IdentificationTuple;
	/// The equivocation handling subsystem, defines methods to report an
	/// offence (after the equivocation has been validated) and for submitting a
	/// transaction to report an equivocation (from an offchain context).
	/// NOTE: when enabling equivocation handling (i.e. this type isn't set to
	/// `()`) you must use this pallet's `ValidateUnsigned` in the runtime
	/// definition.
	type HandleEquivocation = pallet_grandpa::EquivocationHandler<
		Self::KeyOwnerIdentification,
		Offences,
		ReportLongevity,
	>;
	type WeightInfo = ();
	/// Max Authorities in use
	type MaxAuthorities = MaxAuthorities;
	/// The maximum number of entries to keep in the set id to session index mapping.
	///
	/// Since the `SetIdSession` map is only used for validating equivocations this
	/// value should relate to the bonding duration of whatever staking system is
	/// being used (if any). If equivocation handling is not enabled then this value
	/// can be zero.
	type MaxSetIdSessionEntries = MaxSetIdSessionEntries;
}

parameter_types! {
	/// TODO: Documentation
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = Moment;
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
	/// Whether an account can voluntarily transfer any of its balance to another account
	///
	/// Note: This type has been added by Fragnova
	pub const IsTransferable: bool = true;
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = MaxLocks;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
	type IsTransferable = IsTransferable;
}

impl pallet_balances::Config<pallet_balances::Instance2> for Runtime {
	type MaxLocks = MaxLocks;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
	type IsTransferable = IsTransferable;
}

// TODO Review - All the parameters (except `OperationalFeeMultiplier`) have been copied from the repo `substrate-contracts-node`: https://github.com/paritytech/substrate-contracts-node/blob/fcc75b237d85a5f8ad5a492c14d8bd3e065fea8a/runtime/src/lib.rs#L313-L324
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
	/// See for more information: https://paritytech.github.io/substrate/master/pallet_contracts/index.html
	pub const DepositPerByte: Balance = deposit(0, 1);
	/// The maximum number of contracts that can be pending for deletion.
	pub const DeletionQueueDepth: u32 = 128;
	/// The maximum amount of weight that can be consumed per block for lazy trie removal.
	// The lazy deletion runs inside on_initialize.
	pub DeletionWeightLimit: Weight = BlockWeights::get()
		.per_class
		.get(DispatchClass::Normal)
		.max_total
		.unwrap_or(BlockWeights::get().max_block);
	/// Cost schedule and limits.
	pub Schedule: pallet_contracts::Schedule<Runtime> = Default::default();
	/// A fee mulitplier for `Operational` extrinsics to compute "virtual tip" to boost their
	/// `priority`
	pub OperationalFeeMultiplier: u8 = 5;
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
	type RuntimeEvent = RuntimeEvent;
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
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
}

impl pallet_fragments::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

impl pallet_accounts::EthFragContract for Runtime {
	fn get_partner_contracts() -> Vec<String> {
		vec![String::from("0x8a819F380ff18240B5c11010285dF63419bdb2d5")]
	}
}

impl pallet_accounts::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
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
	type RuntimeEvent = RuntimeEvent;
	type OracleProvider = Runtime; // the contract address determines the network to connect (mainnet, goerli, etc.)
	type Threshold = ConstU64<1>;
}

impl pallet_protos::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type StringLimit = StringLimit;
	type DetachAccountLimit = ConstU32<20>; // An ethereum public account address has a length of 20.
	type MaxTags = ConstU32<10>;
}

impl pallet_detach::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type AuthorityId = pallet_detach::crypto::DetachAuthId;
}

impl pallet_clusters::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type NameLimit = ConstU32<20>;
	type DataLimit = ConstU32<300>;
	type MembersLimit = ConstU32<20>;
	type RoleSettingsLimit = ConstU32<20>;
}

parameter_types! {
	pub RootNamespace: Vec<u8> = b"Frag".to_vec();
}

impl pallet_aliases::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type NamespacePrice = ConstU128<100>;
	type NameLimit = ConstU32<20>;
	type RootNamespace = RootNamespace;
}

impl pallet_multisig::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type DepositBase = ConstU128<1>;
	type DepositFactor = ConstU128<1>;
	type MaxSignatories = ConstU32<3>;
	type WeightInfo = ();
}

impl pallet_proxy::Config for Runtime {
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

parameter_types! {
	/// Maximum number of additional fields that may be stored in an ID. Needed to bound the I/O
	/// required to access an identity, but can be pretty high.
	pub const MaxAdditionalFields: u32 = 2;
	/// Maxmimum number of registrars allowed in the system. Needed to bound the complexity
	/// of, e.g., updating judgements.
	pub const MaxRegistrars: u32 = 20;
}

impl pallet_identity::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
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
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
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
	RuntimeCall: From<LocalCall>,
{
	type OverarchingCall = RuntimeCall;
	type Extrinsic = UncheckedExtrinsic;
}

/// Because you configured the Config trait for detach pallet and frag pallet
/// to implement the `CreateSignedTransaction` trait, you also need to implement that trait for the runtime.
impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
	RuntimeCall: From<LocalCall>,
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
		call: RuntimeCall,
		public: <Signature as Verify>::Signer,
		account: AccountId,
		nonce: Index,
	) -> Option<(RuntimeCall, <UncheckedExtrinsic as ExtrinsicT>::SignaturePayload)> {
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
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	/// The safest default is to allow no calls at all.
	///
	/// Runtimes should whitelist dispatchables that are allowed to be called from contracts
	/// and make sure they are stable. Dispatchables exposed to contracts are not allowed to
	/// change because that would break already deployed contracts. The `Call` structure itself
	/// is not allowed to change the indices of existing pallets, too.
	type CallFilter = frame_support::traits::Nothing;
	/// The amount of balance a caller has to pay for each storage item.
	///
	/// # Note
	///
	/// Changing this value for an existing chain might need a storage migration.
	type DepositPerItem = DepositPerItem;
	/// The amount of balance a caller has to pay for each byte of storage.
	///
	/// # Note
	///
	/// Changing this value for an existing chain might need a storage migration.
	type DepositPerByte = DepositPerByte;
	type CallStack = [pallet_contracts::Frame<Self>; 5];
	type WeightPrice = pallet_transaction_payment::Pallet<Self>;
	type WeightInfo = pallet_contracts::weights::SubstrateWeight<Self>;
	/// Type that allows the runtime authors to add new host functions for a contract to call.
	type ChainExtension = chain_extension::MyExtension;
	type DeletionQueueDepth = DeletionQueueDepth;
	type DeletionWeightLimit = DeletionWeightLimit;
	type Schedule = Schedule;
	type AddressGenerator = pallet_contracts::DefaultAddressGenerator;
	type MaxCodeLen = ConstU32<{ 123 * 1024 }>;
	type MaxStorageKeyLen = ConstU32<128>;
	/// Make contract callable functions marked as `#[unstable]` available.
	///
	/// Contracts that use `#[unstable]` functions won't be able to be uploaded unless
	/// this is set to `true`. This is only meant for testnets and dev nodes in order to
	/// experiment with new features.
	///
	/// # Warning
	///
	/// Do **not** set to `true` on productions chains.
	type UnsafeUnstableInterface = ConstBool<false>;
	// TODO Review - Not sure what this is but I've made it `ConstU32<{ 2 * 1024 * 1024 }>` from following https://github.com/paritytech/substrate-contracts-node/blob/fcc75b237d85a5f8ad5a492c14d8bd3e065fea8a/runtime/src/lib.rs#L366
	type MaxDebugBufferLen = ConstU32<{ 2 * 1024 * 1024 }>;
	/// Whether an account can voluntarily transfer any of its balance to another account
	///
	/// Note: This type has been added by Fragnova
	type IsTransferable = ConstBool<false>;
}

parameter_types! {
	/// TODO: Documentation
	pub const IndexDeposit: Balance = 500;
}

impl pallet_indices::Config for Runtime {
	type AccountIndex = AccountIndex;
	type Currency = Balances;
	type Deposit = IndexDeposit;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_indices::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	/// The basic amount of funds that must be reserved for an asset.
	pub const AssetDeposit: Balance = 100 * DOLLARS;
	/// The amount of funds that must be reserved when creating a new approval.
	pub const ApprovalDeposit: Balance = 1 * DOLLARS;
	/// The maximum length of a name or symbol of an asset stored on-chain.
	pub const StringLimit: u32 = 75;
	/// The basic amount of funds that must be reserved when adding metadata to your asset.
	pub const MetadataDepositBase: Balance = 10 * DOLLARS;
	/// The additional funds that must be reserved for the number of bytes you store in your
	/// asset's metadata.
	pub const MetadataDepositPerByte: Balance = 1 * DOLLARS;
}

impl pallet_assets::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type RemoveItemsLimit = ConstU32<1000>;
	type AssetId = u64;
	type AssetIdParameter = u64;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
	type ForceOrigin = EnsureRoot<AccountId>;
	type AssetDeposit = AssetDeposit;
	type AssetAccountDeposit = ConstU128<DOLLARS>;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = StringLimit;
	type Freezer = ();
	type Extra = ();
	type CallbackHandle = ();
	type WeightInfo = pallet_assets::weights::SubstrateWeight<Runtime>;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

/// Authorship tracking for FRAME runtimes.
///
/// This tracks the current author of the block.
///
/// Source: https://paritytech.github.io/substrate/master/pallet_authorship/index.html#
impl pallet_authorship::Config for Runtime {
	/// Find the author of a block.
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
	type EventHandler = (Staking, ImOnline);
}

/// # I'm online Pallet
///
/// If the local node is a validator (i.e. contains an authority key), this pallet
/// gossips a heartbeat transaction with each new session. The heartbeat functions
/// as a simple mechanism to signal that the node is online in the current era.
///
/// Received heartbeats are tracked for one era and reset with each new era. The
/// pallet exposes two public functions to query if a heartbeat has been received
/// in the current era or session.
///
/// The heartbeat is a signed transaction, which was signed using the session key
/// and includes the recent best block number of the local validators chain as well
/// as the [NetworkState](https://paritytech.github.io/substrate/master/sp_core/offchain/struct.OpaqueNetworkState.html#).
/// It is submitted as an Unsigned Transaction via off-chain workers.
///
/// ## Interface
///
/// ### Public Functions
///
/// - `is_online` - True if the validator sent a heartbeat in the current session.
///
/// Source: https://paritytech.github.io/substrate/master/pallet_im_online/index.html#
impl pallet_im_online::Config for Runtime {
	/// The identifier type for an authority.
	type AuthorityId = ImOnlineId;
	type RuntimeEvent = RuntimeEvent;
	/// A trait that allows us to estimate the current session progress and also the
	/// average session length.
	///
	/// This parameter is used to determine the longevity of `heartbeat` transaction and a
	/// rough time when we should start considering sending heartbeats, since the workers
	/// avoids sending them at the very beginning of the session, assuming there is a
	/// chance the authority will produce a block and they won't be necessary.
	type NextSessionRotation = pallet_session::PeriodicSessions<SessionPeriod, Offset>; // Babe;
	/// A type for retrieving the validators supposed to be online in a session.
	type ValidatorSet = Historical;
	/// A type that gives us the ability to submit unresponsiveness offence reports.
	type ReportUnresponsiveness = Offences;
	/// A configuration for base priority of unsigned transactions.
	///
	/// This is exposed so that it can be tuned for particular runtime, when
	/// multiple pallets send unsigned transactions.
	type UnsignedPriority = ImOnlineUnsignedPriority;
	type WeightInfo = pallet_im_online::weights::SubstrateWeight<Runtime>;
	/// The maximum number of keys that can be added.
	type MaxKeys = MaxKeys;
	/// The maximum number of peers to be stored in `ReceivedHeartbeats`
	type MaxPeerInHeartbeats = MaxPeerInHeartbeats;
	/// The maximum size of the encoding of `PeerId` and `MultiAddr` that are coming
	/// from the hearbeat
	type MaxPeerDataEncodingSize = MaxPeerDataEncodingSize;
}

/// # Offences Pallet
///
/// Tracks reported offences
///
/// Source: https://paritytech.github.io/substrate/master/pallet_offences/index.html#
impl pallet_offences::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	/// Full identification of the validator.
	type IdentificationTuple = pallet_session::historical::IdentificationTuple<Self>;
	/// A handler called for every offence report.
	type OnOffenceHandler = Staking;
}


parameter_types! {
	pub const CouncilMotionDuration: BlockNumber = 5 * DAYS;
	pub const CouncilMaxProposals: u32 = 100;
	pub const CouncilMaxMembers: u32 = 100;
}

type CouncilCollective = pallet_collective::Instance1;

/// Collective system: Members of a set of account IDs can make their collective feelings known
/// through dispatched calls from one of two specialized origins.
///
/// The membership can be provided in one of two ways: either directly, using the Root-dispatchable
/// function `set_members`, or indirectly, through implementing the `ChangeMembers`.
/// The pallet assumes that the amount of members stays at or below `MaxMembers` for its weight
/// calculations, but enforces this neither in `set_members` nor in `change_members_sorted`.
///
/// A "prime" member may be set to help determine the default vote behavior based on chain
/// config. If `PrimeDefaultVote` is used, the prime vote acts as the default vote in case of any
/// abstentions after the voting period. If `MoreThanMajorityThenPrimeDefaultVote` is used, then
/// abstentions will first follow the majority of the collective voting, and then the prime
/// member.
///
/// Voting happens through motions comprising a proposal (i.e. a curried dispatchable) plus a
/// number of approvals required for it to pass and be called. Motions are open for members to
/// vote on for a minimum period given by `MotionDuration`. As soon as the needed number of
/// approvals is given, the motion is closed and executed. If the number of approvals is not reached
/// during the voting period, then `close` may be called by any account in order to force the end
/// the motion explicitly. If a prime member is defined then their vote is used in place of any
/// abstentions and the proposal is executed if there are enough approvals counting the new votes.
///
/// If there are not, or if no prime is set, then the motion is dropped without being executed.
///
/// # Footnote
///
/// For a more detailed explanation and live demo of this pallet, see https://www.youtube.com/watch?v=QJ9yuOqwy4s
///
/// Source: https://paritytech.github.io/substrate/master/pallet_collective/index.html#
impl pallet_collective::Config<CouncilCollective> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	/// The runtime call dispatch type.
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	/// The time-out for council motions.
	type MotionDuration = CouncilMotionDuration;
	/// Maximum number of proposals allowed to be active in parallel.
	type MaxProposals = CouncilMaxProposals;
	/// The maximum number of members supported by the pallet. Used for weight estimation.
	///
	/// NOTE:
	/// + Benchmarks will need to be re-run and weights adjusted if this changes.
	/// + This pallet assumes that dependents keep to the limit without enforcing it.
	type MaxMembers = CouncilMaxMembers;
	/// Default vote strategy of this collective.
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
	/// Origin allowed to set collective members
	type SetMembersOrigin = EnsureRoot<Self::AccountId>;
}
type EnsureRootOrHalfCouncil = EitherOfDiverse<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<AccountId, CouncilCollective, 1, 2>,
>;

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const ProposalBondMinimum: Balance = 1 * DOLLARS;
	pub const SpendPeriod: BlockNumber = 1 * DAYS;
	pub const Burn: Permill = Permill::from_percent(50);
	pub const TipCountdown: BlockNumber = 1 * DAYS;
	pub const TipFindersFee: Percent = Percent::from_percent(20);
	pub const TipReportDepositBase: Balance = 1 * DOLLARS;
	pub const DataDepositPerByte: Balance = 1 * CENTS;
	pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
	pub const MaximumReasonLength: u32 = 300;
	pub const MaxApprovals: u32 = 100;
	pub const MaxBalance: Balance = Balance::max_value();
}

/// The Treasury pallet provides a "pot" of funds that can be managed by stakeholders in the system
/// and a structure for making spending proposals from this pot.
///
/// ## Overview
///
/// The Treasury Pallet itself provides the pot to store funds, and a means for stakeholders to
/// propose, approve, and deny expenditures. The chain will need to provide a method (e.g.
/// inflation, fees) for collecting funds.
///
/// By way of example, the Council could vote to fund the Treasury with a portion of the block
/// reward and use the funds to pay developers.
///
///
/// ### Terminology
///
/// - **Proposal:** A suggestion to allocate funds from the pot to a beneficiary.
/// - **Beneficiary:** An account who will receive the funds from a proposal iff the proposal is
///   approved.
/// - **Deposit:** Funds that a proposer must lock when making a proposal. The deposit will be
///   returned or slashed if the proposal is approved or rejected respectively.
/// - **Pot:** Unspent funds accumulated by the treasury pallet.
///
/// ## Footnote
///
/// For a more detailed explanation and live demo of this pallet, see https://www.youtube.com/watch?v=HX7vRpOip5U
///
/// Source: https://paritytech.github.io/substrate/master/pallet_treasury/index.html#
impl pallet_treasury::Config for Runtime {
	type PalletId = TreasuryPalletId;
	/// The staking balance.
	type Currency = Frag;
	/// Origin from which approvals must come.
	type ApproveOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 3, 5>,
	>;
	/// Origin from which rejections must come.
	type RejectOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionMoreThan<AccountId, CouncilCollective, 1, 2>,
	>;
	type RuntimeEvent = RuntimeEvent;
	/// Handler for the unbalanced decrease when slashing for a rejected proposal or bounty.
	type OnSlash = ();
	/// Fraction of a proposal's value that should be bonded in order to place the proposal.
	/// An accepted proposal gets these back. A rejected proposal does not.
	type ProposalBond = ProposalBond;
	/// Minimum amount of funds that should be placed in a deposit for making a proposal.
	type ProposalBondMinimum = ProposalBondMinimum;
	/// Maximum amount of funds that should be placed in a deposit for making a proposal.
	type ProposalBondMaximum = ();
	/// Period between successive spends.
	type SpendPeriod = SpendPeriod;
	/// Percentage of spare funds (if any) that are burnt per spend period.
	type Burn = Burn;
	/// Handler for the unbalanced decrease when treasury funds are burned.
	type BurnDestination = ();
	/// Runtime hooks to external pallet using treasury to compute spend funds.
	type SpendFunds = Bounties;
	type WeightInfo = pallet_treasury::weights::SubstrateWeight<Runtime>;
	/// The maximum number of approvals that can wait in the spending queue.
	///
	/// NOTE: This parameter is also used within the Bounties Pallet extension if enabled.
	type MaxApprovals = MaxApprovals;
	/// The origin required for approving spends from the treasury outside of the proposal
	/// process. The `Success` value is the maximum amount that this origin is allowed to
	/// spend at a time.
	type SpendOrigin = EnsureWithSuccess<EnsureRoot<AccountId>, AccountId, MaxBalance>;
}

parameter_types! {
	pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
	pub const BountyValueMinimum: Balance = 5 * DOLLARS;
	pub const BountyDepositBase: Balance = 1 * DOLLARS;
	pub const CuratorDepositMultiplier: Permill = Permill::from_percent(50);
	pub const CuratorDepositMin: Balance = 1 * DOLLARS;
	pub const CuratorDepositMax: Balance = 100 * DOLLARS;
	pub const BountyDepositPayoutDelay: BlockNumber = 1 * DAYS;
	pub const BountyUpdatePeriod: BlockNumber = 14 * DAYS;
}

/// > NOTE: This pallet is tightly coupled with pallet-treasury.
///
/// This pallet contains logic related to Bounty Spendings.
///
/// A Bounty Spending is a reward for a specified body of work - or specified set of objectives -
/// that needs to be executed (i.e completed) for a predefined Treasury amount to be paid out. A curator is assigned
/// after the bounty is approved and funded by Council, to be delegated with the responsibility of
/// assigning a payout address once the specified set of objectives is completed.
///
/// This pallet may opt into using a [`ChildBountyManager`] that enables bounties to be split into
/// sub-bounties, as children of anh established bounty (called the parent in the context of it's
/// children).
///
/// # Footnote
///
/// For a more detailed explanation and live demo of this pallet, see https://www.youtube.com/watch?v=HX7vRpOip5U
///
/// Source: https://paritytech.github.io/substrate/master/pallet_bounties/index.html#
impl pallet_bounties::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	/// The amount held on deposit for placing a bounty proposal.
	type BountyDepositBase = BountyDepositBase;
	/// The delay period for which a bounty beneficiary need to wait before claim the payout.
	type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
	/// Bounty duration in blocks.
	type BountyUpdatePeriod = BountyUpdatePeriod;
	/// The curator deposit is calculated as a percentage of the curator fee.
	///
	/// This deposit has optional upper and lower bounds with `CuratorDepositMax` and
	/// `CuratorDepositMin`.
	type CuratorDepositMultiplier = CuratorDepositMultiplier;
	/// Minimum amount of funds that should be placed in a deposit for making a proposal.
	type CuratorDepositMin = CuratorDepositMin;
	/// Maximum amount of funds that should be placed in a deposit for making a proposal.
	type CuratorDepositMax = CuratorDepositMax;
	/// Minimum value for a bounty.
	type BountyValueMinimum = BountyValueMinimum;
	/// The amount held on deposit per byte within the tip report reason or bounty description.
	type DataDepositPerByte = DataDepositPerByte;
	/// Maximum acceptable reason length.
	///
	/// Benchmarks depend on this value, be sure to update weights file when changing this value
	type MaximumReasonLength = MaximumReasonLength;
	type WeightInfo = pallet_bounties::weights::SubstrateWeight<Runtime>;
	/// The child bounty manager.
	type ChildBountyManager = ChildBounties;
}

parameter_types! {
	pub const ChildBountyValueMinimum: Balance = 1 * DOLLARS;
}

/// > NOTE: This pallet is tightly coupled with `pallet-treasury` and `pallet-bounties`.
///
/// This pallet contains logic related to Child Bounties.
///
/// With child bounties, a large bounty proposal can be divided into smaller chunks,
/// for parallel execution (i.e parallel completion), and for efficient governance and tracking of spent funds.
/// A child bounty is a smaller piece of work, extracted from a parent bounty.
/// A curator is assigned after the child bounty is created by the parent bounty curator,
/// to be delegated with the responsibility of assigning a payout address once the specified
/// set of tasks is completed.
///
/// # Footnote
///
/// For a more detailed explanation and live demo of this pallet, see https://www.youtube.com/watch?v=HX7vRpOip5U
///
/// Source: https://paritytech.github.io/substrate/master/pallet_child_bounties/index.html#
impl pallet_child_bounties::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	/// Maximum number of child bounties that can be added to a parent bounty.
	type MaxActiveChildBountyCount = ConstU32<5>;
	/// Minimum value for a child-bounty.
	type ChildBountyValueMinimum = ChildBountyValueMinimum;
	type WeightInfo = pallet_child_bounties::weights::SubstrateWeight<Runtime>;
}


/// This pallet is used by the `client/authority-discovery` to retrieve the current and the next set of authorities.
///
/// Source: https://paritytech.github.io/substrate/master/pallet_authority_discovery/index.html#
impl pallet_authority_discovery::Config for Runtime {
	type MaxAuthorities = MaxAuthorities;
}

parameter_types! {
	// phase durations. 1/4 of the last session for each.
	pub const SignedPhase: u64 = EPOCH_DURATION_IN_BLOCKS / 4;
	pub const UnsignedPhase: u64 = EPOCH_DURATION_IN_BLOCKS / 4;

	// signed config
	pub const SignedRewardBase: Balance = 1 * DOLLARS;
	pub const SignedDepositBase: Balance = 1 * DOLLARS;
	pub const SignedDepositByte: Balance = 1 * CENTS;

	pub BetterUnsignedThreshold: Perbill = Perbill::from_rational(1u32, 10_000);

	// miner configs
	pub const MultiPhaseUnsignedPriority: TransactionPriority = StakingUnsignedPriority::get() - 1u64;
	pub MinerMaxWeight: Weight = BlockWeights::get()
		.get(DispatchClass::Normal)
		.max_extrinsic.expect("Normal extrinsics have a weight limit configured; qed")
		.saturating_sub(BlockExecutionWeight::get());
	// Solution can occupy 90% of normal block size
	pub MinerMaxLength: u32 = Perbill::from_rational(9u32, 10) *
		*BlockLength::get()
		.max
		.get(DispatchClass::Normal);
}

frame_election_provider_support::generate_solution_type!(
	#[compact]
	pub struct NposSolution16::<
		VoterIndex = u32,
		TargetIndex = u16,
		Accuracy = sp_runtime::PerU16,
		MaxVoters = MaxElectingVoters,
	>(16)
);

parameter_types! {
	pub MaxNominations: u32 = <NposSolution16 as frame_election_provider_support::NposSolution>::LIMIT as u32;
	pub MaxElectingVoters: u32 = 40_000;
	pub MaxElectableTargets: u16 = 10_000;
	// OnChain values are lower.
	pub MaxOnChainElectingVoters: u32 = 5000;
	pub MaxOnChainElectableTargets: u16 = 1250;
	// The maximum winners that can be elected by the Election pallet which is equivalent to the
	// maximum active validators the staking pallet can have.
	pub MaxActiveValidators: u32 = 1000;
}

/// The numbers configured here could always be more than the the maximum limits of staking pallet
/// to ensure election snapshot will not run out of memory. For now, we set them to smaller values
/// since the staking is bounded and the weight pipeline takes hours for this single pallet.
pub struct ElectionProviderBenchmarkConfig;
/// Configuration for the benchmarks of the pallet `pallet_election_provider_multi_phase`.
///
/// Source: https://paritytech.github.io/substrate/master/pallet_election_provider_multi_phase/trait.BenchmarkingConfig.html#
impl pallet_election_provider_multi_phase::BenchmarkingConfig for ElectionProviderBenchmarkConfig {
	const VOTERS: [u32; 2] = [1000, 2000];
	const TARGETS: [u32; 2] = [500, 1000];
	const ACTIVE_VOTERS: [u32; 2] = [500, 800];
	const DESIRED_TARGETS: [u32; 2] = [200, 400];
	const SNAPSHOT_MAXIMUM_VOTERS: u32 = 1000;
	const MINER_MAXIMUM_VOTERS: u32 = 1000;
	const MAXIMUM_TARGETS: u32 = 300;
}

/// Maximum number of iterations for balancing that will be executed in the embedded OCW
/// miner of election provider multi phase.
pub const MINER_MAX_ITERATIONS: u32 = 10;

/// A source of random balance for NposSolver, which is meant to be run by the OCW election miner.
pub struct OffchainRandomBalancing;
impl Get<Option<BalancingConfig>> for OffchainRandomBalancing {
	fn get() -> Option<BalancingConfig> {
		use sp_runtime::traits::TrailingZeroInput;
		let iterations = match MINER_MAX_ITERATIONS {
			0 => 0,
			max => {
				let seed = sp_io::offchain::random_seed();
				let random = <u32>::decode(&mut TrailingZeroInput::new(&seed))
					.expect("input is padded with zeroes; qed") %
					max.saturating_add(1);
				random as usize
			},
		};

		let config = BalancingConfig { iterations, tolerance: 0 };
		Some(config)
	}
}

pub struct OnChainSeqPhragmen;
impl onchain::Config for OnChainSeqPhragmen {
	type System = Runtime;
	type Solver = SequentialPhragmen<
		AccountId,
		pallet_election_provider_multi_phase::SolutionAccuracyOf<Runtime>,
	>;
	type DataProvider = <Runtime as pallet_election_provider_multi_phase::Config>::DataProvider;
	type WeightInfo = frame_election_provider_support::weights::SubstrateWeight<Runtime>;
	type MaxWinners = <Runtime as pallet_election_provider_multi_phase::Config>::MaxWinners;
	type VotersBound = MaxOnChainElectingVoters;
	type TargetsBound = MaxOnChainElectableTargets;
}

/// Configurations for a miner that comes with this pallet.
///
/// Source: https://paritytech.github.io/substrate/master/pallet_election_provider_multi_phase/unsigned/trait.MinerConfig.html#
impl pallet_election_provider_multi_phase::MinerConfig for Runtime {
	type AccountId = AccountId;
	/// Maximum length of the solution that the miner is allowed to generate.
	///
	/// Solutions are trimmed to respect this.
	type MaxLength = MinerMaxLength;
	/// Maximum weight of the solution that the miner is allowed to generate.
	///
	/// Solutions are trimmed to respect this.
	///
	/// The weight is computed using `solution_weight`.
	type MaxWeight = MinerMaxWeight;
	/// The solution that the miner is mining.
	type Solution = NposSolution16;
	/// Maximum number of votes per voter in the snapshots.
	type MaxVotesPerVoter =
	<<Self as pallet_election_provider_multi_phase::Config>::DataProvider as ElectionDataProvider>::MaxVotesPerVoter;

	// The unsigned submissions have to respect the weight of the submit_unsigned call, thus their
	// weight estimate function is wired to this call's weight.
	fn solution_weight(v: u32, t: u32, a: u32, d: u32) -> Weight {
		<
		<Self as pallet_election_provider_multi_phase::Config>::WeightInfo
		as
		pallet_election_provider_multi_phase::WeightInfo
		>::submit_unsigned(v, t, a, d)
	}
}

/// # Multi phase, offchain election provider pallet.
///
/// Currently, this election-provider has two distinct phases (see [`pallet_election_provider_multi_phase::Phase`]), **signed** and
/// **unsigned**.
///
/// ## Phases
///
/// The timeline of pallet is as follows. At each block,
/// [`frame_election_provider_support::ElectionDataProvider::next_election_prediction`] is used to
/// estimate the time remaining to the next call to
/// [`frame_election_provider_support::ElectionProvider::elect`]. Based on this, a phase is chosen.
/// The timeline is as follows.
///
/// ```ignore
///                                                                    elect()
///                 +   <--T::SignedPhase-->  +  <--T::UnsignedPhase-->   +
///   +-------------------------------------------------------------------+
///    Phase::Off   +       Phase::Signed     +      Phase::Unsigned      +
/// ```
///
/// Note that the unsigned phase starts [`pallet_election_provider_multi_phase::Config::UnsignedPhase`] blocks before the
/// `next_election_prediction`, but only ends when a call to [`pallet_election_provider_multi_phase::ElectionProvider::elect`] happens. If
/// no `elect` happens, the signed phase is extended.
///
/// > Given this, it is rather important for the user of this pallet to ensure it always terminates
/// election via `elect` before requesting a new one.
///
/// Each of the phases can be disabled by essentially setting their length to zero. If both phases
/// have length zero, then the pallet essentially runs only the fallback strategy, denoted by
/// [`Config::Fallback`].
///
/// ### Fallback
///
/// If we reach the end of both phases (i.e. call to [`pallet_election_provider_multi_phase::ElectionProvider::elect`] happens) and no
/// good solution is queued, then the fallback strategy [`pallet_election_provider_multi_phase::Config::Fallback`] is used to
/// determine what needs to be done. The on-chain election is slow, and contains no balancing or
/// reduction post-processing. If [`pallet_election_provider_multi_phase::Config::Fallback`] fails, the next phase
/// [`Phase::Emergency`] is enabled, which is a more *fail-safe* approach.
///
/// # Footnote:
///
/// Check out the diagram here in https://www.youtube.com/watch?v=qVd9lAudynY&t=2502s to see how the "Election Provider" (which is `pallet_election_provider_multi_phase`)
/// communicates with "Election Data Provider" (which is `pallet_staking`) and "Election Consumer" (which is also `pallet_staking`).
///
/// As you can see in the diagram, "Election Provider" expects the "Election Consumer" to call the `elect()` function of the "Election Provider" which returns the winners of the election (i.e the nominated/winning validators)
///
/// Source: https://paritytech.github.io/substrate/master/pallet_election_provider_multi_phase/index.html#
impl pallet_election_provider_multi_phase::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	/// Currency type.
	type Currency = Frag;
	/// Something that can predict the fee of a call. Used to sensibly distribute rewards.
	type EstimateCallFee = TransactionPayment;
	/// Duration of the signed phase.
	type SignedPhase = SignedPhase;
	/// Duration of the unsigned phase.
	type UnsignedPhase = UnsignedPhase;
	/// The minimum amount of improvement to the solution score that defines a solution as
	/// "better" in the Unsigned phase.
	type BetterUnsignedThreshold = BetterUnsignedThreshold;
	/// The minimum amount of improvement to the solution score that defines a solution as
	/// "better" in the Signed phase.
	type BetterSignedThreshold = ();
	type OffchainRepeat = OffchainRepeat;
	/// The priority of the unsigned transaction submitted in the unsigned-phase
	type MinerTxPriority = MultiPhaseUnsignedPriority;
	/// Configurations of the embedded miner.
	///
	/// Any external software implementing this can use the [`pallet_election_provider_multi_phase::unsigned::Miner`] type provided,
	/// which can mine new solutions and trim them accordingly.
	type MinerConfig = Self;
	type SignedMaxSubmissions = ConstU32<10>;
	/// Base reward for a signed solution
	type SignedRewardBase = SignedRewardBase;
	/// Base deposit for a signed solution.
	type SignedDepositBase = SignedDepositBase;
	/// Per-byte deposit for a signed solution.
	type SignedDepositByte = SignedDepositByte;
	/// The maximum amount of unchecked solutions to refund the call fee for.
	type SignedMaxRefunds = ConstU32<3>;
	/// Per-weight deposit for a signed solution.
	type SignedDepositWeight = ();
	/// Maximum weight of a signed solution.
	///
	/// If [`pallet_election_provider_multi_phase::Config::MinerConfig`] is being implemented to submit signed solutions (outside of
	/// this pallet), then [`MinerConfig::solution_weight`] is used to compare against
	/// this value.
	type SignedMaxWeight = MinerMaxWeight;
	/// Handler for the slashed deposits.
	type SlashHandler = (); // burn slashes
	/// Handler for the rewards.
	type RewardHandler = (); // nothing to do upon rewards
	/// Something that will provide the election data.
	type DataProvider = Staking;
	/// Configuration for the fallback.
	type Fallback = onchain::OnChainExecution<OnChainSeqPhragmen>;
	/// Configuration of the governance-only fallback.
	///
	/// As a side-note, it is recommend for test-nets to use `type ElectionProvider =
	/// BoundedExecution<_>` if the test-net is not expected to have thousands of nominators.
	type GovernanceFallback = onchain::OnChainExecution<OnChainSeqPhragmen>;
	/// OCW election solution miner algorithm implementation.
	type Solver = SequentialPhragmen<AccountId, pallet_election_provider_multi_phase::SolutionAccuracyOf<Self>, OffchainRandomBalancing>;
	/// Origin that can control this pallet. Note that any action taken by this origin (such)
	/// as providing an emergency solution is not checked. Thus, it must be a trusted origin.
	type ForceOrigin = EnsureRootOrHalfCouncil;
	/// The maximum number of electable targets to put in the snapshot.
	type MaxElectableTargets = MaxElectableTargets;
	/// The maximum number of winners that can be elected by this `ElectionProvider`
	/// implementation.
	///
	/// Note: This must always be greater or equal to `T::DataProvider::desired_targets()`.
	type MaxWinners = MaxActiveValidators;
	/// The maximum number of electing voters to put in the snapshot. At the moment, snapshots
	/// are only over a single block, but once multi-block elections are introduced they will
	/// take place over multiple blocks.
	type MaxElectingVoters = MaxElectingVoters;
	/// The configuration of benchmarking.
	type BenchmarkingConfig = ElectionProviderBenchmarkConfig;
	type WeightInfo = pallet_election_provider_multi_phase::weights::SubstrateWeight<Self>;
}

parameter_types! {
	/// A Session is defined to have the same duration as an Epoch in Fragnova, just like in Polkadot: https://wiki.polkadot.network/docs/maintain-polkadot-parameters
	///
	/// Note: "Conceptually, a session and epoch are different. Pallet-Babe/Pallet-Aura only cares about epoch and slots,
	/// 	   and Pallet-Grandpa/Pallet-Staking only cares about eras and sessions" - https://www.youtube.com/watch?v=-PSPfbAcpiQ&t=459s
	pub const SessionPeriod: BlockNumber = EPOCH_DURATION_IN_BLOCKS;
	/// The first session will have length of `Offset`, and
	/// the following sessions will have length of `SessionPeriod`.
	/// This may prove nonsensical if `Offset` >= `SessionPeriod`.
	///
	/// Source: https://paritytech.github.io/substrate/master/pallet_session/struct.PeriodicSessions.html#
	pub const Offset: BlockNumber = 0;
}

/// The Session pallet allows validators to manage their session keys, provides a function for changing the session length, and handles session rotation.
///
/// ## Overview
///
/// ### Terminology
///
/// - **Session:** A session is a period of time that has a constant set of validators. Validators
///   can only join or exit the validator set at a session change. It is measured in block numbers.
///   The block where a session is ended is determined by the `ShouldEndSession` trait. When the
///   session is ending, a new validator set can be chosen by `OnSessionEnding` implementations.
///
/// - **Session key:** A session key is actually several keys kept together that provide the various
///   signing functions required by network authorities/validators in pursuit of their duties.
/// - **Validator ID:** Every account has an associated validator ID. For some simple staking
///   systems, this may just be the same as the account ID. For staking systems using a
///   stash/controller model, the validator ID would be the stash account ID of the controller.
///
/// ### Goals
///
/// The Session pallet is designed to make the following possible:
///
/// - Set session keys of the validator set for upcoming sessions.
/// - Control the length of sessions.
/// - Configure and switch between either normal or exceptional session rotations.
///
/// ## Usage
///
/// ### Example from the FRAME
///
/// The [Staking pallet](../pallet_staking/index.html) uses the Session pallet to get the validator
/// set.
///
/// ```
/// use pallet_session as session;
///
/// fn validators<T: pallet_session::Config>() -> Vec<<T as pallet_session::Config>::ValidatorId> {
/// 	<pallet_session::Pallet<T>>::validators()
/// }
/// # fn main(){}
/// ```
///
/// Source: https://paritytech.github.io/substrate/master/pallet_session/index.html#
impl pallet_session::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	/// A stable ID for a validator.
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	/// A conversion from account ID to validator ID.
	///
	/// Its cost must be at most one storage read.
	type ValidatorIdOf = pallet_staking::StashOf<Self>;
	/// Indicator for when to end the session.
	type ShouldEndSession = pallet_session::PeriodicSessions<SessionPeriod, Offset>; // Babe;
	/// Something that can predict the next session rotation. This should typically come from
	/// the same logical unit that provides [`ShouldEndSession`], yet, it gives a best effort
	/// estimate. It is helpful to implement [`EstimateNextNewSession`].
	type NextSessionRotation = pallet_session::PeriodicSessions<SessionPeriod, Offset>; // Babe;
	/// Handler for managing new session.
	type SessionManager = pallet_session::historical::NoteHistoricalRoot<Self, Staking>;
	/// Handler when a session has changed.
	type SessionHandler = <opaque::SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
	/// The keys.
	type Keys = opaque::SessionKeys;
	type WeightInfo = pallet_session::weights::SubstrateWeight<Runtime>;
}

/// `pallet_session::historical::pallet::Config` (re-exported as `pallet_session::historical::Config`) is
/// the **Config necessary for the historical pallet.**
///
/// Source: https://paritytech.github.io/substrate/master/pallet_session/historical/pallet/trait.Config.html#
///
/// # Footnote
///
/// `pallet_session::historical` is an opt-in utility for tracking historical sessions in FRAME-session.
///
/// This is generally useful when implementing blockchains that require accountable
/// safety where validators from some amount f prior sessions must remain slashable.
///
/// Rather than store the full session data for any given session, we instead commit
/// to the roots of merkle tries containing the session data.
///
/// These roots and proofs of inclusion can be generated at any time during the current session.
/// Afterwards, the proofs can be fed to a consensus module when reporting misbehavior.
///
/// Source: https://paritytech.github.io/substrate/master/pallet_session/historical/index.html#
impl pallet_session::historical::Config for Runtime {
	type FullIdentification = pallet_staking::Exposure<AccountId, Balance>;
	type FullIdentificationOf = pallet_staking::ExposureOf<Runtime>;
}

// Accepts a number of expressions to create a instance of PiecewiseLinear which represents the
// NPoS curve (as detailed
// [here](https://research.web3.foundation/en/latest/polkadot/overview/2-token-economics.html#inflation-model))
// for those parameters.
//
// Parameters are:
// - `min_inflation`: the minimal amount to be rewarded between validators, expressed as a fraction
//   of total issuance. Known as `I_0` in the literature. Expressed in millionth, must be between 0
//   and 1_000_000.
//
// - `max_inflation`: the maximum amount to be rewarded between validators, expressed as a fraction
//   of total issuance. This is attained only when `ideal_stake` is achieved. Expressed in
//   millionth, must be between min_inflation and 1_000_000.
//
// - `ideal_stake`: the fraction of total issued tokens that should be actively staked behind
//   validators. Known as `x_ideal` in the literature. Expressed in millionth, must be between
//   0_100_000 and 0_900_000.
//
// - `falloff`: Known as `decay_rate` in the literature. A co-efficient dictating the strength of
//   the global incentivization to get the `ideal_stake`. A higher number results in less typical
//   inflation at the cost of greater volatility for validators. Expressed in millionth, must be
//   between 0 and 1_000_000.
//
// - `max_piece_count`: The maximum number of pieces in the curve. A greater number uses more
//   resources but results in higher accuracy. Must be between 2 and 1_000.
//
// - `test_precision`: The maximum error allowed in the generated test. Expressed in millionth,
//   must be between 0 and 1_000_000.
//
// Source: https://paritytech.github.io/substrate/master/pallet_staking_reward_curve/macro.build.html#
pallet_staking_reward_curve::build! {
	const REWARD_CURVE: PiecewiseLinear<'static> = curve!(
		min_inflation: 0_025_000,
		max_inflation: 0_100_000,
		ideal_stake: 0_500_000,
		falloff: 0_050_000,
		max_piece_count: 40,
		test_precision: 0_005_000,
	);
}

parameter_types! {
	pub const SessionsPerEra: sp_staking::SessionIndex = 6;
	pub const BondingDuration: sp_staking::EraIndex = 24 * 28; // the bonding period is 28 days
	pub const SlashDeferDuration: sp_staking::EraIndex = 24 * 7; // 1/4 the bonding duration.
	pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
	pub const MaxNominatorRewardedPerValidator: u32 = 256;
	pub const OffendingValidatorsThreshold: Perbill = Perbill::from_percent(17);
	pub OffchainRepeat: BlockNumber = 5;
	pub HistoryDepth: u32 = 84;
}

/// A reasonable benchmarking config for staking pallet.
pub struct StakingBenchmarkingConfig;
impl pallet_staking::BenchmarkingConfig for StakingBenchmarkingConfig {
	type MaxNominators = ConstU32<1000>;
	type MaxValidators = ConstU32<1000>;
}

/// The Staking pallet is used to manage funds at stake by network maintainers.
///
/// ## Overview
///
/// The Staking pallet is the means by which a set of network maintainers (known as _authorities_ in
/// some contexts and _validators_ in others) are chosen based upon those who voluntarily place
/// funds under deposit. Under deposit, those funds are rewarded under normal operation but are held
/// at pain of _slash_ (expropriation) should the staked maintainer be found not to be discharging
/// its duties properly.
///
/// ### Terminology
///
/// - Staking: The process of locking up funds for some time, placing them at risk of slashing
///   (loss) in order to become a rewarded maintainer of the network.
/// - Validating: The process of running a node to actively maintain the network, either by
///   producing blocks or guaranteeing finality of the chain.
/// - Nominating: The process of placing staked funds behind one or more validators in order to
///   share in any reward, and punishment, they take.
/// - Stash account: The account holding an owner's funds used for staking.
/// - Controller account: The account that controls an owner's funds for staking.
/// - Era: A (whole) number of sessions, which is the period that the validator set (and each
///   validator's active nominator set) is recalculated and where rewards are paid out.
/// - Slash: The punishment of a staker by reducing its funds.
///
/// ### Goals
///
/// The staking system in Substrate NPoS is designed to make the following possible:
///
/// - Stake funds that are controlled by a cold wallet.
/// - Withdraw some, or deposit more, funds without interrupting the role of an entity.
/// - Switch between roles (nominator, validator, idle) with minimal overhead.
///
/// ### Era payout
///
/// The era payout is computed using yearly inflation curve defined at
/// [`Config::EraPayout`] as such:
///
/// ```nocompile
/// staker_payout = yearly_inflation(npos_token_staked / total_tokens) * total_tokens / era_per_year
/// ```
/// This payout is used to reward stakers as defined in next section
///
/// ```nocompile
/// remaining_payout = max_yearly_inflation * total_tokens / era_per_year - staker_payout
/// ```
/// The remaining reward is send to the configurable end-point
/// [`Config::RewardRemainder`].
///
///
///
/// Source: https://paritytech.github.io/substrate/master/pallet_staking/index.html#
impl pallet_staking::Config for Runtime {
	/// Maximum number of nominations per nominator.
	type MaxNominations = MaxNominations;
	/// The staking balance.
	type Currency = Frag;
	/// Just the `Currency::Balance` type; we have this item to allow us to constrain it to
	/// `From<u64>`.
	type CurrencyBalance = Balance;
	/// Time used for computing era duration.
	///
	/// It is guaranteed to start being called from the first `on_finalize`. Thus value at
	/// genesis is not used.
	type UnixTime = Timestamp;
	/// Convert a balance into a number used for election calculation. This must fit into a
	/// `u64` but is allowed to be sensibly lossy. The `u64` is used to communicate with the
	/// [`frame_election_provider_support`] crate which accepts u64 numbers and does operations
	/// in 128.
	/// Consequently, the backward convert is used convert the u128s from sp-elections back to a
	/// [`BalanceOf`].
	type CurrencyToVote = U128CurrencyToVote;
	/// Tokens have been minted and are unused for validator-reward.
	/// See [Era payout](https://paritytech.github.io/substrate/master/pallet_staking/index.html#era-payout).
	type RewardRemainder = Treasury;
	type RuntimeEvent = RuntimeEvent;
	type Slash = Treasury; // send the slashed funds to the treasury.
	type Reward = (); // rewards are minted from the void
	/// Number of sessions per era.
	type SessionsPerEra = SessionsPerEra;
	/// Number of eras that staked funds must remain bonded for.
	type BondingDuration = BondingDuration;
	/// Number of eras that slashes are deferred by, after computation.
	///
	/// This should be less than the bonding duration. Set to 0 if slashes
	/// should be applied immediately, without opportunity for intervention.
	type SlashDeferDuration = SlashDeferDuration;
	/// A super-majority of the council can cancel the slash.
	type AdminOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 3, 4>,
	>;
	/// Interface for interacting with a session pallet.
	type SessionInterface = Self;
	/// The payout for validators and the system for the current era.
	/// See [Era payout](https://paritytech.github.io/substrate/master/pallet_staking/index.html#era-payout).
	type EraPayout = pallet_staking::ConvertCurve<RewardCurve>;
	/// Something that can estimate the next session change, accurately or as a best effort
	/// guess.
	type NextNewSession = Session;
	/// The maximum number of nominators rewarded for each validator.
	///
	/// For each validator only the `$MaxNominatorRewardedPerValidator` biggest stakers can
	/// claim their reward. This used to limit the i/o cost for the nominator payout.
	type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
	/// The fraction of the validator set that is safe to be offending.
	/// After the threshold is reached a new era will be forced.
	type OffendingValidatorsThreshold = OffendingValidatorsThreshold;
	/// Something that provides the election functionality.
	type ElectionProvider = ElectionProviderMultiPhase;
	/// Something that provides the election functionality at genesis.
	type GenesisElectionProvider = onchain::OnChainExecution<OnChainSeqPhragmen>;
	/// Something that provides a best-effort sorted list of voters aka electing nominators,
	/// used for NPoS election.
	///
	/// The changes to nominators are reported to this. Moreover, each validator's self-vote is
	/// also reported as one independent vote.
	///
	/// To keep the load off the chain as much as possible, changes made to the staked amount
	/// via rewards and slashes are not reported and thus need to be manually fixed by the
	/// staker. In case of `bags-list`, this always means using `rebag` and `putInFrontOf`.
	///
	/// Invariant: what comes out of this list will always be a nominator.
	type VoterList = VoterList;
	// This a placeholder, to be introduced in the next PR as an instance of bags-list
	type TargetList = pallet_staking::UseValidatorsMap<Self>;
	/// The maximum number of `unlocking` chunks a [`StakingLedger`] can
	/// have. Effectively determines how many unique eras a staker may be
	/// unbonding in.
	///
	/// Note: `MaxUnlockingChunks` is used as the upper bound for the
	/// `BoundedVec` item `StakingLedger.unlocking`. Setting this value
	/// lower than the existing value can lead to inconsistencies in the
	/// `StakingLedger` and will need to be handled properly in a runtime
	/// migration. The test `reducing_max_unlocking_chunks_abrupt` shows
	/// this effect.
	type MaxUnlockingChunks = ConstU32<32>;
	/// Number of eras to keep in history.
	type HistoryDepth = HistoryDepth;
	/// A hook called when any staker is slashed. Mostly likely this can be a no-op unless
	/// other pallets exist that are affected by slashing per-staker.
	type OnStakerSlash = NominationPools;
	type WeightInfo = pallet_staking::weights::SubstrateWeight<Runtime>;
	/// Some parameters of the benchmarking.
	type BenchmarkingConfig = StakingBenchmarkingConfig;
}

parameter_types! {
	pub const BagThresholds: &'static [u64] = &voter_bags::THRESHOLDS;
}

type VoterBagsListInstance = pallet_bags_list::Instance1;
/// A semi-sorted list, where items hold an `AccountId` based on some `Score`. The
/// `AccountId` (`id` for short) might be synonym to a `voter` or `nominator` in some context, and
/// `Score` signifies the chance of each id being included in the final
/// `SortedListProvider::iter`.
///
/// Source: https://paritytech.github.io/substrate/master/pallet_bags_list/index.html#
impl pallet_bags_list::Config<VoterBagsListInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	/// The voter bags-list is loosely kept up to date, and the real source of truth for the score
	/// of each node is the staking pallet.
	type ScoreProvider = Staking;
	/// The list of thresholds separating the various bags.
	type BagThresholds = BagThresholds;
	/// The type used to dictate a node position relative to other nodes.
	type Score = frame_election_provider_support::VoteWeight;
	type WeightInfo = pallet_bags_list::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const PostUnbondPoolsWindow: u32 = 4;
	pub const NominationPoolsPalletId: PalletId = PalletId(*b"py/nopls");
	pub const MaxPointsToBalance: u8 = 10;
}

use sp_runtime::traits::Convert;
pub struct BalanceToU256;
impl Convert<Balance, sp_core::U256> for BalanceToU256 {
	fn convert(balance: Balance) -> sp_core::U256 {
		sp_core::U256::from(balance)
	}
}
pub struct U256ToBalance;
impl Convert<sp_core::U256, Balance> for U256ToBalance {
	fn convert(n: sp_core::U256) -> Balance {
		n.try_into().unwrap_or(Balance::max_value())
	}
}

/// A pallet that allows members to delegate their stake to nominating pools. A nomination pool acts
/// as nominator and nominates validators on the members behalf.
///
/// Source: https://paritytech.github.io/substrate/master/pallet_nomination_pools/index.html#
impl pallet_nomination_pools::Config for Runtime {
	type WeightInfo = ();
	type RuntimeEvent = RuntimeEvent;
	/// The nominating balance.
	type Currency = Frag;
	/// The type that is used for reward counter.
	type RewardCounter = FixedU128;
	/// Infallible method for converting `Currency::Balance` to `U256`.
	type BalanceToU256 = BalanceToU256;
	/// Infallible method for converting `U256` to `Currency::Balance`.
	type U256ToBalance = U256ToBalance;
	/// The interface for nominating.
	type Staking = Staking;
	/// The amount of eras a `SubPools::with_era` pool can exist before it gets merged into the
	/// `SubPools::no_era` pool. In other words, this is the amount of eras a member will be
	/// able to withdraw from an unbonding pool which is guaranteed to have the correct ratio of
	/// points to balance; once the `with_era` pool is merged into the `no_era` pool, the ratio
	/// can become skewed due to some slashed ratio getting merged in at some point.
	type PostUnbondingPoolsWindow = PostUnbondPoolsWindow;
	/// The maximum length, in bytes, that a pools metadata maybe.
	type MaxMetadataLen = ConstU32<256>;
	/// The maximum number of simultaneous unbonding chunks that can exist per member.
	type MaxUnbonding = ConstU32<8>;
	/// The nomination pool's pallet id.
	type PalletId = NominationPoolsPalletId;
	/// The maximum pool points-to-balance ratio that an `open` pool can have.
	///
	/// This is important in the event slashing takes place and the pool's points-to-balance
	/// ratio becomes disproportional.
	///
	/// Moreover, this relates to the `RewardCounter` type as well, as the arithmetic operations
	/// are a function of number of points, and by setting this value to e.g. 10, you ensure
	/// that the total number of points in the system are at most 10 times the total_issuance of
	/// the chain, in the absolute worse case.
	///
	/// For a value of 10, the threshold would be a pool points-to-balance ratio of 10:1.
	/// Such a scenario would also be the equivalent of the pool being 90% slashed.
	type MaxPointsToBalance = MaxPointsToBalance;
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
		RandomnessCollectiveFlip: pallet_insecure_randomness_collective_flip,
		Timestamp: pallet_timestamp,
		Aura: pallet_aura,
		Grandpa: pallet_grandpa,
		Balances: pallet_balances,
		Frag: pallet_balances::<Instance2>,
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

		// Authorship must be before session in order to note author in the correct session and era
		// for im-online and staking.
		Authorship: pallet_authorship,
		ImOnline: pallet_im_online,
		Offences: pallet_offences,
		Historical: pallet_session_historical::{Pallet},

		Council: pallet_collective::<Instance1>,
		Treasury: pallet_treasury,
		Bounties: pallet_bounties,
		ChildBounties: pallet_child_bounties,
		AuthorityDiscovery: pallet_authority_discovery,
		ElectionProviderMultiPhase: pallet_election_provider_multi_phase,
		Staking: pallet_staking,
		Session: pallet_session,
		VoterList: pallet_bags_list::<Instance1>,
		NominationPools: pallet_nomination_pools,
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
/// Notice that in any signed transaction/extrinsic that is sent to Fragnova, it will the extra data🥕 "era", "nonce" and "tip": https://polkadot.js.org/apps/#/extrinsics/decode/0xf5018400d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d0150cc530a8f70343680c46687ade61e1e4cdc0dfc6d916c3143828dc588938c1934030b530d9117001260426798d380306ea3a9d04fe7b525a33053a1c31bee86750200000b000000000000003448656c6c6f2c20576f726c6421
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
pub type UncheckedExtrinsic =
	generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
/// The payload being signed in transactions.
///
/// Note: This type is only needed if you want to enable an off-chain worker for the runtime,
/// since it is only used when implementing the trait `frame_system::offchain::CreateSignedTransaction` for `Runtime`.
pub type SignedPayload = generic::SignedPayload<RuntimeCall, SignedExtra>;
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
	///
	/// # Footnote
	///
	/// ## Terminology
	///
	/// - **Session:** A session is a period of time that has a constant set of validators. Validators
	///   can only join or exit the validator set at a session change. It is measured in block numbers.
	///
	/// - **Session key:** A session key is actually several keys kept together that provide the various
	///   signing functions required by network authorities/validators in pursuit of their duties.
	///
	/// Source: https://paritytech.github.io/substrate/master/pallet_session/index.html#
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
			equivocation_proof: fg_primitives::EquivocationProof<
				<Block as BlockT>::Hash,
				NumberFor<Block>,
			>,
			key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			let key_owner_proof = key_owner_proof.decode()?;

			Grandpa::submit_unsigned_equivocation_report(
				equivocation_proof,
				key_owner_proof,
			)
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
			authority_id: GrandpaId,
		) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
			use codec::Encode;

			Historical::prove((fg_primitives::KEY_TYPE, authority_id))
				.map(|p| p.encode())
				.map(fg_primitives::OpaqueKeyOwnershipProof::new)
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
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentCallApi<Block, Balance, RuntimeCall>
		for Runtime
	{
		fn query_call_info(
			call: RuntimeCall,
			len: u32,
		) -> pallet_transaction_payment::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_call_info(call, len)
		}
		fn query_call_fee_details(
			call: RuntimeCall,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_call_fee_details(call, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}



	/// The Runtime API used to dry-run contract interactions.
	///
	/// See: https://paritytech.github.io/substrate/master/pallet_contracts/trait.ContractsApi.html#
	impl pallet_contracts::ContractsApi<Block, AccountId, Balance, BlockNumber, Hash> for Runtime {
		/// Perform a call from a specified account to a given contract.
		///
		/// See [`crate::Pallet::bare_call`].
		fn call(
			origin: AccountId,
			dest: AccountId,
			value: Balance,
			gas_limit: Option<Weight>,
			storage_deposit_limit: Option<Balance>,
			input_data: Vec<u8>,
		) -> pallet_contracts_primitives::ContractExecResult<Balance> {
			let gas_limit = gas_limit.unwrap_or(BlockWeights::get().max_block);
			Contracts::bare_call(
				origin,
				dest,
				value,
				gas_limit,
				storage_deposit_limit,
				input_data,
				CONTRACTS_DEBUG_OUTPUT,
				pallet_contracts::Determinism::Deterministic
			)
		}

		/// Instantiate a new contract.
		///
		/// See `[crate::Pallet::bare_instantiate]`.
		fn instantiate(
			origin: AccountId,
			value: Balance,
			gas_limit: Option<Weight>,
			storage_deposit_limit: Option<Balance>,
			code: pallet_contracts_primitives::Code<Hash>,
			data: Vec<u8>,
			salt: Vec<u8>,
		) -> pallet_contracts_primitives::ContractInstantiateResult<AccountId, Balance>
		{
			let gas_limit = gas_limit.unwrap_or(BlockWeights::get().max_block);
			Contracts::bare_instantiate(
				origin,
				value,
				gas_limit,
				storage_deposit_limit,
				code,
				data,
				salt,
				CONTRACTS_DEBUG_OUTPUT
			)
		}

		/// Upload new code without instantiating a contract from it.
		///
		/// See [`crate::Pallet::bare_upload_code`].
		fn upload_code(
			origin: AccountId,
			code: Vec<u8>,
			storage_deposit_limit: Option<Balance>,
			determinism: pallet_contracts::Determinism,
		) -> pallet_contracts_primitives::CodeUploadResult<Hash, Balance>
		{
			Contracts::bare_upload_code(origin, code, storage_deposit_limit, determinism)
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
