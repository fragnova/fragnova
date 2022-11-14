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
pub struct ClusterMember {
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
pub struct Role {
	pub name: Vec<u8>,
	pub settings: RoleSettings,
	pub members: Vec<ClusterMember>,
	pub rules: Option<Rule>,
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
	pub roles: Vec<Hash256>,
	pub members: Vec<Hash256>,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::{Cluster, Role, RoleSettings};
	use frame_support::traits::fungible;
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
	pub trait Config: frame_system::Config + pallet_balances::Config {
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
	pub type ClustersByOwner<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Vec<Hash256>>;

	/// **StorageMap** that maps a **Cluster** hash with a list of **AccountId** associated to it.
	#[pallet::storage]
	pub type ClusterAccounts<T: Config> = StorageMap<_, Twox64Concat, Hash256, Vec<T::AccountId>>;

	/// **StorageMap** that maps a **Role** with its ID.
	#[pallet::storage]
	pub type Roles<T: Config> = StorageMap<_, Twox64Concat, Hash256, Role>;

	/// **StorageMap** that maps a **Role** with its ID.
	#[pallet::storage]
	pub type Members<T: Config> = StorageMap<_, Twox64Concat, Hash256, ClusterMember>;

	#[allow(missing_docs)]
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ClusterCreated { cluster_hash: Hash256 },
		RoleCreated { role_hash: Hash256 },
		RoleEdited { role_hash: Hash256 },
		RoleDeleted { role_hash: Hash256 },
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
		/// The member already exists in the cluster
		MemberExists,
		/// Member not found in the cluster
		MemberNotFound,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a **Cluster**. A Cluster is stored using its name as key, so there cannot be multiple clusters with the same name.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn create_cluster(origin: OriginFor<T>, name: Vec<u8>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(!name.len().is_zero(), Error::<T>::SystematicFailure);

			let cluster_hash = blake2_256(&name);

			ensure!(!<Clusters<T>>::contains_key(&cluster_hash), Error::<T>::ClusterExists);

			// At creation there are no roles and no members assigned to the cluster
			let cluster = Cluster { owner: who.clone(), name, roles: vec![], members: vec![] };

			// create an account for the cluster
			let vault = Self::get_vault_id(cluster_hash);
			let minimum_balance =
				<pallet_balances::Pallet<T> as fungible::Inspect<T::AccountId>>::minimum_balance();
			<pallet_balances::Pallet<T> as fungible::Mutate<T::AccountId>>::mint_into(
				&vault,
				minimum_balance,
			)?;

			<Clusters<T>>::insert(cluster_hash, cluster);
			<ClustersByOwner<T>>::append(who.clone(), &cluster_hash);

			Self::deposit_event(Event::ClusterCreated { cluster_hash });
			log::trace!("Cluster created: {:?}", cluster_hash.clone());

			Ok(())
		}

		/// Create a **Role** and assign it to an existing **Cluster**.
		/// There cannot be roles with the same name into a cluster.
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

			ensure!(!<Roles<T>>::contains_key(role_hash), Error::<T>::RoleExists);

			let roles_in_cluster =
				<Clusters<T>>::get(&cluster_hash).ok_or(Error::<T>::ClusterNotFound)?;
			ensure!(!roles_in_cluster.roles.contains(&role_hash), Error::<T>::RoleExists);

			// At creation there are no Members and no Rules assigned to a Role
			let role = Role {
				name: role_name,
				settings,
				members: vec![],
				rules: None,
			};

			<Roles<T>>::insert(role_hash, role);

			<Clusters<T>>::mutate(&cluster_hash, |cluster| {
				let cluster = cluster.as_mut().unwrap();
				cluster.roles.push(role_hash);
			});

			Self::deposit_event(Event::RoleCreated { role_hash });
			log::trace!(
				"Role {:?} created and associated to cluster {:?}",
				role_hash,
				cluster_hash
			);

			Ok(())
		}

		/// Edit a **Role**.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn edit_role(
			origin: OriginFor<T>,
			role_hash: Hash256,
			cluster_hash: Hash256,
			new_settings: RoleSettings,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			if new_settings.data.is_empty() || new_settings.name.is_empty() {
				return Err(Error::<T>::InvalidInputs.into());
			}

			// only the owner of the cluster can do this operation
			let cluster = <Clusters<T>>::get(&cluster_hash).ok_or(Error::<T>::SystematicFailure)?;
			ensure!(who == cluster.owner, Error::<T>::NoPermission);

			let roles_in_cluster =
				<Clusters<T>>::get(&cluster_hash).ok_or(Error::<T>::ClusterNotFound)?.roles;
			ensure!(roles_in_cluster.contains(&role_hash), Error::<T>::RoleNotFound);
			ensure!(<Roles<T>>::contains_key(role_hash), Error::<T>::RoleNotFound);

			<Roles<T>>::mutate(&role_hash, |role| {
				let role = role.as_mut().unwrap();
				role.settings = new_settings;
			});

			Self::deposit_event(Event::RoleEdited { role_hash });
			log::trace!("Role edited: {:?}", role_hash);

			Ok(())
		}

		/// Delete a **Role**.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn delete_role(
			origin: OriginFor<T>,
			role_hash: Hash256,
			cluster_hash: Hash256,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// only the owner of the cluster can do this operation
			let cluster = <Clusters<T>>::get(&cluster_hash).ok_or(Error::<T>::SystematicFailure)?;
			ensure!(who == cluster.owner, Error::<T>::NoPermission);

			let roles_in_cluster =
				<Clusters<T>>::get(&cluster_hash).ok_or(Error::<T>::ClusterNotFound)?.roles;
			ensure!(roles_in_cluster.contains(&role_hash), Error::<T>::RoleNotFound);
			ensure!(<Roles<T>>::contains_key(role_hash), Error::<T>::RoleNotFound);

			// Remove from Roles storage
			<Roles<T>>::remove(role_hash);
			// Remove association to Cluster
			<Clusters<T>>::mutate(&cluster_hash, |cluster| {
				let cluster = cluster.as_mut().unwrap();
				let index = cluster.roles.iter().position(|x| x == &role_hash);
				if let Some(index) = index {
					cluster.roles.remove(index);
				}
			});

			Self::deposit_event(Event::RoleDeleted { role_hash });
			log::trace!("Role deleted: {:?}", role_hash);

			Ok(())
		}

		/// Create a **Member**, give it a set of existing **Role** and assign it to an existing **Cluster**.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn add_member(
			origin: OriginFor<T>,
			cluster_name: Vec<u8>,
			roles_name: Vec<Vec<u8>>,
			member_data: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let cluster_hash = blake2_256(&cluster_name.clone());
			let cluster = <Clusters<T>>::get(cluster_hash).ok_or(Error::<T>::ClusterNotFound)?;
			ensure!(who == cluster.owner, Error::<T>::NoPermission);

			let member_hash = blake2_256(&[cluster_name.clone(), member_data.clone()].concat());
			ensure!(!cluster.members.contains(&member_hash), Error::<T>::MemberExists);

			for role in roles_name.clone() {
				let role_hash = blake2_256(&[role.clone(), cluster_name.clone()].concat());
				ensure!(cluster.roles.contains(&role_hash), Error::<T>::RoleNotFound);
				ensure!(<Roles<T>>::contains_key(role_hash), Error::<T>::RoleNotFound);
			}

			let member = ClusterMember { data: member_data };

			<Members<T>>::insert(member_hash, member.clone());
			<Clusters<T>>::mutate(&cluster_hash, |cluster| {
				let cluster = cluster.as_mut().unwrap();
				cluster.members.push(cluster_hash.clone());
			});
			for role in roles_name.clone() {
				let role_hash = blake2_256(&[role.clone(), cluster_name.clone()].concat());
				<Roles<T>>::mutate(&role_hash, |role| {
					let role = role.as_mut().unwrap();
					role.members.push(member.clone());
				});
			}

			Ok(())
		}

		/// Delete a **Member** from an existing **Cluster**.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn delete_member(
			origin: OriginFor<T>,
			cluster_hash: Hash256,
			member_hash: Hash256,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let cluster = <Clusters<T>>::get(cluster_hash).ok_or(Error::<T>::ClusterNotFound)?;
			ensure!(who == cluster.owner, Error::<T>::NoPermission);

			ensure!(cluster.members.contains(&member_hash), Error::<T>::MemberNotFound);

			// Delete member from storage
			<Members<T>>::remove(member_hash);
			// Delete member from Cluster
			<Clusters<T>>::mutate(&cluster_hash, |cluster| {
				let cluster = cluster.as_mut().unwrap();
				let index = cluster.members.iter().position(|x| x == &member_hash);
				if let Some(index) = index {
					cluster.members.remove(index);
				}
			});

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// **Get** the **Account ID** of the Cluster specified by its `cluster_hash`**.
		/// This Account ID is deterministically computed using the `cluster_hash`
		pub fn get_vault_id(cluster_hash: Hash256) -> T::AccountId {
			let hash = blake2_256(&[&b"cluster-vault"[..], &cluster_hash].concat());
			T::AccountId::decode(&mut &hash[..]).expect("T::AccountId should decode")
		}
	}
}
