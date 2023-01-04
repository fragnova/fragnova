#![cfg_attr(not(feature = "std"), no_std)]

extern crate core;

use codec::{Compact, Decode, Encode};
use frame_support::traits::tokens::fungible;
pub use pallet::*;
use pallet_clusters::Clusters;
use pallet_fragments::GetInstanceOwnerParams;
use pallet_protos::{Proto, ProtoOwner, Protos};
use sp_clamor::{Hash128, Hash256, InstanceUnit};
use sp_std::vec::Vec;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod dummy_data;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

/// Enum that indicates the types of target assets linked to an alias
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq)]
pub enum LinkTarget<TAccountId> {
	Proto(Hash256),
	Fragment { definition_hash: Vec<u8>, edition_id: InstanceUnit, copy_id: InstanceUnit }, // struct variant allows a nicer UX on polkadotJS
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

	/// **StorageMap** that maps a **Alias** name to its owner.
	#[pallet::storage]
	pub type Namespaces<T: Config> = StorageMap<_, Twox64Concat, Vec<u8>, T::AccountId>;

	/// **StorageMap** that maps a **Name (of type `Vec<u8>`)** to an **index**.
	/// This ensures no duplicated aliases are used.
	#[pallet::storage]
	pub type Names<T: Config> = StorageMap<_, Twox64Concat, Vec<u8>, Compact<u64>>;

	/// **StorageValue** that **equals** the **total number of unique names** used for aliases in the chain.
	#[pallet::storage]
	pub type NamesIndex<T: Config> = StorageValue<_, u64, ValueQuery>;

	/// **StorageDoubleMap** that maps a (**AccountId**, **Alias** name index) with a **TargetLink**.
	#[pallet::storage]
	pub type Aliases<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		Vec<u8>, // namespace
		Twox64Concat,
		Compact<u64>, // alias name index
		LinkTarget<T::AccountId>, // target asset
	>;

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
		NamespaceAlreadyExists,
		NamespaceNotFound,
		NotAllowed,
		SystematicFailure,
		AliasAlreadyOwned,
		AliasNotExists,
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
			ensure!(!<Namespaces<T>>::contains_key(&namespace), Error::<T>::NamespaceAlreadyExists);

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
		/// - `new_owner`: the AccountId to transfer ownership to
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn transfer_namespace(
			origin: OriginFor<T>,
			namespace: BoundedVec<u8, <T as pallet::Config>::NameLimit>,
			new_owner: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			ensure!(who != new_owner, Error::<T>::NotAllowed);
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

		/// Create an **Alias** that links to a target asset.
		///
		/// Only the owner of the asset can create its alias.
		/// Alias names can be reused by multiple users, but a user can have only one alias with the same name.
		///
		/// - `namespace`: namespace to create
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_alias(
			origin: OriginFor<T>,
			namespace: BoundedVec<u8, <T as pallet::Config>::NameLimit>,
			alias: BoundedVec<u8, <T as pallet::Config>::NameLimit>,
			target: LinkTarget<T::AccountId>,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			ensure!(!namespace.len().is_zero(), Error::<T>::InvalidInput);

			let namespace = namespace.into_inner();

			// Check that the caller is the owner of the namespace
			let owner = <Namespaces<T>>::get(&namespace).ok_or(Error::<T>::NamespaceNotFound)?;
			ensure!(&who == &owner, Error::<T>::NotAllowed);

			let alias_index = Self::take_name_index(&alias);
			// check that the caller does not already own this alias
			ensure!(!<Aliases<T>>::contains_key(&namespace, &alias_index), Error::<T>::AliasAlreadyOwned);

			// check that the caller is the owner of the target asset
			Self::is_target_owner(who, target);

			Ok(())
		}

		/// Replace the **LinkTarget** of an alias with another LinkTarget.
		///
		/// Only the root can execute this.
		///
		/// - `namespace`: namespace related to the alias to update
		/// - `alias`: the alias to update
		/// - `new_target`: the new LinkTarget to link the alias to
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn update_alias_target(
			origin: OriginFor<T>,
			namespace: BoundedVec<u8, <T as pallet::Config>::NameLimit>,
			alias: BoundedVec<u8, <T as pallet::Config>::NameLimit>,
			new_target: LinkTarget<T::AccountId>,
		) -> DispatchResult {
			ensure_root(origin.clone())?;

			ensure!(!namespace.len().is_zero(), Error::<T>::InvalidInput);
			ensure!(!alias.len().is_zero(), Error::<T>::InvalidInput);

			let namespace = namespace.into_inner();

			let alias_index = Self::take_name_index(&alias);
			ensure!(<Aliases<T>>::contains_key(&namespace, &alias_index), Error::<T>::AliasNotExists);
			<Aliases<T>>::insert(&namespace, &alias_index, new_target);

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Utility function that checks for the existence of a name in storage and return its index.
		///
		/// - `name`: the reference of the name to lookup
		///
		/// Returns:
		/// - `Compact<u64>`: the index of the name in storage
		pub fn take_name_index(name: &Vec<u8>) -> Compact<u64> {
			let name_index = <Names<T>>::get(name);
			if let Some(name_index) = name_index {
				<Compact<u64>>::from(name_index)
			} else {
				let next_name_index = <NamesIndex<T>>::try_get().unwrap_or_default() + 1;
				let next_name_index_compact = <Compact<u64>>::from(next_name_index);
				<Names<T>>::insert(name, next_name_index_compact);
				// storing is dangerous inside a closure
				// but after this call we start storing..
				// so it's fine here
				<NamesIndex<T>>::put(next_name_index);
				next_name_index_compact
			}
		}

		pub fn is_target_owner(who: T::AccountId, target: LinkTarget<T::AccountId>) -> DispatchResult {
			match target {
				LinkTarget::Fragment { definition_hash, edition_id, copy_id } => {
					let owner = pallet_fragments::Pallet::<T>::get_instance_owner_account_id(
						GetInstanceOwnerParams { definition_hash, edition_id, copy_id },
					)
						.map_err(|_| Error::<T>::SystematicFailure)?;
					ensure!(who == owner, Error::<T>::NotAllowed);
					Ok(())
				},
				LinkTarget::Proto(proto_hash) => {
					let proto: Proto<T::AccountId, T::BlockNumber> = <Protos<T>>::get(&proto_hash)
						.ok_or(pallet_protos::Error::<T>::ProtoNotFound)?;
					match proto.owner {
						ProtoOwner::User(owner) => ensure!(who == owner, Error::<T>::NotAllowed),
						_ => {
							ensure!(false, Error::<T>::NotAllowed)
						},
					};
					Ok(())
				},
				LinkTarget::Cluster(cluster_id) => {
					let cluster = <Clusters<T>>::get(&cluster_id)
						.ok_or(pallet_clusters::Error::<T>::ClusterNotFound)?;
					ensure!(who == cluster.owner, Error::<T>::NotAllowed);
					Ok(())
				},
				LinkTarget::Account(account_id) => {
					ensure!(who == account_id, Error::<T>::NotAllowed);
					Ok(())
				},
			}
		}
	}
}
