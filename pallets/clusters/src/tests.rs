#![cfg(test)]

use crate as pallet_clusters;
use crate::{dummy_data::*, mock, mock::*, *};

use crate::Event as ClusterEvent;
use create_tests::{create_cluster_, create_role_, edit_role_};
use frame_support::{assert_ok, dispatch::DispatchResult};
use sp_io::hashing::{blake2_256, blake2_128};

mod create_tests {
	use super::*;
	use crate::dummy_data::DummyData;
	use frame_benchmarking::account;
	use frame_support::{assert_noop, ensure, traits::fungible};

	pub fn create_cluster_(
		signer: <Test as frame_system::Config>::AccountId,
		name: Vec<u8>,
	) -> DispatchResult {
		ClustersPallet::create_cluster(RuntimeOrigin::signed(signer), name)
	}

	pub fn create_role_(
		signer: <Test as frame_system::Config>::AccountId,
		cluster: Hash128,
		role: Vec<u8>,
		settings: RoleSetting<<Test as Config>::NameLimit, <Test as Config>::DataLimit>,
	) -> DispatchResult {
		ClustersPallet::create_role(
			RuntimeOrigin::signed(signer),
			cluster.clone(),
			role.clone(),
			settings.clone(),
		)
	}

	pub fn delete_role_(
		signer: <Test as frame_system::Config>::AccountId,
		role_name: Vec<u8>,
		cluster_id: Hash128,
	) -> DispatchResult {
		ClustersPallet::delete_role(
			RuntimeOrigin::signed(signer),
			role_name.clone(),
			cluster_id.clone(),
		)
	}

	pub fn add_member_(
		signer: <Test as frame_system::Config>::AccountId,
		cluster_id: Hash128,
		roles_names: Vec<Vec<u8>>,
		member: <Test as frame_system::Config>::AccountId,
	) -> DispatchResult {
		ClustersPallet::add_member(
			RuntimeOrigin::signed(signer),
			cluster_id.clone(),
			roles_names.clone(),
			member.clone(),
		)
	}

	pub fn edit_role_(
        signer: <Test as frame_system::Config>::AccountId,
        role_name: Vec<u8>,
        cluster_id: Hash128,
        new_members_list: Vec<<Test as frame_system::Config>::AccountId>,
        new_settings: RoleSetting<<Test as Config>::NameLimit, <Test as Config>::DataLimit>,
	) -> DispatchResult {
		ClustersPallet::edit_role(
			RuntimeOrigin::signed(signer),
			role_name.clone(),
			cluster_id.clone(),
			new_members_list.clone(),
			new_settings.clone(),
		)
	}

	#[test]
	fn create_cluster_should_work() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let account_id = dummy.account_id;
			let cluster_name = b"MyCluster".to_vec();
			assert_ok!(create_cluster_(account_id, cluster_name));

			let cluster_id = get_cluster_id(cluster_name.clone(), account_id);

			assert!(<Clusters<Test>>::contains_key(&cluster_id.clone()));

			let cluster = Cluster {
				owner: account_id,
				name: cluster_name.clone(),
				cluster_id,
				roles: vec![],
				members: vec![],
			};
			let result = <Clusters<Test>>::get(&cluster_id.clone()).unwrap();
			assert_eq!(cluster, result);

			let clusters = <ClustersByOwner<Test>>::get(account_id).unwrap();
			assert!(clusters.contains(&cluster_id));

			System::assert_has_event(
				ClusterEvent::ClusterCreated {
					cluster_hash: cluster_id.clone(),
				}
				.into(),
			);

			let minimum_balance = <Balances as fungible::Inspect<
				<Test as frame_system::Config>::AccountId,
			>>::minimum_balance();
			System::assert_has_event(
				frame_system::Event::NewAccount { account: get_vault_account_id(cluster_id) }
					.into(),
			);
			System::assert_has_event(
				pallet_balances::Event::Endowed {
					account: get_vault_account_id(cluster_id),
					free_balance: minimum_balance,
				}
				.into(),
			);
			System::assert_has_event(
				pallet_balances::Event::Deposit {
					who: get_vault_account_id(cluster_id),
					amount: minimum_balance,
				}
				.into(),
			);
		});
	}

	#[test]
	fn create_cluster_with_no_name_fails() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let account_id = dummy.account_id;
			let cluster_name = b"".to_vec();
			assert_noop!(
				create_cluster_(account_id, cluster_name.clone()),
				Error::<Test>::InvalidInputs
			);
		});
	}

	#[test]
	fn create_role_should_work() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let cluster = dummy.cluster.name;
			let role = dummy.role.name;
			let settings = dummy.role_settings;
			let account_id = dummy.account_id;

			assert_ok!(create_cluster_(account_id, cluster.clone()));

			let cluster_id = get_cluster_id(cluster, account_id);

			assert_ok!(create_role_(
				account_id.clone(),
				cluster_id.clone(),
				role,
				settings
			));

			let role_hash = blake2_128(&[&cluster_id[..], &role.clone()[..]].concat());

			let expected_role = Role { name: role.clone(), members: vec![], rules: None };

			let roles_in_cluster = <Clusters<Test>>::get(cluster_id).unwrap().roles;
			assert!(roles_in_cluster.contains(&expected_role));

			System::assert_has_event(
				ClusterEvent::RoleCreated {
					role_hash,
				}
				.into(),
			);
		});
	}

	#[test]
	fn edit_role_with_invalid_inputs_fails() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let cluster = dummy.cluster.name;
			let role = dummy.role.name;
			let settings = dummy.role_settings;
			let account_id = dummy.account_id;

			assert_ok!(create_cluster_(account_id, cluster));

			let cluster_id = get_cluster_id(cluster.clone(), account_id);

			assert_ok!(create_role_(
				account_id.clone(),
				cluster_id.clone(),
				role,
				settings
			));

			let setting_wrong = RoleSetting { name: b"Name".to_vec(),  data: b"".to_vec() };
			assert_noop!(
				edit_role_(
					account_id.clone(),
					role.clone(),
					cluster_id,
					Vec::new(),
					setting_wrong
				),
				Error::<Test>::InvalidInputs
			);
		});
	}

	#[test]
	fn edit_role_that_not_exist_fails() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let cluster = dummy.cluster.name;
			let role = dummy.role.name;
			let settings = dummy.role_settings;
			let account_id = dummy.account_id;

			assert_ok!(create_cluster_(account_id, cluster));
			let cluster_id = get_cluster_id(cluster.clone(), account_id);
			// do not create any role

			// assert there is no role
			assert_noop!(
				edit_role_(
					account_id.clone(),
					role.clone(),
					cluster_id,
					Vec::new(),
					settings,
				),
				Error::<Test>::RoleNotFound
			);
		});
	}

	#[test]
	fn delete_a_role_works() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let cluster = dummy.cluster.name;
			let role = dummy.role.name;
			let account_id = dummy.account_id;
			let settings = dummy.role_settings;

			assert_ok!(create_cluster_(account_id, cluster));
			let cluster_id = get_cluster_id(cluster.clone(), account_id);

			assert_ok!(create_role_(
				account_id.clone(),
				cluster_id.clone(),
				role,
				settings
			));

			let role_hash = get_role_hash(cluster_id.clone(), role.clone());

			assert_ok!(delete_role_(account_id.clone(), role.clone(), cluster_id.clone()));

			let roles_in_cluster = <Clusters<Test>>::get(&cluster_id).unwrap().roles;
			assert!(roles_in_cluster.is_empty());

			System::assert_has_event(
				ClusterEvent::RoleDeleted {
					role_hash,
				}
				.into(),
			);
		});
	}

	#[test]
	fn edit_role_works() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let cluster = dummy.cluster.name;
			let role = dummy.role.name;
			let settings = dummy.role_settings;
			let new_settings = dummy.role_settings_2;
			let account_id = dummy.account_id;

			// create a cluster
			assert_ok!(create_cluster_(account_id, cluster));
			let cluster_id = get_cluster_id(cluster.clone(), account_id);
			// associate the role to the cluster
			assert_ok!(create_role_(
				account_id.clone(),
				cluster_id.clone(),
				role,
				settings
			));

			assert_ok!(edit_role_(
				account_id.clone(),
				role.clone(),
				cluster_id.clone(),
				Vec::new(),
				new_settings.clone(),
			));

			let expected_role =
				Role { name: role.clone(), members: vec![], rules: None };

			let roles_in_cluster = <Clusters<Test>>::get(cluster_id).unwrap().roles;
			assert!(roles_in_cluster.contains(&expected_role));

		});
	}

	#[test]
	fn add_member_to_cluster_should_work() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let cluster = dummy.cluster.name;
			let role = dummy.role.name;
			let settings = dummy.role_settings;
			let account_id = dummy.account_id;
			let member = dummy.account_id_2;

			// create a cluster
			assert_ok!(create_cluster_(account_id, cluster));
			let cluster_id = get_cluster_id(cluster.clone(), account_id);

			// create a role for the cluster
			assert_ok!(create_role_(
				account_id.clone(),
				cluster_id.clone(),
				role.clone(),
				settings.clone()
			));

			// add member into the cluster
			assert_ok!(add_member_(
				account_id.clone(),
				cluster_id.clone(),
				vec![role.clone()],
				member.clone()
			));

			assert!(<Clusters<Test>>::get(&cluster_id)
				.unwrap()
				.members
				.contains(&member));
		});
	}
}
