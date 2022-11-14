#![cfg(test)]

use crate as pallet_clusters;
use crate::{dummy_data::*, mock, mock::*, *};

use create_tests::{create_cluster_, create_role_, edit_role_};
use frame_support::{assert_ok, dispatch::DispatchResult};
use sp_io::hashing::blake2_256;
use crate::Event as ClusterEvent;

mod create_tests {
	use super::*;
	use crate::dummy_data::DummyData;
	use frame_benchmarking::account;
	use frame_support::{assert_noop, ensure};

	pub fn create_cluster_(
		signer: <Test as frame_system::Config>::AccountId,
		name: Vec<u8>,
	) -> DispatchResult {
		ClustersPallet::create_cluster(RuntimeOrigin::signed(signer), name)
	}

	pub fn create_role_(
		signer: <Test as frame_system::Config>::AccountId,
		cluster: Vec<u8>,
		role: Vec<u8>,
		settings: RoleSettings,
	) -> DispatchResult {
		ClustersPallet::create_role(
			RuntimeOrigin::signed(signer),
			cluster.clone(),
			role.clone(),
			settings.clone(),
		)
	}

	pub fn add_member_(
		signer: <Test as frame_system::Config>::AccountId,
		cluster_name: Vec<u8>,
		role_name: Vec<Vec<u8>>,
		member_data: Vec<u8>,
	) -> DispatchResult {
		ClustersPallet::add_member(
			RuntimeOrigin::signed(signer),
			cluster_name.clone(),
			role_name.clone(),
			member_data.clone(),
		)
	}

	pub fn edit_role_(
		signer: <Test as frame_system::Config>::AccountId,
		role_hash: Hash256,
		cluster_hash: Hash256,
		new_settings: RoleSettings,
	) -> DispatchResult {
		ClustersPallet::edit_role(
			RuntimeOrigin::signed(signer),
			role_hash.clone(),
			cluster_hash.clone(),
			new_settings.clone(),
		)
	}

	#[test]
	fn create_cluster_should_work() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let account_id = dummy.account_id;
			let cluster_name = b"MyCluster".to_vec();
			let cluster_hash = get_cluster_hash(cluster_name.clone());
			assert_ok!(create_cluster_(account_id, cluster_name.clone()));

			assert!(<Clusters<Test>>::contains_key(&cluster_hash.clone()));

			let cluster = Cluster {
				owner: account_id,
				name: cluster_name.clone(),
				roles: vec![],
				members: vec![],
			};
			let result = <Clusters<Test>>::get(&cluster_hash.clone()).unwrap();
			assert_eq!(cluster, result);

			let clusters = <ClustersByOwner<Test>>::get(account_id).unwrap();
			assert!(clusters.contains(&cluster_hash));

			System::assert_has_event(
				ClusterEvent::ClusterCreated {
					cluster_hash: get_cluster_hash(cluster_name.clone())
				}.into()
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
				Error::<Test>::SystematicFailure
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

			assert_ok!(create_role_(
				account_id.clone(),
				cluster.clone(),
				role.clone(),
				settings.clone()
			));

			let role_hash = blake2_256(&[role.clone(), cluster.clone()].concat());
			let cluster_hash = blake2_256(&cluster.clone());

			let expected_role = Role {
				name: role.clone(),
				owner: account_id.clone(),
				settings,
				members: vec![],
				rules: None,
			};

			let roles_in_cluster = <Clusters<Test>>::get(cluster_hash).unwrap().roles;
			assert!(roles_in_cluster.contains(&role_hash));

			let stored_role = <Roles<Test>>::get(role_hash).unwrap();
			assert_eq!(expected_role, stored_role);

			System::assert_has_event(
				ClusterEvent::RoleCreated {
					role_hash: get_role_hash(cluster.clone(), role.clone())
				}.into()
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

			assert_ok!(create_cluster_(account_id, cluster.clone()));

			assert_ok!(create_role_(
				account_id.clone(),
				cluster.clone(),
				role.clone(),
				settings.clone()
			));

			let setting_wrong = RoleSettings { data: b"".to_vec(), name: b"Name".to_vec() };
			assert_noop!(
				edit_role_(
					account_id.clone(),
					get_role_hash(cluster.clone(), role.clone()),
					get_cluster_hash(cluster.clone()),
					setting_wrong
				),
				Error::<Test>::InvalidInputs
			);
		});
	}

	#[test]
	fn edit_role_without_permissions_fails() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let cluster = dummy.cluster.name;
			let role = dummy.role.name;
			let settings = dummy.role_settings;
			let account_id = dummy.account_id;
			let account_id_2 = dummy.account_id_2;

			assert_ok!(create_cluster_(account_id, cluster.clone()));

			assert_ok!(create_role_(
				account_id.clone(),
				cluster.clone(),
				role.clone(),
				settings.clone()
			));

			assert_noop!(
				edit_role_(
					account_id_2.clone(),
					get_role_hash(cluster.clone(), role.clone()),
					get_cluster_hash(cluster.clone()),
					settings.clone(),
				),
				Error::<Test>::NoPermission
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

			assert_ok!(create_cluster_(account_id, cluster.clone()));

			// do not create any role

			// assert there is no role
			assert_noop!(
				edit_role_(
					account_id.clone(),
					get_role_hash(cluster.clone(), role.clone()),
					get_cluster_hash(cluster.clone()),
					settings.clone()
				),
				Error::<Test>::RoleNotFound
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
			assert_ok!(create_cluster_(account_id, cluster.clone()));

			let role_hash = get_role_hash(cluster.clone(), role.clone());
			let cluster_hash = get_cluster_hash(cluster.clone());

			// associate the role to the cluster
			assert_ok!(create_role_(
				account_id.clone(),
				cluster.clone(),
				role.clone(),
				settings.clone()
			));

			assert_ok!(edit_role_(
				account_id.clone(),
				role_hash.clone(),
				cluster_hash.clone(),
				new_settings.clone()
			));

			let expected_role = Role {
				name: role.clone(),
				owner: account_id.clone(),
				settings: new_settings,
				members: vec![],
				rules: None,
			};

			let existing_role = <Roles<Test>>::get(role_hash).unwrap();
			assert_eq!(expected_role, existing_role);
		});
	}

	#[test]
	#[ignore]
	fn add_member_to_cluster_should_work() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let cluster = dummy.cluster;
			let role = dummy.role;
			let settings = dummy.role_settings;
			let account_id = dummy.account_id;
			let member_data = dummy.member.data;

			//assert_ok!(add_member_(account_id.clone(), cluster.clone(), role.clone(), member_data.clone()));

			//let member_hash = get_member_hash(cluster.clone().name, role.clone().name, member_data.clone());
			//assert!(<ClusterMembers<Test>>::get(get_cluster_hash(cluster.name)).contains(&member_hash));
		});
	}
}
