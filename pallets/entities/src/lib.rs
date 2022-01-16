#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use codec::{Compact, Decode, Encode};
use sp_std::vec::Vec;

use sp_chainblocks::Hash256;

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
	pub data_hash: Option<Hash256>, // mutable block data hash
	pub metadata: Option<EntityMetadata> // an instance might have metadata
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// fragment-hash to entity-hash-sequence
	#[pallet::storage]
	pub type Fragment2Entities<T: Config> = StorageMap<_, Blake2_128Concat, Hash256, Vec<Hash256>>;

	// entity-hash to entity-data
	#[pallet::storage]
	pub type Entities<T: Config> = StorageMap<_, Blake2_128Concat, Hash256, EntityData>;

	// entity-hash to entity-id to entity-instance-data
	#[pallet::storage]
	pub type EntityInstances<T: Config> = StorageDoubleMap<_, Blake2_128Concat, Hash256, Blake2_128Concat, u128, EntityInstanceData>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create(
			origin: OriginFor<T>,
			fragment_hash: Hash256,
			metadata: EntityMetadata,
			unique: bool,
			mutable: bool,
			max_supply: Option<Compact<u128>>
		) -> DispatchResult {

			Ok(())
		}

	}
}
