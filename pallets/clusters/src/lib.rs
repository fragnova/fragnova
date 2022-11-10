#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Compact, Decode, Encode};
pub use pallet::*;
use sp_clamor::Hash256;
use sp_std::{vec, vec::Vec};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod dummy_data;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

/// **Struct** of a **Member** belonging to a **Role** in 1..N **Clusters**
#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub struct Member {
	pub data: Vec<u8>,
}

/// The **settings** of a **Role**
#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub struct RoleSettings {
	pub name: Vec<u8>,
	pub data: Vec<u8>,
}

/// **Struct** of **Role** belonging to a **Cluster**.
#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub struct Role<TAccountId> {
	pub owner: TAccountId,
	pub name: Vec<u8>,
	pub settings: Vec<RoleSettings>,
}

/// **Struct** of **Rule** belonging to a **Role**.
#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub struct Rule {
	pub name: Vec<u8>,
}

/// **Struct** of a **Cluster**
#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub struct Cluster<TAccountId> {
	pub owner: TAccountId,
	pub name: Vec<u8>,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::{Cluster, Role, RoleSettings};
	use frame_support::{log, pallet_prelude::*, sp_runtime::traits::Zero};
	use frame_system::pallet_prelude::*;
	use sp_clamor::Hash256;
	use sp_io::hashing::blake2_256;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	/// **StorageMap** that maps a **Cluster** with its ID.
	/// The storage key is the name of the cluster, which implies that there cannot be two clusters
	/// with the same name.
	#[pallet::storage]
	pub type Clusters<T: Config> = StorageMap<_, Identity, Hash256, Cluster<T::AccountId>>;

	/// **StorageMap** that maps a **AccountId** with a list of **Cluster** owned by the account.
	#[pallet::storage]
	pub type ClustersToOwner<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Vec<Hash256>>;

	/// **StorageMap** that maps a **Role** with its ID.
	#[pallet::storage]
	pub type Roles<T: Config> = StorageMap<_, Twox64Concat, Hash256, Role<T::AccountId>>;

	/// **StorageMap** that maps a **Cluster** with the list of **Roles** belonging to the cluster.
	#[pallet::storage]
	pub type ClusterRoles<T: Config> = StorageMap<_, Identity, Hash256, Vec<Hash256>>;

	/// **StorageMap** that maps a **Cluster** with the list of **Members** belonging to the cluster.
	#[pallet::storage]
	pub type ClusterMembers<T: Config> = StorageMap<_, Identity, Hash256, Vec<Hash256>>;

	#[allow(missing_docs)]
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ClusterCreated { cluster_hash: Hash256 },
		RoleCreated { role_hash: Hash256 },
		RoleEdited { role_hash: Hash256 },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// The cluster already exists.
		ClusterExists,
		/// Cluster not found
		ClusterNotFound,
		/// The role in the cluster already exists.
		RoleExists,
		/// Element not found
		RoleNotFound,
		/// Missing permission to perform an operation
		NoPermission,
		/// Invalid inputs
		InvalidInputs,
		/// Technical error not categorized
		SystematicFailure,
		/// The owner is not correct.
		OwnerNotCorrect,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a **Cluster**.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn create_cluster(origin: OriginFor<T>, name: Vec<u8>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(!name.len().is_zero(), Error::<T>::SystematicFailure);

			let cluster_hash = blake2_256(&name);

			ensure!(!<Clusters<T>>::contains_key(&cluster_hash), Error::<T>::ClusterExists);

			let cluster = Cluster { owner: who.clone(), name };

			<Clusters<T>>::insert(cluster_hash, cluster);
			<ClustersToOwner<T>>::append(who.clone(), &cluster_hash);

			Self::deposit_event(Event::ClusterCreated { cluster_hash });
			log::trace!("Cluster created: {:?}", cluster_hash.clone());

			Ok(())
		}

		/// Create a **Role** and assign it to an existing **Cluster** specified by name.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn create_role(
			origin: OriginFor<T>,
			cluster_name: Vec<u8>,
			role_name: Vec<u8>,
			settings: RoleSettings,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// not putting the settings in the hash because that would allow roles with same name, but different settings.
			let role_hash = blake2_256(&[role_name.clone(), cluster_name.clone()].concat());
			let cluster_hash = blake2_256(&cluster_name.clone());

			let existing_roles = <ClusterRoles<T>>::get(&cluster_hash)
				.ok_or(|| Error::<T>::SystematicFailure)
				.unwrap_or_default();
			ensure!(!existing_roles.contains(&role_hash), Error::<T>::RoleExists);

			let role = Role { owner: who.clone(), name: role_name, settings: vec![settings] };

			<Roles<T>>::insert(role_hash, role);
			// assign the role to the specified cluster
			<ClusterRoles<T>>::append(cluster_hash, role_hash);

			Self::deposit_event(Event::RoleCreated { role_hash });
			log::trace!("Role created: {:?}", role_hash);

			Ok(())
		}

		/// Edit a **Role**.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn edit_role(
			origin: OriginFor<T>,
			role: Hash256,
			cluster: Vec<u8>,
			new_settings: RoleSettings,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			sp_std::if_std! {
				// This code is only being compiled and executed when the `std` feature is enabled.
				println!("new settings");
			}

			if new_settings.data.is_empty() || new_settings.name.is_empty() {
				return Err(Error::<T>::InvalidInputs.into());
			}

			let stored_role: Role<T::AccountId> =
				<Roles<T>>::get(role).ok_or(Error::<T>::RoleNotFound)?;
			ensure!(who == stored_role.owner, Error::<T>::NoPermission);

			let cluster_hash = blake2_256(&cluster);

			let existing_roles = <ClusterRoles<T>>::get(&cluster_hash)
				.ok_or(|| Error::<T>::SystematicFailure)
				.unwrap_or_default();
			ensure!(existing_roles.contains(&role), Error::<T>::RoleNotFound);

			<Roles<T>>::mutate(&role, |role| {
				let role = role.as_mut().unwrap();
				role.settings = vec![new_settings];
			});

			Self::deposit_event(Event::RoleEdited { role_hash: role });
			log::trace!("Role edited: {:?}", role);

			Ok(())
		}

		/// Create a **Rule** and assign it to an existing **Role**.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn add_member(
			origin: OriginFor<T>,
			cluster_name: Vec<u8>,
			role_name: Vec<u8>,
			member_data: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;




			Ok(())
		}
	}
}
