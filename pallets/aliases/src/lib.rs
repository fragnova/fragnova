//! This pallet `aliases` allows users to create human readable aliases to address their owned existing assets.
//! Example:
//!
//! A user owns a Proto that represents an image of "batman" on chain.
//! This asset is addressable using its hash, which is not human readable and impossible to remember.
//! This pallet-aliases allows the user to create an alias like "DC/batman" that links to that hash
//! and can be used by the owner to demonstrate ownership without the need to use the hash of the asset.
//!
//! "DC/batman" --> f9480f9ead9b82690fodf65546kjmyg730f12763ca2f50ce1792
//!
//! An alias is composed of two parts: <namespace>/<alias>.
//! The namespace is a unique string that identifies the owner.
//! The alias is a string that identifies the asset.
//!
//! The user must be owner of the linked asset in order to create the alias.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate core;

use codec::{Compact, Decode, Encode};
use frame_support::traits::{Currency, ExistenceRequirement, WithdrawReasons};
pub use pallet::*;
use pallet_clusters::Clusters;
use pallet_fragments::Owners;
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
	/// A Proto defined by its hash
	Proto(Hash256),
	/// A Fragment instance defined by
	Fragment {
		/// definition_hash of the Fragment target
		definition_hash: Hash128,
		/// edition_id of the Fragment target
		edition: InstanceUnit,
		/// copy_id of the Fragment target
		copy: InstanceUnit,
	}, // struct variant allows a nicer UX on polkadotJS
	/// An AccountId
	Account(TAccountId),
	/// A Cluster defined by its hash
	Cluster(Hash128),
}

/// Struct that add versioning to a LinkTarget using block number as version number.
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq)]
pub struct LinkTargetVersioned<TAccountId, TBlockNum> {
	/// The linked asset.
	pub link_target: LinkTarget<TAccountId>,
	/// The previous block number indicating the previous version of the linked asset.
	pub prev_block_number: TBlockNum,
	/// The block number indicating the current version of the linked asset.
	pub cur_block_number: TBlockNum,
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

		/// The max size of namespace and alias strings
		#[pallet::constant]
		type NameLimit: Get<u32>;

		/// The root namespace
		#[pallet::constant]
		type RootNamespace: Get<Vec<u8>>;
	}

	/// **StorageMap** that maps a **Namespace** to its owner.
	#[pallet::storage]
	pub type Namespaces<T: Config> = StorageMap<_, Twox64Concat, Vec<u8>, T::AccountId>;

	/// **StorageMap** that maps a **Name (of type `Vec<u8>`)** to an **index**.
	/// This ensures no duplicated aliases are used.
	#[pallet::storage]
	pub type Names<T: Config> = StorageMap<_, Twox64Concat, Vec<u8>, Compact<u64>>;

	/// **StorageValue** that **equals** the **total number of unique names** used for aliases in the chain.
	#[pallet::storage]
	pub type NamesIndex<T: Config> = StorageValue<_, u64, ValueQuery>;

	/// **StorageDoubleMap** that maps a (**Namespace**, **Alias** name index) with a **LinkTarget**.
	#[pallet::storage]
	pub type Aliases<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		Vec<u8>, // namespace
		Twox64Concat,
		Compact<u64>,                                      // alias name index
		LinkTargetVersioned<T::AccountId, T::BlockNumber>, // target asset
	>;

	#[allow(missing_docs)]
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NamespaceCreated { who: T::AccountId, namespace: Vec<u8> },
		NamespaceTransferred { namespace: Vec<u8>, from: T::AccountId, to: T::AccountId },
		NamespaceDeleted { namespace: Vec<u8> },
		AliasCreated { who: T::AccountId, namespace: Vec<u8>, alias: Vec<u8> },
		AliasDeleted { namespace: Vec<u8>, alias: Vec<u8> },
		AliasTargetUpdated { namespace: Vec<u8>, alias: Vec<u8> },
		RootAliasCreated { root_namespace: Vec<u8>, alias: Vec<u8> },
		RootAliasDeleted { root_namespace: Vec<u8>, alias: Vec<u8> },
		RootAliasUpdated { root_namespace: Vec<u8>, alias: Vec<u8> },
	}

	// Errors inform users that something went wrong.
	#[allow(missing_docs)]
	#[pallet::error]
	pub enum Error<T> {
		InvalidInput,
		NamespaceExists,
		NamespaceNotFound,
		NotAllowed,
		SystematicFailure,
		AliasExists,
		AliasNotFound,
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
			// reduce NOVA balance from caller's account. The amount is set in Config.
			let amount: <T as pallet_balances::Config>::Balance =
				<T as Config>::NamespacePrice::get().saturated_into();
			let _ = <pallet_balances::Pallet<T> as Currency<T::AccountId>>::withdraw(
				&who,
				amount,
				WithdrawReasons::TRANSACTION_PAYMENT,
				ExistenceRequirement::KeepAlive,
			)?;

			<Namespaces<T>>::insert(&namespace, &who);

			Self::deposit_event(Event::NamespaceCreated { who, namespace });

			Ok(())
		}

		/// Delete a new **Namespace**.
		///
		/// This also deletes all the aliases linked to this namespace
		///
		/// - `namespace`: namespace to delete
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn delete_namespace(
			origin: OriginFor<T>,
			namespace: BoundedVec<u8, <T as pallet::Config>::NameLimit>,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			ensure!(!namespace.len().is_zero(), Error::<T>::InvalidInput);

			let namespace = namespace.into_inner();

			// Check that the namespace exists
			ensure!(<Namespaces<T>>::contains_key(&namespace), Error::<T>::NamespaceExists);
			// check that the caller is the owner of the namespace
			let owner = <Namespaces<T>>::get(&namespace).ok_or(Error::<T>::NamespaceNotFound)?;
			ensure!(&who == &owner, Error::<T>::NotAllowed);

			<Namespaces<T>>::remove(&namespace);
			let _ = <Aliases<T>>::clear_prefix(&namespace, u32::MAX, None);

			Self::deposit_event(Event::NamespaceDeleted { namespace });

			Ok(())
		}

		/// Transfer the **Namespace ownership** to another AccountId.
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
			ensure!(!<Aliases<T>>::contains_key(&namespace, &alias_index), Error::<T>::AliasExists);

			// check that the caller is the owner of the target asset
			Self::is_target_owner(&who, target.clone())?;

			let current_block_number = <frame_system::Pallet<T>>::block_number();
			let target_versioned = LinkTargetVersioned {
				link_target: target,
				prev_block_number: T::BlockNumber::zero(),
				cur_block_number: current_block_number,
			};
			<Aliases<T>>::insert(&namespace, &alias_index, target_versioned);

			Self::deposit_event(Event::AliasCreated { who, namespace, alias: alias.into_inner() });

			Ok(())
		}

		/// Create a root **Alias** that links to a target asset.
		///
		/// Only the root can execute this.
		///
		/// The Namespace is set in Config.
		///
		/// There is no ownership check of the linked asset.
		///
		/// - `alias`: root alias to create
		/// - `target`: LinkTarget to link the alias to
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_root_alias(
			origin: OriginFor<T>,
			alias: BoundedVec<u8, <T as pallet::Config>::NameLimit>,
			target: LinkTarget<T::AccountId>,
		) -> DispatchResult {
			ensure_root(origin.clone())?;

			let root_namespace = T::RootNamespace::get();
			let alias_index = Self::take_name_index(&alias);
			// check that the caller does not already own this alias
			ensure!(
				!<Aliases<T>>::contains_key(&root_namespace, &alias_index),
				Error::<T>::AliasExists
			);

			let current_block_number = <frame_system::Pallet<T>>::block_number();
			let target_versioned = LinkTargetVersioned {
				link_target: target,
				prev_block_number: T::BlockNumber::zero(),
				cur_block_number: current_block_number,
			};
			<Aliases<T>>::insert(&root_namespace, &alias_index, target_versioned);

			Self::deposit_event(Event::RootAliasCreated {
				root_namespace,
				alias: alias.into_inner(),
			});

			Ok(())
		}

		/// Replace the **LinkTarget** of an alias with another LinkTarget.
		///
		/// Only the root can execute this.
		///
		/// There is no ownership check of the linked new_target.
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

			let stored_alias =
				<Aliases<T>>::get(&namespace, &alias_index).ok_or(Error::<T>::AliasNotFound)?;
			// get stored current block number and set it as previous
			let current_block_number = <frame_system::Pallet<T>>::block_number();
			let new_target_versioned = LinkTargetVersioned {
				link_target: new_target,
				prev_block_number: stored_alias.cur_block_number,
				cur_block_number: current_block_number,
			};

			<Aliases<T>>::insert(&namespace, &alias_index, new_target_versioned);

			Self::deposit_event(Event::AliasTargetUpdated { namespace, alias: alias.into_inner() });

			Ok(())
		}

		/// Replace the **LinkTarget** of a root alias with another LinkTarget.
		///
		/// Only the root can execute this.
		///
		/// There is no ownership check of the linked new_target.
		///
		/// - `alias`: the alias to update
		/// - `new_target`: the new LinkTarget to link the alias to
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn update_root_alias_target(
			origin: OriginFor<T>,
			alias: BoundedVec<u8, <T as pallet::Config>::NameLimit>,
			new_target: LinkTarget<T::AccountId>,
		) -> DispatchResult {
			ensure_root(origin.clone())?;

			ensure!(!alias.len().is_zero(), Error::<T>::InvalidInput);

			let root_namespace = T::RootNamespace::get();
			let alias_index = Self::take_name_index(&alias);

			let stored_alias = <Aliases<T>>::get(&root_namespace, &alias_index)
				.ok_or(Error::<T>::AliasNotFound)?;
			// get stored current block number and set it as previous
			let current_block_number = <frame_system::Pallet<T>>::block_number();
			let new_target_versioned = LinkTargetVersioned {
				link_target: new_target,
				prev_block_number: stored_alias.cur_block_number,
				cur_block_number: current_block_number,
			};

			<Aliases<T>>::insert(&root_namespace, &alias_index, new_target_versioned);

			Self::deposit_event(Event::RootAliasUpdated {
				root_namespace,
				alias: alias.into_inner(),
			});

			Ok(())
		}

		/// Delete an alias.
		///
		/// Only the owner of the namespace linked to the alias can execute this.
		///
		/// - `namespace`: namespace related to the alias to delete
		/// - `alias`: the alias to delete
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn delete_alias(
			origin: OriginFor<T>,
			namespace: BoundedVec<u8, <T as pallet::Config>::NameLimit>,
			alias: BoundedVec<u8, <T as pallet::Config>::NameLimit>,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			ensure!(!namespace.len().is_zero(), Error::<T>::InvalidInput);
			ensure!(!alias.len().is_zero(), Error::<T>::InvalidInput);

			let namespace = namespace.into_inner();

			let owner = <Namespaces<T>>::get(&namespace).ok_or(Error::<T>::NamespaceNotFound)?;
			ensure!(&who == &owner, Error::<T>::NotAllowed);

			let alias_index = Self::take_name_index(&alias);
			ensure!(
				<Aliases<T>>::contains_key(&namespace, &alias_index),
				Error::<T>::AliasNotFound
			);

			<Aliases<T>>::remove(&namespace, &alias_index);

			Self::deposit_event(Event::AliasDeleted { namespace, alias: alias.into_inner() });

			Ok(())
		}

		/// Delete a root alias.
		///
		/// Only the root can execute this.
		///
		/// - `alias`: the root alias to delete
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn delete_root_alias(
			origin: OriginFor<T>,
			alias: BoundedVec<u8, <T as pallet::Config>::NameLimit>,
		) -> DispatchResult {
			ensure_root(origin.clone())?;

			ensure!(!alias.len().is_zero(), Error::<T>::InvalidInput);

			let root_namespace = T::RootNamespace::get();
			let alias_index = Self::take_name_index(&alias);

			ensure!(
				<Aliases<T>>::contains_key(&root_namespace, &alias_index),
				Error::<T>::AliasNotFound
			);

			<Aliases<T>>::remove(&root_namespace, &alias_index);

			Self::deposit_event(Event::RootAliasDeleted {
				root_namespace,
				alias: alias.into_inner(),
			});

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

		/// Utility function that checks if an AccountId is the owner of the assets in the LinkTarget
		///
		/// - `who`: the AccountId
		/// - `target`: the LinkTarget containing the asset to check ownership
		pub fn is_target_owner(
			who: &T::AccountId,
			target: LinkTarget<T::AccountId>,
		) -> DispatchResult {
			match target {
				LinkTarget::Fragment { definition_hash, edition, copy } => {
					let owner = Owners::<T>::iter_prefix(definition_hash)
						.find(|(_owner, vec_instances)| {
							vec_instances.iter().any(|(edition_id, copy_id)| {
								Compact(edition) == *edition_id && Compact(copy) == *copy_id
							})
						})
						.ok_or("Fragment instance owner not found.")?
						.0;
					ensure!(*who == owner, Error::<T>::NotAllowed);
					Ok(())
				},
				LinkTarget::Proto(proto_hash) => {
					let proto: Proto<T::AccountId, T::BlockNumber> = <Protos<T>>::get(&proto_hash)
						.ok_or(pallet_protos::Error::<T>::ProtoNotFound)?;
					match proto.owner {
						ProtoOwner::User(owner) => ensure!(*who == owner, Error::<T>::NotAllowed),
						_ => {
							ensure!(false, Error::<T>::NotAllowed)
						},
					};
					Ok(())
				},
				LinkTarget::Cluster(cluster_id) => {
					let cluster = <Clusters<T>>::get(&cluster_id)
						.ok_or(pallet_clusters::Error::<T>::ClusterNotFound)?;
					ensure!(*who == cluster.owner, Error::<T>::NotAllowed);
					Ok(())
				},
				LinkTarget::Account(account_id) => {
					ensure!(*who == account_id, Error::<T>::NotAllowed);
					Ok(())
				},
			}
		}
	}
}
