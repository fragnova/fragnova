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

use protos::permissions::FragmentPerms;

#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq)]
pub struct FragmentMetadata {
	pub name: Vec<u8>,
	pub external_url: Vec<u8>, // can be 0 len/empty
}

/// Struct of a Fragment
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq)]
pub struct FragmentClass {
	/// The Proto-Fragment that was used to create this Fragment
	pub proto_hash: Hash256,
	pub metadata: FragmentMetadata,
	pub permissions: FragmentPerms,
	pub unique: bool,
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
	/// Storage Map that keeps track of the number of Fragments that were created using a Proto-Fragment.
	/// The key is the hash of the Proto-Fragment, and the value is the list of hash of the Fragments
	#[pallet::storage]
	pub type Proto2Fragments<T: Config> = StorageMap<_, Identity, Hash256, Vec<Hash256>>;

	// fragment-hash to fragment-data
	/// Storage Map of Fragments where the key is the hash of the concatenation of its corresponding Proto-Fragment and the name of the Fragment, and the value is the Fragment struct of the Fragment
	#[pallet::storage]
	pub type Fragments<T: Config> = StorageMap<_, Identity, Hash256, FragmentClass>;

	// fragment-hash to fragment-id to fragment-instance-data
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
		/// Create a Fragment using an existing Proto-Fragment (only the owner of the Proto-Fragment can call this function and create a new Fragment Class based on the Proto)
		///
		/// # Arguments
		/// * `origin` - The origin of the extrinsic/dispatchable function.
		/// * `proto_hash` - The hash of the existing Proto-Fragment
		/// * `metadata` - The metadata (name, external url etc.) of the Fragment that is going to be created
		/// * `permissions` - The permissions that the next owner of the Fragment will have
		/// * `unique` - If the Fragments generated should be unique (only one Fragment can exist with the same exact data)
		/// * `max_supply` (optional) - if scarce, the maximum amount of items that can be ever created of this type
		#[pallet::weight(<T as Config>::WeightInfo::create())]
		pub fn create(
			origin: OriginFor<T>,
			proto_hash: Hash256,
			metadata: FragmentMetadata,
			permissions: FragmentPerms,
			unique: bool,
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

			let fragment_data = FragmentClass { proto_hash, metadata, permissions, unique, max_supply };
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
