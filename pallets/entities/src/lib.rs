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
use sp_chainblocks::Hash256;
use sp_io::hashing::blake2_256;
use sp_std::{vec, vec::Vec};
pub use weights::WeightInfo;

#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq)]
pub struct EntityMetadata {
	pub name: Vec<u8>,
	pub external_url: Vec<u8>, // can be 0 len/empty
}

#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq)]
pub struct EntityData {
	pub fragment_hash: Hash256,
	pub metadata: EntityMetadata,
	pub unique: bool,
	pub mutable: bool,
	pub max_supply: Option<Compact<u128>>,
}

#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq)]
pub struct EntityInstanceData {
	pub data_hash: Option<Hash256>,       // mutable block data hash
	pub metadata: Option<EntityMetadata>, // an instance might have metadata
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use fragments_pallet::{DetachedFragments, Fragment, FragmentOwner, Fragments};
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + fragments_pallet::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// fragment-hash to entity-hash-sequence
	#[pallet::storage]
	pub type Fragment2Entities<T: Config> = StorageMap<_, Identity, Hash256, Vec<Hash256>>;

	// entity-hash to entity-data
	#[pallet::storage]
	pub type Entities<T: Config> = StorageMap<_, Identity, Hash256, EntityData>;

	// entity-hash to entity-id to entity-instance-data
	#[pallet::storage]
	pub type EntityInstances<T: Config> =
		StorageDoubleMap<_, Identity, Hash256, Blake2_128Concat, u128, EntityInstanceData>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		EntityAdded(T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Fragment not found
		FragmentNotFound,
		/// Fragment owner not found
		FragmentOwnerNotFound,
		/// No Permission
		NoPermission,
		/// Fragment is already detached
		FragmentDetached,
		/// Entity already exist
		EntityAlreadyExist,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(<T as Config>::WeightInfo::create())]
		pub fn create(
			origin: OriginFor<T>,
			fragment_hash: Hash256,
			metadata: EntityMetadata,
			unique: bool,
			mutable: bool,
			max_supply: Option<Compact<u128>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let fragment: Fragment<T::AccountId, T::BlockNumber> =
				<Fragments<T>>::get(fragment_hash).ok_or(Error::<T>::FragmentNotFound)?;

			let fragment_owner: T::AccountId = match fragment.owner {
				FragmentOwner::User(owner) => Ok(owner),
				_ => Err(Error::<T>::FragmentOwnerNotFound),
			}?;

			ensure!(who == fragment_owner, Error::<T>::NoPermission);

			ensure!(
				!<DetachedFragments<T>>::contains_key(&fragment_hash),
				Error::<T>::FragmentDetached
			);

			//TODO Need to extend it in future
			let hash = blake2_256(&[&fragment_hash[..], &metadata.name.encode()].concat());

			ensure!(!<Entities<T>>::contains_key(&hash), Error::<T>::EntityAlreadyExist);

			let entity_data = EntityData { fragment_hash, metadata, unique, mutable, max_supply };
			<Entities<T>>::insert(&hash, entity_data);

			Fragment2Entities::<T>::mutate(&fragment_hash, |entity_hash| {
				if let Some(enti_hash) = entity_hash.as_mut() {
					enti_hash.push(hash);
				} else {
					*entity_hash = Some(vec![hash]);
				}
			});

			Self::deposit_event(Event::EntityAdded(who));
			Ok(())
		}
	}
}
