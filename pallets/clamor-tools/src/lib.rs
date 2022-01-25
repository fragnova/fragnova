#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use codec::{Compact, Decode, Encode};
use sp_std::vec::Vec;
use sp_io::hashing::blake2_256;
use sp_chainblocks::Hash256;
use sp_std::vec;
use fragments_pallet::SupportedChains;

#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq)]
pub struct DetachRequest {
	pub fragment_hash: Hash256,
	pub target_chain: SupportedChains,
	pub target_account: Vec<u8>, // an eth address or so
}
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq)]
pub struct DetachInternalData<TPublic> {
	public: TPublic,
	fragment_hash: Hash256,
	target_chain: SupportedChains,
	target_account: Vec<u8>, // an eth address or so
	remote_signature: Vec<u8>,
	nonce: u64,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use fragments_pallet::{Fragment, Fragments, FragmentOwner, ExportData};

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + fragments_pallet::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// fragment-hash to entity-hash-sequence
	#[pallet::storage]
	pub type DetachRequests<T: Config> = StorageValue<_, Vec<DetachRequest>, ValueQuery>;

	#[pallet::storage]
	pub type DetachedFragments<T: Config> = StorageMap<_, Identity, Hash256, ExportData>;

	#[pallet::storage]
	pub type DetachNonces<T: Config> = StorageDoubleMap<_, Blake2_128Concat, Vec<u8>, Blake2_128Concat, SupportedChains, u64>;

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
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(25_000)]
		pub fn internal_finalize_detach(
			origin: OriginFor<T>,
			data: DetachInternalData<T::Public>,
			_signature: T::Signature,
		) -> DispatchResult {

			Ok(())
		}

	}
}
