#![cfg_attr(not(feature = "std"), no_std)]

extern crate core;

use codec::{Decode, Encode};
use frame_support::traits::tokens::fungible;
pub use pallet::*;
use pallet_clusters::Cluster;
use pallet_fragments::GetInstanceOwnerParams;
use sp_clamor::{Hash128, Hash256};
use sp_std::vec::Vec;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod dummy_data;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

/// Enum that indicates the different types of target of an alias
#[derive(Encode, Decode, Clone, scale_info::TypeInfo)]
pub enum LinkTarget<TAccountId, TString> {
	Proto(Hash256),
	Fragment(GetInstanceOwnerParams<TString>),
	Account(TAccountId),
	Cluster(Hash128),
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		pallet_prelude::*,
		sp_runtime::{traits::Zero, SaturatedConversion},
	};
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ pallet_balances::Config
		+ pallet_protos::Config
		+ pallet_fragments::Config
		+ pallet_clusters::Config
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The price (in native NOVA) for a Namespace
		#[pallet::constant]
		type NamespacePrice: Get<u128>;

		/// The max size of namespace string
		#[pallet::constant]
		type NameLimit: Get<u32>;
	}

	/// **StorageMap** that maps a **Cluster** ID to its data.
	#[pallet::storage]
	pub type Namespaces<T: Config> = StorageMap<_, Twox64Concat, Vec<u8>, T::AccountId>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		NamespaceCreated {
			who: T::AccountId,
			namespace: Vec<u8>,
		},
		NamespaceTransferred {
			namespace: Vec<u8>,
			from: T::AccountId,
			to: T::AccountId,
		},
	}

	// Errors inform users that something went wrong.
	#[allow(missing_docs)]
	#[pallet::error]
	pub enum Error<T> {
		InvalidInput,
		NamespaceExists,
		NamespaceNotFound,
		NotAllowed,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new **Namespace**, a unique string that identifies an owner of assets.
		///
		/// The creation of a namespace burns NOVA from the caller's balance.
		/// The amount is set in Config `NamespacePrice`.
		///
		/// - `namespace`: namespace to create
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_namespace(
			origin: OriginFor<T>,
			namespace: BoundedVec<u8, <T as pallet::Config>::NameLimit>,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			ensure!(!namespace.len().is_zero(), Error::<T>::InvalidInput);

			let namespace = namespace.into_inner();

			// Check that the namespace does not exist already
			ensure!(!<Namespaces<T>>::contains_key(&namespace), Error::<T>::NamespaceExists);

			// burn NOVA from account's balance. The amount is set in Config.
			let amount: <T as pallet_balances::Config>::Balance =
				<T as Config>::NamespacePrice::get().saturated_into();
			<pallet_balances::Pallet<T> as fungible::Mutate<T::AccountId>>::burn_from(
				&who, amount,
			)?;

			<Namespaces<T>>::insert(&namespace, &who);

			Self::deposit_event(Event::NamespaceCreated { who, namespace });

			Ok(())
		}

		/// Tranfer the **Namespace ownership** to another AccountId.
		///
		/// Only the owner of the Namespace can execute this.
		///
		/// - `namespace`: namespace to create
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn transfer_namespace(
			origin: OriginFor<T>,
			namespace: BoundedVec<u8, <T as pallet::Config>::NameLimit>,
			new_owner: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			ensure!(!namespace.len().is_zero(), Error::<T>::InvalidInput);

			let namespace = namespace.into_inner();

			// Check that the caller is the owner of the namespace
			let owner = <Namespaces<T>>::get(&namespace).ok_or(Error::<T>::NamespaceNotFound)?;
			ensure!(&who == &owner, Error::<T>::NotAllowed);

			<Namespaces<T>>::insert(&namespace, &new_owner);

			Self::deposit_event(Event::NamespaceTransferred {
				namespace,
				from: who,
				to: new_owner,
			});

			Ok(())
		}
	}
}
