#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod weights;

use codec::{Compact, Decode, Encode};
pub use pallet::*;
use sp_clamor::Hash256;
use sp_io::hashing::blake2_256;
use sp_std::{vec, vec::Vec};
pub use weights::WeightInfo;

/// **Struct** that represents a **Fragment's Metadata**
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq)]
pub struct FragmentMetadata {
	/// **Name** of the **Fragment** (*NOTE: No other Fragment created using the same Proto-Fragment is allowed to have the same name*)
	pub name: Vec<u8>,
	/// **URL** to access the **Metadata Object** (*NOTE: URL can be left empty (i.e an empty string)*)
	pub external_url: Vec<u8>, // can be 0 len/empty
}

/// **Struct** of a **Fragment**
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq)]
pub struct FragmentData {
	/// **Proto-Fragment used** to **create** the **Fragment**
	pub proto_hash: Hash256,
	/// ***FragmentMetadata* Struct** (the **struct** contains the **Fragment's name**, among other things)
	pub metadata: FragmentMetadata,
	/// Whether the **Fragment** is **unique**
	pub unique: bool,
	/// Whether the **Fragment** is **mutable**
	pub mutable: bool,
	/// INC
	pub max_supply: Option<Compact<u128>>,
}

#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq)]
pub struct FragmentInstanceData {
	pub data_hash: Option<Hash256>,         // mutable block data hash
	pub metadata: Option<FragmentMetadata>, // an instance might have metadata
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use pallet_detach::DetachedHashes;
	use pallet_protos::{Proto, ProtoOwner, Protos};

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_protos::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	// proto-hash to fragment-hash-sequence
	/// **StorageMap** that maps a **Proto-Fragment** to a **list of hashes, where each hash is: the hash of the concatenation of the aforementioned Proto-Fragment and a corresponding Fragment's name** 
	#[pallet::storage]
	pub type Proto2Fragments<T: Config> = StorageMap<_, Identity, Hash256, Vec<Hash256>>;

	// fragment-hash to fragment-data
	/// **StorageMap** that maps the **hash of the concatenation of a Proto-Fragment and a corresponding Fragment's name** to a ***Fragment* struct of the aforementioned Fragment**
	#[pallet::storage]
	pub type Fragments<T: Config> = StorageMap<_, Identity, Hash256, FragmentData>;

	// fragment-hash to fragment-id to fragment-instance-data
	/// INC
	#[pallet::storage]
	pub type FragmentInstances<T: Config> =
		StorageDoubleMap<_, Identity, Hash256, Blake2_128Concat, u128, FragmentInstanceData>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		FragmentAdded(T::AccountId),
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
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// **Create** a **Fragment** using an **existing Proto-Fragment** (NOTE: ***Only* the Proto-Fragment's owner** is **allowed to create a Fragment using the Proto-Fragment**)
		///
		/// # Arguments
		/// * `origin` - **Origin** of the **extrinsic function**
		/// * `proto_hash` - **Hash** of an **existing Proto-Fragment**
		/// * `metadata` -  **Metadata** of the **Fragment**
		/// * `unique` - **Whether** the **Fragment** is **unique**
		/// * `mutable` - **Whether** the **Fragment** is **mutable** 
		/// * `max_supply` (*optional*) - **Maximum amount of items** that **can ever be created** using the **Fragment** (INCDT)
		#[pallet::weight(<T as Config>::WeightInfo::create())]
		pub fn create(
			origin: OriginFor<T>,
			proto_hash: Hash256,
			metadata: FragmentMetadata,
			unique: bool,
			mutable: bool,
			max_supply: Option<Compact<u128>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let proto: Proto<T::AccountId, T::BlockNumber> =
				<Protos<T>>::get(proto_hash).ok_or(Error::<T>::ProtoNotFound)?;

			let proto_owner: T::AccountId = match proto.owner {
				ProtoOwner::User(owner) => Ok(owner),
				_ => Err(Error::<T>::ProtoOwnerNotFound),
			}?;

			ensure!(who == proto_owner, Error::<T>::NoPermission);

			ensure!(!<DetachedHashes<T>>::contains_key(&proto_hash), Error::<T>::Detached);

			//TODO Need to extend it in future
			let hash = blake2_256(&[&proto_hash[..], &metadata.name.encode()].concat());

			ensure!(!<Fragments<T>>::contains_key(&hash), Error::<T>::AlreadyExist);

			let fragment_data = FragmentData { proto_hash, metadata, unique, mutable, max_supply };
			<Fragments<T>>::insert(&hash, fragment_data);

			Proto2Fragments::<T>::mutate(&proto_hash, |fragment_hash| {
				if let Some(entity_hash) = fragment_hash.as_mut() {
					entity_hash.push(hash);
				} else {
					*fragment_hash = Some(vec![hash]);
				}
			});

			Self::deposit_event(Event::FragmentAdded(who));
			Ok(())
		}
	}
}
