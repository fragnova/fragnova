#![cfg(test)]

use crate as pallet_clusters;
use crate::mock;
use crate::mock::*;
use crate::*;

use create_tests::create_cluster_;
use frame_support::{assert_ok, dispatch::DispatchResult};
use sp_io::hashing::blake2_256;

mod create_tests {
	use super::*;
	use crate::dummy_data::DummyData;
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
		settings: &RoleSettings,
	) -> DispatchResult {
		ClustersPallet::create_role(
			RuntimeOrigin::signed(signer),
			cluster.clone(),
			role.clone(),
			settings.clone(),
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
				&settings.clone()
			));

			let role_hash = blake2_256(&[role.clone(), cluster.clone()].concat());
			let cluster_hash = blake2_256(&cluster.clone());
			assert!(<ClusterRoles<Test>>::get(&cluster_hash).unwrap().contains(&role_hash));

			assert_eq!(
				System::events()[System::events().len() - 1].event,
				mock::RuntimeEvent::from(Event::RoleCreated {
					role_hash: get_role_hash(cluster.clone(), role.clone())
				})
			);
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
