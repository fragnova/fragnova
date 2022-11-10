#![cfg(test)]

use crate as pallet_clusters;
use crate::{mock, mock::*, *};

use create_tests::{create_cluster_, create_role_, edit_role_};
use frame_support::{assert_ok, dispatch::DispatchResult};
use sp_io::hashing::blake2_256;

mod create_tests {
	use super::*;
	use crate::dummy_data::DummyData;
	use frame_benchmarking::account;
	use frame_support::{assert_noop, ensure};

	fn get_role_hash(cluster: Vec<u8>, role: Vec<u8>) -> Hash256 {
		blake2_256(&[role, cluster].concat())
	}

	fn get_cluster_hash(name: Vec<u8>) -> Hash256 {
		blake2_256(&name)
	}

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

	pub fn edit_role_(
		signer: <Test as frame_system::Config>::AccountId,
		role_hash: Hash256,
		new_cluster: Option<Vec<u8>>,
		new_settings: Option<RoleSettings>,
	) -> DispatchResult {
		ClustersPallet::edit_role(
			RuntimeOrigin::signed(signer),
			role_hash.clone(),
			new_cluster.clone(),
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

			let cluster = Cluster { owner: account_id, name: cluster_name.clone() };
			let result = <Clusters<Test>>::get(&cluster_hash.clone()).unwrap();

			assert_eq!(cluster, result);

			assert_eq!(
				System::events()[System::events().len() - 1].event,
				mock::RuntimeEvent::from(Event::ClusterCreated {
					cluster_hash: get_cluster_hash(cluster_name.clone())
				})
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

			assert_ok!(create_role_(
				account_id.clone(),
				cluster.clone(),
				role.clone(),
				settings.clone()
			));

			let role_hash = blake2_256(&[role.clone(), cluster.clone()].concat());
			let cluster_hash = blake2_256(&cluster.clone());

			let expected_role =
				Role { name: role.clone(), owner: account_id.clone(), settings: vec![settings] };

			let existing_role = <Roles<Test>>::get(role_hash).unwrap();
			assert_eq!(expected_role, existing_role);
			assert!(<ClusterRoles<Test>>::get(&cluster_hash).unwrap().contains(&role_hash));

			assert_eq!(
				System::events()[System::events().len() - 1].event,
				mock::RuntimeEvent::from(Event::RoleCreated {
					role_hash: get_role_hash(cluster.clone(), role.clone())
				})
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

			assert_ok!(create_role_(
				account_id.clone(),
				cluster.clone(),
				role.clone(),
				settings.clone()
			));

			let role_hash = blake2_256(&[role, cluster].concat());
			assert_noop!(
				edit_role_(account_id.clone(), role_hash.clone(), None, None),
				Error::<Test>::InvalidInputs
			);
		});
	}

	#[test]
	fn edit_role_to_not_existing_cluster_fails() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let cluster = dummy.cluster.name;
			let role = dummy.role.name;
			let settings = dummy.role_settings;
			let account_id = dummy.account_id;

			assert_ok!(create_role_(
				account_id.clone(),
				cluster.clone(),
				role.clone(),
				settings.clone()
			));

			let role_hash = blake2_256(&[role, cluster].concat());
			assert_noop!(
				edit_role_(
					account_id.clone(),
					role_hash.clone(),
					Some(b"NotExistingCluster".to_vec()),
					Some(settings.clone())
				),
				Error::<Test>::ClusterNotFound
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

			assert_ok!(create_role_(
				account_id.clone(),
				cluster.clone(),
				role.clone(),
				settings.clone()
			));

			let role_hash = blake2_256(&[role.clone(), cluster.clone()].concat());
			assert_noop!(
				edit_role_(
					account_id_2.clone(),
					role_hash.clone(),
					Some(cluster.clone()),
					Some(settings.clone()),
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

			let role_hash = blake2_256(&[role.clone(), cluster.clone()].concat());
			assert_noop!(
				edit_role_(
					account_id.clone(),
					role_hash.clone(),
					Some(cluster.clone()),
					Some(settings.clone())
				),
				Error::<Test>::RoleNotFound
			);
		});
	}

	#[test] #[ignore]
	fn edit_role_works() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let cluster = dummy.cluster.name;
			let role = dummy.role.name;
			let settings = dummy.role_settings;
			let new_settings = dummy.role_settings_2;
			let account_id = dummy.account_id;

			// create a first cluster
			assert_ok!(create_cluster_(account_id, cluster.clone()));

			// create a second cluster
			let new_cluster_name = b"AnotherCluster".to_vec();
			assert_ok!(create_cluster_(account_id.clone(), new_cluster_name.clone()));

			// associate the role to the first cluster
			assert_ok!(create_role_(
				account_id.clone(),
				cluster.clone(),
				role.clone(),
				settings.clone()
			));

/*			let role_hash = blake2_256(&[role.clone(), cluster.clone()].concat());
			let cluster_hash = blake2_256(&cluster.clone());
			assert_ok!(edit_role_(
				account_id.clone(),
				role_hash.clone(),
				cluster.clone(),
				Some(new_settings.clone())
			));

*/
		});
	}

	/*#[test]
	fn add_member_to_cluster_should_work() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let cluster = dummy.cluster;
			let role = dummy.role;
			let settings = dummy.role_settings;
			let account_id = dummy.account_id;

			assert_ok!(create_role_(account_id, &role, &cluster, &settings));

			assert!(<Roles<Test>>::contains_key(get_role_hash(&cluster, &role, &settings)));

			assert_eq!(
				System::events()[System::events().len() - 1].event,
				mock::RuntimeEvent::from(Event::RoleCreated {
					role_hash: blake2_256(
						&[role.clone().encode(), cluster.clone().name, settings.clone().encode()]
							.concat()
					)
				})
			);
		});
	}*/
}
