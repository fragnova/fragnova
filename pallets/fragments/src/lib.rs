//! This pallet `fragments` performs logic related to Fragment Definitions and Fragment Instances
//!
//! IMPORTANT STUFF TO KNOW:
//!
//! # Fragment Definition
//!
//! A Fragment Definition is created using a Proto-Fragment (see pallet `protos`).
//! A Fragment Definition's ID can be determinstically computed using its Proto-Fragment hash and
//! its metadata struct `DefinitionMetadata`.
//!
//! A Fragment Definition is essentially a digital asset that can be used to enhance the user experience in a game or application,
//! like an in-game item or user account. A Fragment has its own storage, metadata and digital life on its own.
//!
//!
//! # Fragment Instance
//!
//! A Fragment Instance is created from a Fragment Definition.
//!
//! It is a digital asset that can be used to enhance the user experience in a game or application,
//! like an in-game item or user account.
//!
//! Each Fragment Instance also has an edition number.
//!
//! Therefore, a Fragment Instance can be uniquely identified using its Fragment Definition's ID, its Edition ID and its Copy ID.
//!
//! The Copy ID allows us to distinguish a Fragment Instance that has the same Fragment Definition ID and the same Edition ID.

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod dummy_data;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[allow(missing_docs)]
mod weights;

use codec::{Compact, Decode, Encode};
pub use pallet::*;
use sp_clamor::{Hash128, Hash256, InstanceUnit};
use sp_core::crypto::UncheckedFrom;
use sp_io::{
	hashing::{blake2_128, blake2_256},
	transaction_index,
};
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};
pub use weights::WeightInfo;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use protos::permissions::FragmentPerms;

use frame_support::dispatch::DispatchResult;
use sp_runtime::traits::StaticLookup;

use frame_support::traits::tokens::{
	fungible,
	fungibles,
	ExistenceRequirement,
};

use sp_runtime::SaturatedConversion;

use scale_info::prelude::{
	format,
	string::{String, ToString},
};
use serde_json::{json, Map, Value};

/// **Data Type** used to **Query and Filter for Fragment Definitions**
#[derive(Encode, Decode, Clone, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct GetDefinitionsParams<TAccountId, TString> {
	/// Whether to order the results in descending or ascending order
	pub desc: bool,
	/// Number of FD Results to skip
	pub from: u64,
	/// Number of FDs to retrieve
	pub limit: u64,
	/// List of Custom-Metadata Keys of the FD that should also be returned
	pub metadata_keys: Vec<TString>,
	/// Owner of the FD
	pub owner: Option<TAccountId>,
	/// Whether to return the owner(s) of all the returned FDs
	pub return_owners: bool,
	// pub categories: Vec<Categories>,
	// pub tags: Vec<TString>,
}
#[cfg(test)]
impl<TAccountId, TString> Default for GetDefinitionsParams<TAccountId, TString> {
	fn default() -> Self {
		Self {
			desc: Default::default(),
			from: Default::default(),
			limit: Default::default(),
			metadata_keys: Default::default(),
			owner: None,
			return_owners: false,
		}
	}
}

/// **Data Type** used to **Query and Filter for Fragment Instances**
#[derive(Encode, Decode, Clone, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct GetInstancesParams<TAccountId, TString> {
	/// Whether to order the results in descending or ascending order
	pub desc: bool,
	/// Number of FI Results to skip
	pub from: u64,
	/// Number of FIs to retrieve
	pub limit: u64,
	/// The Fragment Definition/Collection that all the FIs must be in
	pub definition_hash: TString,
	/// List of Metadata Keys of the FI that should also be returned
	pub metadata_keys: Vec<TString>,
	/// Owner of the FIs
	pub owner: Option<TAccountId>,
	/// Whether to only return FIs that have a Copy ID of 1
	pub only_return_first_copies: bool,
}
#[cfg(test)]
impl<TAccountId, TString: Default> Default for GetInstancesParams<TAccountId, TString> {
	fn default() -> Self {
		Self {
			desc: Default::default(),
			from: Default::default(),
			limit: Default::default(),
			definition_hash: Default::default(),
			metadata_keys: Default::default(),
			owner: None,
			only_return_first_copies: Default::default(),
		}
	}
}

/// **Data Type** used to **Query the owner of a Fragment Instance**
#[derive(Encode, Decode, Clone, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct GetInstanceOwnerParams<TString> {
	/// Fragment Definition/Collection that the Fragment Instance is in
	pub definition_hash: TString,
	/// Edition ID of the Fragment Instance
	pub edition_id: InstanceUnit,
	/// Copy ID of the Fragment Instance
	pub copy_id: InstanceUnit,
}

/// Enum can be used to represent a currency that exists on the Clamor Blockchain
#[derive(Encode, Decode, Copy, Clone, scale_info::TypeInfo, Debug, PartialEq)] // REVIEW - should it implement the trait `Copy`?
pub enum Currency<TFungibleAsset> {
	/// Clamor's Native Currency (i.e NOVA token)
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
	// Notes from Giovanni:
	//
	// If Fragment Instances (created from the Fragment Definition) must contain unique data when created (injected by buyers, validated by the system)
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

/// Struct **representing** a sale of the **Fragment Definition** .
///
/// Note: When a Fragment Definition is put on sale, users can create Fragment Instances from it for a fee.
///
/// Footnotes:
///
/// #### Remarks
///
///`price` is using `u128` and not `T::Balance` because the latter requires a whole lot of traits to be satisfied.. rust headakes.
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq)]
pub struct PublishingData<TBlockNum> {
	/// **Fee** that is **needed to be paid** to create a **single Fragment Instance** from the **Fragment Definition**
	pub price: Compact<u128>,
	/// **Amount of Fragment Instances** that **can be bought**
	pub units_left: Option<Compact<InstanceUnit>>,
	/// Block number that the sale ends at (*optional*)
	pub expiration: Option<TBlockNum>,
	/// If the Fragment instance represents a **stack of stackable items** (for e.g gold coins or arrows - https://runescape.fandom.com/wiki/Stackable_items),
	/// the **number of items** to **top up** in the **stack of stackable items**
	pub stack_amount: Option<Compact<InstanceUnit>>,
}

/// Enum indicating the different ways to put a Fragment Instance on sale.
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq, Eq)]
pub enum SecondarySaleType {
	/// Put the Fragment Instance on sale with with a fixed price (where the fixed price is specified in the associated `u128` value of this enum variant)
	Normal(u128),
	// /// Auction (Starting Price, Current Price, Timeout)
	// Auction(Compact<u128>, Compact<u128>, TBlockNum),
}
/// Struct representing the **Details of a Fragment Instance that is put on sale**
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq)]
pub struct SecondarySaleData<TAccountId, TBlockNum> {
	/// Current Owner of the Fragment Instance
	pub owner: TAccountId,
	/// New Permissions of the Fragment Instance after it is sold
	pub new_permissions: Option<FragmentPerms>,
	/// Block number that the newly-copied Fragment Instance expires at. If the Fragment Instance is not copyable, this field is practically irrelevant.
	pub expiration: Option<TBlockNum>,
	/// Type of Sale
	pub secondary_sale_type: SecondarySaleType,
}

/// Enum indicating the different ways that one can purchase a Fragment Instance
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq, Eq)]
pub enum SecondarySaleBuyOptions {
	/// Normal Purchase: Purchase the Fragment Instance based on the fixed-price that was defined by its seller
	Normal,
	// Auction (Bid Price)
	// Auction(Compact<u128>),
}

/// **Enum** indicating whether to
/// **create one Fragment Instance with custom data attached to it**
/// or whether to
/// **create multiple Fragment Instances (with no custom data attached to them)**
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq)]
pub enum FragmentBuyOptions {
	/// Create multiple Fragment Instances (where the number of Fragment Instances to create is specified in the associated `u64` value of this enum variant)
	Quantity(u64),
	/// Create a single Fragment Instance with some custom data attached to it (where the custom data is specified in the associated `Vec<u8>` value of this enum variant)
	UniqueData(Vec<u8>),
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, Twox64Concat};
	use frame_system::pallet_prelude::*;
	use pallet_protos::{MetaKeys, MetaKeysIndex, Proto, ProtoOwner, Protos, ProtosByOwner};
	use sp_clamor::get_vault_id;
	use pallet_detach::{DetachRequest, DetachRequests, DetachHash, DetachedHashes, SupportedChains};

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_protos::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// Weight functions needed for pallet_fragments.
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// **StorageMap** that maps a **Proto-Fragment**
	/// to a
	/// **list of Fragment Definitions that were created using the aforementioned Proto-Fragment**
	#[pallet::storage]
	pub type Proto2Fragments<T: Config> = StorageMap<_, Identity, Hash256, Vec<Hash128>>;

	// fragment-hash to fragment-data
	/// **StorageMap** that maps a **Fragment Definition ID (which is determinstically computed using its Proto-Fragment hash and its metadata struct `DefinitionMetadata`)**
	/// to a
	/// ***FragmentDefinition* struct**
	#[pallet::storage]
	pub type Definitions<T: Config> = StorageMap<
		_,
		Identity,
		Hash128,
		FragmentDefinition<Vec<u8>, T::AssetId, T::AccountId, T::BlockNumber>,
	>;

	/// **StorageMap** that maps a **Fragment Definition ID**
	/// to a
	/// ***PublishingData* struct (of the aforementioned Fragment Definition)**
	#[pallet::storage]
	pub type Publishing<T: Config> =
	StorageMap<_, Identity, Hash128, PublishingData<T::BlockNumber>>;

	/// **StorageNMap** that maps a
	/// **Fragment Instance's Fragment Definition ID, Edition ID and Copy ID**
	/// to a
	/// ***`SecondarySaleData`* struct**
	#[pallet::storage]
	pub type Definition2SecondarySales<T: Config> = StorageNMap<
		_,
		(
			// Definition Hash
			storage::Key<Identity, Hash128>,
			// Edition ID
			storage::Key<Identity, InstanceUnit>,
			// Copy ID
			storage::Key<Identity, InstanceUnit>,
		),
		SecondarySaleData<T::AccountId, T::BlockNumber>,
	>;

	/// **StorageMap** that maps a **Fragment Definition ID**
	/// to the
	/// **total number of unique Edition IDs** found in the
	/// **Fragment Instances that have the aforementioned Fragment Definition ID**
	#[pallet::storage]
	pub type EditionsCount<T: Config> = StorageMap<_, Identity, Hash128, Compact<InstanceUnit>>;

	/// **StorageMap** that maps a **tuple that contains a Fragment Definition ID and an Edition ID**
	/// to the
	/// **total number of Fragment Instances that have the Fragment Definition ID and the Edition ID**
	#[pallet::storage]
	pub type CopiesCount<T: Config> = StorageMap<_, Identity, (Hash128, InstanceUnit), Compact<InstanceUnit>>;

	/// **StorageNMap** that maps a
	/// **Fragment Instance's Fragment Definition ID, Edition ID and Copy ID**
	/// to a
	/// ***`FragmentInstance`* struct**
	///
	/// Footnotes:
	///
	///  #### Keys hashing reasoning
	///
	/// Very long key, means takes a lot of redundant storage (because we will have **many** Instances!), we try to limit the  damage by using `Identity` so that the final key will be:
	/// `[16 bytes of Fragment class hash]+[8 bytes of u64, edition]+[8 bytes of u64, copy id]` for a total of 32 bytes.
	#[pallet::storage]
	pub type Fragments<T: Config> = StorageNMap<
		_,
		// Keys are using Identity for compression, as we deteministically create fragments
		(
			storage::Key<Identity, Hash128>,
			// Editions
			storage::Key<Identity, InstanceUnit>,
			// Copies
			storage::Key<Identity, InstanceUnit>,
		),
		FragmentInstance<T::BlockNumber>,
	>;

	/// StorageMap that maps a **Fragment Definition and a ***Unique Data*** Hash**
	/// to **an Existing Edition of the aforementioned Fragment Definition**
	#[pallet::storage]
	pub type UniqueData2Edition<T: Config> = StorageDoubleMap<
		_,
		Identity,
		Hash128, // Fragment Definition ID
		Identity,
		Hash256, // Unique Data's Hash
		InstanceUnit,    // Edition ID
	>;

	/// StorageDoubleMap that maps a **Fragment Definition and a Clamor Account ID**
	/// to a
	/// **list of Fragment Instances of the Fragment Definition that is owned by the Clamor Account ID**
	///
	/// This storage item stores the exact same thing as `Inventory`, except that the primary key and the secondary key are swapped
	///
	/// Footnotes:
	///
	/// Notice this pulls from memory (and deserializes (scale)) the whole `Vec<_,_>`, this is on purpose as it should not be too big.
	#[pallet::storage]
	pub type Owners<T: Config> = StorageDoubleMap<
		_,
		Identity,
		Hash128,
		Twox64Concat,
		T::AccountId,
		Vec<(Compact<InstanceUnit>, Compact<InstanceUnit>)>,
	>;

	/// StorageDoubleMap that maps a **Clamor Account ID and a Fragment Definition**
	/// to a
	/// **list of Fragment Instances of the Fragment Definition that is owned by the Clamor Account ID**
	///
	/// This storage item stores the exact same thing as `Owners`, except that the primary key and the secondary key are swapped
	///
	/// Footnotes:
	///
	/// Notice this pulls from memory (and deserializes (scale)) the whole `Vec<_,_>`, this is on purpose as it should not be too big.
	#[pallet::storage]
	pub type Inventory<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::AccountId,
		Identity,
		Hash128,
		Vec<(Compact<InstanceUnit>, Compact<InstanceUnit>)>,
	>;

	/// StorageMap that maps the **Block Number**
	/// to a
	/// **list of Fragment Instances that expire on that Block
	/// (note: each FI in the list is represented as a tuple that contains the Fragment Instance's Fragment Definition ID, the Fragment Instance's Edition ID and
	/// the Fragment Instance's Copy ID)**
	///
	/// Footnotes:
	///
	///  Fragment Instances can expire, we process expirations every `on_finalize`
	#[pallet::storage]
	pub type Expirations<T: Config> =
	StorageMap<_, Twox64Concat, T::BlockNumber, Vec<(Hash128, Compact<InstanceUnit>, Compact<InstanceUnit>)>>;

	/// **StorageMap** that maps a **Fragment Definition ID and a Number** to a **Data Hash**
	#[pallet::storage]
	pub type DataHashMap<T: Config> =
	StorageDoubleMap<_, Identity, Hash128, Identity, Compact<u64>, Hash256>;
	/// **StorageMap** that maps a **Fragment Definition ID** to the **total number of "Numbers" (see `DataHashMap` to understand what "Numbers" means) that fall under it**
	#[pallet::storage]
	pub type DataHashMapIndex<T: Config> = StorageMap<_, Identity, Hash128, u64>;

	#[allow(missing_docs)]
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New definition created by account, definition hash
		DefinitionCreated { definition_hash: Hash128 },
		/// A Fragment Definition metadata has changed
		DefinitionMetadataChanged { definition_hash: Hash128, metadata_key: Vec<u8> },
		/// A Fragment Instance metadata has changed
		InstanceMetadataChanged {
			definition_hash: Hash128,
			edition_id: InstanceUnit,
			copy_id: InstanceUnit,
			metadata_key: Vec<u8>,
		},
		/// Fragment sale has been opened
		Publishing { definition_hash: Hash128 },
		/// Fragment sale has been closed
		Unpublishing { definition_hash: Hash128 },
		/// Inventory item has been added to account
		InventoryAdded {
			account_id: T::AccountId,
			definition_hash: Hash128,
			fragment_id: (InstanceUnit, InstanceUnit),
		},
		/// Inventory item has removed added from account
		InventoryRemoved {
			account_id: T::AccountId,
			definition_hash: Hash128,
			fragment_id: (InstanceUnit, InstanceUnit),
		},
		/// Inventory has been updated
		InventoryUpdated {
			account_id: T::AccountId,
			definition_hash: Hash128,
			fragment_id: (InstanceUnit, InstanceUnit),
		},
		/// Fragment Expiration event
		Expired {
			account_id: T::AccountId,
			definition_hash: Hash128,
			fragment_id: (InstanceUnit, InstanceUnit),
		},
		/// Resell Instance
		Resell {
			definition_hash: Hash128,
			fragment_id: (InstanceUnit, InstanceUnit),
		},
		/// End Resale of Instance
		EndResale {
			definition_hash: Hash128,
			fragment_id: (InstanceUnit, InstanceUnit),
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Proto not found
		ProtoNotFound,
		/// Proto owner not found
		ProtoOwnerNotFound,
		/// No Permission
		NoPermission,
		/// Detach Request's Target Account is empty
		DetachAccountIsEmpty,
		/// Detach Request Already Submitted
		DetachRequestAlreadyExists,
		/// Already detached
		Detached,
		/// Already exist
		AlreadyExist,
		/// Metadata Name is Empty
		MetadataNameIsEmpty,
		/// Not found
		NotFound,
		/// Sale has expired
		Expired,
		/// Insufficient funds
		InsufficientBalance,
		/// Account cannot exist with the funds that would be given.
		ReceiverBelowMinimumBalance,
		/// Fragment sale sold out
		SoldOut,
		/// Sale already open
		SaleAlreadyOpen,
		/// Max supply reached
		MaxSupplyReached,
		/// Published quantity reached
		PublishedQuantityReached, // Need to think of a better name!
		/// Params not valid
		ParamsNotValid,
		/// This should not really happen
		SystematicFailure,
		/// Fragment Instance already uploaded with the same unique data
		UniqueDataExists,
		/// Currency not found
		CurrencyNotFound,
		/// Fragment Definition's Metadata key is empty
		DefinitionMetadataKeyIsEmpty,
		/// Fragment Instance's Metadata key is empty
		InstanceMetadataKeyIsEmpty,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// **Create** a **Fragment Definition** using an **existing Proto-Fragment**.
		///
		/// Note: **Only** the **Proto-Fragment's owner** is **allowed** to **create** a **Fragment Definition**
		/// using the **Proto-Fragment**
		///
		/// # Arguments
		///
		/// * `origin` - **Origin** of the **extrinsic function**
		/// * `proto_hash` - **Hash** of an **existing Proto-Fragment**
		/// * `metadata` -  **Metadata** of the **Fragment Definition**
		///
		/// * `permissions` - **Set of Actions** (encapsulated in a `FragmentPerms` bitflag enum)
		/// that are **allowed to be done** to **any Fragment Instance** when it **first gets created**
		/// from the **Fragment Definition that is created in this extrnisic function** (e.g edit, transfer etc.)
		///
		/// Note: These **allowed set of actions of a created Fragment Instance** ***may change***
		/// when the **Fragment Instance is given to another account ID** (see the `give` extrinsic).
		///
		/// * `unique` (*optional*) - **Whether** the **Fragment Definiton** is **unique**
		/// * `max_supply` (*optional*) - **Maximum amount of Fragment instances (where each Fragment instance has a different Edition ID)**
		/// that **can be created** using the **Fragment Definition**
		#[pallet::weight(<T as Config>::WeightInfo::create(metadata.name.len() as u32))]
		pub fn create(
			origin: OriginFor<T>,
			proto_hash: Hash256,
			metadata: DefinitionMetadata<BoundedVec<u8, <T as pallet_protos::Config>::StringLimit>, T::AssetId>,
			permissions: FragmentPerms,
			unique: Option<UniqueOptions>,
			max_supply: Option<InstanceUnit>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let metadata = DefinitionMetadata::<Vec<u8>, T::AssetId> {
				name: metadata.name.into(),
				currency: metadata.currency,
			};

			let proto: Proto<T::AccountId, T::BlockNumber> =
				<Protos<T>>::get(proto_hash).ok_or(Error::<T>::ProtoNotFound)?; // Get `Proto` struct from `proto_hash`

			let proto_owner: T::AccountId = match proto.owner {
				// Get `proto_owner` from `proto.owner`
				ProtoOwner::User(owner) => Ok(owner),
				_ => Err(Error::<T>::ProtoOwnerNotFound),
			}?;

			ensure!(who == proto_owner, Error::<T>::NoPermission); // Only proto owner can create a fragment definition from proto

			ensure!(!<DetachedHashes<T>>::contains_key(&DetachHash::Proto(proto_hash)), Error::<T>::Detached); // proto must not have been detached

			ensure!(metadata.name.len() > 0, Error::<T>::MetadataNameIsEmpty);

			let hash = blake2_128(
				// This is the unique id of the Fragment Definition that will be created
				&[&proto_hash[..], &metadata.name.encode(), &metadata.currency.encode()].concat(),
			);

			ensure!(!<Definitions<T>>::contains_key(&hash), Error::<T>::AlreadyExist); // If fragment already exists, throw error

			if let Currency::Custom(currency) = metadata.currency {
				ensure!(
					pallet_assets::Pallet::<T>::maybe_total_supply(currency).is_some(),
					Error::<T>::CurrencyNotFound
				); // If it is `None`, this means the asset ID `currency` doesn't exist
			}

			let current_block_number = <frame_system::Pallet<T>>::block_number();

			// ! Writing

			// create vault account
			// we need an existential amount deposit to be able to create the vault account
			let vault: T::AccountId = get_vault_id(hash);

			match metadata.currency {
				Currency::Native => {
					let minimum_balance =
						<pallet_balances::Pallet<T> as fungible::Inspect<T::AccountId>>::minimum_balance();
					<pallet_balances::Pallet<T> as fungible::Mutate<T::AccountId>>::mint_into(
						&vault,
						minimum_balance,
					)?;
				},
				Currency::Custom(currency) => {
					let minimum_balance =
						<pallet_assets::Pallet<T> as fungibles::Inspect<T::AccountId>>::minimum_balance(currency);
					<pallet_assets::Pallet<T> as fungibles::Mutate<T::AccountId>>::mint_into(
						currency,
						&vault,
						minimum_balance,
					)?;
				}
			};

			let fragment_data = FragmentDefinition {
				proto_hash,
				metadata,
				permissions,
				unique,
				max_supply: max_supply.map(|x| Compact(x)),
				creator: who.clone(),
				created_at: current_block_number,
				custom_metadata: BTreeMap::new(),
			};
			<Definitions<T>>::insert(&hash, fragment_data);

			Proto2Fragments::<T>::append(&proto_hash, hash);

			Self::deposit_event(Event::DefinitionCreated { definition_hash: hash });
			Ok(())
		}

		/// **Alters** the **custom metadata** of a **Fragment Definition** (whose ID is `definition_hash`) by **adding or modifying a key-value pair** (`metadata_key.clone`,`blake2_256(&data.encode())`)
		/// to the **BTreeMap field `custom_metadata`** of the **existing Fragment Definition's Struct Instance**.
		/// Furthermore, this function also indexes `data` in the Blockchain's Database and stores it in the IPFS
		///
		/// # Arguments
		///
		/// * `origin` - The origin of the extrinsic / dispatchable function
		/// * `definition_hash` - **ID of the Fragment Definition**
		/// * `metadata_key` - The key (of the key-value pair) that is added in the BTreeMap field `custom_metadata` of the existing Fragment Definition's Struct Instance
		/// * `data` - The hash of `data` is used as the value (of the key-value pair) that is added in the BTreeMap field `custom_metadata` of the existing Fragment Definition's Struct Instance
		#[pallet::weight(50_000)]
		pub fn set_definition_metadata(
			origin: OriginFor<T>,
			// fragment hash we want to update
			definition_hash: Hash128,
			// Think of "Vec<u8>" as String (something to do with WASM - that's why we use Vec<u8>)
			metadata_key: BoundedVec<u8, <T as pallet_protos::Config>::StringLimit>,
			// data we want to update last because of the way we store blocks (storage chain)
			data: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(!metadata_key.is_empty(), Error::<T>::DefinitionMetadataKeyIsEmpty);

			let proto_hash =
				<Definitions<T>>::get(definition_hash).ok_or(Error::<T>::NotFound)?.proto_hash; // Get `proto_hash` from `definition_hash`
			let proto: Proto<T::AccountId, T::BlockNumber> =
				<Protos<T>>::get(proto_hash).ok_or(Error::<T>::ProtoNotFound)?;
			let proto_owner: T::AccountId = match proto.owner {
				// Get `proto_owner` from `proto`
				ProtoOwner::User(owner) => Ok(owner),
				_ => Err(Error::<T>::ProtoOwnerNotFound),
			}?;
			ensure!(who == proto_owner, Error::<T>::NoPermission); // Ensure `who` is `proto_owner`

			// TO REVIEW
			ensure!(!<DetachedHashes<T>>::contains_key(&DetachHash::Proto(proto_hash)), Error::<T>::Detached); // Ensure `proto_hash` isn't detached

			let data_hash = blake2_256(&data);

			// we need this to index transactions
			let extrinsic_index = <frame_system::Pallet<T>>::extrinsic_index()
				.ok_or(Error::<T>::SystematicFailure)?;

			// Write STATE from now, ensure no errors from now...

			let metadata_key_index = {
				let index = <MetaKeys<T>>::get(metadata_key.clone());
				if let Some(index) = index {
					<Compact<u64>>::from(index)
				} else {
					let next_index = <MetaKeysIndex<T>>::try_get().unwrap_or_default() + 1;
					<MetaKeys<T>>::insert(metadata_key.clone(), next_index);
					// storing is dangerous inside a closure
					// but after this call we start storing..
					// so it's fine here
					<MetaKeysIndex<T>>::put(next_index);
					<Compact<u64>>::from(next_index)
				}
			};

			<Definitions<T>>::mutate(&definition_hash, |definition| {
				let definition = definition.as_mut().unwrap();
				// update custom metadata
				definition.custom_metadata.insert(metadata_key_index, data_hash);
			});

			// index data
			transaction_index::index(extrinsic_index, data.len() as u32, data_hash);

			// also emit event
			Self::deposit_event(Event::DefinitionMetadataChanged {
				definition_hash,
				metadata_key: metadata_key.clone().into(),
			});

			log::debug!(
				"Added metadata to fragment definition: {:x?} with key: {:x?}",
				definition_hash,
				metadata_key
			);

			Ok(())
		}

		/// **Alters** the **metadata** of a **Fragment Instance** (whose Fragment Definition ID is `definition_hash`,
		/// whose Edition ID is `edition_id` and whose Copy ID is `copy_id`).
		/// Furthermore, this function also indexes `data` in the Blockchain's Database and stores it in the IPFS
		///
		/// # Arguments
		///
		/// * `origin` - The origin of the extrinsic / dispatchable function
		/// * `definition_hash` - **ID of the Fragment Instance's Fragment Definition**
		/// * `edition_id` - **Edition ID of the Fragment Instance**
		/// * `copy_id` - **Copy ID of the Fragment Instance**
		/// * `metadata_key` - The key (of the key-value pair) that is added in the BTreeMap field `metadata` of the existing Fragment Instance's Struct Instance
		/// * `data` - The hash of `data` is used as the value (of the key-value pair) that is added in the BTreeMap field `metadata` of the existing Fragment Instance's Struct Instance
		#[pallet::weight(50_000)]
		pub fn set_instance_metadata(
			origin: OriginFor<T>,
			definition_hash: Hash128,
			edition_id: InstanceUnit,
			copy_id: InstanceUnit,
			// Think of "Vec<u8>" as String (something to do with WASM - that's why we use Vec<u8>)
			metadata_key: BoundedVec<u8, <T as pallet_protos::Config>::StringLimit>,
			// data we want to update last because of the way we store blocks (storage chain)
			data: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(!metadata_key.is_empty(), Error::<T>::InstanceMetadataKeyIsEmpty);

			ensure!(!<DetachedHashes<T>>::contains_key(&DetachHash::Instance(definition_hash, Compact(edition_id), Compact(copy_id))), Error::<T>::Detached);

			let instance_struct = <Fragments<T>>::get((definition_hash, edition_id, copy_id))
				.ok_or(Error::<T>::NotFound)?;

			let owned_instances =
				<Inventory<T>>::get(who.clone(), definition_hash).ok_or(Error::<T>::NotFound)?;
			ensure!(
				owned_instances.contains(&(Compact(edition_id), Compact(copy_id))),
				Error::<T>::NoPermission
			);

			let data_hash = blake2_256(&data);

			// we need this to index transactions
			let extrinsic_index = <frame_system::Pallet<T>>::extrinsic_index()
				.ok_or(Error::<T>::SystematicFailure)?;

			// Write STATE from now, ensure no errors from now...

			let metadata_key_index = {
				let index = <MetaKeys<T>>::get(metadata_key.clone());
				if let Some(index) = index {
					<Compact<u64>>::from(index)
				} else {
					let next_index = <MetaKeysIndex<T>>::try_get().unwrap_or_default() + 1;
					<MetaKeys<T>>::insert(metadata_key.clone(), next_index);
					// storing is dangerous inside a closure
					// but after this call we start storing..
					// so it's fine here
					<MetaKeysIndex<T>>::put(next_index);
					<Compact<u64>>::from(next_index)
				}
			};

			let (index, should_update_metadata_field) = {
				if let Some(existing_index) = instance_struct.metadata.get(&metadata_key_index) {
					(existing_index.clone(), false)
				} else {
					let next_index =
						<DataHashMapIndex<T>>::try_get(definition_hash).unwrap_or_default() + 1;
					<DataHashMapIndex<T>>::insert(definition_hash, next_index);
					(Compact(next_index), true)
				}
			};

			<DataHashMap<T>>::insert(definition_hash, index, data_hash);

			if should_update_metadata_field {
				<Fragments<T>>::mutate(&(definition_hash, edition_id, copy_id), |instance| {
					let instance = instance.as_mut().unwrap();
					// update custom metadata
					instance.metadata.insert(metadata_key_index, index);
				});
			}

			// index data
			transaction_index::index(extrinsic_index, data.len() as u32, data_hash);

			// also emit event
			Self::deposit_event(Event::InstanceMetadataChanged {
				definition_hash: definition_hash,
				edition_id,
				copy_id,
				metadata_key: metadata_key.clone().into(),
			});

			log::debug!(
				"Added metadata to fragment instance: {:x?}, {}, {} with key: {:x?}",
				definition_hash,
				edition_id,
				copy_id,
				metadata_key
			);

			Ok(())
		}

		/// Put the **Fragment Definition `definition_hash`** on sale. When a Fragment Definition is put on sale, users can create Fragment Instances from it for a fee.
		///
		/// Note: **Only** the **Fragment's Proto-Fragment's owner** is **allowed** to put the **Fragment** on sale
		///
		/// # Arguments
		///
		/// * `origin` - **Origin** of the **extrinsic function**
		/// * `definition_hash` - ID of the **Fragment Definition** to put on sale
		/// * `price` -  **Price** to **buy** a **single Fragment Instance** that is created from the **Fragment Definition*
		/// * `quantity` (*optional*) - **Maximum amount of Fragment Instances** that **can be bought**
		/// * `expires` (*optional*) - **Block number** that the sale ends at (*optional*)
		/// * `amount` (*optional*) - If the Fragment instance represents a **stack of stackable items** (for e.g gold coins or arrows - https://runescape.fandom.com/wiki/Stackable_items),
		/// the **number of items** to **top up** in the **stack of stackable items**
		#[pallet::weight(<T as Config>::WeightInfo::publish())]
		pub fn publish(
			origin: OriginFor<T>,
			definition_hash: Hash128,
			price: u128,
			quantity: Option<InstanceUnit>,
			expires: Option<T::BlockNumber>,
			stack_amount: Option<InstanceUnit>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let proto_hash =
				<Definitions<T>>::get(definition_hash).ok_or(Error::<T>::NotFound)?.proto_hash; // Get `proto_hash` from `definition_hash`
			let proto: Proto<T::AccountId, T::BlockNumber> =
				<Protos<T>>::get(proto_hash).ok_or(Error::<T>::ProtoNotFound)?; // Get `proto` from `proto_hash`

			let proto_owner: T::AccountId = match proto.owner {
				// Get `proto_owner` from `proto`
				ProtoOwner::User(owner) => Ok(owner),
				_ => Err(Error::<T>::ProtoOwnerNotFound),
			}?;

			ensure!(who == proto_owner, Error::<T>::NoPermission); // Ensure `who` is `proto_owner`

			// TO REVIEW
			ensure!(!<DetachedHashes<T>>::contains_key(&DetachHash::Proto(proto_hash)), Error::<T>::Detached); // Ensure `proto_hash` isn't detached

			ensure!(!<Publishing<T>>::contains_key(&definition_hash), Error::<T>::SaleAlreadyOpen); // Ensure `definition_hash` isn't already published

			let fragment_data =
				<Definitions<T>>::get(definition_hash).ok_or(Error::<T>::NotFound)?; // Get `FragmentDefinition` struct from `definition_hash`

			if let Some(max_supply) = fragment_data.max_supply {
				let max: InstanceUnit = max_supply.into();
				let existing: InstanceUnit =
					<EditionsCount<T>>::get(&definition_hash).unwrap_or(Compact(0)).into();
				let left = max.saturating_sub(existing); // `left` = `max` - `existing`
				if left == 0 {
					return Err(Error::<T>::MaxSupplyReached.into());
				}
				if let Some(quantity) = quantity {
					let quantity: InstanceUnit = quantity.into();
					ensure!(quantity <= left, Error::<T>::MaxSupplyReached); // Ensure that the function parameter `quantity` is smaller than or equal to `left`
				} else {
					// Ensure that if `fragment_data.max_supply` exists, the function parameter `quantity` must also exist
					return Err(Error::<T>::ParamsNotValid.into());
				}
			}

			// ! Writing

			<Publishing<T>>::insert(
				definition_hash,
				PublishingData {
					price: Compact(price),
					units_left: quantity.map(|x| Compact(x)),
					expiration: expires,
					stack_amount: stack_amount.map(|x| Compact(x)),
				},
			);

			Self::deposit_event(Event::Publishing { definition_hash });

			Ok(())
		}

		/// Take the **Fragment Definition `definition_hash`** off sale.
		/// When a Fragment Definition is put on sale, users can create Fragment Instances from it for a fee.
		///
		/// Note: **Only** the **Fragment's Proto-Fragment's owner** is **allowed** to take the Fragment off sale
		///
		/// # Arguments
		///
		/// * `origin` - **Origin** of the **extrinsic function**
		/// * `definition_hash` - **ID** of the **Fragment Definition** to take off sale
		#[pallet::weight(<T as Config>::WeightInfo::unpublish())]
		pub fn unpublish(origin: OriginFor<T>, definition_hash: Hash128) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let proto_hash =
				<Definitions<T>>::get(definition_hash).ok_or(Error::<T>::NotFound)?.proto_hash; // Get `proto_hash` from `definition_hash`
			let proto: Proto<T::AccountId, T::BlockNumber> =
				<Protos<T>>::get(proto_hash).ok_or(Error::<T>::ProtoNotFound)?;

			let proto_owner: T::AccountId = match proto.owner {
				// Get `proto_owner` from `proto`
				ProtoOwner::User(owner) => Ok(owner),
				_ => Err(Error::<T>::ProtoOwnerNotFound),
			}?;

			ensure!(who == proto_owner, Error::<T>::NoPermission); // Ensure `who` is `proto_owner`

			// TO REVIEW
			ensure!(!<DetachedHashes<T>>::contains_key(&DetachHash::Proto(proto_hash)), Error::<T>::Detached); // Ensure `proto_hash` isn't detached

			ensure!(<Publishing<T>>::contains_key(&definition_hash), Error::<T>::NotFound); // Ensure `definition_hash` is currently published

			// ! Writing

			<Publishing<T>>::remove(&definition_hash); // Remove Fragment Definition `definition_hash` from `Publishing`

			Self::deposit_event(Event::Unpublishing { definition_hash });

			Ok(())
		}

		/// Create **Fragment instance(s)** from the **Fragment Definition `definition_hash`** and
		/// **assign their ownership** to **`origin`**
		///
		/// Note: **Each created Fragment instance** will have a **different Edition ID** and a **Copy ID of "1"**.
		///
		/// Note: **Only** the **Fragment Definition's Proto-Fragment's owner** is **allowed** to
		/// create instance(s) of the Fragment in this extrinsic function.
		///
		/// # Arguments
		///
		/// * `origin` - **Origin** of the **extrinsic function**
		/// * `definition_hash` - **ID* of the **Fragment Definition**
		/// * `options` - **Enum** indicating whether to
		/// **create one Fragment Instance with custom data attached to it** or whether to
		/// **create multiple Fragment Instances (with no custom data attached)**
		/// * `amount` (*optional*) - If the Fragment Instance(s) represent a **stack of stackable items**
		/// (for e.g gold coins or arrows - https://runescape.fandom.com/wiki/Stackable_items),
		/// `amount` is the **number of items** to **top up** in the **stack of stackable items**
		///
		/// TODO - `*q as u32` might cause problems if q is too big (since q is u64)!!!
		#[pallet::weight(match options {
		FragmentBuyOptions::Quantity(q) => <T as Config>::WeightInfo::mint_definition_that_has_non_unique_capability(*q as u32),
		FragmentBuyOptions::UniqueData(d) => <T as Config>::WeightInfo::mint_definition_that_has_unique_capability(d.len() as u32)
		})]
		pub fn mint(
			origin: OriginFor<T>,
			definition_hash: Hash128,
			options: FragmentBuyOptions,
			stack_amount: Option<InstanceUnit>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let current_block_number = <frame_system::Pallet<T>>::block_number();

			let proto_hash =
				<Definitions<T>>::get(definition_hash).ok_or(Error::<T>::NotFound)?.proto_hash; // Get `proto_hash` from `definition_hash`
			let proto: Proto<T::AccountId, T::BlockNumber> =
				<Protos<T>>::get(proto_hash).ok_or(Error::<T>::ProtoNotFound)?; // Get `proto` from `proto_hash`

			let proto_owner: T::AccountId = match proto.owner {
				// Get `proto_owner` from `proto`
				ProtoOwner::User(owner) => Ok(owner),
				_ => Err(Error::<T>::ProtoOwnerNotFound),
			}?;

			ensure!(who == proto_owner, Error::<T>::NoPermission); // Ensure `who` is `proto_owner`

			// TO REVIEW
			ensure!(!<DetachedHashes<T>>::contains_key(&DetachHash::Proto(proto_hash)), Error::<T>::Detached); // Ensure `proto_hash` isn't detached

			let quantity = match options {
				// Number of fragment instances to mint
				FragmentBuyOptions::Quantity(quantity) => u64::from(quantity),
				_ => 1u64,
			};

			// ! Writing

			Self::mint_fragments(
				&who,
				&definition_hash,
				None, // PublishingData (optional)
				&options,
				quantity,
				current_block_number,
				None, // Block Number the Fragment(s) expire at (optional)
				stack_amount.map(|x| Compact(x)),
			)
		}

		/// Allows the Caller Account ID `origin` to create Fragment instance(s) of the Fragment Definition `definition_hash`,
		/// for a fee. The ownership of the created Fragment instance(s) is assigned to the Caller Account ID.
		///
		/// Note: Each created Fragment instance will have a different Edition ID and a Copy ID of "1".
		///
		/// Note: The total fee that the buyer (i.e the Caller Account ID `origin`) must pay is the
		/// specified price-per-instance multiplied by the total number of instance(s) that the buyer wants to create. (@karan)
		/// This amount will be transferred to the Fragment Definition's Vault's Account ID.
		///
		///
		///
		/// # Arguments
		///
		/// * `origin` - **Origin** of the **extrinsic function**
		/// * `definition_hash` - **ID** of the **Fragment Definition**
		/// * `options` - **Enum** indicating whether to
		/// **create one Fragment Instance with custom data attached to it** or whether to
		/// **create multiple Fragment Instances (with no custom data attached)**
		///
		/// TODO - `*=q as u32` might cause problems if q is too big (since q is u64)!!!
		#[pallet::weight(match options {
		FragmentBuyOptions::Quantity(q) => <T as Config>::WeightInfo::buy_definition_that_has_non_unique_capability(*q as u32),
		FragmentBuyOptions::UniqueData(d) => <T as Config>::WeightInfo::buy_definition_that_has_unique_capability(d.len() as u32)
		})]
		pub fn buy(
			origin: OriginFor<T>,
			definition_hash: Hash128,
			options: FragmentBuyOptions,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let current_block_number = <frame_system::Pallet<T>>::block_number();

			let sale = <Publishing<T>>::get(&definition_hash).ok_or(Error::<T>::NotFound)?; // if Fragment Definition `definition_hash` is not published (i.e on sale), you cannot buy it
			if let Some(expiration) = sale.expiration {
				ensure!(current_block_number < expiration, Error::<T>::Expired);
			}

			if let Some(units_left) = sale.units_left {
				ensure!(units_left > Compact(0), Error::<T>::SoldOut);
			}

			let price: u128 = sale.price.into();

			let fragment_data =
				<Definitions<T>>::get(definition_hash).ok_or(Error::<T>::NotFound)?;

			let vault: T::AccountId = get_vault_id(definition_hash); // Get the Vault Account ID of `definition_hash`

			let quantity = match options {
				FragmentBuyOptions::Quantity(amount) => u64::from(amount),
				_ => 1u64,
			};

			let price = price.saturating_mul(quantity as u128); // `price` = `price` * `quantity`

			Self::can_transfer_currency(&who, &vault, price, fragment_data.metadata.currency)?;

			// ! Writing

			Self::mint_fragments(
				&who,
				&definition_hash,
				Some(&sale), // PublishingData (optional)
				&options,
				quantity,
				current_block_number,
				None, // Block Number that the Fragment Instance will expire at (optional)
				sale.stack_amount,
			)?;

			Self::transfer_currency(&who, &vault, price, fragment_data.metadata.currency)?;

			Ok(())
		}

		/// Give a **Fragment Instance** to **`to`**.
		///
		/// If the **current permitted actions of the Fragment Instance** allows for it to be copied (i.e if it has the permission **FragmentPerms::COPY**),
		/// then it is copied and the copy's ownership is assigned to `to`.
		/// Otherwise, its ownership is transferred from `origin` to `to`.
		///
		/// Note: **Only** the **Fragment Instance's owner** is **allowed** to give the Fragment Instance
		///
		/// # Arguments
		///
		/// * `origin` - **Origin** of the **extrinsic function**
		/// * `definition_hash` - Fragment Definition ID of the Fragment Instance
		/// * `edition` - Edition ID of the Fragment Instance
		/// * `copy` - Copy ID of the Fragment instance
		/// * `to` - **Account ID** to give the Fragment instance
		/// * `new_permissions` (*optional*) - The permitted set of actions that the account that is given the Fragment instance can do with it. Note: `new_permissions` must be a subset of the current `permissions` field of the Fragment Instance.
		/// * `expiration` (*optional*) - Block number that the newly-copied Fragment Instance expires at. If the Fragment Instance is not copyable, this parameter is practically irrelevant.
		#[pallet::weight(
		<T as Config>::WeightInfo::benchmark_give_instance_that_has_copy_perms()
		.max(<T as Config>::WeightInfo::benchmark_give_instance_that_does_not_have_copy_perms())
		)] // Since both weight functions return a static value, we should not be doing a `max()` and just manually choose the one with a greater weight!
		pub fn give(
			origin: OriginFor<T>,
			definition_hash: Hash128,
			edition_id: InstanceUnit,
			copy_id: InstanceUnit,
			to: <T::Lookup as StaticLookup>::Source,
			new_permissions: Option<FragmentPerms>,
			expiration: Option<T::BlockNumber>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::can_transfer_instance(&who, &definition_hash, edition_id, copy_id, new_permissions, expiration)?;

			let to = T::Lookup::lookup(to)?;

			// now we take two different paths if item can be copied or not
			Self::transfer_instance(&who, &to, &definition_hash, edition_id, copy_id, new_permissions, expiration)?;

			Ok(())
		}

		/// Create the **Account ID** of the **Fragment Instance whose Fragment Definition ID is `class`,
		/// whose Edition ID is `edition`** and whose Copy ID is `copy`**
		///
		/// # Arguments
		///
		/// * `origin` - **Origin** of the **extrinsic function**
		/// * `definition_hash` - **Fragment Definition 	ID** of the **Fragment Instance**
		/// * `edition` - **Edition ID** of the **Fragment Instance**
		/// * `copy` - **Copy ID** of the **Fragment Instance**
		#[pallet::weight(50_000)]
		pub fn create_account(
			origin: OriginFor<T>,
			definition_hash: Hash128,
			edition: InstanceUnit,
			copy: InstanceUnit,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Only the owner of this fragment can transfer it
			let ids =
				<Inventory<T>>::get(who.clone(), definition_hash).ok_or(Error::<T>::NotFound)?;

			ensure!(ids.contains(&(Compact(edition), Compact(copy))), Error::<T>::NoPermission);

			// create an account for a specific fragment
			// we need an existential amount deposit to be able to create the vault account
			let frag_account = Self::get_fragment_account_id(definition_hash, edition, copy);
			let minimum_balance =
				<pallet_balances::Pallet<T> as fungible::Inspect<T::AccountId>>::minimum_balance();
			let _ = <pallet_balances::Pallet<T> as fungible::Mutate<T::AccountId>>::mint_into(
				&frag_account,
				minimum_balance,
			);

			// TODO Make owner pay for deposit actually!
			// TODO setup proxy

			Ok(())
		}


		/// Put a Fragment Instance on sale.
		///
		///
		/// Note: **Only** the **Fragment Instance's owner** is **allowed** to call this extrinsic
		///
		/// # Arguments
		///
		/// * `origin` - **Origin** of the **extrinsic function**
		/// * `definition_hash` - Fragment Definition ID of the Fragment Instance
		/// * `edition` - Edition ID of the Fragment Instance
		/// * `copy` - Copy ID of the Fragment instance
		/// * `to` - **Account ID** to give the Fragment instance
		/// * `new_permissions` (*optional*) - The permitted set of actions that the account that is given the Fragment instance can do with it. Note: `new_permissions` must be a subset of the current `permissions` field of the Fragment Instance.
		/// * `expiration` (*optional*) - Block number that the newly-copied Fragment Instance expires at. If the Fragment Instance is not copyable, this parameter is practically irrelevant.
		/// * `secondary_sale_type` - Type of Sale
		#[pallet::weight(50_000)]
		pub fn resell(
			origin: OriginFor<T>,
			definition_hash: Hash128,
			edition_id: InstanceUnit,
			copy_id: InstanceUnit,
			new_permissions: Option<FragmentPerms>,
			expiration: Option<T::BlockNumber>,
			secondary_sale_type: SecondarySaleType
		) -> DispatchResult {

			let who = ensure_signed(origin)?;

			ensure!(
				!<Definition2SecondarySales<T>>::contains_key((definition_hash, edition_id, copy_id)),
				Error::<T>::SaleAlreadyOpen
			);

			Self::can_transfer_instance(&who, &definition_hash, edition_id, copy_id, new_permissions, expiration)?;

			// ! Writing

			Definition2SecondarySales::<T>::insert(
				(definition_hash, edition_id, copy_id),
				SecondarySaleData {
					owner: who,
					new_permissions,
					expiration,
					secondary_sale_type
				}
			);

			Self::deposit_event(Event::Resell { definition_hash, fragment_id: (edition_id, copy_id) });

			Ok(())
		}

		/// End the sale of a Fragment Instance that is currently on sale.
		///
		/// # Arguments
		///
		/// * `origin` - **Origin** of the **extrinsic function**
		/// * `definition_hash` - Fragment Definition ID of the Fragment Instance
		/// * `edition` - Edition ID of the Fragment Instance
		/// * `copy` - Copy ID of the Fragment instance
		#[pallet::weight(50_000)]
		pub fn end_resale(
			origin: OriginFor<T>,
			definition_hash: Hash128,
			edition_id: InstanceUnit,
			copy_id: InstanceUnit,
		) -> DispatchResult {

			let who = ensure_signed(origin)?;

			ensure!(
				who == <Definition2SecondarySales<T>>::get((definition_hash, edition_id, copy_id)).ok_or(Error::<T>::NotFound)?.owner,
				Error::<T>::NoPermission
			);

			// ! Writing

			Definition2SecondarySales::<T>::remove((definition_hash, edition_id, copy_id));

			Self::deposit_event(Event::EndResale { definition_hash, fragment_id: (edition_id, copy_id) });

			Ok(())
		}

		/// Buy an existing Fragment Instance that is on sale.
		///
		/// # Arguments
		///
		/// * `origin` - **Origin** of the **extrinsic function**
		/// * `definition_hash` - Fragment Definition ID of the Fragment Instance
		/// * `edition` - Edition ID of the Fragment Instance
		/// * `copy` - Copy ID of the Fragment instance
		/// * `options` - Enum indicating how to buy the instance
		#[pallet::weight(50_000)]
		pub fn secondary_buy(
			origin: OriginFor<T>,
			definition_hash: Hash128,
			edition_id: InstanceUnit,
			copy_id: InstanceUnit,
			options: SecondarySaleBuyOptions
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let secondary_sale_data = Definition2SecondarySales::<T>::get((definition_hash, edition_id, copy_id))
				.ok_or(Error::<T>::NotFound)?;

			let currency = Definitions::<T>::get(definition_hash).ok_or(Error::<T>::SystematicFailure)?.metadata.currency;

			match (secondary_sale_data.secondary_sale_type, options) {
				(SecondarySaleType::Normal(price), SecondarySaleBuyOptions::Normal) => {
					Self::can_transfer_currency(&who, &secondary_sale_data.owner, price, currency)?;

					// ! Writing

					Self::transfer_currency(&who, &secondary_sale_data.owner, price, currency)?;
					Self::transfer_instance(
						&secondary_sale_data.owner,
						&who,
						&definition_hash,
						edition_id,
						copy_id,
						secondary_sale_data.new_permissions,
						secondary_sale_data.expiration
					)?;

					// remove secondary sale data from `Definition2SecondarySales`
					Definition2SecondarySales::<T>::remove((definition_hash, edition_id, copy_id));
				}
				_ => return Err(Error::<T>::ParamsNotValid.into()),
			};

			Ok(())
		}

		/// Request to detach a **Fragment** from **Clamor**.
		///
		/// Note: The Fragment may actually get detached after one or more Clamor blocks since when this extrinsic is called.
		///
		/// Note: **Once the Fragment is detached**, an **event is emitted that includes a signature**.
		/// This signature can then be used to attach the Proto-Fragment to an External Blockchain `target_chain`.
		///
		/// # Arguments
		///
		/// * `origin` - The origin of the extrinsic function
		/// * `definition_hash` - **ID of the Fragment Definition** to **detach**
		/// * `target_chain` - **External Blockchain** to attach the Proto-Fragment into
		/// * `target_account` - **Public Account Address in the External Blockchain `target_chain`**
		///   to assign ownership of the Proto-Fragment to
		#[pallet::weight(25_000)] // TODO - weight
		pub fn detach(
			origin: OriginFor<T>,
			definition_hash: Hash128,
			edition_id: InstanceUnit,
			copy_id: InstanceUnit,
			target_chain: SupportedChains,
			target_account: BoundedVec<u8, T::DetachAccountLimit>, // an eth address or so
		) -> DispatchResult {

			let who = ensure_signed(origin)?;

			ensure!(!target_account.is_empty(), Error::<T>::DetachAccountIsEmpty);

			let current_block_number = <frame_system::Pallet<T>>::block_number();

			// owner must own instance
			let owned_instances = <Inventory<T>>::get(who, definition_hash).ok_or(Error::<T>::NoPermission)?;
			ensure!(
				owned_instances.contains(&(Compact(edition_id), Compact(copy_id))),
				Error::<T>::NoPermission
			);

			// REVIEW - no go if will expire this block
			let instance = <Fragments<T>>::get((definition_hash, edition_id, copy_id))
				.ok_or(Error::<T>::NotFound)?;
			if let Some(expiring_at) = instance.expiring_at {
				ensure!(current_block_number < expiring_at, Error::<T>::NotFound);
			}

			let detach_hash = DetachHash::Instance(definition_hash, Compact(edition_id), Compact(copy_id));
			let detach_request = DetachRequest { hash: detach_hash.clone(), target_chain, target_account: target_account.into() };

			ensure!(!<DetachedHashes<T>>::contains_key(&detach_hash),Error::<T>::Detached);
			ensure!(!<DetachRequests<T>>::get().contains(&detach_request), Error::<T>::DetachRequestAlreadyExists);

			<DetachRequests<T>>::mutate(|requests| {
				requests.push(detach_request);
			});

			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// During the block finalization phase,
		/// clear all the *Fragment instance*-related Storage Items of any information regarding
		/// Fragment instances that have already expired
		fn on_finalize(n: T::BlockNumber) {
			let expiring = <Expirations<T>>::take(n);
			if let Some(expiring) = expiring {
				for item in expiring {
					// remove from Fragments
					<Fragments<T>>::remove((item.0, u64::from(item.1), u64::from(item.2)));
					for (owner, items) in <Owners<T>>::iter_prefix(item.0) {
						let index = items.iter().position(|x| x == &(item.1, item.2));
						if let Some(index) = index {
							// remove from Owners
							<Owners<T>>::mutate(item.0, owner.clone(), |x| {
								if let Some(x) = x {
									x.remove(index);
								}
							});

							// remove from Inventory
							<Inventory<T>>::mutate(owner.clone(), item.0, |x| {
								if let Some(x) = x {
									let index = x.iter().position(|y| y == &(item.1, item.2));
									if let Some(index) = index {
										x.remove(index);
									}
								}
							});

							// trigger an Event
							Self::deposit_event(Event::Expired {
								account_id: owner,
								definition_hash: item.0,
								fragment_id: (item.1.into(), item.2.into()),
							});

							// fragments are unique so we are done here
							break;
						}
					}
				}
			}
		}
	}

	impl<T: Config> Pallet<T> {

		/// Get the **Account ID** of the **Fragment Instance whose Fragment Definition ID is `definition_hash`,
		/// whose Edition ID is `edition`** and whose Copy ID is `copy`**
		///
		/// This Account ID is determinstically computed using the Fragment Definition ID `class_hash`, the Edition ID `edition` and the Copy ID `copy`
		pub fn get_fragment_account_id(
			definition_hash: Hash128,
			edition: InstanceUnit,
			copy: InstanceUnit,
		) -> T::AccountId {
			let hash = blake2_256(
				&[&b"fragments-account"[..], &definition_hash, &edition.encode(), &copy.encode()]
					.concat(),
			);
			T::AccountId::decode(&mut &hash[..]).expect("T::AccountId should decode")
		}

		/// Create `quantity` number of Fragment Instances from the Fragment Definition `definition_hash` and assigns their ownership to `to`
		///
		/// # Arguments
		///
		/// * `to` - **Account ID** to assign ownernship of the created Fragment instances to
		/// * `definition_hash` - ID of the Fragment Definition
		/// * `sale` - Struct **representing** a **sale of the Fragment Definition**, if the **Fragment Definition** is **currently on sale**
		/// * `options` - **Enum** indicating whether to
		/// **create one Fragment Instance with custom data attached to it** or whether to
		/// **create multiple Fragment Instances (with no custom data attached)**
		/// * `quantity` - **Number of Fragment Instances** to **create**
		/// * `current_block_number` - **Current block number** of the **Clamor Blockchain**
		/// * `expiring_at` (*optional*) - **Block Number** that the **Fragment Instance** will **expire at**
		/// * `amount` (*optional*) - If the Fragment Instance(s) represent a **stack of stackable items**
		/// (for e.g gold coins or arrows - https://runescape.fandom.com/wiki/Stackable_items),
		/// `amount` is the **number of items** to **top up** in the **stack of stackable items**
		pub fn mint_fragments(
			to: &T::AccountId,
			definition_hash: &Hash128,
			sale: Option<&PublishingData<T::BlockNumber>>,
			options: &FragmentBuyOptions,
			quantity: u64,
			current_block_number: T::BlockNumber,
			expiring_at: Option<T::BlockNumber>,
			stack_amount: Option<Compact<InstanceUnit>>,
		) -> DispatchResult {
			use frame_support::ensure;

			if let Some(expiring_at) = expiring_at {
				ensure!(expiring_at > current_block_number, Error::<T>::ParamsNotValid); // Ensure `expiring_at` > `current_block_number`
			}

			let fragment_data =
				<Definitions<T>>::get(definition_hash).ok_or(Error::<T>::NotFound)?;

			// we need this to index transactions
			let extrinsic_index = <frame_system::Pallet<T>>::extrinsic_index() // `<frame_system::Pallet<T>>::extrinsic_index()` is defined as: "Gets the index of extrinsic that is currently executing." (https://paritytech.github.io/substrate/master/frame_system/pallet/struct.Pallet.html#method.extrinsic_index)
				.ok_or(Error::<T>::SystematicFailure)?;

			let (data_hash, data_len) = match options {
				FragmentBuyOptions::UniqueData(data) => {
					if fragment_data.unique.is_none() || quantity != 1 {
						return Err(Error::<T>::ParamsNotValid.into());
					}

					let data_hash = blake2_256(&data);

					ensure!(
						!<UniqueData2Edition<T>>::contains_key(definition_hash, data_hash),
						Error::<T>::UniqueDataExists
					);

					(Some(data_hash), Some(data.len()))
				},
				FragmentBuyOptions::Quantity(_) => {
					if fragment_data.unique.is_some() {
						return Err(Error::<T>::ParamsNotValid.into());
					}

					(None, None)
				},
			};

			let existing: InstanceUnit =
				<EditionsCount<T>>::get(&definition_hash).unwrap_or(Compact(0)).into();

			if let Some(sale) = sale {
				// if limited amount let's reduce the amount of units left
				if let Some(units_left) = sale.units_left {
					if quantity > units_left.into() {
						return Err(Error::<T>::PublishedQuantityReached.into());
					} else {
						<Publishing<T>>::mutate(&*definition_hash, |sale| {
							if let Some(sale) = sale {
								let left: InstanceUnit = units_left.into();
								sale.units_left = Some(Compact(left - quantity));
							}
						});
					}
				}
			} else {
				// We still don't wanna go over supply limit
				if let Some(max_supply) = fragment_data.max_supply {
					let max: InstanceUnit = max_supply.into();
					let left = max.saturating_sub(existing); // `left` = `max` - `existing`
					if quantity > left {
						// Ensure the function parameter `quantity` is smaller than or equal to `left`
						return Err(Error::<T>::MaxSupplyReached.into());
					}
				}
			}

			// ! Writing if successful

			<Definitions<T>>::mutate(definition_hash, |fragment| {
				// Get the `FragmentDefinition` struct from `definition_hash`
				if let Some(fragment) = fragment {
					for id in existing..(existing + quantity) {
						let id = id + 1u64;
						let cid = Compact(id); // `cid` stands for "compact id"

						<Fragments<T>>::insert(
							(definition_hash, id, 1),
							FragmentInstance {
								permissions: fragment.permissions,
								created_at: current_block_number,
								custom_data: data_hash,
								expiring_at,
								stack_amount,
								metadata: BTreeMap::new(),
							},
						);

						<CopiesCount<T>>::insert((definition_hash, id), Compact(1));

						<Inventory<T>>::append(to.clone(), definition_hash, (cid, Compact(1))); // **Add** the **Fragment Intstance whose Fragment Definition is `definition_hash`, Edition ID is `cid` and Copy ID is 1**  to the **inventory of `to`**

						<Owners<T>>::append(definition_hash, to.clone(), (cid, Compact(1)));

						if let Some(expiring_at) = expiring_at {
							<Expirations<T>>::append(
								expiring_at,
								(*definition_hash, cid, Compact(1)),
							);
						}
						Self::deposit_event(Event::InventoryAdded {
							account_id: to.clone(),
							definition_hash: *definition_hash,
							fragment_id: (id, 1),
						});
					}

					if let (Some(data_hash), Some(data_len)) = (data_hash, data_len) {
						<UniqueData2Edition<T>>::insert(definition_hash, data_hash, existing); // if `data` exists, `quantity` is ensured to be 1

						// index immutable data for IPFS discovery
						transaction_index::index(extrinsic_index, data_len as u32, data_hash);
					}

					<EditionsCount<T>>::insert(definition_hash, Compact(existing + quantity));
				}
			});

			Ok(())
		}

		/// Whether `amount` amount of token `currency` can be transferred from `from` to `to`
		pub fn can_transfer_currency(
			from: &T::AccountId,
			to: &T::AccountId,
			amount: u128,
			currency: Currency<T::AssetId>
		) -> DispatchResult {

			match currency {
				Currency::Custom(currency) => {
					let minimum_balance_needed_to_exist =
						<pallet_assets::Pallet<T> as fungibles::Inspect<T::AccountId>>::minimum_balance(currency);
					let price_balance: <pallet_assets::Pallet<T> as fungibles::Inspect<T::AccountId>>::Balance =
						amount.saturated_into();

					ensure!(
						<pallet_assets::Pallet<T> as fungibles::Inspect<T::AccountId>>::balance(currency, from) >=
							price_balance + minimum_balance_needed_to_exist,
						Error::<T>::InsufficientBalance
					);
					ensure!(
						<pallet_assets::Pallet<T> as fungibles::Inspect<T::AccountId>>::balance(currency, to) +
							price_balance >= minimum_balance_needed_to_exist,
						Error::<T>::ReceiverBelowMinimumBalance
					);
				},
				Currency::Native => {
					let minimum_balance_needed_to_exist =
						<pallet_balances::Pallet<T> as fungible::Inspect<T::AccountId>>::minimum_balance();
					let price_balance: <pallet_balances::Pallet<T> as fungible::Inspect<T::AccountId>>::Balance =
						amount.saturated_into();

					ensure!(
						pallet_balances::Pallet::<T>::free_balance(from) >=
							price_balance + minimum_balance_needed_to_exist,
						Error::<T>::InsufficientBalance
					);
					ensure!(
						pallet_balances::Pallet::<T>::free_balance(to) +
							price_balance >= minimum_balance_needed_to_exist,
						Error::<T>::ReceiverBelowMinimumBalance
					);
				}
			}

			Ok(())
		}

		/// Transfer `amount` amount of token `currency` from `from` to `to`
		pub fn transfer_currency(
			from: &T::AccountId,
			to: &T::AccountId,
			amount: u128,
			currency: Currency<T::AssetId>
		) -> DispatchResult {

			match currency {
				Currency::Custom(currency) => {
					<pallet_assets::Pallet<T> as fungibles::Transfer<T::AccountId>>::transfer(
						// transfer `amount` units of `currency` from `from` to `to`
						currency,
						from,
						to,
						amount.saturated_into(),
						true, // The debited account must stay alive at the end of the operation; an error is returned if this cannot be achieved legally.
					)
						.map_err(|_| Error::<T>::InsufficientBalance)?;
				},
				Currency::Native => {
					pallet_balances::Pallet::<T>::do_transfer(
						// transfer `amount` units of NOVA from `from` to `to`
						from,
						to,
						amount.saturated_into(),
						ExistenceRequirement::KeepAlive,
					)
						.map_err(|_| Error::<T>::InsufficientBalance)?;
				}
			}

			Ok(())
		}

		/// Whether a Fragment Instance can be transferred
		///
		/// * `from` - Clamor Account ID to transfer the Fragment Instance from
		/// * `definition_hash` - Fragment Definition of the Fragment Instance
		/// * `edition_id` - Edition ID of the Fragment Instance
		/// * `copy_id` - Copy ID of the Fragment Instance
		/// * `new_permissions` - New Permissions of the Fragment Instance after it is sold
		/// * `expiration` -  Block number that the newly-copied Fragment Instance expires at. If the Fragment Instance is not copyable, this field is practically irrelevant.
		pub fn can_transfer_instance(
			from: &T::AccountId,
			definition_hash: &Hash128,
			edition_id: InstanceUnit,
			copy_id: InstanceUnit,
			new_permissions: Option<FragmentPerms>,
			expiration: Option<T::BlockNumber>,
		) -> DispatchResult {

			ensure!(!<DetachedHashes<T>>::contains_key(&DetachHash::Instance(*definition_hash, Compact(edition_id), Compact(copy_id))), Error::<T>::Detached);

			let current_block_number = <frame_system::Pallet<T>>::block_number();

			let item_data = <Fragments<T>>::get((definition_hash, edition_id, copy_id))
				.ok_or(Error::<T>::NotFound)?;

			// no go if will expire this block
			if let Some(item_expiration) = item_data.expiring_at {
				ensure!(current_block_number < item_expiration, Error::<T>::NotFound);
			}

			if let Some(expiration) = expiration {
				ensure!(current_block_number < expiration, Error::<T>::ParamsNotValid);
			}

			// Only the owner of this fragment can transfer it
			let ids =
				<Inventory<T>>::get(from.clone(), definition_hash).ok_or(Error::<T>::NoPermission)?;

			ensure!(ids.contains(&(Compact(edition_id), Compact(copy_id))), Error::<T>::NoPermission);

			// first of all make sure the item can be transferred
			ensure!(
				(item_data.permissions & FragmentPerms::TRANSFER) == FragmentPerms::TRANSFER,
				Error::<T>::NoPermission
			);

			if let Some(new_perms) = new_permissions {
				// ensure we only allow more restrictive permissions
				if (item_data.permissions & FragmentPerms::EDIT) != FragmentPerms::EDIT {
					ensure!(
						(new_perms & FragmentPerms::EDIT) != FragmentPerms::EDIT,
						Error::<T>::NoPermission
					);
				}
				if (item_data.permissions & FragmentPerms::COPY) != FragmentPerms::COPY {
					ensure!(
						(new_perms & FragmentPerms::COPY) != FragmentPerms::COPY,
						Error::<T>::NoPermission
					);
				}
				if (item_data.permissions & FragmentPerms::TRANSFER) != FragmentPerms::TRANSFER {
					ensure!(
						(new_perms & FragmentPerms::TRANSFER) != FragmentPerms::TRANSFER,
						Error::<T>::NoPermission
					);
				}
			}

			Ok(())

		}

		/// Transfer a Fragment Instance from `from` to `to`
		///
		/// * `from` - Clamor Account ID to transfer the Fragment Instance from
		/// * `to` - Clamor Account ID to transfer the Fragment Instance to
		/// * `definition_hash` - Fragment Definition of the Fragment Instance
		/// * `edition_id` - Edition ID of the Fragment Instance
		/// * `copy_id` - Copy ID of the Fragment Instance
		/// * `new_permissions` - New Permissions of the Fragment Instance after it is sold
		/// * `expiration` -  Block number that the newly-copied Fragment Instance expires at. If the Fragment Instance is not copyable, this field is practically irrelevant.
		pub fn transfer_instance(
			from: &T::AccountId,
			to: &T::AccountId,
			definition_hash: &Hash128,
			edition_id: InstanceUnit,
			copy_id: InstanceUnit,
			new_permissions: Option<FragmentPerms>,
			expiration: Option<T::BlockNumber>,
		) -> DispatchResult {

			let mut item_data = <Fragments<T>>::get((definition_hash, edition_id, copy_id))
				.ok_or(Error::<T>::NotFound)?;

			let perms = if let Some(new_perms) = new_permissions {
				// ensure we only allow more restrictive permissions
				new_perms
			} else {
				item_data.permissions
			};

			// now we take two different paths if item can be copied or not
			if (item_data.permissions & FragmentPerms::COPY) == FragmentPerms::COPY {
				// we will copy the item to the new account
				item_data.permissions = perms;

				let copy: u64 = <CopiesCount<T>>::get((definition_hash, edition_id))
					.ok_or(Error::<T>::NotFound)?
					.into();

				let copy = copy + 1;

				<CopiesCount<T>>::insert((definition_hash, edition_id), Compact(copy));

				<Owners<T>>::append(definition_hash, to.clone(), (Compact(edition_id), Compact(copy)));

				<Inventory<T>>::append(
					to.clone(),
					definition_hash,
					(Compact(edition_id), Compact(copy)),
				);

				// handle expiration
				if let Some(expiring_at) = item_data.expiring_at {
					let expiration = if let Some(expiration) = expiration {
						if expiration < expiring_at {
							item_data.expiring_at = Some(expiration);
							expiration
						} else {
							expiring_at
						}
					} else {
						expiring_at
					};
					<Expirations<T>>::append(
						expiration,
						(definition_hash, Compact(edition_id), Compact(copy)),
					);
				} else if let Some(expiration) = expiration {
					item_data.expiring_at = Some(expiration);
					<Expirations<T>>::append(
						expiration,
						(definition_hash, Compact(edition_id), Compact(copy)),
					);
				}

				<Fragments<T>>::insert((definition_hash, edition_id, copy), item_data);

				Self::deposit_event(Event::InventoryAdded {
					account_id: to.clone(),
					definition_hash: *definition_hash,
					fragment_id: (edition_id, copy),
				});
			} else {
				// we will remove from this account to give to new account
				<Owners<T>>::mutate(definition_hash, from.clone(), |ids| {
					if let Some(ids) = ids {
						ids.retain(|cid| *cid != (Compact(edition_id), Compact(copy_id)))
					}
				});

				<Inventory<T>>::mutate(from.clone(), definition_hash, |ids| {
					if let Some(ids) = ids {
						ids.retain(|cid| *cid != (Compact(edition_id), Compact(copy_id)))
					}
				});

				Self::deposit_event(Event::InventoryRemoved {
					account_id: from.clone(),
					definition_hash: *definition_hash,
					fragment_id: (edition_id, copy_id),
				});

				<Owners<T>>::append(definition_hash, to.clone(), (Compact(edition_id), Compact(copy_id)));

				<Inventory<T>>::append(
					to.clone(),
					definition_hash,
					(Compact(edition_id), Compact(copy_id)),
				);

				Self::deposit_event(Event::InventoryAdded {
					account_id: to.clone(),
					definition_hash: *definition_hash,
					fragment_id: (edition_id, copy_id),
				});

				// finally fix permissions that might have changed
				<Fragments<T>>::mutate((definition_hash, edition_id, copy_id), |item_data| {
					if let Some(item_data) = item_data {
						item_data.permissions = perms;
					}
				});
			}

			Ok(())

		}

		/// Query the owner of a Fragment Instance. The return type is AccountId
		pub fn get_instance_owner_account_id(
			params: GetInstanceOwnerParams<Vec<u8>>,
		) -> Result<T::AccountId, Vec<u8>> {
			let definition_hash: Hash128 = hex::decode(params.definition_hash)
				.map_err(|_| "Failed to convert string to u8 slice")?
				.try_into()
				.map_err(|_| "Failed to convert u8 slice to Hash128")?;

			if params.copy_id
				> CopiesCount::<T>::get((definition_hash, params.edition_id))
				.unwrap_or(Compact(0))
				.into()
			{
				return Err("Instance not found".into());
			}

			let owner = Owners::<T>::iter_prefix(definition_hash)
				.find(|(_owner, vec_instances)| {
					vec_instances.iter().any(|(edition_id, copy_id)| {
						Compact(params.edition_id) == *edition_id
							&& Compact(params.copy_id) == *copy_id
					})
				})
				.ok_or("Owner not found (this should never happen)")?
				.0;

			Ok(owner)
		}
	}

	/// Implementation Block of `Pallet` specifically for RPC-related functions
	impl<T: Config> Pallet<T>
		where
			T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
	{
		// pub fn get_definitions_old(params: GetDefinitionsParams<T::AccountId, Vec<u8>>) -> Result<Vec<u8>, Vec<u8>> {
		//
		// 	let get_protos_params = GetProtosParams {
		// 		desc: params.desc,
		// 		from: params.from,
		// 		limit: params.limit,
		// 		metadata_keys: Vec::new(),
		// 		owner: params.owner,
		// 		return_owners: params.return_owners,
		// 		categories: params.categories,
		// 		tags: params.tags,
		// 		available: None,
		// 	};
		//
		// 	let map_protos: Map<String, Value> = pallet_protos::Pallet::<T>::get_protos_map(get_protos_params)?;
		// 	let map_protos_that_have_defs = map_protos
		// 		.into_iter()
		// 		.filter(|(proto_id, map_proto)| {
		// 			let array_proto_id: Hash256 = hex::decode(&proto_id).unwrap().try_into().unwrap(); // using `unwrao()` can lead to panicking
		// 			<Proto2Fragments<T>>::contains_key(&array_proto_id)
		// 		})
		// 		// .filter_map(|(proto_id, map_proto)| -> Option<_> {
		// 		// 	if let Ok(array_proto_id) = hex::decode(&proto_id) {
		// 		// 		if Ok(array_proto_id) = array_proto_id.try_into() {
		// 		// 			if <Proto2Fragments<T>>::contains_key(&array_proto_id) {
		// 		// 				Some((proto_id, map_proto))
		// 		// 			} else {
		// 		// 				None
		// 		// 			}
		// 		// 		} else {
		// 		// 			Some(Err("Failed to convert `proto_id` to Hash256".into()))
		// 		// 		}
		// 		// 	} else {
		// 		// 		Some(Err("`Failed to decode `proto_id``".into()))
		// 		// 	}
		// 		// })
		// 		.skip(params.from as usize)
		// 		.take(params.limit as usize)
		// 		.collect::<Map<_, _>>();
		//
		// 	let mut map_definitions = Map::new();
		//
		// 	for (proto_id, value_map_proto) in map_protos_that_have_defs.into_iter() {
		// 		let mut map_proto = match value_map_proto {
		// 			Value::Object(mp) => mp,
		// 			_ => return Err("Failed to get map_proto".into()),
		// 		};
		//
		// 		let array_proto_id = hex::decode(&proto_id).or(Err("`Failed to decode `proto_id``"))?;
		// 		let array_proto_id: Hash256 = array_proto_id.try_into().or(Err("Failed to convert `proto_id` to Hash256"))?;
		//
		// 		map_proto.insert(String::from("proto"), Value::String(proto_id));
		//
		// 		let list_definitions = <Proto2Fragments<T>>::get(&array_proto_id).ok_or("`proto_id` not found in `Proto2Fragments`")?;
		//
		// 		for definition in list_definitions.iter() {
		// 			map_definitions.insert(hex::encode(definition), Value::Object(map_proto.clone())); // TODO: currently using `map_proto.clone()` as a temp fix
		// 		}
		// 	}
		//
		// 	if !params.metadata_keys.is_empty() {
		// 		for (definition_id, map_definition) in map_definitions.iter_mut() {
		//
		// 			let map_definition = match map_definition {
		// 				Value::Object(map_definition) => map_definition,
		// 				_ => return Err("Failed to get map_definition".into()),
		// 			};
		//
		// 			let array_definition_id: Hash128 = if let Ok(array_definition_id) = hex::decode(definition_id) {
		// 				if let Ok(array_definition_id) = array_definition_id.try_into() {
		// 					array_definition_id
		// 				} else {
		// 					return Err("Failed to convert definition to Hash128".into());
		// 				}
		// 			} else {
		// 				return Err("Failed to decode definition_id".into());
		// 			};
		// 			let definition_metadata = if let Some(definition) = <Definitions<T>>::get(array_definition_id) {
		// 				definition.custom_metadata
		// 			} else {
		// 				return Err("Failed to get definition".into());
		// 			};
		// 			let mut map_of_matching_metadata_keys = pallet_protos::Pallet::<T>::get_map_of_matching_metadata_keys(&params.metadata_keys, &definition_metadata);
		// 			(*map_definition).append(&mut map_of_matching_metadata_keys);
		// 		}
		// 	}
		//
		// 	let result = json!(map_definitions).to_string();
		//
		// 	Ok(result.into_bytes())
		// }

		/// **Query** and **Return** **Fragment Definition(s)** based on **`params`**
		///
		/// The returned JSON string has the following format:
		///
		/// {
		/// 	<definition-hash>: {
		///			"name": <definition-name>,
		///			"owner": <definition-owner>,
		///			"metadata": {
		///				<metadata-key>: <data-hash>,
		/// 			...
		///			},
		/// 	...
		/// }
		pub fn get_definitions(
			params: GetDefinitionsParams<T::AccountId, Vec<u8>>,
		) -> Result<Vec<u8>, Vec<u8>> {
			let mut map = Map::new();

			let list_definitions_final: Vec<Hash128> = if let Some(owner) = params.owner {
				let list_protos_owner =
					<ProtosByOwner<T>>::get(ProtoOwner::<T::AccountId>::User(owner))
						.ok_or("Owner not found")?; // `owner` exists in `ProtosByOwner`
				if params.desc {
					// Sort in descending order
					list_protos_owner
						.into_iter()
						.rev()
						.filter_map(|proto_id| <Proto2Fragments<T>>::get(&proto_id))
						.flatten()
						.skip(params.from as usize)
						.take(params.limit as usize)
						.collect()
				} else {
					// Sort in ascending order
					list_protos_owner
						.into_iter()
						.filter_map(|proto_id| <Proto2Fragments<T>>::get(&proto_id))
						.flatten()
						.skip(params.from as usize)
						.take(params.limit as usize)
						.collect()
				}
			} else {
				<Definitions<T>>::iter_keys()
					.skip(params.from as usize)
					.take(params.limit as usize)
					.collect()
			};

			for definition_id in list_definitions_final.into_iter() {
				map.insert(hex::encode(definition_id), Value::Object(Map::new()));
			}

			for (definition_id, map_definition) in map.iter_mut() {
				let map_definition = match map_definition {
					Value::Object(map_definition) => map_definition,
					_ => return Err("Failed to get map_definition".into()),
				};

				let array_definition_id: Hash128 = hex::decode(definition_id)
					.or(Err("`Failed to decode `definition_id``"))?
					.try_into()
					.or(Err("Failed to convert `definition_id` to Hash128"))?;

				let num_instances: InstanceUnit =
					if let Some(editions) = <EditionsCount<T>>::get(array_definition_id) {
						let editions: InstanceUnit = editions.into();
						(1..=editions)
							.map(|edition_id| -> Result<InstanceUnit, _> {
								<CopiesCount<T>>::get((array_definition_id, edition_id))
									.map(Into::<InstanceUnit>::into)
									.ok_or("Number of Copies not found for an existing edition")
							})
							.sum::<Result<InstanceUnit, _>>()?
					} else {
						0
					};

				(*map_definition).insert("num_instances".into(), num_instances.into());

				let definition_struct = <Definitions<T>>::get(array_definition_id)
					.ok_or("Failed to get definition struct")?;

				(*map_definition).insert(
					"name".into(),
					String::from_utf8(definition_struct.metadata.name)
						.map_err(|_| "Failed to convert u8 vec to sring")?
						.into(),
				);
				// (*map_definition).insert("currency".into(), definition_struct.metadata.currency.into());

				if params.return_owners {
					let owner = <Protos<T>>::get(definition_struct.proto_hash)
						.ok_or("Failed to get proto struct")?
						.owner;
					let json_owner = pallet_protos::Pallet::<T>::get_owner_in_json_format(owner);
					(*map_definition).insert(String::from("owner"), json_owner);
				}

				if !params.metadata_keys.is_empty() {
					let definition_metadata = definition_struct.custom_metadata;
					let map_of_matching_metadata_keys =
						pallet_protos::Pallet::<T>::get_map_of_matching_metadata_keys(
							&params.metadata_keys,
							&definition_metadata,
						);
					(*map_definition)
						.insert("metadata".into(), map_of_matching_metadata_keys.into());
					// (*map_definition).append(&mut map_of_matching_metadata_keys);
				}
			}

			let result = json!(map).to_string();

			Ok(result.into_bytes())
		}

		/// **Query** and **Return** **Fragment Instance(s)** based on **`params`**
		///
		/// The returned JSON string has the following format:
		///
		/// {
		/// 	"<edition-id>.<copy-id>": {
		///			"name": <metadata-name>,
		///			"metadata": {
		///				<metadata-key>: <data-hash>,
		/// 			...
		///			},
		/// 	...
		/// }
		pub fn get_instances(
			params: GetInstancesParams<T::AccountId, Vec<u8>>,
		) -> Result<Vec<u8>, Vec<u8>> {
			let mut map = Map::new();

			let definition_hash: Hash128 = hex::decode(params.definition_hash)
				.map_err(|_| "Failed to convert string to u8 slice")?
				.try_into()
				.map_err(|_| "Failed to convert u8 slice to Hash128")?;

			let editions: u64 =
				<EditionsCount<T>>::get(&definition_hash).unwrap_or(Compact(0)).into();

			let list_tuple_edition_id_copy_id = if let Some(owner) = params.owner {
				<Inventory<T>>::get(owner, definition_hash)
					.unwrap_or_default()
					.into_iter()
					.map(|(c1, c2)| (c1.into(), c2.into()))
					.collect::<Vec<(InstanceUnit, InstanceUnit)>>()
			} else {
				(1..=editions)
					.map(|edition_id| -> Result<_, _> {
						let copies = if params.only_return_first_copies {
							1
						} else {
							<CopiesCount<T>>::get((definition_hash, edition_id))
								.ok_or("No Copies Found!")?
								.into()
						};
						Ok((edition_id, copies))
					})
					.collect::<Result<Vec<(u64, u64)>, Vec<u8>>>()?
					.into_iter()
					.flat_map(|(edition_id, copies)| {
						(1..=copies)
							.map(|copy_id| (edition_id, copy_id))
							.collect::<Vec<(u64, u64)>>()
					})
					.collect::<Vec<(InstanceUnit, InstanceUnit)>>()
			};

			list_tuple_edition_id_copy_id
				.into_iter()
				.skip(params.from as usize)
				.take(params.limit as usize)
				.try_for_each(|(edition_id, copy_id)| -> Result<(), Vec<u8>> {
					let mut map_instance = Map::new();

					let instance_struct =
						<Fragments<T>>::get((definition_hash, edition_id, copy_id))
							.ok_or("Instance not found")?;

					if !params.metadata_keys.is_empty() {
						let metadata = instance_struct
							.metadata
							.iter()
							.map(|(metadata_key_index, data_hash_index)| {
								let data_hash =
									<DataHashMap<T>>::get(definition_hash, data_hash_index)
										.ok_or::<Vec<u8>>("Data hash not found".into())?;
								Ok((metadata_key_index.clone(), data_hash))
							})
							.collect::<Result<BTreeMap<Compact<u64>, Hash256>, Vec<u8>>>()?;
						let map_of_matching_metadata_keys =
							pallet_protos::Pallet::<T>::get_map_of_matching_metadata_keys(
								&params.metadata_keys,
								&metadata,
							);
						map_instance
							.insert("metadata".into(), map_of_matching_metadata_keys.into());
					}

					map.insert(format!("{}.{}", edition_id, copy_id), map_instance.into());

					Ok(())
				})?;

			let result = json!(map).to_string();

			Ok(result.into_bytes())
		}

		/// Query the owner of a Fragment Instance. The return type is a String
		pub fn get_instance_owner(
			params: GetInstanceOwnerParams<Vec<u8>>,
		) -> Result<Vec<u8>, Vec<u8>> {
			let owner = Self::get_instance_owner_account_id(params)?;

			Ok(hex::encode(owner).into_bytes())
		}
	}
}
