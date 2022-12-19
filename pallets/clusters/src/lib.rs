//! This pallet `clusters` performs logic related to **Clusters**.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate core;

use codec::{Decode, Encode};
use frame_support::BoundedVec;
pub use pallet::*;
use sp_clamor::Hash128;
use sp_std::{vec, vec::Vec, collections::btree_map::BTreeMap};

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
pub struct Role {
	/// Name of the role
	pub name: Vec<u8>,
	/// The optional list of Rules associated to the Role
	pub rules: Option<Rule>,
	/// The settings of the Role
	pub settings: Vec<RoleSetting>,
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
	/// The map that contains the list of Roles in Cluster, each associated with an index
	pub roles: BTreeMap<u64, Role>,
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

	/// **StorageDoubleMap** that maps a (**AccountId**, **Cluster hash**) with a list of **Role** indexes.
	#[pallet::storage]
	pub type Members<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Identity, Hash128, Vec<u64>>;

	/// **StorageMap** that maps a **(Cluster hash, role name)** to an **index number**
	#[pallet::storage]
	pub type ClusterRoleKeys<T: Config> = StorageMap<_, Twox64Concat, (Hash128, Vec<u8>), u64>;

	/// **StorageValue** that **equals** the **total number of unique ClusterKeys in the blockchain**
	#[pallet::storage]
	pub type ClusterRoleKeysIndex<T: Config> = StorageValue<_, u64, ValueQuery>;

	/// Enum that indicates the different types of actions that user can do when call `edit_role`.
	#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq, Eq)]
	pub enum Action {
		/// Delete from the Role
		DELETE,
		/// Add into the Role
		ADD,
	}

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
		RoleEdited {
			cluster_hash: Hash128,
			role_name: Vec<u8>,
		},
		RoleDeleted {
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
		/// - `name`: name of the cluster.
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

			// At creation there are no roles and no members assigned to the cluster
			let cluster = Cluster {
				owner: who.clone(),
				name: name.into_inner(), // `self.0`
				cluster_id,
				roles: BTreeMap::new(),
			};

			let minimum_balance =
				<pallet_balances::Pallet<T> as fungible::Inspect<T::AccountId>>::minimum_balance();

			// write
			// create an account for the cluster
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

			let role_name = role_name.into_inner();
			ensure!(
				!<ClusterRoleKeys<T>>::contains_key((&cluster_id, &role_name)),
				Error::<T>::RoleExists
			);

			let cluster = <Clusters<T>>::get(&cluster_id).ok_or(Error::<T>::ClusterNotFound)?;

			// Check that the caller is the owner of the cluster
			ensure!(who == cluster.owner, Error::<T>::NoPermission);

			// At creation there are no Members and no Rules assigned to a Role
			let new_role =
				Role { name: role_name.clone(), rules: None, settings: settings.into_inner() };

			// write
			let next_index = <ClusterRoleKeysIndex<T>>::try_get().unwrap_or_default() + 1;
			<ClusterRoleKeys<T>>::insert((&cluster_id, &role_name), next_index);
			<ClusterRoleKeysIndex<T>>::put(next_index);

			<Clusters<T>>::mutate(&cluster_id, |cluster| {
				let cluster = cluster.as_mut().expect("Should find the cluster");
				cluster.roles.insert(next_index, new_role);
			});

			Self::deposit_event(Event::RoleCreated {
				cluster_hash: cluster_id,
				role_name: role_name.clone(),
			});
			log::trace!("Role {:?} created and associated to cluster {:?}", &role_name, cluster_id);

			Ok(())
		}

		/// Edit a **Role**.
		///
		/// Adds new members to the role (BoundedVec limited to T::MembersList).
		/// Adds new settings to the role (BoundedVec limited to T::RoleSettingsLimit).
		///
		/// - `role_name`: name of the role to edit.
		/// - `cluster_id`: hash of the cluster that the role belongs to.
		/// - `action`: DELETE or ADD
		/// - `members`: new list of members to be added to the existing list.
		/// - `settings`: new list of settings to be added to the existing list.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn edit_role(
			origin: OriginFor<T>,
			role_name: BoundedVec<u8, T::NameLimit>,
			cluster_id: Hash128,
			action: Action,
			members: BoundedVec<T::AccountId, T::MembersLimit>,
			settings: BoundedVec<RoleSetting, T::RoleSettingsLimit>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(!role_name.len().is_zero(), Error::<T>::InvalidInput);

			let cluster = <Clusters<T>>::get(&cluster_id).ok_or(Error::<T>::SystematicFailure)?;
			// Check that the caller is the owner of the cluster;
			ensure!(who == cluster.owner, Error::<T>::NoPermission);

			// Either the list of members or the list of new settings must have values.
			if members.len().is_zero() && (settings.len().is_zero()) {
				return Err(Error::<T>::InvalidInput.into());
			}

			let role_name = role_name.into_inner();

			ensure!(
				<ClusterRoleKeys<T>>::contains_key((&cluster_id, &role_name)),
				Error::<T>::RoleNotFound
			);

			let role_index = <ClusterRoleKeys<T>>::get((&cluster_id, &role_name))
				.ok_or(Error::<T>::SystematicFailure)?;

			// write
			match action {
				Action::ADD => {
					<Clusters<T>>::mutate(&cluster_id, |cluster| {
						let cluster = cluster.as_mut().expect("Should find the cluster");
						let role = cluster
							.roles
							.get_mut(&role_index)
							.expect("Should find the role")
							.settings
							.extend(settings.into_inner());
					});
					for member in members {
						// new members have no roles
						<Members<T>>::insert(member, cluster_id, vec![role_index]);
					}
				},
				Action::DELETE => {
					<Clusters<T>>::mutate(&cluster_id, |cluster| {
						let cluster = cluster.as_mut().expect("Should find the cluster");
						let role_settings = &mut cluster
							.roles
							.get_mut(&role_index)
							.expect("Should find the role")
							.settings;
						for setting in settings.into_inner() {
							role_settings
								.retain(|role_setting| !role_setting.name.eq(&setting.name));
						}
					});
					for member in members {
						<Members<T>>::insert(member, cluster_id, vec![role_index]);
					}
				},
			}

			Self::deposit_event(Event::RoleEdited {
				cluster_hash: cluster_id,
				role_name: role_name.clone(),
			});
			log::trace!("Role edited: {:?}", &role_name);

			Ok(())
		}

		/// Delete a **Role**.
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

			// Check that the role exists in the cluster
			ensure!(
				<ClusterRoleKeys<T>>::contains_key((&cluster_id, &role_name)),
				Error::<T>::RoleNotFound
			);

			// write
			// Remove Role from Cluster
			let role_index = <ClusterRoleKeys<T>>::get((&cluster_id, &role_name))
				.ok_or(Error::<T>::SystematicFailure)?;

			<Clusters<T>>::mutate(&cluster_id, |cluster| {
				if let Some(cluster) = cluster {
					cluster.roles.remove(&role_index);
				}
			});

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
			ensure!(
				!<Members<T>>::contains_key(&member, &cluster_id),
				Error::<T>::MemberExists
			);

			for role in &roles_names {
				// Check that the roles actually exist in the cluster
				ensure!(
					<ClusterRoleKeys<T>>::contains_key((&cluster_id, &role)),
					Error::<T>::RoleNotFound
				);
			}

			// write
			let mut role_indexes = Vec::new();
			for role_name in roles_names {
				let index = <ClusterRoleKeys<T>>::get((&cluster_id, &role_name));
				if let Some(index) = index {
					role_indexes.push(index);
				} else {
					let next_index = <ClusterRoleKeysIndex<T>>::try_get().unwrap_or_default() + 1;
					<ClusterRoleKeys<T>>::insert((&cluster_id, &role_name), next_index);
					<ClusterRoleKeysIndex<T>>::put(next_index);
					role_indexes.push(next_index);
				}
			}

			<Members<T>>::insert(member, cluster_id, role_indexes);

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
			ensure!(
				<Members<T>>::contains_key(&member, &cluster_id),
				Error::<T>::MemberExists
			);

			// write
			// Delete member from Cluster
			<Members<T>>::remove(member, cluster_id);

			Ok(())
		}
	}
}
