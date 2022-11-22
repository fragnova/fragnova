use crate::{mock, mock::*, *};
use sp_core::ed25519::Public;
use sp_io::hashing::{blake2_128, blake2_256};

pub struct DummyCluster {
	pub owner: Public,
	pub name: Vec<u8>,
}

pub struct DummyRole {
	pub name: Vec<u8>,
	pub settings: Vec<RoleSetting>,
}

pub struct DummyMember {
	pub data: Vec<u8>,
}

pub struct DummyData {
	pub cluster: DummyCluster,
	pub role: DummyRole,
	pub role_settings: RoleSetting,
	pub role_settings_2: RoleSetting,
	pub member: DummyMember,
	pub account_id: Public,
	pub account_id_2: Public,
}

pub fn get_role_hash(cluster_id: Hash128, role: Vec<u8>) -> Hash128 {
	blake2_128(&[&cluster_id[..], &role.clone()[..]].concat())
}

pub fn get_cluster_id(cluster_name: Vec<u8>, account_id: sp_core::ed25519::Public) -> Hash128 {
	let extrinsic_index = 2;
	System::set_extrinsic_index(extrinsic_index);

	blake2_128(
		&[cluster_name.clone(), extrinsic_index.clone().encode(), account_id.clone().encode()]
			.concat(),
	)
}

pub fn get_vault_account_id(cluster_id: Hash128) -> sp_core::ed25519::Public {
	let hash = blake2_256(&[&b"cluster-vault"[..], &cluster_id].concat());
	sp_core::ed25519::Public::from_raw(hash)
}

impl DummyData {
	pub fn new() -> Self {
		let role_settings =
			RoleSetting { name: b"Setting One".to_vec(), data: b"Data One".to_vec() };
		let role_settings_2 =
			RoleSetting { name: b"Setting Two".to_vec(), data: b"Data Two".to_vec() };

		let role = DummyRole { name: b"RoleOne".to_vec(), settings: vec![role_settings.clone()] };

		let cluster =
			DummyCluster { owner: Public::from_raw([1u8; 32]), name: b"ClusterName".to_vec() };

		let member = DummyMember { data: Vec::new() };

		Self {
			cluster,
			role,
			role_settings,
			role_settings_2,
			member,
			account_id: sp_core::ed25519::Public::from_raw([1u8; 32]),
			account_id_2: sp_core::ed25519::Public::from_raw([2u8; 32]),
		}
	}
}
