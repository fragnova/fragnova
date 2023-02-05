#![cfg(test)]

use crate::{dummy_data::*, mock::*, *};

use crate::Event as ClusterEvent;
use frame_support::{assert_ok, dispatch::DispatchResult};

mod create_tests {
	use super::*;
	use crate::dummy_data::DummyData;
	use frame_support::{
		assert_noop,
		traits::{fungible, Currency},
	};

	pub fn create_cluster_(
		signer: <Test as frame_system::Config>::AccountId,
		name: Vec<u8>,
	) -> DispatchResult {
		let bounded_name: BoundedVec<u8, <Test as Config>::NameLimit> =
			name.clone().try_into().expect("cluster name is too long");
		// fund the account to be able to create the proxy
		pallet_balances::Pallet::<Test>::make_free_balance_be(&signer, 1000000);
		ClustersPallet::create_cluster(Origin::signed(signer), bounded_name)
	}

	pub fn create_role_(
		signer: <Test as frame_system::Config>::AccountId,
		cluster: Hash128,
		role: Vec<u8>,
		settings: Vec<Setting>,
	) -> DispatchResult {
		let bounded_name: BoundedVec<u8, <Test as Config>::NameLimit> =
			role.clone().try_into().expect("role name is too long");
		let bounded_settings: BoundedVec<Setting, <Test as Config>::RoleSettingsLimit> =
			settings.clone().try_into().expect("role settings is too long");
		ClustersPallet::create_role(Origin::signed(signer), cluster, bounded_name, bounded_settings)
	}

	pub fn delete_role_(
		signer: <Test as frame_system::Config>::AccountId,
		role_name: Vec<u8>,
		cluster_id: Hash128,
	) -> DispatchResult {
		let bounded_name: BoundedVec<u8, <Test as Config>::NameLimit> =
			role_name.clone().try_into().expect("role name is too long");
		ClustersPallet::delete_role(Origin::signed(signer), bounded_name, cluster_id)
	}

	pub fn add_member_(
		signer: <Test as frame_system::Config>::AccountId,
		cluster_id: Hash128,
		roles_names: Vec<Vec<u8>>,
		member: <Test as frame_system::Config>::AccountId,
	) -> DispatchResult {
		ClustersPallet::add_member(
			Origin::signed(signer),
			cluster_id,
			roles_names.iter().map(|x| x.clone().try_into().unwrap()).collect(),
			member,
		)
	}

	pub fn add_role_members_(
		signer: <Test as frame_system::Config>::AccountId,
		cluster_id: Hash128,
		role_name: Vec<u8>,
		members: Vec<<Test as frame_system::Config>::AccountId>,
	) -> DispatchResult {
		let bounded_name: BoundedVec<u8, <Test as Config>::NameLimit> =
			role_name.clone().try_into().expect("role name is too long");
		let bounded_members_list: BoundedVec<
			<Test as frame_system::Config>::AccountId,
			<Test as Config>::MembersLimit,
		> = members.clone().try_into().expect("too many accounts");

		ClustersPallet::add_role_members(
			Origin::signed(signer),
			bounded_name,
			cluster_id,
			bounded_members_list,
		)
	}

	pub fn delete_role_members_(
		signer: <Test as frame_system::Config>::AccountId,
		cluster_id: Hash128,
		role_name: Vec<u8>,
		members: Vec<<Test as frame_system::Config>::AccountId>,
	) -> DispatchResult {
		let bounded_name: BoundedVec<u8, <Test as Config>::NameLimit> =
			role_name.clone().try_into().expect("role name is too long");
		let bounded_members_list: BoundedVec<
			<Test as frame_system::Config>::AccountId,
			<Test as Config>::MembersLimit,
		> = members.clone().try_into().expect("too many accounts");

		ClustersPallet::delete_role_members(
			Origin::signed(signer),
			bounded_name,
			cluster_id,
			bounded_members_list,
		)
	}

	pub fn add_role_settings_(
		signer: <Test as frame_system::Config>::AccountId,
		cluster_id: Hash128,
		role_name: Vec<u8>,
		settings: Vec<Setting>,
	) -> DispatchResult {
		let bounded_name: BoundedVec<u8, <Test as Config>::NameLimit> =
			role_name.clone().try_into().expect("role name is too long");
		let bounded_settings: BoundedVec<Setting, <Test as Config>::RoleSettingsLimit> =
			settings.clone().try_into().expect("too many settings");

		ClustersPallet::add_role_settings(
			Origin::signed(signer),
			bounded_name,
			cluster_id,
			bounded_settings,
		)
	}

	pub fn delete_role_settings_(
		signer: <Test as frame_system::Config>::AccountId,
		cluster_id: Hash128,
		role_name: Vec<u8>,
		settings: Vec<Vec<u8>>,
	) -> DispatchResult {
		let bounded_name: BoundedVec<u8, <Test as Config>::NameLimit> =
			role_name.clone().try_into().expect("role name is too long");
		let bounded_settings: BoundedVec<BoundedVec<u8,  <Test as Config>::NameLimit>, <Test as Config>::RoleSettingsLimit> =
			settings.iter().map(|x| x.clone().try_into().unwrap()).collect::<Vec<_>>().try_into().expect("too many settings");

		ClustersPallet::delete_role_settings(
			Origin::signed(signer),
			bounded_name,
			cluster_id,
			settings.iter().map(|x| x.clone().try_into().unwrap()).collect(),
		)
	}

	#[test]
	fn create_cluster_should_work() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let account_id = dummy.account_id;
			let cluster_name = b"Name".to_vec();

			assert_ok!(create_cluster_(account_id, cluster_name.clone()));

			let extrinsic_index = <frame_system::Pallet<Test>>::extrinsic_index().unwrap();
			let cluster_id = get_cluster_id(cluster_name.clone(), account_id, extrinsic_index);

			assert!(<Clusters<Test>>::contains_key(&cluster_id.clone()));

			let name_index = take_name_index_(&cluster_name);

			let cluster =
				Cluster { owner: account_id, name: name_index, cluster_id, roles: Vec::new() };
			let result = <Clusters<Test>>::get(&cluster_id.clone()).unwrap();
			assert_eq!(cluster, result);

			let clusters = <ClustersByOwner<Test>>::get(account_id).unwrap();
			assert!(clusters.contains(&cluster_id));

			System::assert_has_event(
				ClusterEvent::ClusterCreated { cluster_hash: cluster_id.clone() }.into(),
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
	fn create_role_should_work() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let cluster = dummy.cluster.name;
			let role = dummy.role.name;
			let settings = dummy.role_settings;
			let account_id = dummy.account_id;

			assert_ok!(create_cluster_(account_id, cluster.clone()));

			let extrinsic_index = <frame_system::Pallet<Test>>::extrinsic_index().unwrap();
			let cluster_id = get_cluster_id(cluster.clone(), account_id, extrinsic_index);

			let role_setting = Setting { name: settings.name, data: settings.data };

			assert_ok!(create_role_(
				account_id.clone(),
				cluster_id.clone(),
				role.clone(),
				vec![role_setting]
			));

			let role_name_index = take_name_index_(&role);

			let roles_in_cluster = <Clusters<Test>>::get(cluster_id).unwrap().roles;
			assert!(roles_in_cluster.contains(&role_name_index));
			//assert_eq!(<Roles<Test>>::get(&cluster_id, &name_index).unwrap(), expected_role);

			System::assert_has_event(
				ClusterEvent::RoleCreated { cluster_hash: cluster_id, role_name: role }.into(),
			);
		});
	}

	#[test]
	fn delete_role_with_invalid_inputs_fails() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let cluster = dummy.cluster.name;
			let role_name = dummy.role.name;
			let role_settings = dummy.role_settings;
			let account_id = dummy.account_id;

			assert_ok!(create_cluster_(account_id, cluster.clone()));

			let extrinsic_index = <frame_system::Pallet<Test>>::extrinsic_index().unwrap();

			let cluster_id = get_cluster_id(cluster.clone(), account_id, extrinsic_index);

			let role_setting = Setting { name: role_settings.name, data: role_settings.data };

			assert_ok!(create_role_(
				account_id.clone(),
				cluster_id.clone(),
				role_name.clone(),
				vec![role_setting]
			));

			assert_noop!(
				delete_role_(account_id.clone(), Vec::new(), cluster_id,),
				Error::<Test>::InvalidInput
			);
		});
	}

	#[test]
	fn delete_role_that_not_exist_fails() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let cluster = dummy.cluster.name;
			let account_id = dummy.account_id;

			assert_ok!(create_cluster_(account_id, cluster.clone()));

			let extrinsic_index = <frame_system::Pallet<Test>>::extrinsic_index().unwrap();

			let cluster_id = get_cluster_id(cluster.clone(), account_id, extrinsic_index);

			// assert there is no role
			assert_noop!(
				delete_role_(account_id.clone(), b"NotExistingRole".to_vec(), cluster_id,),
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

			assert_ok!(create_cluster_(account_id, cluster.clone()));

			let extrinsic_index = <frame_system::Pallet<Test>>::extrinsic_index().unwrap();

			let cluster_id = get_cluster_id(cluster.clone(), account_id, extrinsic_index);

			let name_index = take_name_index_(&role);
			let role_setting = Setting { name: settings.name, data: settings.data };

			assert_ok!(create_role_(
				account_id.clone(),
				cluster_id.clone(),
				role.clone(),
				vec![role_setting]
			));

			assert_ok!(delete_role_(account_id.clone(), role.clone(), cluster_id.clone()));

			let roles_in_cluster = <Clusters<Test>>::get(&cluster_id).unwrap().roles;
			assert!(roles_in_cluster.is_empty());
			assert!(!<Roles<Test>>::contains_key(&cluster_id, &name_index));

			System::assert_has_event(
				ClusterEvent::RoleDeleted { cluster_hash: cluster_id, role_name: role }.into(),
			);
		});
	}

	#[test]
	fn delete_role_settings_works() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let cluster = dummy.cluster.name;
			let role = dummy.role.name;
			let settings = dummy.role_settings;
			let account_id = dummy.account_id;
			let name_index = take_name_index_(&role);

			// create a cluster
			assert_ok!(create_cluster_(account_id, cluster.clone()));

			let extrinsic_index = <frame_system::Pallet<Test>>::extrinsic_index().unwrap();

			let cluster_id = get_cluster_id(cluster.clone(), account_id, extrinsic_index);
			let setting_name = settings.name;
			let setting_data = settings.data;
			let role_setting = Setting { name: setting_name.clone(), data: setting_data.clone() };
			// associate the role to the cluster
			assert_ok!(create_role_(
				account_id.clone(),
				cluster_id.clone(),
				role.clone(),
				vec![role_setting.clone()]
			));

			assert_ok!(delete_role_settings_(
				account_id.clone(),
				cluster_id.clone(),
				role.clone(),
				vec![setting_name.clone()],
			));

			let name_setting_index = take_name_index_(&setting_name);
			let role_setting =
				CompactSetting { name: name_setting_index, data: setting_data.clone() };
			assert!(!<Roles<Test>>::get(&cluster_id, &name_index)
				.unwrap()
				.settings
				.contains(&role_setting));
		});
	}

	#[test]
	fn add_role_settings_works() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let cluster = dummy.cluster.name;
			let role = dummy.role.name;
			let settings = dummy.role_settings;
			let settings2 = dummy.role_settings_2;
			let account_id = dummy.account_id;

			// create a cluster
			assert_ok!(create_cluster_(account_id, cluster.clone()));

			let extrinsic_index = <frame_system::Pallet<Test>>::extrinsic_index().unwrap();

			let cluster_id = get_cluster_id(cluster.clone(), account_id, extrinsic_index);

			let role_name_index = take_name_index_(&role);

			let setting_name = settings.name;
			let setting_data = settings.data;
			let setting2_name = settings2.name;
			let setting2_data = settings2.data;

			let role_setting1 = Setting { name: setting_name.clone(), data: setting_data.clone() };
			// associate the role to the cluster
			assert_ok!(create_role_(
				account_id.clone(),
				cluster_id.clone(),
				role.clone(),
				vec![role_setting1.clone()]
			));

			let role_setting2 =
				Setting { name: setting2_name.clone(), data: setting2_data.clone() };

			assert_ok!(add_role_settings_(
				account_id.clone(),
				cluster_id.clone(),
				role.clone(),
				vec![role_setting2.clone()],
			));

			let name_setting_index = take_name_index_(&setting_name);
			let role_setting_1 =
				CompactSetting { name: name_setting_index, data: setting_data.clone() };

			assert!(<Roles<Test>>::get(&cluster_id, &role_name_index)
				.unwrap()
				.settings
				.contains(&role_setting_1));
			let name_setting_index2 = take_name_index_(&setting2_name);
			let role_setting_2 =
				CompactSetting { name: name_setting_index2, data: setting2_data.clone() };
			assert!(<Roles<Test>>::get(&cluster_id, &role_name_index)
				.unwrap()
				.settings
				.contains(&role_setting_2));
		});
	}

	#[test]
	fn add_role_settings_with_same_name_fails() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let cluster = dummy.cluster.name;
			let role = dummy.role.name;
			let settings = dummy.role_settings;
			let account_id = dummy.account_id;

			// create a cluster
			assert_ok!(create_cluster_(account_id, cluster.clone()));

			let extrinsic_index = <frame_system::Pallet<Test>>::extrinsic_index().unwrap();
			let cluster_id = get_cluster_id(cluster.clone(), account_id, extrinsic_index);
			let role_name_index = take_name_index_(&role);

			let setting_name = settings.name;
			let setting_data = settings.data;
			let setting2_name = setting_name.clone();
			let setting2_data = setting_data.clone();

			let role_setting1 = Setting { name: setting_name.clone(), data: setting_data.clone() };
			// associate the role to the cluster
			assert_ok!(create_role_(
				account_id.clone(),
				cluster_id.clone(),
				role.clone(),
				vec![role_setting1.clone()]
			));

			let role_setting2 =
				Setting { name: setting2_name.clone(), data: setting2_data.clone() };

			assert_noop!(
				add_role_settings_(
					account_id.clone(),
					cluster_id.clone(),
					role.clone(),
					vec![role_setting2.clone()],
				),
				Error::<Test>::RoleSettingsExists
			);

			assert_eq!(
				<Roles<Test>>::get(&cluster_id, &role_name_index).unwrap().settings.len(),
				1
			);
		});
	}

	#[test]
	fn add_role_members_works() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let cluster = dummy.cluster.name;
			let role = dummy.role.name;
			let settings = dummy.role_settings;
			let account_id = dummy.account_id;
			let account_id_2 = dummy.account_id_2;

			// create a cluster
			assert_ok!(create_cluster_(account_id, cluster.clone()));

			let extrinsic_index = <frame_system::Pallet<Test>>::extrinsic_index().unwrap();

			let cluster_id = get_cluster_id(cluster.clone(), account_id, extrinsic_index);

			let role_setting = Setting { name: settings.name, data: settings.data };
			// associate the role to the cluster
			assert_ok!(create_role_(
				account_id.clone(),
				cluster_id.clone(),
				role.clone(),
				vec![role_setting.clone()]
			));

			assert_ok!(add_role_members_(
				account_id.clone(),
				cluster_id.clone(),
				role.clone(),
				vec![account_id.clone(), account_id_2],
			));

			assert!(<Members<Test>>::contains_key(&cluster_id, &account_id));
			assert!(<Members<Test>>::contains_key(&cluster_id, &account_id));
		});
	}

	#[test]
	fn delete_role_members_works() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let cluster = dummy.cluster.name;
			let role = dummy.role.name;
			let settings = dummy.role_settings;
			let account_id = dummy.account_id;
			let account_id_2 = dummy.account_id_2;

			// create a cluster
			assert_ok!(create_cluster_(account_id, cluster.clone()));

			let extrinsic_index = <frame_system::Pallet<Test>>::extrinsic_index().unwrap();

			let cluster_id = get_cluster_id(cluster.clone(), account_id, extrinsic_index);

			let role_setting = Setting { name: settings.name, data: settings.data };
			// associate the role to the cluster
			assert_ok!(create_role_(
				account_id.clone(),
				cluster_id.clone(),
				role.clone(),
				vec![role_setting.clone()]
			));

			assert_ok!(add_role_members_(
				account_id.clone(),
				cluster_id.clone(),
				role.clone(),
				vec![account_id.clone(), account_id_2],
			));

			assert_ok!(delete_role_members_(
				account_id.clone(),
				cluster_id.clone(),
				role.clone(),
				vec![account_id_2.clone()],
			));

			assert!(<Members<Test>>::contains_key(&cluster_id, &account_id));
			assert!(!<Members<Test>>::contains_key(&cluster_id, &account_id_2));
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
			assert_ok!(create_cluster_(account_id, cluster.clone()));

			let extrinsic_index = <frame_system::Pallet<Test>>::extrinsic_index().unwrap();

			let cluster_id = get_cluster_id(cluster.clone(), account_id, extrinsic_index);
			let role_setting = Setting { name: settings.name, data: settings.data };

			// create a role for the cluster
			assert_ok!(create_role_(
				account_id.clone(),
				cluster_id.clone(),
				role.clone(),
				vec![role_setting.clone()]
			));

			// add member into the cluster
			assert_ok!(add_member_(
				account_id.clone(),
				cluster_id.clone(),
				vec![role.clone()],
				member.clone()
			));

			assert!(<Members<Test>>::contains_key(&cluster_id, &member));
		});
	}
}
