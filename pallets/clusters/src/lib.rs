//! This pallet `clusters` performs logic related to **Clusters**.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::BoundedVec;
pub use pallet::*;
use sp_clamor::Hash128;
use sp_std::{vec, vec::Vec};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::{Get, RuntimeDebug};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod dummy_data;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

/// The **settings** of a **Role**
#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub struct RoleSetting {
	/// Name of the setting
	pub name: Vec<u8>,
	/// The data associated with the Role
	pub data: Vec<u8>,
}

/// **Struct** of **Role** belonging to a **Cluster**.
#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub struct Role<TAccountId> {
	/// Name of the role
	pub name: Vec<u8>,
	/// The list of Members associated with the role
	pub members: Vec<TAccountId>,
	/// The optional list of Rules associated to the Role
	pub rules: Option<Rule>,
}

/// **Struct** of **Rule** belonging to a **Role**.
#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub struct Rule {
	/// The name of the Rule
	pub name: Vec<u8>,
}

/// **Struct** of a **Cluster**
#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub struct Cluster<TAccountId> {
	/// The owner of the Cluster
	pub owner: TAccountId,
	/// The name of the Cluster
	pub name: Vec<u8>,
	/// The ID of the cluster
	pub cluster_id: Hash128,
	/// The list of Roles associated to the Cluster
	pub roles: Vec<Role<TAccountId>>,
	/// The list of Members of the Cluster
	pub members: Vec<TAccountId>,
}

/// **Struct** representing the details about accounts created off-chain by various parties and integrations.
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq, Eq)]
pub struct AccountInfo<TAccountID, TMoment> {
	/// The actual account ID
	pub account_id: TAccountID,
	/// The timestamp when this account was created
	pub created_at: TMoment,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::{Cluster, Role, RoleSetting};
	use frame_support::{
		log,
		pallet_prelude::*,
		sp_runtime::traits::Zero,
		traits::{fungible, ReservableCurrency},
	};
	use frame_system::pallet_prelude::*;
	use sp_clamor::get_vault_id;
	use sp_io::hashing::{blake2_128, blake2_256};

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ pallet_balances::Config
		+ pallet_proxy::Config
		+ pallet_timestamp::Config
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The max size of name
		#[pallet::constant]
		type NameLimit: Get<u32>;

		/// The max size of data
		#[pallet::constant]
		type DataLimit: Get<u32>;

		/// The max number of members
		#[pallet::constant]
		type MembersLimit: Get<u32>;
	}

	/// **StorageMap** that maps a **Cluster** ID to its data.
	#[pallet::storage]
	pub type Clusters<T: Config> =
		StorageMap<_, Identity, Hash128, Cluster<T::AccountId>>;

	/// **StorageMap** that maps a **AccountId** with a list of **Cluster** owned by the account.
	#[pallet::storage]
	pub type ClustersByOwner<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Vec<Hash128>>;

	/// **StorageMap** that maps a **Role** with its **RoleSettings**.
	#[pallet::storage]
	pub type RoleToSettings<T: Config> =
		StorageMap<_, Twox64Concat, Hash128, Vec<RoleSetting>>;

	#[allow(missing_docs)]
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ClusterCreated {
			cluster_hash: Hash128,
		},
		RoleCreated {
			role_hash: Hash128,
		},
		RoleEdited {
			role_hash: Hash128,
		},
		RoleDeleted {
			role_hash: Hash128,
		},
		/// A new sponsored account was added
		ProxyAccountAdded {
			cluster_account: T::AccountId,
			proxy_account: T::AccountId,
		},
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
		InvalidInput,
		/// Technical error not categorized
		SystematicFailure,
		/// The owner is not correct.
		OwnerNotCorrect,
		/// The member already exists in the cluster
		MemberExists,
		/// Too many members
		MembersLimitReached,
		/// Member not found in the cluster
		MemberNotFound,
		/// Account proxy already associated with the cluster account
		AccountProxyAlreadyExists,
		/// Too many proxies associated with the cluster
		TooManyProxies,
		/// InsufficientBalance
		InsufficientFunds,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a **Cluster** passing a name as input.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn create_cluster(origin: OriginFor<T>, name: BoundedVec<u8, T::NameLimit>) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			ensure!(!name.len().is_zero(), Error::<T>::InvalidInput);

			let extrinsic_index = <frame_system::Pallet<T>>::extrinsic_index()
				.ok_or(Error::<T>::SystematicFailure)?;
			let current_block_number = <frame_system::Pallet<T>>::block_number();

			let cluster_id = blake2_128(
				&[current_block_number.encode(), Vec::from(name.clone()), extrinsic_index.clone().encode(), who.clone().encode()].concat(),
			);

			// Check that the cluster does not exist already
			ensure!(!<Clusters<T>>::contains_key(&cluster_id), Error::<T>::ClusterExists);

			// At creation there are no roles and no members assigned to the cluster
			let cluster = Cluster {
				owner: who.clone(),
				name: Vec::from(name),
				cluster_id,
				roles: vec![],
				members: vec![],
			};

			let minimum_balance =
				<pallet_balances::Pallet<T> as fungible::Inspect<T::AccountId>>::minimum_balance();
			let origin_balance = <pallet_balances::Pallet<T> as fungible::Inspect<T::AccountId>>::balance(&who);
			ensure!(origin_balance > minimum_balance, Error::<T>::InsufficientFunds);

			// write
			// create an account for the cluster
			let vault = get_vault_id(cluster_id);
			<pallet_balances::Pallet<T> as fungible::Mutate<T::AccountId>>::mint_into(
				&vault,
				minimum_balance,
			)?;

			// create a pure proxy
			pallet_proxy::Pallet::<T>::create_pure(
				origin.clone(),
				T::ProxyType::default(),
				T::BlockNumber::zero(),
				0,
			)?;

			<Clusters<T>>::insert(cluster_id, cluster);
			<ClustersByOwner<T>>::append(who.clone(), &cluster_id);

			Self::deposit_event(Event::ClusterCreated { cluster_hash: cluster_id });
			log::trace!("Cluster created: {:?}", hex::encode(cluster_id.clone()));

			Ok(())
		}

		/// Create a **Role** and assign it to an existing **Cluster**.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn create_role(
			origin: OriginFor<T>,
			cluster_id: Hash128,
			role_name: BoundedVec<u8, T::NameLimit>,
			settings: RoleSetting,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(!cluster_id.len().is_zero(), Error::<T>::InvalidInput);
			ensure!(!role_name.len().is_zero(), Error::<T>::InvalidInput);
			ensure!(!settings.name.len().is_zero(), Error::<T>::InvalidInput);
			ensure!(!settings.data.len().is_zero(), Error::<T>::InvalidInput);

			let role_name_vec = Vec::from(role_name.clone());
			let role_hash = blake2_128(&[&cluster_id[..], &role_name_vec[..]].concat());

			// Check that the caller is the owner of the cluster
			let cluster = <Clusters<T>>::get(&cluster_id).ok_or(Error::<T>::ClusterNotFound)?;
			ensure!(who == cluster.owner, Error::<T>::NoPermission);

			// At creation there are no Members and no Rules assigned to a Role
			let role = Role { name: role_name_vec.clone(), members: vec![], rules: None };

			// Check that the role does not exists already in the cluster
			let roles_in_cluster = <Clusters<T>>::get(&cluster_id)
				.ok_or(Error::<T>::ClusterNotFound)?
				.roles
				.into_iter()
				.filter(|role| role_name_vec.eq(&role.name))
				.collect::<Vec<Role<T::AccountId>>>();
			ensure!(roles_in_cluster.is_empty(), Error::<T>::RoleExists);

			// write
			<Clusters<T>>::mutate(&cluster_id, |cluster| {
				let cluster = cluster.as_mut().unwrap();
				cluster.roles.push(role);
			});

			<RoleToSettings<T>>::append(role_hash, settings);

			Self::deposit_event(Event::RoleCreated { role_hash });
			log::trace!("Role {:?} created and associated to cluster {:?}", role_hash, cluster_id);

			Ok(())
		}

		/// Edit a **Role**.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn edit_role(
			origin: OriginFor<T>,
			role_name: BoundedVec<u8, T::NameLimit>,
			cluster_id: Hash128,
			new_members_list: BoundedVec<T::AccountId, T::MembersLimit>,
			new_settings: RoleSetting,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let role_name_vec = Vec::from(role_name.clone());
			ensure!(!role_name.len().is_zero(), Error::<T>::InvalidInput);

			if new_members_list.len().is_zero() &&
				(new_settings.data.is_empty() || new_settings.name.is_empty())
			{
				return Err(Error::<T>::InvalidInput.into())
			}

			// Check that the caller is the owner of the cluster
			let cluster = <Clusters<T>>::get(&cluster_id).ok_or(Error::<T>::SystematicFailure)?;
			ensure!(who == cluster.owner, Error::<T>::NoPermission);

			// Check that the role exists in the cluster and in storage
			let roles_in_cluster = <Clusters<T>>::get(&cluster_id)
				.ok_or(Error::<T>::ClusterNotFound)?
				.roles
				.into_iter()
				.filter(|role| role_name_vec.eq(&role.name))
				.collect::<Vec<Role<T::AccountId>>>();
			ensure!(!roles_in_cluster.is_empty(), Error::<T>::RoleNotFound);

			let role_hash = blake2_128(&[&cluster_id[..], &role_name_vec].concat());

			// write
			<Clusters<T>>::mutate(&cluster_id, |cluster| {
				let cluster = cluster.as_mut().unwrap();
				let index = cluster.roles.iter().position(|x| role_name_vec.eq(&x.name));
				if let Some(index) = index {
					let role = cluster.roles.get(index).unwrap();
					let mut members = role.clone().members;
					for member in new_members_list {
						members.push(member);
					}
				}
			});
			<RoleToSettings<T>>::mutate(&role_hash, |role| {
				let role_settings = role.as_mut().unwrap();
				role_settings.push(new_settings);
			});

			Self::deposit_event(Event::RoleEdited { role_hash });
			log::trace!("Role edited: {:?}", role_hash);

			Ok(())
		}

		/// Delete a **Role**.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn delete_role(
			origin: OriginFor<T>,
			role_name: BoundedVec<u8, T::NameLimit>,
			cluster_id: Hash128,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let role_name_vec = Vec::from(role_name.clone());
			ensure!(!role_name.len().is_zero(), Error::<T>::InvalidInput);

			// only the owner of the cluster can do this operation
			let cluster = <Clusters<T>>::get(&cluster_id).ok_or(Error::<T>::ClusterNotFound)?;
			ensure!(who == cluster.owner, Error::<T>::NoPermission);

			// Check that the role exists in the cluster and in storage
			let roles_in_cluster = cluster
				.roles
				.into_iter()
				.filter(|role| role_name_vec.eq(&role.name))
				.collect::<Vec<Role<T::AccountId>>>();
			ensure!(!roles_in_cluster.is_empty(), Error::<T>::RoleNotFound);

			// write
			// Remove Role from Cluster
			<Clusters<T>>::mutate(&cluster_id, |cluster| {
				let cluster = cluster.as_mut().unwrap();
				let index = cluster.roles.iter().position(|x| role_name_vec.eq(&x.name));
				if let Some(index) = index {
					cluster.roles.remove(index);
				}
			});

			let role_hash = blake2_128(&[&cluster_id[..], &role_name_vec].concat());
			if !roles_in_cluster.is_empty() {
				<RoleToSettings<T>>::remove(&role_hash);
			}

			Self::deposit_event(Event::RoleDeleted { role_hash });
			log::trace!("Role deleted: {:?}", role_hash);

			Ok(())
		}

		/// Create a **Member**, give it a set of existing **Role** and assign it to an existing **Cluster**.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn add_member(
			origin: OriginFor<T>,
			cluster_id: Hash128,
			roles: Vec<Vec<u8>>,
			member: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Check that the cluster exists and the caller is the owner of the cluster
			let cluster = <Clusters<T>>::get(cluster_id).ok_or(Error::<T>::ClusterNotFound)?;
			ensure!(who == cluster.owner, Error::<T>::NoPermission);

			// Check that the cluster does not already contain the member
			ensure!(!cluster.members.contains(&member), Error::<T>::MemberExists);

			ensure!(cluster.members.len() < T::MembersLimit::get() as usize, Error::<T>::MembersLimitReached);

			// Check that the roles for the member already exists in the cluster
			let roles_in_cluster: Vec<Vec<u8>> =
				cluster.roles.iter().map(|role| role.name.clone()).collect();
			for role in roles.clone() {
				ensure!(
					roles_in_cluster.contains(&role),
					Error::<T>::RoleNotFound
				);
			}

			// write
			<Clusters<T>>::mutate(&cluster_id, |cluster| {
				let cluster = cluster.as_mut().unwrap();
				// Add member into the cluster
				cluster.members.push(member.clone());

				// Associate the member with its roles in the cluster
				for role in roles.clone() {
					let index = cluster.roles.iter().position(|x| x.name == role);
					if let Some(index) = index {
						let role =
							cluster.roles.get(index).ok_or(Error::<T>::SystematicFailure).unwrap();
						let mut role_members = role.clone().members;
						role_members.push(member.clone());
					}
				}
			});

			Ok(())
		}

		/// Delete a **Member** from an existing **Cluster**.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn delete_member(
			origin: OriginFor<T>,
			cluster_id: Hash128,
			member: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let cluster = <Clusters<T>>::get(cluster_id).ok_or(Error::<T>::ClusterNotFound)?;
			ensure!(who == cluster.owner, Error::<T>::NoPermission);
			// Check that the member is in the cluster
			ensure!(cluster.members.contains(&member), Error::<T>::MemberNotFound);

			// write
			// Delete member from Cluster
			<Clusters<T>>::mutate(&cluster_id, |cluster| {
				let cluster = cluster.as_mut().unwrap();
				let index = cluster.members.iter().position(|x| x == &member);
				if let Some(index) = index {
					cluster.members.remove(index);
				}
			});

			Ok(())
		}
	}
}
