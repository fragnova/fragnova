use crate::{mock, mock::*, *};
use sp_core::ed25519::Public;

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
	pub cluster: Vec<DummyCluster>,
	pub role: Vec<DummyRole>,
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

impl DummyData {
	pub fn new() -> Self {
		let role_settings = RoleSettings { name: b"Setting One".to_vec(), data: Vec::new() };
		let role_settings_2 = RoleSettings { name: b"Setting Two".to_vec(), data: Vec::new() };

		let role = DummyRole {
			owner: Public::from_raw([1u8; 32]),
			name: b"RoleOne".to_vec(),
			settings: vec![role_settings.clone()],
		};

		let cluster =
			DummyCluster { owner: Public::from_raw([1u8; 32]), name: b"ClusterName".to_vec() };

		let member = DummyMember { cluster: Vec::new(), role: Vec::new(), data: Vec::new() };

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
