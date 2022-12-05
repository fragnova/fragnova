//! This pallet `protos` performs logic related to Proto-Fragments.
//!
//! IMPORTANT STUFF TO KNOW:
//!/**/
//! A Proto-Fragment is a digital asset that can be used to build a game or application

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod dummy_data;

#[cfg(test)]
mod mock;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod tests;

#[allow(missing_docs)]
mod weights;

use protos::{categories::Categories, traits::Trait};

use sp_core::{crypto::UncheckedFrom, ecdsa, H160, U256};

use codec::{Compact, Decode, Encode};
pub use pallet::*;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use sp_io::{
	hashing::{blake2_256, twox_64},
	transaction_index,
};
use sp_std::{
	collections::{btree_map::BTreeMap, vec_deque::VecDeque},
	vec,
	vec::Vec,
};

pub use weights::WeightInfo;

use sp_clamor::{Hash256, Hash64};

use scale_info::prelude::{
	format,
	string::{String, ToString},
};
use serde_json::{json, Map, Value};

use base58::ToBase58;

use frame_support::traits::tokens::fungibles::{Inspect, Mutate};

/// TODO: Documentation
#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub enum LinkSource {
	// Generally we just store this data, we don't verify it as we assume auth service did it.
	// (Link signature, Linked block number, EIP155 Chain ID)
	/// TODO: Documentation
	Evm(ecdsa::Signature, u64, U256),
}

/// **Types** of **Assets that are linked to a Proto-Fragment** (e.g an ERC-721 Contract etc.)
#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub enum LinkedAsset {
	/// Ethereum (ERC721 Contract address, Token ID, Link source)
	Erc721(H160, U256, LinkSource),
}

/// **Types** of **Proto-Fragment Owners**
#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub enum ProtoOwner<TAccountId> {
	/// A **regular account** on **this chain**
	User(TAccountId),
	/// An **external asset** not on this chain
	ExternalAsset(LinkedAsset),
}

/// **Data Type** used to **Query and Filter for Proto-Fragments**
#[derive(Encode, Decode, Clone, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct GetProtosParams<TAccountId, TString> {
	/// Whether to order the results in descending or ascending order
	pub desc: bool,
	/// Number of Proto-Fragment Results to skip
	pub from: u64,
	/// Number of Proto-Fragments to retrieve
	pub limit: u64,
	/// List of Metadata Keys of the Proto-Fragment that should also be returned
	pub metadata_keys: Vec<TString>,
	/// Owner of the Proto-Fragment
	pub owner: Option<TAccountId>,
	/// Whether to return the owner(s) of all the returned Proto-Fragments
	pub return_owners: bool,
	/// List of categories to filter by
	pub categories: Vec<Categories>,
	/// List of tags to filter by
	pub tags: Vec<TString>,
	/// The returned Proto-Fragments must not have any tag that is specified in the `exclude_tags` field
	pub exclude_tags: Vec<TString>,
	/// Whether the Proto-Fragments should be available or not
	pub available: Option<bool>,
}
#[cfg(test)]
impl<TAccountId, TString> Default for GetProtosParams<TAccountId, TString> {
	fn default() -> Self {
		Self {
			desc: Default::default(),
			from: Default::default(),
			limit: Default::default(),
			metadata_keys: Default::default(),
			owner: None,
			return_owners: Default::default(),
			categories: Default::default(),
			tags: Default::default(),
			exclude_tags: Default::default(),
			available: Default::default(),
		}
	}
}

/// **Data Type** used to **Query the Genealogy of a Proto-Fragment**
#[derive(Encode, Decode, Clone, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct GetGenealogyParams<TString> {
	/// The Proto-Fragment whose Genealogy will be retrieved
	pub proto_hash: TString,
	/// Whether to retrieve the ancestors of the Proto-Fragment. If `false`, the descendants are retrieved instead
	pub get_ancestors: bool,
}

/// **Struct** of a **Proto-Fragment Patch**
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq, Eq)]
pub struct ProtoPatch<TBlockNumber> {
	/// **Block Number** in which the **patch was created**
	pub block: TBlockNumber,
	/// **Hash** of patch data
	pub data_hash: Hash256,
	/// **List of New Proto-Fragments** that was **used** to **create** the **patch** (INCDT)
	pub references: Vec<Hash256>,
}

/// Struct that represents the account information of a Proto-Fragment
#[derive(Default, Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq, Eq)]
pub struct AccountsInfo {
	/// TODO: Documentation
	pub active_accounts: u128,
	/// TODO: Documentation
	pub lifetime_accounts: u128,
}

/// **Enum** that indicates **how a Proto-Fragment can be used**
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq, Eq)]
pub enum UsageLicense<TContractAddress> {
	/// Proto-Fragment is not available for use (owners can always use it)
	Closed,
	/// Proto-Fragment is available for use freely
	Open,
	/// Proto-Fragment is available for use if an amount of tickets is under curation
	Tickets(Compact<u64>),
	/// Proto-Fragment is available for use if a custom contract returns true
	Contract(TContractAddress),
}

/// **Struct** of a **Proto-Fragment**
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq, Eq)]
pub struct Proto<TAccountId, TBlockNumber> {
	/// **Block Number** in which the **Proto-Fragment was minted in**
	pub block: TBlockNumber,
	/// **List of *ProtoPatch* structs** of the **Proto-Fragment**
	pub patches: Vec<ProtoPatch<TBlockNumber>>,
	/// **License** details of the **Proto-Fragment**
	pub license: UsageLicense<TAccountId>,
	/// **Original Creator** of the **Proto-Fragment**
	pub creator: TAccountId,
	/// *Current Owner** of the **Proto-Fragment**
	pub owner: ProtoOwner<TAccountId>,
	/// **List of other Proto-Fragments** used to create the **Proto-Fragment**
	pub references: Vec<Hash256>,
	/// **Category** of the **Proto-Fragment**
	pub category: Categories,
	/// **List of Tags** associated with the **Proto-Fragment**
	pub tags: Vec<Compact<u64>>,
	/// **Map** that maps the **Key of a Proto-Fragment's Metadata Object** to the **Hash of the
	/// aforementioned Metadata Object**
	pub metadata: BTreeMap<Compact<u64>, Hash256>,
	/// Accounts information for this proto.
	pub accounts_info: AccountsInfo,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, Twox64Concat};
	use frame_system::pallet_prelude::*;
	use sp_clamor::CID_PREFIX;
	use sp_runtime::SaturatedConversion;
	use pallet_detach::{DetachRequest, DetachRequests, DetachHash, DetachedHashes, SupportedChains};

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ pallet_detach::Config
		+ pallet_accounts::Config
		+ pallet_assets::Config
		+ pallet_contracts::Config
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Weight functions needed for pallet_protos.
		type WeightInfo: WeightInfo;

		/// Weight for adding a a byte worth of storage in certain extrinsics such as `upload()`.
		#[pallet::constant]
		type StorageBytesMultiplier: Get<u64>;

		/// The number of blocks after which a curation period is over
		#[pallet::constant]
		type CurationExpiration: Get<u64>;

		/// Asset ID of the fungible asset "TICKET"
		#[pallet::constant]
		type TicketsAssetId: Get<<Self as pallet_assets::Config>::AssetId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// **StorageMap** that maps a **Tag (of type `Vec<u8>`)** to an **index number**
	#[pallet::storage]
	pub type Tags<T: Config> = StorageMap<_, Twox64Concat, Vec<u8>, u64>;

	/// **StorageValue** that **equals** the **total number of unique tags in the blockchain**
	#[pallet::storage]
	pub type TagsIndex<T: Config> = StorageValue<_, u64, ValueQuery>;

	/// **StorageMap** that maps a **Metadata Key (of type `Vec<u8>`)** to an **index number**
	#[pallet::storage]
	pub type MetaKeys<T: Config> = StorageMap<_, Twox64Concat, Vec<u8>, u64>;

	/// **StorageValue** that **equals** the **total number of unique Metadata Keys in the
	/// blockchain**
	#[pallet::storage]
	pub type MetaKeysIndex<T: Config> = StorageValue<_, u64, ValueQuery>;

	/// **StorageMap** that maps a **Trait ID** to the **name of the Trait**
	#[pallet::storage]
	pub type Traits<T: Config> = StorageMap<_, Identity, Hash64, Vec<u8>, ValueQuery>;

	/// **StorageMap** that maps a **Shards-Proto** (i.e a Proto-Fragment that is a Shards Script) to
	/// **all the traits** that are
	/// **in the "implementing" field (`ShardsScriptInfo::implementing`) of the Shards-Proto's category** or **in the "implementing" field of one Shard-Proto's Shard-Proto ancestor's category**
	#[pallet::storage]
	pub type TraitImplsByShard<T: Config> = StorageMap<_, Identity, Hash256, Vec<Hash64>>;

	/// **StorageMap** that maps a **Proto-Fragment's data's hash** to a ***Proto* struct (of the
	/// aforementioned Proto-Fragment)**
	#[pallet::storage]
	pub type Protos<T: Config> =
		StorageMap<_, Identity, Hash256, Proto<T::AccountId, T::BlockNumber>>;

	/// **StorageMap** that maps a **Proto-Fragment** to a **list of other Proto-Fragments that reference the Proto-Fragment**
	#[pallet::storage]
	pub type ProtosByParent<T: Config> = StorageMap<_, Identity, Hash256, Vec<Hash256>>;

	/// **StorageMap** that maps a **variant of the *Category* enum** to a **list of Proto-Fragment
	/// hashes (that have the aforementioned variant)**
	// Not ideal but to have it iterable...
	#[pallet::storage]
	pub type ProtosByCategory<T: Config> = StorageMap<_, Twox64Concat, Categories, Vec<Hash256>>;

	/// **StorageMap** that maps a **variant of the *ProtoOwner* enum** to a **list of
	/// Proto-Fragment hashes (that have the aforementioned variant)**
	#[pallet::storage]
	pub type ProtosByOwner<T: Config> =
		StorageMap<_, Twox64Concat, ProtoOwner<T::AccountId>, Vec<Hash256>>;

	/// **StorageDoubleMap** that maps a **Proto-Fragment and a Clamor Account ID** to a **tuple
	/// that contains the Curated Amount (tickets burned by the aforementioned Clamor Account ID)
	/// and the Block Number**
	// Curation management
	// (Amount burned, Last burn time)
	#[pallet::storage]
	pub type ProtoCurations<T: Config> = StorageDoubleMap<
		_,
		Identity,
		Hash256,
		Twox64Concat,
		T::AccountId,
		(<T as pallet_assets::Config>::Balance, T::BlockNumber),
	>;

	/// **StorageMap** that maps a **Clamor Account ID** to a **list of Proto-Fragments that was
	/// staked on by the aforementioned Clamor Account ID**
	#[pallet::storage]
	pub type AccountCurations<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Vec<Hash256>>;

	/// **StorageMap** that maps a **Block number** to a list of accounts that have curations
	/// expiring on that block number
	#[pallet::storage]
	pub type ExpiringCurations<T: Config> =
		StorageMap<_, Twox64Concat, T::BlockNumber, Vec<T::AccountId>>;

	#[allow(missing_docs)]
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A Proto-Fragment was uploaded
		Uploaded { proto_hash: Hash256, cid: Vec<u8> },
		/// A Proto-Fragment was patched
		Patched { proto_hash: Hash256, cid: Vec<u8> },
		/// A Proto-Fragment metadata has changed
		MetadataChanged { proto_hash: Hash256, cid: Vec<u8> },
		/// A Proto-Fragment was detached
		Detached { proto_hash: Hash256, cid: Vec<u8> },
		/// A Proto-Fragment was transferred
		Transferred { proto_hash: Hash256, owner_id: T::AccountId },
		/// Stake was created
		Staked {
			proto_hash: Hash256,
			account_id: T::AccountId,
			balance: <T as pallet_assets::Config>::Balance,
		},
		/// Stake was unlocked
		Unstaked {
			proto_hash: Hash256,
			account_id: T::AccountId,
			balance: <T as pallet_assets::Config>::Balance,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Systematic failure - those errors should not happen.
		SystematicFailure,
		/// Proto not found
		ProtoNotFound,
		/// Proto already uploaded
		ProtoExists,
		/// Detach Request Already Submitted
		DetachRequestAlreadyExists,
		/// Already detached
		Detached,
		/// Not the owner of the proto
		Unauthorized,
		/// Not enough tickets burned on the proto
		NotEnoughTickets,
		/// Curation not found
		CurationNotFound,
		/// Reference not found
		ReferenceNotFound,
		/// Not enough tokens to stake
		InsufficientBalance,
		/// Proto-Fragment's References includes itself!
		CircularReference,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
	{
		/// **Upload** a **Proto-Fragment** onto the **Blockchain**.
		/// Furthermore, this function also indexes `data` in the Blockchain's Database and makes it
		/// available via bitswap (IPFS) directly from every chain node permanently.
		///
		/// # Arguments
		///
		/// * `origin` - The origin of the extrinsic function
		/// * `references` - **List of other Proto-Fragments** used to create the **Proto-Fragment**
		/// * `categories` - **Category type** of the **Proto-Fragment**
		/// * `tags` - **List of tags** to **tag** the **Proto-Fragment** **with**
		/// * `linked_asset` (*optional*) - An **asset that is linked with the Proto-Fragment** (e.g
		///   an ERC-721 Contract)
		/// * `license` - **Enum** indicating **how the Proto-Fragment can be used**. NOTE: If None, the
		///   **Proto-Fragment** *<u>can't be included</u>* into **other Proto-Fragments**
		/// * `data` - **Data** of the **Proto-Fragment**
		#[pallet::weight(<T as pallet::Config>::WeightInfo::upload(references.len() as u32, tags.len() as u32, data.len() as u32))]
		pub fn upload(
			origin: OriginFor<T>,
			// we store this in the state as well
			references: Vec<Hash256>,
			category: Categories,
			tags: Vec<Vec<u8>>,
			linked_asset: Option<LinkedAsset>,
			license: UsageLicense<T::AccountId>,
			// let data come last as we record this size in blocks db (storage chain)
			// and the offset is calculated like
			// https://github.com/paritytech/substrate/blob/a57bc4445a4e0bfd5c79c111add9d0db1a265507/client/db/src/lib.rs#L1678
			data: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let current_block_number = <frame_system::Pallet<T>>::block_number();

			// hash the immutable data, this is also the unique proto id
			// to compose the V1 Cid add this prefix to the hash: (str "z" (base58
			// "0x0155a0e40220"))
			let proto_hash = blake2_256(&data);
			let data_len = data.len();

			// make sure the proto does not exist already!
			ensure!(!<Protos<T>>::contains_key(&proto_hash), Error::<T>::ProtoExists);

			// proto cannot refer itself!
			ensure!(!references.contains(&proto_hash), Error::<T>::CircularReference);

			// we need this to index transactions
			let extrinsic_index = <frame_system::Pallet<T>>::extrinsic_index()
				.ok_or(Error::<T>::SystematicFailure)?;

			// Check license requirements
			Self::check_license(&references, &who)?;

			// Store Trait if trait, also hash properly the data and decode name
			let category = match category {
				Categories::Trait(_) => {
					let trait_id = twox_64(&data);
					ensure!(!<Traits<T>>::contains_key(&trait_id), Error::<T>::ProtoExists);

					let info =
						Trait::decode(&mut &data[..]).map_err(|_| Error::<T>::SystematicFailure)?;

					ensure!(info.name.len() > 0, Error::<T>::SystematicFailure);

					// Write STATE from now, ensure no errors from now...
					<Traits<T>>::insert(trait_id, info.name.encode());

					Categories::Trait(Some(trait_id))
				},
				Categories::Shards(shards_script_info_struct) => {
					// let format = &shards_script_info_struct.format;

					let trait_implementations = [&shards_script_info_struct.requiring[..], &shards_script_info_struct.implementing[..]].concat();
					TraitImplsByShard::<T>::insert(proto_hash, trait_implementations);

					Categories::Shards(shards_script_info_struct)
				},
				_ => category,
			};

			let owner = if let Some(link) = linked_asset {
				ProtoOwner::ExternalAsset(link)
			} else {
				ProtoOwner::User(who.clone())
			};

			let tags = tags
				.iter()
				.map(|s| {
					let tag_index = <Tags<T>>::get(s);
					if let Some(tag_index) = tag_index {
						<Compact<u64>>::from(tag_index)
					} else {
						let next_index = <TagsIndex<T>>::try_get().unwrap_or_default() + 1;
						<Tags<T>>::insert(s, next_index);
						// storing is dangerous inside a closure
						// but after this call we start storing..
						// so it's fine here
						<TagsIndex<T>>::put(next_index);
						<Compact<u64>>::from(next_index)
					}
				})
				.collect();

			// store in the state the proto
			let proto = Proto {
				block: current_block_number,
				patches: vec![],
				license,
				creator: who.clone(),
				owner: owner.clone(),
				references: references.clone(),
				category: category.clone(),
				tags,
				metadata: BTreeMap::new(),
				accounts_info: AccountsInfo::default(),
			};

			// store proto
			<Protos<T>>::insert(proto_hash, proto);

			// store by parent
			for reference in references.into_iter() {
				<ProtosByParent<T>>::append(reference, proto_hash);
			}

			// store by category
			<ProtosByCategory<T>>::append(category, proto_hash);

			// store by owner
			<ProtosByOwner<T>>::append(owner, proto_hash);

			// index immutable data for IPFS discovery
			transaction_index::index(extrinsic_index, data_len as u32, proto_hash);

			let cid = [&CID_PREFIX[..], &proto_hash[..]].concat();
			let cid = cid.to_base58();
			let cid = [&b"z"[..], cid.as_bytes()].concat();

			// also emit event
			Self::deposit_event(Event::Uploaded { proto_hash, cid });

			log::debug!("Uploaded proto: {:?}", proto_hash);

			Ok(())
		}

		/// **Patch** an **existing Proto-Fragment** (*by appending the hash of `data` to the Vector
		/// field `patches` of the existing Proto-Fragment's Struct Instance*) Furthermore, this
		/// function also indexes `data` in the Blockchain's Database and stores it in the IPFS
		///
		/// # Arguments
		///
		/// * `origin` - The origin of the extrinsic function
		/// * `proto_hash` - Existing Proto-Fragment's hash
		/// * `license` (optional) - If **this value** is **not None**, the **existing Proto-Fragment's current license** is overwritten to **this value**
		/// * `new_references` - **List of New Proto-Fragments** that was **used** to **create** the
		///   **patch**
		/// * `tags` (optional) - **List of tags** to **overwrite** the **Proto-Fragment's current list of tags** with, if not None.
		/// * `data` - **Data** of the **Proto-Fragment**
		#[pallet::weight(<T as pallet::Config>::WeightInfo::patch(new_references.len() as u32,
		tags.as_ref().map(|tags| tags.len() as u32).unwrap_or_default(), data.len() as u32))]
		pub fn patch(
			origin: OriginFor<T>,
			// proto hash we want to patch
			proto_hash: Hash256,
			license: Option<UsageLicense<T::AccountId>>,
			new_references: Vec<Hash256>,
			tags: Option<Vec<Vec<u8>>>,
			// data we want to patch last because of the way we store blocks (storage chain)
			data: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let proto: Proto<T::AccountId, T::BlockNumber> =
				<Protos<T>>::get(&proto_hash).ok_or(Error::<T>::ProtoNotFound)?;

			ensure!(!new_references.contains(&proto_hash), Error::<T>::CircularReference);

			match proto.owner {
				ProtoOwner::User(owner) => ensure!(owner == who, Error::<T>::Unauthorized),
				ProtoOwner::ExternalAsset(_ext_asset) =>
				// We don't allow updating external assets
				{
					ensure!(false, Error::<T>::Unauthorized)
				},
			};

			ensure!(!<DetachedHashes<T>>::contains_key(&DetachHash::Proto(proto_hash)), Error::<T>::Detached);

			let data_hash = blake2_256(&data);

			let current_block_number = <frame_system::Pallet<T>>::block_number();

			// we need this to index transactions
			let extrinsic_index = <frame_system::Pallet<T>>::extrinsic_index()
				.ok_or(Error::<T>::SystematicFailure)?;

			// Write STATE from now, ensure no errors from now...

			// Check license requirements
			Self::check_license(&new_references, &who)?;

			<Protos<T>>::mutate(&proto_hash, |proto| {
				let proto = proto.as_mut().unwrap();

				// Add a data patch if not empty
				if !data.is_empty() {
					// No failures from here on out
					proto.patches.push(ProtoPatch {
						block: current_block_number,
						data_hash,
						references: new_references.clone(),
					});
					// index mutable data for IPFS discovery as well
					transaction_index::index(extrinsic_index, data.len() as u32, data_hash);
				}

				// Overwrite license if not None
				if let Some(license) = license {
					proto.license = license;
				}

				// Replace previous tags if not None
				if let Some(tags) = tags {
					let tags = tags
						.iter()
						.map(|s| {
							let tag_index = <Tags<T>>::get(s);
							if let Some(tag_index) = tag_index {
								<Compact<u64>>::from(tag_index)
							} else {
								let next_index = <TagsIndex<T>>::try_get().unwrap_or_default() + 1;
								<Tags<T>>::insert(s, next_index);
								// storing is dangerous inside a closure
								// but after this call we start storing..
								// so it's fine here
								<TagsIndex<T>>::put(next_index);
								<Compact<u64>>::from(next_index)
							}
						})
						.collect();

					proto.tags = tags;
				}
			});

			for new_reference in new_references.into_iter() {
				<ProtosByParent<T>>::append(new_reference, proto_hash);
			}

			let cid = [&CID_PREFIX[..], &data_hash[..]].concat();
			let cid = cid.to_base58();
			let cid = [&b"z"[..], cid.as_bytes()].concat();

			// also emit event
			Self::deposit_event(Event::Patched { proto_hash, cid });

			log::debug!("Updated proto: {:?}", proto_hash);

			Ok(())
		}

		/// **Transfer** the **ownership** of a **Proto-Fragment** from **`origin`** to
		/// **`new_owner`**
		///
		/// # Arguments
		///
		/// * `origin` - The origin of the extrinsic function
		/// * `proto_hash` - The **hash of the data of the Proto-Fragment** to **transfer**
		/// * `new_owner` - The **Account ID** to **transfer the Proto-Fragment to**
		#[pallet::weight(<T as pallet::Config>::WeightInfo::transfer())]
		pub fn transfer(
			origin: OriginFor<T>,
			proto_hash: Hash256,
			new_owner: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// make sure the proto exists
			let proto: Proto<T::AccountId, T::BlockNumber> =
				<Protos<T>>::get(&proto_hash).ok_or(Error::<T>::ProtoNotFound)?;

			// make sure the caller is the owner
			match proto.owner.clone() {
				ProtoOwner::User(owner) => ensure!(owner == who, Error::<T>::Unauthorized),
				ProtoOwner::ExternalAsset(_ext_asset) =>
				// We don't allow updating external assets
				{
					ensure!(false, Error::<T>::Unauthorized)
				},
			};

			// make sure the proto is not detached
			ensure!(!<DetachedHashes<T>>::contains_key(&DetachHash::Proto(proto_hash)), Error::<T>::Detached);

			// collect new owner
			let new_owner_s = ProtoOwner::User(new_owner.clone());

			// WRITING STATE FROM NOW

			// remove proto from old owner
			<ProtosByOwner<T>>::mutate(proto.owner, |protos_by_owner| {
				if let Some(list) = protos_by_owner {
					list.retain(|current_hash| proto_hash != *current_hash);
				}
			});

			// add proto to new owner
			<ProtosByOwner<T>>::append(new_owner_s.clone(), proto_hash);

			// update proto
			<Protos<T>>::mutate(&proto_hash, |proto| {
				let proto = proto.as_mut().unwrap();
				proto.owner = new_owner_s;
			});

			// emit event
			Self::deposit_event(Event::Transferred { proto_hash, owner_id: new_owner });

			Ok(())
		}

		/// **Alters** the **metadata** of a **Proto-Fragment** (whose hash is `proto_hash`) by
		/// **adding or modifying a key-value pair**
		/// (`metadata_key.clone`,`blake2_256(&data.encode())`) to the **BTreeMap field `metadata`**
		/// of the **existing Proto-Fragment's Struct Instance**. Furthermore, this function also
		/// indexes `data` in the Blockchain's Database and stores it in the IPFS To successfully
		/// patch a Proto-Fragment, the `auth` provided must be valid. Otherwise, an error is
		/// returned (问Gio - what happened to auth?)
		///
		/// # Arguments
		///
		/// * `origin` - The origin of the extrinsic / dispatchable function
		/// * `proto_hash` - The **hash of the Proto-Fragment**
		/// * `metadata_key` - The key (of the key-value pair) that is added in the BTreeMap field
		///   `metadata` of the existing Proto-Fragment's Struct Instance
		/// * `data` - The hash of `data` is used as the value (of the key-value pair) that is added
		///   in the BTreeMap field `metadata` of the existing Proto-Fragment's Struct Instance
		#[pallet::weight(<T as pallet::Config>::WeightInfo::set_metadata(metadata_key.len() as u32, data.len() as u32))]
		pub fn set_metadata(
			origin: OriginFor<T>,
			// proto hash we want to update
			proto_hash: Hash256,
			// Think of "Vec<u8>" as String (something to do with WASM - that's why we use Vec<u8>)
			metadata_key: Vec<u8>,
			// data we want to update last because of the way we store blocks (storage chain)
			data: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let proto: Proto<T::AccountId, T::BlockNumber> =
				<Protos<T>>::get(&proto_hash).ok_or(Error::<T>::ProtoNotFound)?;

			match proto.owner {
				ProtoOwner::User(owner) => ensure!(owner == who, Error::<T>::Unauthorized),
				ProtoOwner::ExternalAsset(_ext_asset) =>
				// We don't allow updating external assets
				{
					ensure!(false, Error::<T>::Unauthorized)
				},
			};

			ensure!(!<DetachedHashes<T>>::contains_key(&DetachHash::Proto(proto_hash)), Error::<T>::Detached);

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

			<Protos<T>>::mutate(&proto_hash, |proto| {
				let proto = proto.as_mut().unwrap();
				// update metadata
				proto.metadata.insert(metadata_key_index, data_hash);
			});

			// index data
			transaction_index::index(extrinsic_index, data.len() as u32, data_hash);

			// also emit event
			Self::deposit_event(Event::MetadataChanged { proto_hash, cid: metadata_key.clone() });

			log::debug!("Added metadata to proto: {:x?} with key: {:x?}", proto_hash, metadata_key);

			Ok(())
		}

		/// Request to detach a **Proto-Fragment** from **Clamor**.
		///
		/// Note: The Proto-Fragment may actually get detached after one or more Clamor blocks since when this extrinsic is called.
		///
		/// Note: **Once the Proto-Fragment is detached**, an **event is emitted that includes a signature**.
		/// This signature can then be used to attach the Proto-Fragment to an External Blockchain `target_chain`.
		///
		/// # Arguments
		///
		/// * `origin` - The origin of the extrinsic function
		/// * `proto_hash` - **ID of the Proto-Fragment** to **detach**
		/// * `target_chain` - **External Blockchain** to attach the Proto-Fragment into
		/// * `target_account` - **Public Account Address in the External Blockchain `target_chain`**
		///   to assign ownership of the Proto-Fragment to
		#[pallet::weight(<T as pallet::Config>::WeightInfo::detach())]
		pub fn detach(
			origin: OriginFor<T>,
			proto_hash: Hash256,
			target_chain: SupportedChains,
			target_account: Vec<u8>, // an eth address or so
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// make sure the proto exists
			let proto: Proto<T::AccountId, T::BlockNumber> =
				<Protos<T>>::get(&proto_hash).ok_or(Error::<T>::ProtoNotFound)?;

			match proto.owner {
				ProtoOwner::User(owner) => ensure!(owner == who, Error::<T>::Unauthorized),
				ProtoOwner::ExternalAsset(_ext_asset) =>
				// We don't allow detaching external assets
				{
					ensure!(false, Error::<T>::Unauthorized)
				},
			};

			let detach_hash = DetachHash::Proto(proto_hash);
			let detach_request = DetachRequest { hash: detach_hash.clone(), target_chain, target_account };

			ensure!(!<DetachedHashes<T>>::contains_key(&detach_hash), Error::<T>::Detached);
			ensure!(!<DetachRequests<T>>::get().contains(&detach_request), Error::<T>::DetachRequestAlreadyExists);

			<DetachRequests<T>>::mutate(|requests| {
				requests.push(detach_request);
			});

			Ok(())
		}

		/// Curate, burning tickets on a Proto-Fragment
		///
		/// # Arguments
		///
		/// * `origin` - The origin of the extrinsic function
		/// * `proto_hash` - **Hash of the Proto-Fragment** to **stake on**
		/// * `amount` - **Amount of tickets** to **burn**
		#[pallet::weight(50_000)]
		pub fn curate(
			origin: OriginFor<T>,
			proto_hash: Hash256,
			amount: <T as pallet_assets::Config>::Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			// make sure the proto exists
			ensure!(<Protos<T>>::contains_key(&proto_hash), Error::<T>::ProtoNotFound);

			// make sure the user has enough tickets
			let balance = <pallet_assets::Pallet<T> as Inspect<T::AccountId>>::balance(
				<T as pallet::Config>::TicketsAssetId::get(),
				&who,
			);
			ensure!(balance >= amount, Error::<T>::InsufficientBalance);

			let current_block_number = <frame_system::Pallet<T>>::block_number();

			// ! from now we write...

			// Burn tickets from account and record the stake locally
			let _ = <pallet_assets::Pallet<T> as Mutate<T::AccountId>>::burn_from(
				<T as pallet::Config>::TicketsAssetId::get(),
				&who,
				amount.saturated_into(),
			)?;

			// take record of the stake
			<ProtoCurations<T>>::insert(proto_hash, &who, (amount, current_block_number));
			<AccountCurations<T>>::append(who.clone(), proto_hash.clone());

			// also emit event
			Self::deposit_event(Event::Staked { proto_hash, account_id: who, balance: amount }); // 问Gio

			Ok(())
		}

		/// Delete Proto-Fragment `proto_hash` from all relevant Storage Items
		#[pallet::weight(50_000)]
		pub fn ban(origin: OriginFor<T>, proto_hash: Hash256) -> DispatchResult {
			ensure_root(origin)?;

			let proto_struct = <Protos<T>>::get(&proto_hash).ok_or(Error::<T>::ProtoNotFound)?;

			<ProtosByCategory<T>>::mutate(&proto_struct.category, |list_protos| {
				if let Some(list_protos) = list_protos {
					list_protos.retain(|current_hash| proto_hash != *current_hash);
				}
			});
			<ProtosByOwner<T>>::mutate(proto_struct.owner, |list_protos| {
				if let Some(list_protos) = list_protos {
					list_protos.retain(|current_hash| proto_hash != *current_hash);
				}
			});

			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// During the block finalization phase
		fn on_finalize(n: T::BlockNumber) {
			// drain expired curations
			let expiring = <ExpiringCurations<T>>::take(n);
			if let Some(expiring) = expiring {
				for account in &expiring {
					let curations = <AccountCurations<T>>::get(account);
					if let Some(curations) = curations {
						for proto in curations {
							let curation = <ProtoCurations<T>>::get(proto, account);
							if let Some(curation) = curation {
								if curation.1 + T::CurationExpiration::get().saturated_into() >= n {
									<ProtoCurations<T>>::remove(proto, account);
									<AccountCurations<T>>::mutate(account, |curations| {
										if let Some(curations) = curations {
											curations.retain(|p| p != &proto);
										}
									});
								}
							}
						}
					}
				}
			}
		}
	}

	impl<T: Config> Pallet<T>
	where
		T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
	{
		fn check_license(references: &[Hash256], who: &T::AccountId) -> DispatchResult {
			// TODO this is not tested properly
			for reference in references.iter() {
				let proto = <Protos<T>>::get(reference);
				if let Some(proto) = proto {
					let owner = match proto.owner {
						ProtoOwner::User(owner) => Some(owner),
						_ => None,
					};

					if let Some(owner) = owner {
						if owner == *who {
							// owner can include freely
							continue;
						}
					}

					let license = proto.license;
					match license {
						UsageLicense::Closed => return Err(Error::<T>::Unauthorized.into()),
						UsageLicense::Open => continue,
						UsageLicense::Tickets(amount) => {
							let amount: u64 = amount.into();
							let amount = amount.saturated_into();

							let curation = <ProtoCurations<T>>::get(reference, who.clone());
							if let Some(curation) = curation {
								// Check if the user curated enough tickets
								ensure!(curation.0 >= amount, Error::<T>::NotEnoughTickets);
							} else {
								// Curation not found
								return Err(Error::<T>::CurationNotFound.into());
							}
						},
						UsageLicense::Contract(contract_address) => {
							let data = (reference, who.clone()).encode();
							let res: Result<pallet_contracts_primitives::ExecReturnValue, _> =
								<pallet_contracts::Pallet<T>>::bare_call(
									who.clone(),
									contract_address,
									0u32.saturated_into(),
									Weight::from_ref_time(1_000_000), // TODO determine this limit better should not be too high indeed
									None,
									data,
									false,
								)
								.result
								.map_err(|e| {
									log::debug!("UsageLicense::Contract error: {:?}", e);
									e
								});

							if let Ok(res) = res {
								let allowed = bool::decode(&mut &res.data.0[..]);
								if let Ok(allowed) = allowed {
									if !allowed {
										return Err(Error::<T>::Unauthorized.into());
									}
								} else {
									return Err(Error::<T>::Unauthorized.into());
								}
							} else {
								return Err(Error::<T>::Unauthorized.into());
							}
						},
					}
				} else {
					// Proto not found
					return Err(Error::<T>::ReferenceNotFound.into());
				}
			}
			Ok(())
		}

		fn filter_proto(
			proto_id: &Hash256,
			tags: &[Vec<u8>],
			categories: &[Categories],
			avail: Option<bool>,
			exclude_tags: &[Vec<u8>],
		) -> bool {
			if let Some(struct_proto) = <Protos<T>>::get(proto_id) {
				if let Some(avail) = avail {
					if avail && struct_proto.license == UsageLicense::Closed {
						return false;
					} else if !avail && struct_proto.license != UsageLicense::Closed {
						return false;
					}
				}

				if categories.len() == 0 {
					return Self::filter_tags(tags, &struct_proto, exclude_tags);
				} else {
					return Self::filter_category(tags, &struct_proto, categories, exclude_tags);
				}
			} else {
				false
			}
		}

		/// Whether `struct_proto` has all the tags `tags` and categories `categories`
		fn filter_category(
			tags: &[Vec<u8>],
			struct_proto: &Proto<T::AccountId, T::BlockNumber>,
			categories: &[Categories],
			exclude_tags: &[Vec<u8>],
		) -> bool {
			let found: Vec<_> = categories
				.into_iter()
				.filter(|cat| match cat {
					Categories::Shards(param_script_info) => {
						if let Categories::Shards(stored_script_info) = &struct_proto.category {
							let implementing_diffs: Vec<_> = param_script_info
								.implementing
								.clone()
								.into_iter()
								.filter(|item| stored_script_info.implementing.contains(item))
								.collect();
							let requiring_diffs: Vec<_> = param_script_info
								.requiring
								.clone()
								.into_iter()
								.filter(|item| stored_script_info.requiring.contains(item))
								.collect();

							let zero_vec = [0u8; 8];

							// Specific query:
							// Partial or full match {requiring, implementing}. Same format {Edn|Binary}.
							if !implementing_diffs.is_empty() || !requiring_diffs.is_empty() {
								if param_script_info.format == stored_script_info.format {
									return Self::filter_tags(tags, struct_proto, exclude_tags);
								} else {
									return false;
								}
							}
							// Generic query:
							// Get all with same format. {Edn|Binary}. No match {requiring, implementing}.
							else if param_script_info.implementing.contains(&zero_vec)
								&& param_script_info.requiring.contains(&zero_vec)
								&& param_script_info.format == stored_script_info.format
							{
								return Self::filter_tags(tags, struct_proto, exclude_tags);
							} else {
								return false;
							}
						} else {
							// it should never go here
							return false;
						}
					},
					_ => {
						if *cat == &struct_proto.category {
							return Self::filter_tags(tags, struct_proto, exclude_tags);
						} else {
							return false;
						}
					},
				})
				.collect();

			if found.is_empty() {
				return false;
			} else {
				return true;
			}
		}

		/// Whether `struct_proto` has all the tags `tags`
		fn filter_tags(
			tags: &[Vec<u8>],
			struct_proto: &Proto<T::AccountId, T::BlockNumber>,
			exclude_tags: &[Vec<u8>],
		) -> bool {
			// empty iterator returns `false` for `Iterator::any()`
			let proto_has_any_unwanted_tag = exclude_tags.into_iter().any(|tag| {
				if let Some(tag_idx) = <Tags<T>>::get(tag) {
					struct_proto.tags.contains(&Compact::from(tag_idx))
				} else {
					false
				}
			});
			// empty iterator returns `true` for `Iterator::all()`
			let proto_has_all_wanted_tags = tags.into_iter().all(|tag| {
				if let Some(tag_idx) = <Tags<T>>::get(tag) {
					struct_proto.tags.contains(&Compact::from(tag_idx))
				} else {
					false
				}
			});

			proto_has_all_wanted_tags && !proto_has_any_unwanted_tag
		}

		fn get_list_of_matching_categories(
			params: &GetProtosParams<T::AccountId, Vec<u8>>,
			category: &Categories,
		) -> Vec<Categories> {
			let found: Vec<Categories> = params
				.categories
				.clone()
				.into_iter()
				.filter(|cat| match cat {
					Categories::Shards(param_script_info) => {
						if let Categories::Shards(stored_script_info) = &category {
							let implementing_diffs: Vec<_> = param_script_info
								.implementing
								.clone()
								.into_iter()
								.filter(|item| stored_script_info.implementing.contains(item))
								.collect();
							let requiring_diffs: Vec<_> = param_script_info
								.requiring
								.clone()
								.into_iter()
								.filter(|item| stored_script_info.requiring.contains(item))
								.collect();

							let zero_vec = [0u8; 8];

							// Specific query:
							// Partial or full match {requiring, implementing}. Same format {Edn|Binary}.
							if !implementing_diffs.is_empty() || !requiring_diffs.is_empty() {
								if param_script_info.format == stored_script_info.format {
									return true;
								} else {
									return false;
								}
							}
							// Generic query:
							// Get all with same format. {Edn|Binary}. No match {requiring, implementing}.
							else if param_script_info.implementing.contains(&zero_vec)
								&& param_script_info.requiring.contains(&zero_vec)
								&& param_script_info.format == stored_script_info.format
							{
								return true;
							} else if !(&cat == &category) {
								return false;
							} else {
								return false;
							}
						} else {
							return false;
						}
					},
					// for all other types of Categories
					_ => {
						if !(&cat == &category) {
							return false;
						} else {
							return true;
						}
					},
				})
				.collect();

			return found;
		}

		/// Converts a `ProtoOwner` struct into a JSON
		pub fn get_owner_in_json_format(owner: ProtoOwner<T::AccountId>) -> Value {
			let json_owner = match owner {
				ProtoOwner::User(account_id) => json!({
					"type": "internal",
					"value": hex::encode(account_id)
				}),
				ProtoOwner::ExternalAsset(linked_asset) => {
					let value = match linked_asset {
						LinkedAsset::Erc721(contract, token_id, source) => {
							let chain_id = match source {
								LinkSource::Evm(_sig, _block, chain_id) => chain_id,
							};
							json!({
								"type": "erc721",
								"value": {
									"contract": format!("0x{:x}", contract),
									"token_id": format!("0x{:x}", token_id),
									"chain_id": format!("0x{:x}", chain_id)
								}
							})
						},
					};
					json!({
						"type": "external",
						"value": value,
					})
				},
			};

			json_owner
		}

		/// Queries the `metadata_keys` that exist in the map `metadata` and returns them as a JSON (along with their corresponding data hashes)
		pub fn get_map_of_matching_metadata_keys(
			metadata_keys: &Vec<Vec<u8>>,
			metadata: &BTreeMap<Compact<u64>, Hash256>,
		) -> Map<String, Value> {
			let mut map = Map::new();

			for metadata_key in metadata_keys.clone().iter() {
				let metadata_value =
					if let Some(metadata_key_index) = <MetaKeys<T>>::get(metadata_key) {
						if let Some(data_hash) = metadata.get(&Compact(metadata_key_index)) {
							Value::String(hex::encode(data_hash))
						} else {
							Value::Null
						}
					} else {
						Value::Null
					};

				if let Ok(string_metadata_key) = String::from_utf8(metadata_key.clone()) {
					map.insert(string_metadata_key, metadata_value);
				}
			}

			map
		}

		/// **Query** and **Return** **Proto-Fragment(s)** based on **`params`**. The **return
		/// type** is a **JSON string**
		///
		/// # Arguments
		///
		/// * `params` - A ***GetProtosParams* struct**
		pub fn get_protos(
			params: GetProtosParams<T::AccountId, Vec<u8>>,
		) -> Result<Vec<u8>, Vec<u8>> {
			let protos_map: Map<String, Value> = Self::get_protos_map(params)?;

			let result = json!(protos_map).to_string();

			Ok(result.into_bytes())
		}

		/// **Query** and **Return** **Proto-Fragment(s)** based on **`params`**. The **return
		/// type** is a **JSON string**
		///
		/// # Arguments
		///
		/// * `params` - A ***GetProtosParams* struct**
		pub fn get_protos_map(
			params: GetProtosParams<T::AccountId, Vec<u8>>,
		) -> Result<Map<String, Value>, Vec<u8>> {
			let mut map = Map::new();

			let list_protos_final: Vec<Hash256> = if let Some(owner) = params.owner {
				// `owner` exists
				let list_protos_owner =
					<ProtosByOwner<T>>::get(ProtoOwner::<T::AccountId>::User(owner))
						.ok_or("Owner not found")?; // `owner` exists in `ProtosByOwner`
				if params.desc {
					// Sort in descending order
					list_protos_owner
						.into_iter()
						.rev()
						.filter(|proto_id| {
							Self::filter_proto(
								proto_id,
								&params.tags,
								&params.categories,
								params.available,
								&params.exclude_tags,
							)
						})
						.skip(params.from as usize)
						.take(params.limit as usize)
						.collect()
				} else {
					// Sort in ascending order
					list_protos_owner
						.into_iter()
						.filter(|proto_id| {
							Self::filter_proto(
								proto_id,
								&params.tags,
								&params.categories,
								params.available,
								&params.exclude_tags,
							)
						})
						.skip(params.from as usize)
						.take(params.limit as usize)
						.collect()
				}
			} else {
				// Notice this wastes time and memory and needs a better implementation
				let mut flat = Vec::<Hash256>::new();

				let cats = <ProtosByCategory<T>>::iter_keys().collect::<Vec<Categories>>();
				let cats: Vec<Categories> =
					if params.desc { cats.iter().rev().map(|x| x.clone()).collect() } else { cats };

				for category in cats {
					if params.categories.len() != 0 {
						let found: Vec<Categories> =
							Self::get_list_of_matching_categories(&params, &category);
						// if the current stored category does not match with any of the categories
						// in input, it can be discarded from this search.
						if found.is_empty() {
							continue;
						}
					}
					// Found the category.
					// Now collect all the protos linked to the category type in input
					let protos = <ProtosByCategory<T>>::get(category);
					if let Some(protos) = protos {
						let collection: Vec<Hash256> = if params.desc {
							// Sort in descending order
							protos
								.into_iter()
								.rev()
								.filter(|proto_id| {
									Self::filter_proto(
										proto_id,
										&params.tags,
										&params.categories,
										params.available,
										&params.exclude_tags,
									)
								})
								.collect()
						} else {
							// Sort in ascending order
							protos
								.into_iter()
								.filter(|proto_id| {
									Self::filter_proto(
										proto_id,
										&params.tags,
										&params.categories,
										params.available,
										&params.exclude_tags,
									)
								})
								.collect()
						};
						flat.extend(collection);
					}
				}
				flat.iter()
					.skip(params.from as usize)
					.take(params.limit as usize)
					.map(|x| *x)
					.collect()
			};

			for proto_id in list_protos_final.into_iter() {
				map.insert(hex::encode(proto_id), Value::Object(Map::new()));
			}

			if params.return_owners || !params.metadata_keys.is_empty() {
				for (proto_id, map_proto) in map.iter_mut() {
					let map_proto = match map_proto {
						Value::Object(map_proto) => map_proto,
						_ => return Err("Failed to get map_proto".into()),
					};

					let array_proto_id: Hash256 = hex::decode(proto_id)
						.or(Err("`Failed to decode `proto_id``"))?
						.try_into()
						.or(Err("Failed to convert `proto_id` to Hash256"))?;

					let proto_struct =
						<Protos<T>>::get(array_proto_id).ok_or("Failed to get proto")?;

					match proto_struct.license {
						UsageLicense::Tickets(amount) => {
							let n: u64 = amount.into();
							(*map_proto).insert(String::from("tickets"), Value::Number(n.into()));
						},
						_ => {
							(*map_proto).insert(String::from("tickets"), Value::Null);
						},
					}

					if params.return_owners {
						let owner = proto_struct.owner;
						let json_owner = Self::get_owner_in_json_format(owner);
						(*map_proto).insert("owner".into(), json_owner);
					}

					if !params.metadata_keys.is_empty() {
						let proto_metadata = proto_struct.metadata;
						let map_of_matching_metadata_keys = Self::get_map_of_matching_metadata_keys(
							&params.metadata_keys,
							&proto_metadata,
						);
						(*map_proto)
							.insert("metadata".into(), map_of_matching_metadata_keys.into());
						// (*map_proto).append(&mut map_of_matching_metadata_keys);
					}
				}
			}

			Ok(map)
		}

		/// **Query** the Genealogy of a Proto-Fragment based on **`params`**. The **return
		/// type** is a **JSON string** that represents an Adjacency List.
		///
		/// # Arguments
		///
		/// * `params` - A ***GetGenealogyParams* struct**
		pub fn get_genealogy(params: GetGenealogyParams<Vec<u8>>) -> Result<Vec<u8>, Vec<u8>> {
			let proto_hash: Hash256 = hex::decode(params.proto_hash)
				.map_err(|_| "Failed to convert string to u8 slice")?
				.try_into()
				.map_err(|_| "Failed to convert u8 slice to Hash256")?;

			let mut adjacency_list = BTreeMap::<String, Vec<String>>::new();

			let mut queue = VecDeque::<Hash256>::new();
			queue.push_back(proto_hash);

			let mut visited = BTreeMap::<Hash256, bool>::new();
			visited.insert(proto_hash, true);

			while let Some(proto) = queue.pop_front() {
				let neighbors = if params.get_ancestors {
					let proto_struct =
						<Protos<T>>::get(proto).ok_or("Proto Hash Does Not Exist!")?;
					let mut parents = proto_struct.references;
					let mut references_from_patches = proto_struct
						.patches
						.into_iter()
						.flat_map(|pp: ProtoPatch<_>| pp.references)
						.collect::<Vec<Hash256>>();
					parents.append(&mut references_from_patches);
					parents
				} else {
					let children = <ProtosByParent<T>>::get(proto).unwrap_or_default();
					children
				};

				adjacency_list
					.insert(hex::encode(proto), neighbors.iter().map(|p| hex::encode(p)).collect());

				for neighbor in neighbors.into_iter() {
					if !visited.contains_key(&neighbor) {
						visited.insert(neighbor, true);
						queue.push_back(neighbor);
					}
				}
			}

			Ok(json!(adjacency_list).to_string().into_bytes())
		}
	}
}
