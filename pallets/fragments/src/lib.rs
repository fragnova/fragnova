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
pub struct FragmentMetadata {
	pub name: Vec<u8>,
	pub external_url: Vec<u8>, // can be 0 len/empty
}

#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq)]
pub struct FragmentData {
	pub proto_hash: Hash256,
	pub metadata: FragmentMetadata,
	pub unique: bool,
	pub mutable: bool,
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
	#[pallet::storage]
	pub type Proto2Fragments<T: Config> = StorageMap<_, Identity, Hash256, Vec<Hash256>>;

	// fragment-hash to fragment-data
	#[pallet::storage]
	pub type Fragments<T: Config> = StorageMap<_, Identity, Hash256, FragmentData>;

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
				if let Some(enti_hash) = fragment_hash.as_mut() {
					enti_hash.push(hash);
				} else {
					*fragment_hash = Some(vec![hash]);
				}
			});

			Self::deposit_event(Event::FragmentAdded(who));
			Ok(())
		}
	}
}
