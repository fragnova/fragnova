use sp_core::{bounded_vec, ConstU32};
use crate::{mock, mock::*, *};
use sp_core::ed25519::Public;
use sp_io::hashing::{blake2_128, blake2_256};

pub struct DummyCluster {
	pub owner: Public,
	pub name: BoundedVec<u8, <Test as Config>::NameLimit>,
}

pub struct DummyRole {
	pub name: BoundedVec<u8, <Test as Config>::NameLimit>,
	pub settings: Vec<RoleSetting<<Test as Config>::NameLimit, <Test as Config>::DataLimit>>,
}

pub struct DummyData {
	pub cluster: DummyCluster,
	pub role: DummyRole,
	pub role_settings: RoleSetting<<Test as Config>::NameLimit, <Test as Config>::DataLimit>,
	pub role_settings_2: RoleSetting<<Test as Config>::NameLimit, <Test as Config>::DataLimit>,
	pub account_id: Public,
	pub account_id_2: Public,
}

pub fn get_role_hash(cluster_id: Hash128, role: BoundedVec<u8, <Test as Config>::NameLimit>) -> Hash128 {
	blake2_128(&[&cluster_id[..], &role.clone()[..]].concat())
}

pub fn get_cluster_id(cluster_name: BoundedVec<u8, <Test as Config>::NameLimit>, account_id: sp_core::ed25519::Public) -> Hash128 {
	let extrinsic_index = 2;
	System::set_extrinsic_index(extrinsic_index);

	blake2_128(
		&[cluster_name.clone(), extrinsic_index.clone().encode(), account_id.clone().encode()]
			.concat(),
	)
}

pub fn get_vault_account_id(cluster_id: Hash128) -> sp_core::ed25519::Public {
	let hash = blake2_256(&[&b"fragnova-vault"[..], &cluster_id].concat());
	sp_core::ed25519::Public::from_raw(hash)
}

impl DummyData {
	pub fn new() -> Self {
		let setting_name: BoundedVec<u8, <Test as Config>::NameLimit> = bounded_vec![b"Setting One"];
		let setting_data: BoundedVec<u8, <Test as Config>::DataLimit> = bounded_vec![b"Data One"];
		let role_settings =
			RoleSetting { name: setting_name, data: setting_data };

		let setting_name_2: BoundedVec<u8, <Test as Config>::NameLimit> = bounded_vec![b"Setting Two"];
		let setting_data_2: BoundedVec<u8, <Test as Config>::DataLimit> = bounded_vec![b"Data Two"];
		let role_settings_2 =
			RoleSetting { name: setting_name_2, data: setting_data_2 };

		let role_name: BoundedVec<u8, <Test as Config>::NameLimit> = bounded_vec![b"RoleOne"];
		let role = DummyRole { name: role_name, settings: vec![role_settings.clone()] };

		let cluster_name: BoundedVec<u8, <Test as Config>::NameLimit> = bounded_vec![b"ClusterName"];
		let cluster =
			DummyCluster { owner: Public::from_raw([1u8; 32]), name: cluster_name };

		Self {
			cluster,
			role,
			role_settings,
			role_settings_2,
			account_id: sp_core::ed25519::Public::from_raw([1u8; 32]),
			account_id_2: sp_core::ed25519::Public::from_raw([2u8; 32]),
		}
	}
}
