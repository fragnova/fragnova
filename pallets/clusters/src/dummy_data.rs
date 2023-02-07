use crate::{mock::*, *};
use sp_core::ed25519::Public;
use sp_io::hashing::blake2_128;

pub struct DummyCluster {
	pub owner: Public,
	pub name: Vec<u8>,
}

pub struct DummyRole {
	pub name: Vec<u8>,
	pub settings: Vec<DummyRoleSetting>,
}

pub struct DummyRoleSetting {
	pub name: Vec<u8>,
	pub data: Vec<u8>,
}

pub struct DummyData {
	pub cluster: DummyCluster,
	pub role: DummyRole,
	pub role_settings: DummyRoleSetting,
	pub role_settings_2: DummyRoleSetting,
	pub account_id: Public,
	pub account_id_2: Public,
}

pub fn get_cluster_id(
	cluster_name: Vec<u8>,
	account_id: sp_core::ed25519::Public,
	index: u32,
) -> Hash128 {
	let extrinsic_index = 2;
	System::set_extrinsic_index(extrinsic_index);
	let block_number = System::block_number();

	blake2_128(
		&[
			&block_number.encode(),
			cluster_name.as_slice(),
			&index.to_be_bytes(),
			&account_id.encode(),
		]
		.concat(),
	)
}

pub fn take_name_index_(name: &Vec<u8>) -> Compact<u64> {
	let name_index = <Names<Test>>::get(name);
	if let Some(name_index) = name_index {
		<Compact<u64>>::from(name_index)
	} else {
		let next_name_index = <NamesIndex<Test>>::try_get().unwrap_or_default() + 1;
		let next_name_index_compact = <Compact<u64>>::from(next_name_index);
		<Names<Test>>::insert(name, next_name_index_compact);
		// storing is dangerous inside a closure
		// but after this call we start storing..
		// so it's fine here
		<NamesIndex<Test>>::put(next_name_index);
		next_name_index_compact
	}
}

impl DummyData {
	pub fn new() -> Self {
		let role_settings =
			DummyRoleSetting { name: b"Setting One".to_vec(), data: b"Data One".to_vec() };
		let role_settings_ =
			DummyRoleSetting { name: b"Setting One".to_vec(), data: b"Data One".to_vec() };
		let role_settings_2 =
			DummyRoleSetting { name: b"Setting Two".to_vec(), data: b"Data Two".to_vec() };

		let role = DummyRole { name: b"Role1".to_vec(), settings: vec![role_settings_] };

		let cluster =
			DummyCluster { owner: Public::from_raw([1u8; 32]), name: b"Cluster1".to_vec() };

		let account_id = sp_core::ed25519::Public::from_raw([1u8; 32]);
		let account_id_2 = sp_core::ed25519::Public::from_raw([2u8; 32]);

		Self { cluster, role, role_settings, role_settings_2, account_id, account_id_2 }
	}
}
