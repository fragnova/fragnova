//! A Cluster in Fragnova represents a "group" where users (i.e. Accounts) can belong to.
//!
//! A Cluster has an owner, a name and a set of Roles. The owner of the cluster can create unlimited roles and associate users to each role.
//!
//! A Role has a name, some settings and a set of members (i.e. Accounts) associated to it. A user can be associated with multiple roles.
//!
//! A Cluster can be also associated to a **Proto** during its **upload**.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate core;

use codec::{Compact, Decode, Encode};
use frame_support::{transactional, BoundedVec};
pub use pallet::*;
use sp_fragnova::Hash128;
use sp_std::{vec, vec::Vec};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod dummy_data;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

/// **Struct** of a **Cluster**
#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub struct Cluster<TAccountId> {
	/// The owner of the Cluster
	pub owner: TAccountId,
	/// The ID of the Cluster
	pub name: Compact<u64>,
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
	use frame_support::{
		log,
		pallet_prelude::*,
		sp_runtime::traits::Zero,
		traits::{fungible, ReservableCurrency},
	};
	use frame_system::pallet_prelude::*;
	use sp_fragnova::get_vault_id;
	use sp_io::{hashing::blake2_128};

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
	}

	/// **StorageMap** that maps a **Cluster** ID to its data.
	#[pallet::storage]
	pub type Clusters<T: Config> = StorageMap<_, Identity, Hash128, Cluster<T::AccountId>>;

	/// **StorageMap** that maps a **AccountId** with a list of **Cluster** owned by the account.
	#[pallet::storage]
	pub type ClustersByOwner<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Vec<Hash128>>;

	#[pallet::storage]
	pub type RoleMembers<T: Config> = StorageNMap<
		_,
		(
			// Cluster Hash
			storage::Key<Identity, Hash128>,
			// Role name index
			storage::Key<Twox64Concat, Compact<u64>>,
			// Account ID
			storage::Key<Twox64Concat, T::AccountId>,
		),
		T::BlockNumber, // since when the member is part of the role
	>;

	#[pallet::storage]
	pub type RoleSettings<T: Config> = StorageNMap<
		_,
		(
			// Cluster Hash
			storage::Key<Identity, Hash128>,
			// Role name index
			storage::Key<Twox64Concat, Compact<u64>>,
			// Setting name index
			storage::Key<Twox64Concat, Compact<u64>>,
		),
		BoundedVec<u8, T::DataLimit>,
	>;

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
		/// RoleSetting already exists
		RoleSettingsExists,
		// RoleSetting not found
		RoleSettingNotFound,
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
		#[transactional] // not ideal but need to refactor `take_name_index` if we want to remove it
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
			let cluster = Cluster { owner: who.clone(), name: name_index };

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

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))] // TODO BENCHMARKS as we allow "infinite" members and settings to be added
		#[transactional] // not ideal but need to refactor `take_name_index` if we want to remove it
		pub fn emplace_role(
			origin: OriginFor<T>,
			cluster_id: Hash128,
			role_name: BoundedVec<u8, T::NameLimit>,
			settings: Vec<(BoundedVec<u8, T::NameLimit>, BoundedVec<u8, T::DataLimit>)>,
			members: Vec<T::AccountId>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(!cluster_id.len().is_zero(), Error::<T>::InvalidInput);
			ensure!(!role_name.len().is_zero(), Error::<T>::InvalidInput);

			let cluster = <Clusters<T>>::get(&cluster_id).ok_or(Error::<T>::ClusterNotFound)?;
			// Check that the caller is the owner of the cluster
			ensure!(who == cluster.owner, Error::<T>::NoPermission);

			let role_name = role_name.into_inner();
			let name_index = Self::take_name_index(&role_name);

			// this is a storage hack but we add/replce a 0 as setting name index to the role anyway just in case
			// in this call settings and members are both len == 0
			<RoleSettings<T>>::insert(
				(&cluster_id, &name_index, &Compact(0)),
				BoundedVec::default(),
			);

			for setting in &settings {
				let setting_index = Self::take_name_index(&setting.0);
				<RoleSettings<T>>::insert((&cluster_id, &name_index, &setting_index), &setting.1);
			}

			let current_block_number = <frame_system::Pallet<T>>::block_number();

			for member in &members {
				<RoleMembers<T>>::insert((&cluster_id, &name_index, &member), current_block_number);
			}

			Self::deposit_event(Event::RoleCreated {
				cluster_hash: cluster_id,
				role_name: role_name.clone(),
			});

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))] // TODO BENCHMARKS as we allow "infinite" members and settings to be removed
		#[transactional] // not ideal but need to refactor `take_name_index` if we want to remove it
		pub fn remove_members(
			origin: OriginFor<T>,
			cluster_id: Hash128,
			role_name: BoundedVec<u8, T::NameLimit>,
			members: Vec<T::AccountId>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(!cluster_id.len().is_zero(), Error::<T>::InvalidInput);
			ensure!(!role_name.len().is_zero(), Error::<T>::InvalidInput);

			let cluster = <Clusters<T>>::get(&cluster_id).ok_or(Error::<T>::ClusterNotFound)?;
			// Check that the caller is the owner of the cluster
			ensure!(who == cluster.owner, Error::<T>::NoPermission);

			let role_name = role_name.into_inner();
			let name_index = Self::take_name_index(&role_name);

			// ensure that the special 0 key is set to the role aka role exists
			ensure!(
				<RoleSettings<T>>::contains_key((&cluster_id, &name_index, &Compact(0))),
				Error::<T>::RoleNotFound
			);

			for member in &members {
				<RoleMembers<T>>::remove((&cluster_id, &name_index, &member));
			}

			Self::deposit_event(Event::RoleCreated {
				cluster_hash: cluster_id,
				role_name: role_name.clone(),
			});

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))] // TODO BENCHMARKS as we allow "infinite" removals
		#[transactional] // not ideal but need to refactor `take_name_index` if we want to remove it
		pub fn remove_settings(
			origin: OriginFor<T>,
			cluster_id: Hash128,
			role_name: BoundedVec<u8, T::NameLimit>,
			settings: Vec<BoundedVec<u8, T::NameLimit>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(!cluster_id.len().is_zero(), Error::<T>::InvalidInput);
			ensure!(!role_name.len().is_zero(), Error::<T>::InvalidInput);

			let cluster = <Clusters<T>>::get(&cluster_id).ok_or(Error::<T>::ClusterNotFound)?;
			// Check that the caller is the owner of the cluster
			ensure!(who == cluster.owner, Error::<T>::NoPermission);

			let role_name = role_name.into_inner();
			let name_index = Self::take_name_index(&role_name);

			// ensure that the special 0 key is set to the role aka role exists
			ensure!(
				<RoleSettings<T>>::contains_key((&cluster_id, &name_index, &Compact(0))),
				Error::<T>::RoleNotFound
			);

			for setting in &settings {
				let setting_index = Self::take_name_index(&setting);
				<RoleSettings<T>>::remove((&cluster_id, &name_index, &setting_index));
			}

			Self::deposit_event(Event::RoleCreated {
				cluster_hash: cluster_id,
				role_name: role_name.clone(),
			});

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))] // TODO BENCHMARKS as we allow "infinite" removals
		#[transactional] // not ideal but need to refactor `take_name_index` if we want to remove it
		pub fn kill_role(
			origin: OriginFor<T>,
			cluster_id: Hash128,
			role_name: BoundedVec<u8, T::NameLimit>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(!cluster_id.len().is_zero(), Error::<T>::InvalidInput);
			ensure!(!role_name.len().is_zero(), Error::<T>::InvalidInput);

			let cluster = <Clusters<T>>::get(&cluster_id).ok_or(Error::<T>::ClusterNotFound)?;
			// Check that the caller is the owner of the cluster
			ensure!(who == cluster.owner, Error::<T>::NoPermission);

			let role_name = role_name.into_inner();
			let name_index = Self::take_name_index(&role_name);

			// ensure that the special 0 key is set to the role aka role exists
			ensure!(
				<RoleSettings<T>>::contains_key((&cluster_id, &name_index, &Compact(0))),
				Error::<T>::RoleNotFound
			);

			let _ = <RoleSettings<T>>::clear_prefix((&cluster_id, &name_index), u32::MAX, None);
			let _ = <RoleMembers<T>>::clear_prefix((&cluster_id, &name_index), u32::MAX, None);

			Self::deposit_event(Event::RoleDeleted {
				cluster_hash: cluster_id,
				role_name: role_name.clone(),
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
	}
}
