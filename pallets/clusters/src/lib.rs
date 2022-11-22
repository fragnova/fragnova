//! This pallet `clusters` performs logic related to **Clusters**.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
pub use pallet::*;
use sp_clamor::Hash128;
use sp_core::bounded::BoundedVec;
use sp_std::{vec, vec::Vec};

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
	use frame_support::traits::ReservableCurrency;
	use frame_support::{log, pallet_prelude::*, sp_runtime::traits::Zero, traits::fungible};
	use frame_system::pallet_prelude::*;
	use sp_io::{
		hashing::{blake2_128, blake2_256},
	};

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
	}

	/// **StorageMap** that maps a **Cluster** ID to its data.
	#[pallet::storage]
	pub type Clusters<T: Config> = StorageMap<_, Identity, Hash128, Cluster<T::AccountId>>;

	/// **StorageMap** that maps a **AccountId** with a list of **Cluster** owned by the account.
	#[pallet::storage]
	pub type ClustersByOwner<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Vec<Hash128>>;

	/// **StorageMap** that maps a **Role** with its **RoleSettings**.
	#[pallet::storage]
	pub type RoleToSettings<T: Config> = StorageMap<_, Twox64Concat, Hash128, Vec<RoleSetting>>;

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
		InvalidInputs,
		/// Technical error not categorized
		SystematicFailure,
		/// The owner is not correct.
		OwnerNotCorrect,
		/// The member already exists in the cluster
		MemberExists,
		/// Member not found in the cluster
		MemberNotFound,
		/// Account proxy already associated with the cluster account
		AccountProxyAlreadyExists,
		/// Too many proxies associated with the cluster
		TooManyProxies,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a **Cluster** passing a name as input.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn create_cluster(origin: OriginFor<T>, name: Vec<u8>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(!name.len().is_zero(), Error::<T>::InvalidInputs);

			let extrinsic_index = <frame_system::Pallet<T>>::extrinsic_index()
				.ok_or(Error::<T>::SystematicFailure)?;

			let cluster_id = blake2_128(
				&[name.clone(), extrinsic_index.clone().encode(), who.clone().encode()].concat(),
			);

			sp_std::if_std! {
				// This code is only being compiled and executed when the `std` feature is enabled.
				println!("CLUSTER_ID: {:?}", cluster_id);
			}
			// Check that the cluster does not exist already
			ensure!(!<Clusters<T>>::contains_key(&cluster_id), Error::<T>::ClusterExists);

			// At creation there are no roles and no members assigned to the cluster
			let cluster =
				Cluster { owner: who.clone(), name, cluster_id, roles: vec![], members: vec![] };

			// create an account for the cluster
			let vault = Self::get_vault_id(cluster_id);
			let minimum_balance =
				<pallet_balances::Pallet<T> as fungible::Inspect<T::AccountId>>::minimum_balance();
			<pallet_balances::Pallet<T> as fungible::Mutate<T::AccountId>>::mint_into(
				&vault,
				minimum_balance,
			)?;

			// write
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
			role_name: Vec<u8>,
			settings: RoleSetting,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(!cluster_id.len().is_zero(), Error::<T>::InvalidInputs);
			ensure!(!role_name.len().is_zero(), Error::<T>::InvalidInputs);
			ensure!(!settings.name.len().is_zero(), Error::<T>::InvalidInputs);
			ensure!(!settings.data.len().is_zero(), Error::<T>::InvalidInputs);

			let role_hash = blake2_128(&[&cluster_id[..], &role_name.clone()[..]].concat());

			// Check that the caller is the owner of the cluster
			let cluster = <Clusters<T>>::get(&cluster_id).ok_or(Error::<T>::ClusterNotFound)?;
			ensure!(who == cluster.owner, Error::<T>::NoPermission);

			// At creation there are no Members and no Rules assigned to a Role
			let role = Role { name: role_name.clone(), members: vec![], rules: None };

			// Check that the role does not exists already in the cluster
			let roles_in_cluster = <Clusters<T>>::get(&cluster_id)
				.ok_or(Error::<T>::ClusterNotFound)?
				.roles
				.into_iter()
				.filter(|role| role.name == role_name)
				.collect::<Vec<Role<T::AccountId>>>();
			ensure!(roles_in_cluster.is_empty(), Error::<T>::RoleExists);

			// write
			<Clusters<T>>::mutate(&cluster_id, |cluster| {
				let cluster = cluster.as_mut().unwrap();
				cluster.roles.push(role.clone());
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
			role_name: Vec<u8>,
			cluster_id: Hash128,
			new_members_list: Vec<T::AccountId>,
			new_settings: RoleSetting,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			if new_members_list.is_empty()
				&& (new_settings.data.is_empty() || new_settings.name.is_empty())
			{
				return Err(Error::<T>::InvalidInputs.into());
			}

			// Check that the caller is the owner of the cluster
			let cluster = <Clusters<T>>::get(&cluster_id).ok_or(Error::<T>::SystematicFailure)?;
			ensure!(who == cluster.owner, Error::<T>::NoPermission);

			// Check that the role exists in the cluster and in storage
			let roles_in_cluster = <Clusters<T>>::get(&cluster_id)
				.ok_or(Error::<T>::ClusterNotFound)?
				.roles
				.into_iter()
				.filter(|role| role.name == role_name)
				.collect::<Vec<Role<T::AccountId>>>();
			ensure!(!roles_in_cluster.is_empty(), Error::<T>::RoleNotFound);

			let role_hash = blake2_128(&[&cluster_id[..], &role_name.clone()].concat());

			// write
			<Clusters<T>>::mutate(&cluster_id, |cluster| {
				let cluster = cluster.as_mut().unwrap();
				let index = cluster.clone().roles.iter().position(|x| x.name == role_name);
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
			role_name: Vec<u8>,
			cluster_id: Hash128,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// only the owner of the cluster can do this operation
			let cluster = <Clusters<T>>::get(&cluster_id).ok_or(Error::<T>::ClusterNotFound)?;
			ensure!(who == cluster.owner, Error::<T>::NoPermission);

			// Check that the role exists in the cluster and in storage
			let roles_in_cluster = cluster
				.roles
				.into_iter()
				.filter(|role| role.name == role_name)
				.collect::<Vec<Role<T::AccountId>>>();
			ensure!(!roles_in_cluster.is_empty(), Error::<T>::RoleNotFound);

			// write
			// Remove Role from Cluster
			<Clusters<T>>::mutate(&cluster_id, |cluster| {
				let cluster = cluster.as_mut().unwrap();
				let index = cluster.roles.iter().position(|x| x.name == role_name);
				if let Some(index) = index {
					cluster.roles.remove(index);
				}
			});

			let role_hash = blake2_128(&[&cluster_id[..], &role_name.clone()].concat());
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

			// Check that the roles for the member already exists in the cluster
			let roles_in_cluster: Vec<Vec<u8>> =
				cluster.roles.iter().map(|role| role.name.clone()).collect();
			for role in &roles {
				ensure!(roles_in_cluster.contains(&role), Error::<T>::RoleNotFound);
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

		/// Allow another Account ID `proxy_account` to be used as a proxy for the Account ID of the cluster
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn add_proxy_account(
			origin: OriginFor<T>,
			cluster_id: Hash128,
			proxy_account: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Check that the caller is the owner of the cluster
			let cluster = <Clusters<T>>::get(&cluster_id).ok_or(Error::<T>::ClusterNotFound)?;
			ensure!(who == cluster.owner, Error::<T>::NoPermission);

			ensure!(
				!pallet_proxy::Proxies::<T>::contains_key(&proxy_account),
				Error::<T>::AccountProxyAlreadyExists
			);

			let cluster_account = Self::get_vault_id(cluster_id);

			let proxy_def = pallet_proxy::ProxyDefinition {
				delegate: cluster_account.clone(),
				proxy_type: T::ProxyType::default(),
				delay: T::BlockNumber::default(),
			};
			let bounded_proxies: BoundedVec<_, T::MaxProxies> =
				vec![proxy_def].try_into().map_err(|_| Error::<T>::TooManyProxies)?;

			// ! Writing state

			let deposit = T::ProxyDepositBase::get() + T::ProxyDepositFactor::get();
			<T as pallet_proxy::Config>::Currency::reserve(&cluster_account, deposit.clone())?;

			pallet_proxy::Proxies::<T>::insert(&proxy_account, (bounded_proxies, deposit.clone()));

			Self::deposit_event(Event::ProxyAccountAdded { cluster_account, proxy_account });

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// **Get** the **Account ID** of the Cluster specified by its `cluster_hash`**.
		/// This Account ID is deterministically computed using the `cluster_hash`
		pub fn get_vault_id(cluster_hash: Hash128) -> T::AccountId {
			let hash = blake2_256(&[&b"cluster-vault"[..], &cluster_hash].concat());
			T::AccountId::decode(&mut &hash[..]).expect("T::AccountId should decode")
		}
	}
}
