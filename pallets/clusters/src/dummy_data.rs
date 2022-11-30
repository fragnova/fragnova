use frame_support::traits::fungible;
use sp_core::{bounded_vec, ConstU32};
use crate::{mock, mock::*, *};
use sp_core::ed25519::Public;
use sp_io::hashing::{blake2_128, blake2_256};

pub struct DummyCluster {
	pub owner: Public,
	pub name: Vec<u8>,
}

pub struct DummyRole {
	pub name: Vec<u8>,
	pub settings: Vec<RoleSetting<BoundedVec<u8, <Test as Config>::NameLimit>, BoundedVec<u8, <Test as Config>::DataLimit>>>,
}

pub struct DummyData {
	pub cluster: DummyCluster,
	pub role: DummyRole,
	pub role_settings: RoleSetting<BoundedVec<u8, <Test as Config>::NameLimit>, BoundedVec<u8, <Test as Config>::DataLimit>>,
	pub role_settings_2: RoleSetting<BoundedVec<u8, <Test as Config>::NameLimit>, BoundedVec<u8, <Test as Config>::DataLimit>>,
	pub account_id: Public,
	pub account_id_2: Public,
}

pub fn get_role_hash(cluster_id: Hash128, role: BoundedVec<u8, <Test as Config>::NameLimit>) -> Hash128 {
	blake2_128(&[&cluster_id[..], &role.clone()[..]].concat())
}

pub fn get_cluster_id(cluster_name: Vec<u8>, account_id: sp_core::ed25519::Public) -> Hash128 {
	let extrinsic_index = 2;
	System::set_extrinsic_index(extrinsic_index);
	let block_number = System::block_number();

	blake2_128(
		&[block_number.encode(), cluster_name.clone(), extrinsic_index.clone().encode(), account_id.clone().encode()]
			.concat(),
	)
}

pub fn get_vault_account_id(cluster_id: Hash128) -> sp_core::ed25519::Public {
	let hash = blake2_256(&[&b"fragnova-vault"[..], &cluster_id].concat());
	sp_core::ed25519::Public::from_raw(hash)
}

impl DummyData {
	pub fn new() -> Self {

		let setting_name: BoundedVec<u8, <Test as Config>::NameLimit> = bounded_vec![1, 2, 3, 4, 5];
		let setting_name_2: BoundedVec<u8, <Test as Config>::DataLimit> = bounded_vec![1, 2, 3, 4, 5];
		let setting_data: BoundedVec<u8, <Test as Config>::NameLimit> = bounded_vec![1, 2, 3, 4, 5];
		let setting_data_2: BoundedVec<u8, <Test as Config>::DataLimit> = bounded_vec![1, 2, 3, 4, 5];

		let role_settings =
			RoleSetting { name: setting_name, data: setting_data };

		let role_settings_2 =
			RoleSetting { name: setting_name_2, data: setting_data_2 };

		let role = DummyRole { name: b"Role1".to_vec(), settings: vec![role_settings.clone()] };

		let cluster =
			DummyCluster { owner: Public::from_raw([1u8; 32]), name: b"MyCl".to_vec() };

		let account_id = sp_core::ed25519::Public::from_raw([1u8; 32]);
		let account_id_2 = sp_core::ed25519::Public::from_raw([2u8; 32]);

		Self {
			cluster,
			role,
			role_settings,
			role_settings_2,
			account_id,
			account_id_2
		}
	}
}
