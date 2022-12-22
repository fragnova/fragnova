//! This pallet `clusters` performs logic related to **Clusters**.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate core;

use codec::{Compact, Decode, Encode};
use frame_support::BoundedVec;
pub use pallet::*;
use sp_clamor::Hash128;
use sp_std::{collections::btree_map::BTreeMap, vec, vec::Vec};

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
	pub name: Compact<u64>,
	/// The data associated with the Role
	pub data: Vec<u8>,
}

/// **Struct** of **Role** belonging to a **Cluster**.
#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub struct Role {
	/// Name of the role
	pub name: Compact<u64>,
	/// The settings of the Role
	pub settings: Vec<RoleSetting>,
}

/// **Struct** of a **Cluster**
#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub struct Cluster<TAccountId> {
	/// The owner of the Cluster
	pub owner: TAccountId,
	/// The ID of the Cluster
	pub name: Compact<u64>,
	/// The ID of the cluster
	pub cluster_id: Hash128,
	/// The map that contains the list of Role IDs belonging to the in Cluster
	pub roles: Vec<Compact<u64>>,
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
	use sp_io::hashing::blake2_128;

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
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The max size of name
		#[pallet::constant]
		type NameLimit: Get<u32>;

		/// The max size of data
		#[pallet::constant]
		type DataLimit: Get<u32>;

		/// The max number of members
		#[pallet::constant]
		type MembersLimit: Get<u32>;

		/// The max number of role settings
		#[pallet::constant]
		type RoleSettingsLimit: Get<u32>;
	}

	/// **StorageMap** that maps a **Cluster** ID to its data.
	#[pallet::storage]
	pub type Clusters<T: Config> = StorageMap<_, Identity, Hash128, Cluster<T::AccountId>>;

	/// **StorageMap** that maps a **AccountId** with a list of **Cluster** owned by the account.
	#[pallet::storage]
	pub type ClustersByOwner<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Vec<Hash128>>;

	/// **StorageDoubleMap** that maps a (**Cluster hash**, **AccountId**) with a list of **Role** indexes.
	#[pallet::storage]
	pub type Members<T: Config> =
		StorageDoubleMap<_, Identity, Hash128, Twox64Concat, T::AccountId, Vec<Compact<u64>>>;

	/// **StorageDoubleMap** that maps a (**Cluster hash**, **Role ID**) with a **Role**.
	#[pallet::storage]
	pub type Roles<T: Config> =
		StorageDoubleMap<_, Identity, Hash128, Twox64Concat, Compact<u64>, Role>;

	/// **StorageMap** that maps a **Name (of type `Vec<u8>`)** to an **index**.
	/// This ensures no duplicated names are used.
	#[pallet::storage]
	pub type Names<T: Config> = StorageMap<_, Twox64Concat, Vec<u8>, Compact<u64>>;

	/// **StorageValue** that **equals** the **total number of unique names** used for Roles and Clusters
	/// in the chain.
	#[pallet::storage]
	pub type NamesIndex<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[allow(missing_docs)]
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ClusterCreated {
			cluster_hash: Hash128,
		},
		RoleCreated {
			cluster_hash: Hash128,
			role_name: Vec<u8>,
		},
		RoleSettingsEdited {
			cluster_hash: Hash128,
			role_name: Vec<u8>,
		},
		RoleDeleted {
			cluster_hash: Hash128,
			role_name: Vec<u8>,
		},
		RoleMemberEdited {
			cluster_hash: Hash128,
			role_name: Vec<u8>,
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
		/// RoleSettings not found
		RoleSettingsNotFound,
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
		ClusterMembersLimitReached,
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
		/// Create a new **Cluster**.
		///
		/// - `name`: name of the cluster (BoundedVec limited to T::NameLimit).
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_cluster(
			origin: OriginFor<T>,
			name: BoundedVec<u8, T::NameLimit>,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			ensure!(!name.len().is_zero(), Error::<T>::InvalidInput);

			// as_be_bytes to get bytes without allocations
			let extrinsic_index = <frame_system::Pallet<T>>::extrinsic_index()
				.ok_or(Error::<T>::SystematicFailure)?
				.to_be_bytes();
			// BlockNumber type is non concrete, so we need to encode it to get bytes
			let current_block_number = <frame_system::Pallet<T>>::block_number().encode();
			// Who is not concrete, so we need to encode it to get bytes
			let who_bytes = who.encode();

			let cluster_id = blake2_128(
				&[&current_block_number, name.as_slice(), &extrinsic_index, &who_bytes].concat(),
			);

			// Check that the cluster does not exist already
			ensure!(!<Clusters<T>>::contains_key(&cluster_id), Error::<T>::ClusterExists);

			let cluster_name = name.into_inner();
			let name_index = Self::take_name_index(&cluster_name);

			// At creation there are no roles and no members assigned to the cluster
			let cluster =
				Cluster { owner: who.clone(), name: name_index, cluster_id, roles: Vec::new() };

			let minimum_balance =
				<pallet_balances::Pallet<T> as fungible::Inspect<T::AccountId>>::minimum_balance();

			// write
			// create an account for the cluster, so that the cluster will be able to receive funds.
			let vault = get_vault_id(cluster_id);
			<pallet_balances::Pallet<T> as fungible::Mutate<T::AccountId>>::mint_into(
				&vault,
				minimum_balance,
			)?;

			// create a proxy
			let proxy_def = pallet_proxy::ProxyDefinition {
				delegate: who.clone(),
				proxy_type: T::ProxyType::default(),
				delay: T::BlockNumber::default(),
			};

			let bounded_proxies: BoundedVec<_, T::MaxProxies> =
				vec![proxy_def].try_into().map_err(|_| Error::<T>::TooManyProxies)?;

			// ! Writing state

			let deposit = T::ProxyDepositBase::get() + T::ProxyDepositFactor::get();
			<T as pallet_proxy::Config>::Currency::reserve(&who, deposit)?;

			pallet_proxy::Proxies::<T>::insert(&vault, (bounded_proxies, deposit));

			<Clusters<T>>::insert(cluster_id, cluster);
			<ClustersByOwner<T>>::append(who, &cluster_id);

			Self::deposit_event(Event::ClusterCreated { cluster_hash: cluster_id });
			log::trace!("Cluster created: {:?}", hex::encode(&cluster_id));

			Ok(())
		}

		/// Create a **Role** and assign it to an existing **Cluster**.
		///
		/// - `cluster_id`: hash of the cluster
		/// - `role_name`: name of the role to add into the cluster (BoundedVec limited to T::NameLimit).
		/// - `settings`: settings of the role (BoundedVec limited to T::RoleSettingsLimit).
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_role(
			origin: OriginFor<T>,
			cluster_id: Hash128,
			role_name: BoundedVec<u8, T::NameLimit>,
			settings: BoundedVec<RoleSetting, T::RoleSettingsLimit>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(!cluster_id.len().is_zero(), Error::<T>::InvalidInput);
			ensure!(!role_name.len().is_zero(), Error::<T>::InvalidInput);

			let cluster = <Clusters<T>>::get(&cluster_id).ok_or(Error::<T>::ClusterNotFound)?;
			// Check that the caller is the owner of the cluster
			ensure!(who == cluster.owner, Error::<T>::NoPermission);

			let role_name = role_name.into_inner();
			let name_index = Self::take_name_index(&role_name);

			ensure!(!<Roles<T>>::contains_key(&cluster_id, &name_index), Error::<T>::RoleExists);

			// At creation there are no Members assigned to a Role
			let new_role = Role { name: name_index.clone(), settings: settings.into_inner() };

			// write
			<Clusters<T>>::mutate(&cluster_id, |cluster| {
				let cluster = cluster.as_mut().expect("Should find the cluster");
				cluster.roles.push(name_index.clone());
			});

			<Roles<T>>::insert(cluster_id, name_index, new_role);

			Self::deposit_event(Event::RoleCreated {
				cluster_hash: cluster_id,
				role_name: role_name.clone(),
			});
			log::trace!("Role {:?} created and associated to cluster {:?}", &role_name, cluster_id);

			Ok(())
		}

		/// Associate a new list of **AccountId** to a **Role** in a cluster.
		///
		/// - `role_name`: name of the role to edit (BoundedVec limited to T::NameLimit).
		/// - `cluster_id`: hash of the cluster that the role belongs to.
		/// - `members`: new list of members to be added to the existing list.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn add_role_members(
			origin: OriginFor<T>,
			role_name: BoundedVec<u8, T::NameLimit>,
			cluster_id: Hash128,
			members: BoundedVec<T::AccountId, T::MembersLimit>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(!role_name.len().is_zero(), Error::<T>::InvalidInput);

			let cluster = <Clusters<T>>::get(&cluster_id).ok_or(Error::<T>::SystematicFailure)?;
			// Check that the caller is the owner of the cluster;
			ensure!(who == cluster.owner, Error::<T>::NoPermission);

			// Either the list of members or the list of new settings must have values.
			if members.len().is_zero() {
				return Err(Error::<T>::InvalidInput.into());
			}

			let role_name = role_name.into_inner();
			let name_index = Self::take_name_index(&role_name);

			ensure!(<Roles<T>>::contains_key(&cluster_id, &name_index), Error::<T>::RoleNotFound);

			// write
			for member in members {
				<Members<T>>::insert(cluster_id, member, vec![name_index]);
			}

			Self::deposit_event(Event::RoleMemberEdited {
				cluster_hash: cluster_id,
				role_name: role_name.clone(),
			});
			log::trace!("New members added to the role: {:?}", &role_name);

			Ok(())
		}

		/// Remove the **Role** from a **Member**.
		///
		/// - `role_name`: name of the role to edit (BoundedVec limited to T::NameLimit).
		/// - `cluster_id`: hash of the cluster that the role belongs to.
		/// - `members`: new list of members to be deleted (BoundedVec limited to T::MembersLimit).
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn delete_role_members(
			origin: OriginFor<T>,
			role_name: BoundedVec<u8, T::NameLimit>,
			cluster_id: Hash128,
			members: BoundedVec<T::AccountId, T::MembersLimit>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(!role_name.len().is_zero(), Error::<T>::InvalidInput);

			let cluster = <Clusters<T>>::get(&cluster_id).ok_or(Error::<T>::SystematicFailure)?;
			// Check that the caller is the owner of the cluster;
			ensure!(who == cluster.owner, Error::<T>::NoPermission);

			// Either the list of members or the list of new settings must have values.
			if members.len().is_zero() {
				return Err(Error::<T>::InvalidInput.into());
			}

			let role_name = role_name.into_inner();
			let name_index = Self::take_name_index(&role_name);

			ensure!(<Roles<T>>::contains_key(&cluster_id, &name_index), Error::<T>::RoleNotFound);

			// write
			for member in members {
				<Members<T>>::remove(&cluster_id, &member);
				log::trace!("Member {:?} deleted from role {:?}", &member, &role_name);
			}

			Self::deposit_event(Event::RoleMemberEdited {
				cluster_hash: cluster_id,
				role_name: role_name.clone(),
			});

			Ok(())
		}

		/// Add a list of **RoleSetting** into an existing **Role** in a Cluster
		///
		/// - `role_name`: name of the role to edit (BoundedVec limited to T::NameLimit).
		/// - `cluster_id`: hash of the cluster that the role belongs to.
		/// - `settings`: new list of settings to be added to the existing list (BoundedVec limited to T::RoleSettingsLimit).
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn add_role_settings(
			origin: OriginFor<T>,
			role_name: BoundedVec<u8, T::NameLimit>,
			cluster_id: Hash128,
			settings: BoundedVec<RoleSetting, T::RoleSettingsLimit>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(!role_name.len().is_zero(), Error::<T>::InvalidInput);

			let cluster = <Clusters<T>>::get(&cluster_id).ok_or(Error::<T>::SystematicFailure)?;
			// Check that the caller is the owner of the cluster;
			ensure!(who == cluster.owner, Error::<T>::NoPermission);

			let role_name = role_name.into_inner();
			let name_index = Self::take_name_index(&role_name);

			ensure!(<Roles<T>>::contains_key(&cluster_id, &name_index), Error::<T>::RoleNotFound);

			// write
			<Roles<T>>::mutate(&cluster_id, &name_index, |role| {
				let role = role.as_mut().expect("Should find the role");
				role.settings.extend(settings.into_inner());
			});

			Self::deposit_event(Event::RoleSettingsEdited {
				cluster_hash: cluster_id,
				role_name: role_name.clone(),
			});
			log::trace!("New role settings added into the role {:?}", &role_name);

			Ok(())
		}

		/// Delete a list of **RoleSettings** from a **Role** in a Cluster.
		///
		/// - `role_name`: name of the role to edit (BoundedVec limited to T::NameLimit).
		/// - `cluster_id`: hash of the cluster that the role belongs to.
		/// - `settings`: new list of settings to be added to the existing list (BoundedVec limited to T::RoleSettingsLimit).
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn delete_role_settings(
			origin: OriginFor<T>,
			role_name: BoundedVec<u8, T::NameLimit>,
			cluster_id: Hash128,
			settings: BoundedVec<Vec<u8>, T::RoleSettingsLimit>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(!role_name.len().is_zero(), Error::<T>::InvalidInput);

			let cluster = <Clusters<T>>::get(&cluster_id).ok_or(Error::<T>::SystematicFailure)?;
			// Check that the caller is the owner of the cluster;
			ensure!(who == cluster.owner, Error::<T>::NoPermission);

			let role_name = role_name.into_inner();
			let name_index = Self::take_name_index(&role_name);

			let mut settings_name_indexes = Vec::new();
			for setting_name in settings.into_inner() {
				settings_name_indexes.push(Self::take_name_index(&setting_name));
			}

			ensure!(<Roles<T>>::contains_key(&cluster_id, &name_index), Error::<T>::RoleNotFound);

			// write
			<Roles<T>>::mutate(&cluster_id, &name_index, |role| {
				let role_settings = &mut role.as_mut().expect("Should find the role").settings;
				for setting_name in settings_name_indexes {
					role_settings.retain(|role_setting| !role_setting.name.eq(&setting_name));
				}
			});

			Self::deposit_event(Event::RoleSettingsEdited {
				cluster_hash: cluster_id,
				role_name: role_name.clone(),
			});
			log::trace!("Role edited: {:?}", &role_name);

			Ok(())
		}

		/// Delete a **Role** from a Cluster.
		///
		/// - `role_name`: name of the role to delete.
		/// - `cluster_id`: hash of the cluster that the role belongs to.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn delete_role(
			origin: OriginFor<T>,
			role_name: BoundedVec<u8, T::NameLimit>,
			cluster_id: Hash128,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(!role_name.len().is_zero(), Error::<T>::InvalidInput);

			// only the owner of the cluster can do this operation
			let cluster = <Clusters<T>>::get(&cluster_id).ok_or(Error::<T>::ClusterNotFound)?;
			ensure!(who == cluster.owner, Error::<T>::NoPermission);

			let role_name = role_name.into_inner();
			let name_index = Self::take_name_index(&role_name);

			ensure!(<Roles<T>>::contains_key(&cluster_id, &name_index), Error::<T>::RoleNotFound);

			// write
			// Remove Role from Cluster
			<Clusters<T>>::mutate(&cluster_id, |cluster| {
				let roles = &mut cluster.as_mut().expect("should find cluster").roles;
				roles.retain(|x| !x.eq(&name_index));
			});

			<Roles<T>>::remove(&cluster_id, &name_index);

			Self::deposit_event(Event::RoleDeleted {
				cluster_hash: cluster_id,
				role_name: role_name.clone(),
			});
			log::trace!("Role deleted: {:?}", &role_name);

			Ok(())
		}

		/// Add a **Member** into a cluster.
		///
		/// It also assigns a list of **Role** to the new member.
		///
		/// - `cluster_id`: hash of the cluster where to add the new member.
		/// - `roles`: list of role names to be assigned to the new member.
		/// - `member`: AccountId of the new member.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn add_member(
			origin: OriginFor<T>,
			cluster_id: Hash128,
			roles_names: Vec<Vec<u8>>,
			member: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Check that the cluster exists and the caller is the owner of the cluster
			let cluster = <Clusters<T>>::get(cluster_id).ok_or(Error::<T>::ClusterNotFound)?;
			ensure!(who == cluster.owner, Error::<T>::NoPermission);

			// Check that the cluster does not already contain the member
			ensure!(!<Members<T>>::contains_key(&cluster_id, &member), Error::<T>::MemberExists);

			for role in &roles_names {
				// Check that the roles actually exist in the cluster
				let name_index = Self::take_name_index(role);
				ensure!(
					<Roles<T>>::contains_key(&cluster_id, &name_index),
					Error::<T>::RoleNotFound
				);
			}

			// write
			let mut role_indexes = Vec::new();
			for role_name in roles_names {
				role_indexes.push(Self::take_name_index(&role_name));
			}

			<Members<T>>::insert(cluster_id, member, role_indexes);

			Ok(())
		}

		/// Delete a **Member** from a **Cluster**.
		///
		/// - `cluster_id`: hash of the cluster.
		/// - `member`: AccountId of the member to be removed.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn delete_member(
			origin: OriginFor<T>,
			cluster_id: Hash128,
			member: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let cluster = <Clusters<T>>::get(cluster_id).ok_or(Error::<T>::ClusterNotFound)?;
			ensure!(who == cluster.owner, Error::<T>::NoPermission);
			// Check that the member is in the cluster
			ensure!(<Members<T>>::contains_key(&cluster_id, &member), Error::<T>::MemberExists);

			// write
			// Delete member from Cluster
			<Members<T>>::remove(cluster_id, member);

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
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
	}
}
