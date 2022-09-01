//! This pallet `fragments` performs logic related to Fragment Definitions and Fragment Instances
//!
//! IMPORTANT STUFF TO KNOW:
//!
//! # Fragment Definition
//!
//! A Fragment Definition is created using a Proto-Fragment (see pallet `protos`).
//! A Fragment Definition's ID can be determinstically computed using its Proto-Fragment hash and
//! its metadata struct `FragmentMetadata`.
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

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod dummy_data;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod weights;

use codec::{Compact, Decode, Encode};
pub use pallet::*;
use sp_clamor::{Hash128, Hash256};
use sp_io::{
	hashing::{blake2_128, blake2_256},
	transaction_index,
};
use sp_std::{vec::Vec, collections::btree_map::BTreeMap};
pub use weights::WeightInfo;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use protos::{permissions::FragmentPerms, categories::Categories};

use frame_support::dispatch::DispatchResult;
use sp_runtime::traits::StaticLookup;

use frame_support::traits::{
	tokens::fungibles::Inspect, tokens::fungibles::Transfer, Currency, ExistenceRequirement,
};
use sp_runtime::SaturatedConversion;

use pallet_protos::GetProtosParams;
use scale_info::prelude::{
	format,
	string::{String, ToString},
};
use serde_json::{json, Map, Value};

type Unit = u64;

/// **Data Type** used to **Query and Filter for Fragment Definitions**
#[derive(Encode, Decode, Clone, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct GetDefinitionsParams<TAccountId, TString> {
	pub desc: bool,
	pub from: u64,
	pub limit: u64,
	pub metadata_keys: Vec<TString>,
	pub owner: Option<TAccountId>,
	pub return_owners: bool,
	pub categories: Vec<Categories>,
	pub tags: Vec<TString>,
}

/// **Data Type** used to **Query and Filter for Fragment Instances**
#[derive(Encode, Decode, Clone, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct GetInstancessParams {
	pub desc: bool,
	pub from: u64,
	pub limit: u64,
	pub definition_hash: Hash128,
}

/// **Struct** of a **Fragment Definition's Metadata**
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq)]
pub struct FragmentMetadata<TFungibleAsset> {
	/// **Name** of the **Fragment Definition**
	pub name: Vec<u8>,
	/// **Currency** that the **buyer** of a **Fragment Instance that is created from the Fragment Definition** must **pay in**.
	/// If this field is `None`, the currency the buyer must pay in is NOVA.
	pub currency: Option<TFungibleAsset>,
}

#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq)]
pub struct UniqueOptions {
	pub mutable: bool,
}

/// **Struct** of a **Fragment Definition**
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq)]
pub struct FragmentDefinition<TFungibleAsset, TAccountId, TBlockNum> {
	/// **Proto-Fragment used** to **create** the **Fragment**
	pub proto_hash: Hash256,
	/// ***FragmentMetadata* Struct** (the **struct** contains the **Fragment's name**, among other things)
	pub metadata: FragmentMetadata<TFungibleAsset>,
	/// **Set of Actions** (encapsulated in a `FragmentPerms` bitflag enum) that are **allowed to be done** to
	/// **any Fragment Instance** when it **first gets created** from the **Fragment Definition** (e.g edit, transfer etc.)
	///
	/// These **allowed set of actions of the Fragment Instance** ***may change***
	/// when the **Fragment Instance is given to another account ID** (see the `give` extrinsic).
	pub permissions: FragmentPerms,
	// Notes from Giovanni:
	//
	// If Fragment Instances (created from the Fragment Definition) must contain unique data when created (injected by buyers, validated by the system)
	/// Whether the **Fragment Definition** is **mutable**
	pub unique: Option<UniqueOptions>,
	/// If scarce, the max supply of the Fragment
	pub max_supply: Option<Compact<Unit>>,
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
	/// Next owner permissions, owners can change those if they want to more restrictive ones, never more permissive
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
	pub amount: Option<Compact<Unit>>,
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
	pub units_left: Option<Compact<Unit>>,
	/// Block number that the sale ends at (*optional*)
	pub expiration: Option<TBlockNum>,
	/// If the Fragment instance represents a **stack of stackable items** (for e.g gold coins or arrows - https://runescape.fandom.com/wiki/Stackable_items),
	/// the **number of items** to **top up** in the **stack of stackable items** // EMERICK
	pub amount: Option<Compact<Unit>>,
}

/// **Enum** indicating whether to
/// **create one Fragment Instance with custom data attached to it**
/// or whether to
/// **create multiple Fragment Instances (with no custom data attached)**
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq)]
pub enum FragmentBuyOptions {
	/// Create create *"x"* Number of Fragment Instances to create,
	/// where *"x"* is the associated `u64` value inside the enum variant
	Quantity(u64),
	/// Create a single Fragment Instance with custom data *"x"* attached to it,
	/// where *"x"* is the assosicated `Vec<u8>` value inside the enum variant
	UniqueData(Vec<u8>),
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, Twox64Concat};
	use frame_system::pallet_prelude::*;
	use pallet_detach::DetachedHashes;
	use pallet_protos::{Proto, ProtoOwner, Protos, MetaKeys, MetaKeysIndex};

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_protos::Config + pallet_assets::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
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
	/// **StorageMap** that maps a **Fragment Definition ID (which is determinstically computed using its Proto-Fragment hash and its metadata struct `FragmentMetadata`)**
	/// to a
	/// ***FragmentDefinition* struct**
	#[pallet::storage]
	pub type Definitions<T: Config> = StorageMap<
		_,
		Identity,
		Hash128,
		FragmentDefinition<T::AssetId, T::AccountId, T::BlockNumber>,
	>;

	/// **StorageMap** that maps a **Fragment Definition ID (which is determinstically computed using its Proto-Fragment hash and its metadata struct `FragmentMetadata`)**
	/// to a
	/// ***PublishingData* struct (of the aforementioned Fragment Definition)**
	#[pallet::storage]
	pub type Publishing<T: Config> =
		StorageMap<_, Identity, Hash128, PublishingData<T::BlockNumber>>;

	/// **StorageMap** that maps a **Fragment Definition ID**
	/// to the
	/// **total number of unique Edition IDs** found in the
	/// **Fragment Instances that have the aforementioned Fragment Definition ID**
	#[pallet::storage]
	pub type EditionsCount<T: Config> = StorageMap<_, Identity, Hash128, Compact<Unit>>;

	/// **StorageMap** that maps a **tuple that contains a Fragment Definition ID and an Edition ID**
	/// to the
	/// **total number of Fragment Instances that have the Fragment Definition ID and the Edition ID**
	#[pallet::storage]
	pub type CopiesCount<T: Config> = StorageMap<_, Identity, (Hash128, Unit), Compact<Unit>>;

	/// **StorageNMap** that maps the **Fragment Definition ID of a Fragment Instance,
	/// the Fragment Edition ID of the aforementioned Fragment Instance and
	/// the Copy ID of the aforementioned Fragment Instance**
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
			storage::Key<Identity, Unit>,
			// Copies
			storage::Key<Identity, Unit>,
		),
		FragmentInstance<T::BlockNumber>,
	>;

	#[pallet::storage]
	pub type UniqueData2Edition<T: Config> = StorageDoubleMap<
		_,
		Identity,
		Hash128, // Fragment Definition ID
		Identity,
		Hash256, // Unique Data's Hash
		Unit,    // Edition ID
	>;

	/// StorageDoubleMap that maps a **Fragment Definition and the
	/// Owner of a Fragment Instance that was created from the aforementioned Fragment Definition**
	/// to a
	/// **tuple that contains the Fragment Instance's Edition ID and the Fragment Instance's Copy ID**
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
		Vec<(Compact<Unit>, Compact<Unit>)>,
	>;

	/// StorageDoubleMap that maps the **Owner of a Fragment Instance and the Fragment Instance's Fragment Definition**
	/// to a
	/// **tuple that contains the Fragment Instance's Edition ID and the Fragment Instance's Copy ID**
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
		Vec<(Compact<Unit>, Compact<Unit>)>,
	>;

	/// StorageMap that maps the **Block Number that a Fragment Instance expires at**
	/// to a
	/// **tuple that contains the Fragment Instance's Fragment Definition ID, the Fragment Instance's Edition ID and
	/// the Fragment Instance's Copy ID**
	///
	/// Footnotes:
	///
	///  Fragment Instances can expire, we process expirations every `on_finalize`
	#[pallet::storage]
	pub type Expirations<T: Config> =
		StorageMap<_, Twox64Concat, T::BlockNumber, Vec<(Hash128, Compact<Unit>, Compact<Unit>)>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New definition created by account, definition hash
		DefinitionCreated { fragment_hash: Hash128 },
		/// A Proto-Fragment metadata has changed
		DefinitionMetadataChanged { fragment_hash: Hash128, metadata_key: Vec<u8> },
		/// Fragment sale has been opened
		Publishing { fragment_hash: Hash128 },
		/// Fragment sale has been closed
		Unpublishing { fragment_hash: Hash128 },
		/// Inventory item has been added to account
		InventoryAdded {
			account_id: T::AccountId,
			fragment_hash: Hash128,
			fragment_id: (Unit, Unit),
		},
		/// Inventory item has removed added from account
		InventoryRemoved {
			account_id: T::AccountId,
			fragment_hash: Hash128,
			fragment_id: (Unit, Unit),
		},
		/// Inventory has been updated
		InventoryUpdated {
			account_id: T::AccountId,
			fragment_hash: Hash128,
			fragment_id: (Unit, Unit),
		},
		/// Fragment Expiration event
		Expired { account_id: T::AccountId, fragment_hash: Hash128, fragment_id: (Unit, Unit) },
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
		/// Already detached
		Detached,
		/// Already exist
		AlreadyExist,
		/// Not found
		NotFound,
		/// Sale has expired
		Expired,
		/// Insufficient funds
		InsufficientBalance,
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
		#[pallet::weight(<T as Config>::WeightInfo::create())]
		pub fn create(
			origin: OriginFor<T>,
			proto_hash: Hash256,
			metadata: FragmentMetadata<T::AssetId>,
			permissions: FragmentPerms,
			unique: Option<UniqueOptions>,
			max_supply: Option<Unit>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let proto: Proto<T::AccountId, T::BlockNumber> =
				<Protos<T>>::get(proto_hash).ok_or(Error::<T>::ProtoNotFound)?; // Get `Proto` struct from `proto_hash`

			let proto_owner: T::AccountId = match proto.owner {
				// Get `proto_owner` from `proto.owner`
				ProtoOwner::User(owner) => Ok(owner),
				_ => Err(Error::<T>::ProtoOwnerNotFound),
			}?;

			ensure!(who == proto_owner, Error::<T>::NoPermission); // Only proto owner can create a fragment definition from proto

			ensure!(!<DetachedHashes<T>>::contains_key(&proto_hash), Error::<T>::Detached); // proto must not have been detached

			let hash = blake2_128(
				// This is the unique id of the Fragment Definition that will be created
				&[&proto_hash[..], &metadata.name.encode(), &metadata.currency.encode()].concat(),
			);

			ensure!(!<Definitions<T>>::contains_key(&hash), Error::<T>::AlreadyExist); // If fragment already exists, throw error

			if let Some(currency) = metadata.currency {
				ensure!(pallet_assets::Pallet::<T>::maybe_total_supply(currency).is_some(), Error::<T>::CurrencyNotFound); // If it is `None`, this means the asset ID `currency` doesn't exist
			}

			let current_block_number = <frame_system::Pallet<T>>::block_number();

			// ! Writing

			// create vault account
			// we need an existential amount deposit to be able to create the vault account
			let vault = Self::get_vault_id(hash);
			let min_balance =
				<pallet_balances::Pallet<T> as Currency<T::AccountId>>::minimum_balance();
			let _ = <pallet_balances::Pallet<T> as Currency<T::AccountId>>::deposit_creating(
				&vault,
				min_balance,
			);

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

			Self::deposit_event(Event::DefinitionCreated { fragment_hash: hash });
			Ok(())
		}

		/// **Alters** the **custom metadata** of a **Fragment Definition** (whose ID is `fragment_hash`) by **adding or modifying a key-value pair** (`metadata_key.clone`,`blake2_256(&data.encode())`)
		/// to the **BTreeMap field `custom_metadata`** of the **existing Fragment Definition's Struct Instance**.
		/// Furthermore, this function also indexes `data` in the Blockchain's Database and stores it in the IPFS
		///
		/// # Arguments
		///
		/// * `origin` - The origin of the extrinsic / dispatchable function
		/// * `fragment_hash` - **ID of the Fragment Definition**
		/// * `metadata_key` - The key (of the key-value pair) that is added in the BTreeMap field `custom_metadata` of the existing Fragment Definition's Struct Instance
		/// * `data` - The hash of `data` is used as the value (of the key-value pair) that is added in the BTreeMap field `custom_metadata` of the existing Fragment Definition's Struct Instance
		#[pallet::weight(50_000)]
		pub fn set_custom_metadata(
			origin: OriginFor<T>,
			// fragment hash we want to update
			fragment_hash: Hash128,
			// Think of "Vec<u8>" as String (something to do with WASM - that's why we use Vec<u8>)
			metadata_key: Vec<u8>,
			// data we want to update last because of the way we store blocks (storage chain)
			data: Vec<u8>,
		) -> DispatchResult {

			let who = ensure_signed(origin)?;

			let proto_hash =
				<Definitions<T>>::get(fragment_hash).ok_or(Error::<T>::NotFound)?.proto_hash; // Get `proto_hash` from `fragment_hash`
			let proto: Proto<T::AccountId, T::BlockNumber> =
				<Protos<T>>::get(proto_hash).ok_or(Error::<T>::ProtoNotFound)?;
			let proto_owner: T::AccountId = match proto.owner { // Get `proto_owner` from `proto`
				ProtoOwner::User(owner) => Ok(owner),
				_ => Err(Error::<T>::ProtoOwnerNotFound),
			}?;
			ensure!(who == proto_owner, Error::<T>::NoPermission); // Ensure `who` is `proto_owner`

			// TO REVIEW
			ensure!(!<DetachedHashes<T>>::contains_key(&proto_hash), Error::<T>::Detached); // Ensure `proto_hash` isn't detached

			let data_hash = blake2_256(&data);

			// we need this to index transactions
			let extrinsic_index = <frame_system::Pallet<T>>::extrinsic_index()
				.ok_or(Error::<T>::SystematicFailure)?;

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

			// Write STATE from now, ensure no errors from now...

			<Definitions<T>>::mutate(&fragment_hash, |definition| {
				let definition = definition.as_mut().unwrap();
				// update custom metadata
				definition.custom_metadata.insert(metadata_key_index, data_hash);
			});

			// index data
			transaction_index::index(extrinsic_index, data.len() as u32, data_hash);

			// also emit event
			Self::deposit_event(Event::DefinitionMetadataChanged { fragment_hash, metadata_key: metadata_key.clone() });

			log::debug!("Added metadata to fragment definition: {:x?} with key: {:x?}", fragment_hash, metadata_key);

			Ok(())
		}

		/// Put the **Fragment Definition `fragment_hash`** on sale. When a Fragment Definition is put on sale, users can create Fragment Instances from it for a fee.
		///
		/// Note: **Only** the **Fragment's Proto-Fragment's owner** is **allowed** to put the **Fragment** on sale
		///
		/// # Arguments
		///
		/// * `origin` - **Origin** of the **extrinsic function**
		/// * `fragment_hash` - ID of the **Fragment Definition** to put on sale
		/// * `price` -  **Price** to **buy** a **single Fragment Instance** that is created from the **Fragment Definition*
		/// * `quantity` (*optional*) - **Maximum amount of Fragment Instances** that **can be bought**
		/// * `expires` (*optional*) - **Block number** that the sale ends at (*optional*)
		/// * `amount` (*optional*) - If the Fragment instance represents a **stack of stackable items** (for e.g gold coins or arrows - https://runescape.fandom.com/wiki/Stackable_items),
		/// the **number of items** to **top up** in the **stack of stackable items**
		#[pallet::weight(50_000)]
		pub fn publish(
			origin: OriginFor<T>,
			fragment_hash: Hash128,
			price: u128,
			quantity: Option<Unit>,
			expires: Option<T::BlockNumber>,
			amount: Option<Unit>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let proto_hash =
				<Definitions<T>>::get(fragment_hash).ok_or(Error::<T>::NotFound)?.proto_hash; // Get `proto_hash` from `fragment_hash`
			let proto: Proto<T::AccountId, T::BlockNumber> =
				<Protos<T>>::get(proto_hash).ok_or(Error::<T>::ProtoNotFound)?; // Get `proto` from `proto_hash`

			let proto_owner: T::AccountId = match proto.owner {
				// Get `proto_owner` from `proto`
				ProtoOwner::User(owner) => Ok(owner),
				_ => Err(Error::<T>::ProtoOwnerNotFound),
			}?;

			ensure!(who == proto_owner, Error::<T>::NoPermission); // Ensure `who` is `proto_owner`

			// TO REVIEW
			ensure!(!<DetachedHashes<T>>::contains_key(&proto_hash), Error::<T>::Detached); // Ensure `proto_hash` isn't detached

			ensure!(!<Publishing<T>>::contains_key(&fragment_hash), Error::<T>::SaleAlreadyOpen); // Ensure `fragment_hash` isn't already published

			let fragment_data = <Definitions<T>>::get(fragment_hash).ok_or(Error::<T>::NotFound)?; // Get `FragmentDefinition` struct from `fragment_hash`

			if let Some(max_supply) = fragment_data.max_supply {
				let max: Unit = max_supply.into();
				let existing: Unit =
					<EditionsCount<T>>::get(&fragment_hash).unwrap_or(Compact(0)).into();
				let left = max.saturating_sub(existing); // `left` = `max` - `existing`
				if let Some(quantity) = quantity {
					let quantity: Unit = quantity.into();
					ensure!(quantity <= left, Error::<T>::MaxSupplyReached); // Ensure that the function parameter `quantity` is smaller than or equal to `left`
				} else {
					// Ensure that if `fragment_data.max_supply` exists, the function parameter `quantity` must also exist
					return Err(Error::<T>::ParamsNotValid.into());
				}
				if left == 0 {
					return Err(Error::<T>::MaxSupplyReached.into());
				}
			}

			// ! Writing

			<Publishing<T>>::insert(
				fragment_hash,
				PublishingData {
					price: Compact(price),
					units_left: quantity.map(|x| Compact(x)),
					expiration: expires,
					amount: amount.map(|x| Compact(x)),
				},
			);

			Self::deposit_event(Event::Publishing { fragment_hash });

			Ok(())
		}

		/// Take the **Fragment Definition `fragment_hash`** off sale.
		/// When a Fragment Definition is put on sale, users can create Fragment Instances from it for a fee.
		///
		/// Note: **Only** the **Fragment's Proto-Fragment's owner** is **allowed** to take the Fragment off sale
		///
		/// # Arguments
		///
		/// * `origin` - **Origin** of the **extrinsic function**
		/// * `fragment_hash` - **ID** of the **Fragment Definition** to take off sale
		#[pallet::weight(50_000)]
		pub fn unpublish(origin: OriginFor<T>, fragment_hash: Hash128) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let proto_hash =
				<Definitions<T>>::get(fragment_hash).ok_or(Error::<T>::NotFound)?.proto_hash; // Get `proto_hash` from `fragment_hash`
			let proto: Proto<T::AccountId, T::BlockNumber> =
				<Protos<T>>::get(proto_hash).ok_or(Error::<T>::ProtoNotFound)?;

			let proto_owner: T::AccountId = match proto.owner {
				// Get `proto_owner` from `proto`
				ProtoOwner::User(owner) => Ok(owner),
				_ => Err(Error::<T>::ProtoOwnerNotFound),
			}?;

			ensure!(who == proto_owner, Error::<T>::NoPermission); // Ensure `who` is `proto_owner`

			// TO REVIEW
			ensure!(!<DetachedHashes<T>>::contains_key(&proto_hash), Error::<T>::Detached); // Ensure `proto_hash` isn't detached

			ensure!(<Publishing<T>>::contains_key(&fragment_hash), Error::<T>::NotFound); // Ensure `fragment_hash` is currently published

			// ! Writing

			<Publishing<T>>::remove(&fragment_hash); // Remove Fragment Definition `fragment_hash` from `Publishing`

			Self::deposit_event(Event::Unpublishing { fragment_hash });

			Ok(())
		}

		/// Create **Fragment instance(s)** from the **Fragment Definition `fragment_hash`** and
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
		/// * `fragment_hash` - **ID* of the **Fragment Definition**
		/// * `options` - **Enum** indicating whether to
		/// **create one Fragment Instance with custom data attached to it** or whether to
		/// **create multiple Fragment Instances (with no custom data attached)**
		/// * `amount` (*optional*) - If the Fragment Instance(s) represent a **stack of stackable items**
		/// (for e.g gold coins or arrows - https://runescape.fandom.com/wiki/Stackable_items),
		/// `amount` is the **number of items** to **top up** in the **stack of stackable items**
		#[pallet::weight(50_000)]
		pub fn mint(
			origin: OriginFor<T>,
			fragment_hash: Hash128,
			options: FragmentBuyOptions,
			amount: Option<Unit>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let current_block_number = <frame_system::Pallet<T>>::block_number();

			let proto_hash =
				<Definitions<T>>::get(fragment_hash).ok_or(Error::<T>::NotFound)?.proto_hash; // Get `proto_hash` from `fragment_hash`
			let proto: Proto<T::AccountId, T::BlockNumber> =
				<Protos<T>>::get(proto_hash).ok_or(Error::<T>::ProtoNotFound)?; // Get `proto` from `proto_hash`

			let proto_owner: T::AccountId = match proto.owner {
				// Get `proto_owner` from `proto`
				ProtoOwner::User(owner) => Ok(owner),
				_ => Err(Error::<T>::ProtoOwnerNotFound),
			}?;

			ensure!(who == proto_owner, Error::<T>::NoPermission); // Ensure `who` is `proto_owner`

			// TO REVIEW
			ensure!(!<DetachedHashes<T>>::contains_key(&proto_hash), Error::<T>::Detached); // Ensure `proto_hash` isn't detached

			let quantity = match options {
				// Number of fragment instances to mint
				FragmentBuyOptions::Quantity(amount) => u64::from(amount),
				_ => 1u64,
			};

			// ! Writing

			Self::mint_fragments(
				&who,
				&fragment_hash,
				None, // PublishingData (optional)
				&options,
				quantity,
				current_block_number,
				None, // Block Number the Fragment(s) expire at (optional)
				amount.map(|x| Compact(x)),
			)
		}

		/// Allows the Caller Account ID `origin` to create Fragment instance(s) of the Fragment Definition `fragment_hash`,
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
		/// * `fragment_hash` - **ID** of the **Fragment Definition**
		/// * `options` - **Enum** indicating whether to
		/// **create one Fragment Instance with custom data attached to it** or whether to
		/// **create multiple Fragment Instances (with no custom data attached)**
		#[pallet::weight(50_000)]
		pub fn buy(
			origin: OriginFor<T>,
			fragment_hash: Hash128,
			options: FragmentBuyOptions,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let current_block_number = <frame_system::Pallet<T>>::block_number();

			let sale = <Publishing<T>>::get(&fragment_hash).ok_or(Error::<T>::NotFound)?; // if Fragment Definition `fragment_hash` is not published (i.e on sale), you cannot buy it
			if let Some(expiration) = sale.expiration {
				ensure!(current_block_number < expiration, Error::<T>::Expired);
			}

			if let Some(units_left) = sale.units_left {
				ensure!(units_left > Compact(0), Error::<T>::SoldOut);
			}

			let price: u128 = sale.price.into();

			let fragment_data = <Definitions<T>>::get(fragment_hash).ok_or(Error::<T>::NotFound)?;

			let vault = &Self::get_vault_id(fragment_hash); // Get the Vault Account ID of `fragment_hash`

			let quantity = match options {
				FragmentBuyOptions::Quantity(amount) => u64::from(amount),
				_ => 1u64,
			};

			let price = price.saturating_mul(quantity as u128); // `price` = `price` * `quantity`

			if let Some(currency) = fragment_data.metadata.currency {
				let minimum_balance_needed_to_exist =
					<pallet_assets::Pallet<T> as Inspect<T::AccountId>>::minimum_balance(currency);
				let price_balance: <pallet_assets::Pallet<T> as Inspect<T::AccountId>>::Balance =
					price.saturated_into();

				ensure!(
					<pallet_assets::Pallet<T> as Inspect<T::AccountId>>::balance(currency, &who)
						>= price_balance + minimum_balance_needed_to_exist,
					Error::<T>::InsufficientBalance
				);
			} else {
				let minimum_balance_needed_to_exist =
					<pallet_balances::Pallet<T> as Currency<T::AccountId>>::minimum_balance();
				let price_balance: <pallet_balances::Pallet<T> as Currency<T::AccountId>>::Balance =
					price.saturated_into();

				ensure!(
					<pallet_balances::Pallet<T> as Currency<T::AccountId>>::free_balance(&who)
						>= price_balance + minimum_balance_needed_to_exist,
					Error::<T>::InsufficientBalance
				);
			}

			// ! Writing

			Self::mint_fragments(
				&who,
				&fragment_hash,
				Some(&sale), // PublishingData (optional)
				&options,
				quantity,
				current_block_number,
				None, // Block Number that the Fragment Instance will expire at (optional)
				sale.amount,
			)?;

			if let Some(currency) = fragment_data.metadata.currency {
				<pallet_assets::Pallet<T> as Transfer<T::AccountId>>::transfer(
					// transfer `price` units of `currency` from `who` to `vault`
					currency,
					&who,
					&vault,
					price.saturated_into(),
					true, // The debited account must stay alive at the end of the operation; an error is returned if this cannot be achieved legally.
				)
				.map_err(|_| Error::<T>::InsufficientBalance)?;
			} else {
				<pallet_balances::Pallet<T> as Currency<T::AccountId>>::transfer(
					// transfer `price` units of NOVA from `who` to `vault`
					&who,
					&vault,
					price.saturated_into(),
					ExistenceRequirement::KeepAlive,
				)
				.map_err(|_| Error::<T>::InsufficientBalance)?;
			}

			Ok(())
		}

		/// Give the **Fragment Instance whose Fragment Definition ID is `class`, whose Edition ID is `edition` and whose Copy ID is `copy`** to **`to`**.
		///
		/// If the **current permitted actions of the Fragment Instance** allows for it to be duplicated (i.e if it has the permission **FragmentPerms::COPY**),
		/// then it is duplicated and the duplicate's ownership is assigned to `to`.
		/// Otherwise, its ownernship is transferred from `origin` to `to`.
		///
		/// Note: **Only** the **Fragment Instance's owner** is **allowed** to give the Fragment Instance
		///
		/// # Arguments
		///
		/// * `origin` - **Origin** of the **extrinsic function**
		/// * `class` - Fragment Definition ID of the Fragment Instance to give
		/// * `edition` - Edition ID of the Fragment Insance to give
		/// * `copy` - Copy ID of the Fragment instance to give
		/// * `to` - **Account ID** to give the Fragment instance to
		///
		/// * `new_permissions` (*optional*) - The permitted set of actions (encapsulated in a `FragmentPerms` bitflag enum)
		/// that the account that is given the Fragment instance can do with it.
		///
		/// Note: `new_permissions` must be a subset of the current `permissions` field of the Fragment Instance;
		/// therefore, the `new_permissions` can only be more restrictive (than the current `permissions` field of the Fragment Instance),
		/// never more permissive
		///
		/// * `expiration` (*optional*) - Block number that the duplicated Fragment Instance expires at.
		/// If the Fragment Instance was not duplicated, this parameter is irrelevant.
		#[pallet::weight(50_000)]
		pub fn give(
			origin: OriginFor<T>,
			class: Hash128,
			edition: Unit,
			copy: Unit,
			to: <T::Lookup as StaticLookup>::Source,
			new_permissions: Option<FragmentPerms>,
			expiration: Option<T::BlockNumber>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let current_block_number = <frame_system::Pallet<T>>::block_number();

			let mut item_data =
				<Fragments<T>>::get((class, edition, copy)).ok_or(Error::<T>::NotFound)?;

			// no go if will expire this block
			if let Some(item_expiration) = item_data.expiring_at {
				ensure!(current_block_number < item_expiration, Error::<T>::NotFound);
			}

			if let Some(expiration) = expiration {
				ensure!(current_block_number < expiration, Error::<T>::ParamsNotValid);
			}

			// Only the owner of this fragment can transfer it
			let ids = <Inventory<T>>::get(who.clone(), class).ok_or(Error::<T>::NotFound)?;

			ensure!(ids.contains(&(Compact(edition), Compact(copy))), Error::<T>::NoPermission);

			// first of all make sure the item can be transferred
			ensure!(
				(item_data.permissions & FragmentPerms::TRANSFER) == FragmentPerms::TRANSFER,
				Error::<T>::NoPermission
			);

			let perms = if let Some(new_perms) = new_permissions {
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
				new_perms
			} else {
				item_data.permissions
			};

			let to = T::Lookup::lookup(to)?;

			// now we take two different paths if item can be copied or not
			if (item_data.permissions & FragmentPerms::COPY) == FragmentPerms::COPY {
				// we will copy the item to the new account
				item_data.permissions = perms;

				let copy: u64 =
					<CopiesCount<T>>::get((class, edition)).ok_or(Error::<T>::NotFound)?.into();

				let copy = copy + 1;

				<CopiesCount<T>>::insert((class, edition), Compact(copy));

				<Owners<T>>::append(class, to.clone(), (Compact(edition), Compact(copy)));

				<Inventory<T>>::append(to.clone(), class, (Compact(edition), Compact(copy)));

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
					<Expirations<T>>::append(expiration, (class, Compact(edition), Compact(copy)));
				} else if let Some(expiration) = expiration {
					item_data.expiring_at = Some(expiration);
					<Expirations<T>>::append(expiration, (class, Compact(edition), Compact(copy)));
				}

				<Fragments<T>>::insert((class, edition, copy), item_data);

				Self::deposit_event(Event::InventoryAdded {
					account_id: to,
					fragment_hash: class,
					fragment_id: (edition, copy),
				});
			} else {
				// we will remove from this account to give to new account
				<Owners<T>>::mutate(class, who.clone(), |ids| {
					if let Some(ids) = ids {
						ids.retain(|cid| *cid != (Compact(edition), Compact(copy)))
					}
				});

				<Inventory<T>>::mutate(who.clone(), class, |ids| {
					if let Some(ids) = ids {
						ids.retain(|cid| *cid != (Compact(edition), Compact(copy)))
					}
				});

				Self::deposit_event(Event::InventoryRemoved {
					account_id: who.clone(),
					fragment_hash: class,
					fragment_id: (edition, copy),
				});

				<Owners<T>>::append(class, to.clone(), (Compact(edition), Compact(copy)));

				<Inventory<T>>::append(to.clone(), class, (Compact(edition), Compact(copy)));

				Self::deposit_event(Event::InventoryAdded {
					account_id: to,
					fragment_hash: class,
					fragment_id: (edition, copy),
				});

				// finally fix permissions that might have changed
				<Fragments<T>>::mutate((class, edition, copy), |item_data| {
					if let Some(item_data) = item_data {
						item_data.permissions = perms;
					}
				});
			}

			Ok(())
		}

		/// Create the **Account ID** of the **Fragment Instance whose Fragment Definition ID is `class`,
		/// whose Edition ID is `edition`** and whose Copy ID is `copy`**
		///
		/// # Arguments
		///
		/// * `origin` - **Origin** of the **extrinsic function**
		/// * `class` - **Fragment Definition 	ID** of the **Fragment Instance**
		/// * `edition` - **Edition ID** of the **Fragment Instance**
		/// * `copy` - **Copy ID** of the **Fragment Instance**
		#[pallet::weight(50_000)]
		pub fn create_account(
			origin: OriginFor<T>,
			class: Hash128,
			edition: Unit,
			copy: Unit,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Only the owner of this fragment can transfer it
			let ids = <Inventory<T>>::get(who.clone(), class).ok_or(Error::<T>::NotFound)?;

			ensure!(ids.contains(&(Compact(edition), Compact(copy))), Error::<T>::NoPermission);

			// create an account for a specific fragment
			// we need an existential amount deposit to be able to create the vault account
			let frag_account = Self::get_fragment_account_id(class, edition, copy);
			let min_balance =
				<pallet_balances::Pallet<T> as Currency<T::AccountId>>::minimum_balance();
			let _ = <pallet_balances::Pallet<T> as Currency<T::AccountId>>::deposit_creating(
				&frag_account,
				min_balance,
			);

			// TODO Make owner pay for deposit actually!
			// TODO setup proxy

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
								fragment_hash: item.0,
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
}

impl<T: Config> Pallet<T> {
	/// **Get** the **Account ID** of the Fragment Definition `class_hash`**
	///
	/// This Account ID is determinstically computed using the Fragment Definition `class_hash`
	pub fn get_vault_id(class_hash: Hash128) -> T::AccountId {
		let hash = blake2_256(&[&b"fragments-vault"[..], &class_hash].concat());
		T::AccountId::decode(&mut &hash[..]).expect("T::AccountId should decode")
	}

	/// Get the **Account ID** of the **Fragment Instance whose Fragment Definition ID is `class_hash`,
	/// whose Edition ID is `edition`** and whose Copy ID is `copy`**
	///
	/// This Account ID is determinstically computed using the Fragment Definition ID `class_hash`, the Edition ID `edition` and the Copy ID `copy`
	pub fn get_fragment_account_id(class_hash: Hash128, edition: Unit, copy: Unit) -> T::AccountId {
		let hash = blake2_256(
			&[&b"fragments-account"[..], &class_hash, &edition.encode(), &copy.encode()].concat(),
		);
		T::AccountId::decode(&mut &hash[..]).expect("T::AccountId should decode")
	}

	/// Create `quantity` number of Fragment Instances from the Fragment Definition `fragment_hash` and assigns their ownership to `to`
	///
	/// # Arguments
	///
	/// * `to` - **Account ID** to assign ownernship of the created Fragment instances to
	/// * `fragment_hash` - ID of the Fragment Definition
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
		fragment_hash: &Hash128,
		sale: Option<&PublishingData<T::BlockNumber>>,
		options: &FragmentBuyOptions,
		quantity: u64,
		current_block_number: T::BlockNumber,
		expiring_at: Option<T::BlockNumber>,
		amount: Option<Compact<Unit>>,
	) -> DispatchResult {
		use frame_support::ensure;

		if let Some(expiring_at) = expiring_at {
			ensure!(expiring_at > current_block_number, Error::<T>::ParamsNotValid); // Ensure `expiring_at` > `current_block_number`
		}

		let fragment_data = <Definitions<T>>::get(fragment_hash).ok_or(Error::<T>::NotFound)?;

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
					!<UniqueData2Edition<T>>::contains_key(fragment_hash, data_hash),
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

		let existing: Unit = <EditionsCount<T>>::get(&fragment_hash).unwrap_or(Compact(0)).into();

		if let Some(sale) = sale {
			// if limited amount let's reduce the amount of units left
			if let Some(units_left) = sale.units_left {
				if quantity > units_left.into() {
					return Err(Error::<T>::PublishedQuantityReached.into());
				} else {
					<Publishing<T>>::mutate(&*fragment_hash, |sale| {
						if let Some(sale) = sale {
							let left: Unit = units_left.into();
							sale.units_left = Some(Compact(left - quantity));
						}
					});
				}
			}
		} else {
			// We still don't wanna go over supply limit
			if let Some(max_supply) = fragment_data.max_supply {
				let max: Unit = max_supply.into();
				let left = max.saturating_sub(existing); // `left` = `max` - `existing`
				if quantity > left {
					// Ensure the function parameter `quantity` is smaller than or equal to `left`
					return Err(Error::<T>::MaxSupplyReached.into());
				}
			}
		}

		// ! Writing if successful

		<Definitions<T>>::mutate(fragment_hash, |fragment| {
			// Get the `FragmentDefinition` struct from `fragment_hash`
			if let Some(fragment) = fragment {
				for id in existing..(existing + quantity) {
					let id = id + 1u64;
					let cid = Compact(id); // `cid` stands for "compact id"

					<Fragments<T>>::insert(
						(fragment_hash, id, 1),
						FragmentInstance {
							permissions: fragment.permissions,
							created_at: current_block_number,
							custom_data: data_hash,
							expiring_at,
							amount,
						},
					);

					<CopiesCount<T>>::insert((fragment_hash, id), Compact(1));

					<Inventory<T>>::append(to.clone(), fragment_hash, (cid, Compact(1))); // **Add** the **Fragment Intstance whose Fragment Definition is `fragment_hash`, Edition ID is `cid` and Copy ID is 1**  to the **inventory of `to`**

					<Owners<T>>::append(fragment_hash, to.clone(), (cid, Compact(1)));

					if let Some(expiring_at) = expiring_at {
						<Expirations<T>>::append(expiring_at, (*fragment_hash, cid, Compact(1)));
					}
					Self::deposit_event(Event::InventoryAdded {
						account_id: to.clone(),
						fragment_hash: *fragment_hash,
						fragment_id: (id, 1),
					});
				}

				if let (Some(data_hash), Some(data_len)) = (data_hash, data_len) {
					<UniqueData2Edition<T>>::insert(fragment_hash, data_hash, existing); // if `data` exists, `quantity` is ensured to be 1

					// index immutable data for IPFS discovery
					transaction_index::index(extrinsic_index, data_len as u32, data_hash);
				}

				<EditionsCount<T>>::insert(fragment_hash, Compact(existing + quantity));
			}
		});

		Ok(())
	}

}

impl<T : Config> Pallet<T>
	where
		T::AccountId: AsRef<[u8]>
{


	pub fn get_definitions(params: GetDefinitionsParams<T::AccountId, Vec<u8>>) -> Result<Vec<u8>, Vec<u8>> {

		let get_protos_params = GetProtosParams {
			desc: params.desc,
			from: params.from,
			limit: params.limit,
			metadata_keys: Vec::new(),
			owner: params.owner,
			return_owners: params.return_owners,
			categories: params.categories,
			tags: params.tags,
			available: None,
		};

		let map_protos: Map<String, Value> = pallet_protos::Pallet::<T>::get_protos_map(get_protos_params)?;
		let map_protos_that_have_defs = map_protos
			.into_iter()
			.filter(|(proto_id, map_proto)| {
				let array_proto_id: Hash256 = hex::decode(&proto_id).unwrap().try_into().unwrap(); // using `unwrao()` can lead to panicking
				<Proto2Fragments<T>>::contains_key(&array_proto_id)
			})
			// .filter_map(|(proto_id, map_proto)| -> Option<_> {
			// 	if let Ok(array_proto_id) = hex::decode(&proto_id) {
			// 		if Ok(array_proto_id) = array_proto_id.try_into() {
			// 			if <Proto2Fragments<T>>::contains_key(&array_proto_id) {
			// 				Some((proto_id, map_proto))
			// 			} else {
			// 				None
			// 			}
			// 		} else {
			// 			Some(Err("Failed to convert `proto_id` to Hash256".into()))
			// 		}
			// 	} else {
			// 		Some(Err("`Failed to decode `proto_id``".into()))
			// 	}
			// })
			.skip(params.from as usize)
			.take(params.limit as usize)
			.collect::<Map<_, _>>();

		let mut map_definitions = Map::new();

		for (proto_id, value_map_proto) in map_protos_that_have_defs.into_iter() {

			let mut map_proto = match map_proto {
				Value::Object(map_proto) => map_proto,
				_ => return Err("Failed to get map_proto".into()),
			};

			let array_proto_id = hex::decode(&proto_id).or(Err("`Failed to decode `proto_id``"))?;
			let array_proto_id: Hash256 = array_proto_id.try_into().or(Err("Failed to convert `proto_id` to Hash256"))?;

			map_proto.insert(String::from("proto"), Value::String(proto_id));

			let list_definitions = <Proto2Fragments<T>>::get(&array_proto_id).ok_or("`proto_id` not found in `Proto2Fragments`")?;

			for definition in list_definitions.iter() {
				map_definitions.insert(hex::encode(definition), Value::Object(map_proto));
			}
		}

		if !params.metadata_keys.is_empty() {
			for (definition_id, map_definition) in map_definitions.iter_mut() {

				let map_definition = match map_definition {
					Value::Object(map_definition) => map_definition,
					_ => return Err("Failed to get map_definition".into()),
				};

				let array_definition_id: Hash128 = if let Ok(array_definition_id) = hex::decode(definition_id) {
					if let Ok(array_definition_id) = array_definition_id.try_into() {
						array_definition_id
					} else {
						return Err("Failed to convert definition to Hash128".into());
					}
				} else {
					return Err("Failed to decode definition_id".into());
				};
				let definition_metadata = if let Some(definition) = <Definitions<T>>::get(array_definition_id) {
					definition.custom_metadata
				} else {
					return Err("Failed to get definition".into());
				};
				let mut map_of_matching_metadata_keys = pallet_protos::Pallet::<T>::get_map_of_matching_metadata_keys(&params.metadata_keys, &definition_metadata);
				(*map_definition).append(&mut map_of_matching_metadata_keys);
			}
		}

		let result = json!(map_definitions).to_string();

		Ok(result.into_bytes())







	}


	pub fn get_instances(
		params: GetInstancessParams
	) -> Result<Vec<u8>, Vec<u8>> {

		let map = Map::new();

		let definition_hash = params.definition_hash;

		let editions: u64 = <EditionsCount<T>>::get(&definition_hash).unwrap_or(Compact(0)).into();

		for edition_id in 0..editions {
			let copies: u64 = <CopiesCount<T>>::get((definition_hash, edition_id)).ok_or("No Copies Found!")?.into();

			for copy_id in 0..copies {
				// Add to the `map`
			}
		}

		let result = json!(map).to_string();

		Ok(result.into_bytes())

	}
}
