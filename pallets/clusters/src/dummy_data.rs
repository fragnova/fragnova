use crate::{mock, mock::*, *};
use sp_core::ed25519::Public;
use sp_io::hashing::blake2_256;

pub struct DummyCluster {
	pub owner: Public,
	pub name: Vec<u8>,
}

pub struct DummyRole {
	pub owner: Public,
	pub name: Vec<u8>,
	pub settings: Vec<RoleSettings>,
}

pub struct DummyMember {
	pub data: Vec<u8>,
}

pub struct DummyData {
	pub cluster: DummyCluster,
	pub role: DummyRole,
	pub role_settings: RoleSettings,
	pub role_settings_2: RoleSettings,
	pub member: DummyMember,
	pub account_id: Public,
	pub account_id_2: Public,
}

pub fn get_role_hash(cluster: Vec<u8>, role: Vec<u8>) -> Hash256 {
	blake2_256(&[role, cluster].concat())
}

pub fn get_cluster_hash(name: Vec<u8>) -> Hash256 {
	blake2_256(&name)
}

pub fn get_member_hash(cluster: Vec<u8>, role: Vec<u8>, data: Vec<u8>) -> Hash256 {
	blake2_256(&[cluster, role, data].concat())
}

impl DummyData {
	pub fn new() -> Self {
		let role_settings =
			RoleSettings { name: b"Setting One".to_vec(), data: b"Data One".to_vec() };
		let role_settings_2 =
			RoleSettings { name: b"Setting Two".to_vec(), data: b"Data Two".to_vec() };

		let role = DummyRole {
			owner: Public::from_raw([1u8; 32]),
			name: b"RoleOne".to_vec(),
			settings: vec![role_settings.clone()],
		};

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
