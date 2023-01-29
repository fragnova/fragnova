// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use crate::{Hash256};
use codec::{Compact, Decode, Encode};
use sp_std::{
	collections::btree_map::BTreeMap,
};
use protos::permissions::FragmentPerms;

/// Type used to represent an Instance's Edition ID and an Instance's Copy ID
pub type InstanceUnit = u64;

/// Enum can be used to represent a currency that exists on the Fragnova Blockchain
#[derive(Encode, Decode, Copy, Clone, scale_info::TypeInfo, Debug, PartialEq)] // REVIEW - should it implement the trait `Copy`?
pub enum Currency<TFungibleAsset> {
	/// Fragnova's Native Currency (i.e NOVA token)
	Native,
	/// A Custom Currency
	Custom(TFungibleAsset),
}

/// **Struct** of a **Fragment Definition's Metadata**
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq)]
pub struct DefinitionMetadata<TU8Vector, TFungibleAsset> {
	/// **Name** of the **Fragment Definition**
	pub name: TU8Vector,
	/// **Currency** that must be used to buy **any and all Fragment Instances created from the Fragment Definition**
	pub currency: Currency<TFungibleAsset>,
}

/// TODO
/// **Enum** that represents the **settings** for a **Fragment Definition whose Fragment instance(s) must contain unique data when created**
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq)]
pub struct UniqueOptions {
	/// Whether the unique data of the Fragment instance(s) are mutable
	pub mutable: bool,
}

/// **Struct** of a **Fragment Definition**
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq)]
pub struct FragmentDefinition<TU8Array, TFungibleAsset, TAccountId, TBlockNum> {
	/// **Proto-Fragment used** to **create** the **Fragment**
	pub proto_hash: Hash256,
	/// ***DefinitionMetadata* Struct** (the **struct** contains the **Fragment Definition's name**, among other things)
	pub metadata: DefinitionMetadata<TU8Array, TFungibleAsset>,
	/// **Set of Actions** (encapsulated in a `FragmentPerms` bitflag enum) that are **allowed to be done** to
	/// **any Fragment Instance** when it **first gets created** from the **Fragment Definition** (e.g edit, transfer etc.)
	///
	/// These **allowed set of actions of the Fragment Instance** ***may change***
	/// when the **Fragment Instance is given to another account ID** (see the `give()` extrinsic).
	pub permissions: FragmentPerms,
	// Note: If Fragment Instances (created from the Fragment Definition) must contain unique data when created (injected by buyers, validated by the system)
	/// Whether the **Fragment Definition** is **mutable**
	pub unique: Option<UniqueOptions>,
	/// If scarce, the max supply of the Fragment
	pub max_supply: Option<Compact<InstanceUnit>>,
	/// The creator of this class
	pub creator: TAccountId,
	/// The block number when the item was created
	pub created_at: TBlockNum,
	/// **Map** that maps the **Key of a Proto-Fragment's Custom Metadata Object** to the **Hash of the aforementioned Custom Metadata Object**
	pub custom_metadata: BTreeMap<Compact<u64>, Hash256>,
}

/// **Struct** of a **Fragment Instance**
///
/// Footnotes:
///
/// #### Remarks
///
/// * On purpose not storing owner because:
///   * Big, 32 bytes
///   * Most of use cases will definitely already have the owner available when using this structure, as likely going thru `Inventory` etc.
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq)]
pub struct FragmentInstance<TBlockNum> {
	// Next owner permissions, owners can change those if they want to more restrictive ones, never more permissive
	/// **Set of Actions** (encapsulated in a `FragmentPerms` bitflag enum) **allowed to be done**
	/// to the **Fragment Instance** (e.g edit, transfer etc.)
	///
	/// These **allowed set of actions of the Fragment Instance** ***may change***
	/// when the **Fragment Instance is given to another account ID** (see the `give` extrinsic).
	pub permissions: FragmentPerms,
	/// Block number in which the Fragment Instance was created
	pub created_at: TBlockNum,
	/// Custom data, if unique, this is the hash of the data that can be fetched using bitswap directly on our nodes
	pub custom_data: Option<Hash256>,
	/// Block number that the Fragment Instance expires at (*optional*)
	pub expiring_at: Option<TBlockNum>,
	/// If the Fragment instance represents a **stack of stackable items** (for e.g gold coins or arrows - https://runescape.fandom.com/wiki/Stackable_items),
	/// the **number of items** that are **left** in the **stack of stackable items**
	pub stack_amount: Option<Compact<InstanceUnit>>,
	/// TODO: Documentation
	/// **Map** that maps the **Key of a Proto-Fragment's Metadata Object** to an **Index of the Hash of the aforementioned Metadata Object**
	pub metadata: BTreeMap<Compact<u64>, Compact<u64>>,
}
