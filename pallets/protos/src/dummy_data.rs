use crate::*;

pub use pallet_accounts::dummy_data::{
	create_link_signature, create_lock_signature, get_ethereum_account_id_from_ecdsa_public_struct,
	Link, Lock,
};

use sp_core::{
	Pair,
	H160, // Ethereum Account Addresses use this type
	U256,
};

use sp_clamor::{Hash256, CID_PREFIX};

use protos::categories::{Categories, ShardsTraitInfo, TextCategories};

pub fn compute_data_hash(data: &Vec<u8>) -> Hash256 {
	blake2_256(&data)
}

pub fn compute_data_cid(data: &Vec<u8>) -> Vec<u8> {
	let hash = compute_data_hash(data);

	let cid = [&CID_PREFIX[..], &hash[..]].concat();
	let cid = cid.to_base58();
	let cid = [&b"z"[..], cid.as_bytes()].concat();

	cid
}

#[derive(Clone)]
pub struct ProtoFragment {
	pub references: Vec<Hash256>,
	pub category: Categories,
	pub tags: Vec<Vec<u8>>,
	pub linked_asset: Option<LinkedAsset>,
	pub include_cost: Option<u64>,
	pub data: Vec<u8>,
}
impl ProtoFragment {
	pub fn get_proto_hash(&self) -> Hash256 {
		compute_data_hash(&self.data)
	}

	pub fn get_proto_cid(&self) -> Vec<u8> {
		compute_data_cid(&self.data)
	}
}

pub struct Patch {
	pub proto_fragment: ProtoFragment,
	pub include_cost: Option<u64>,
	pub new_references: Vec<Hash256>,
	pub new_data: Vec<u8>,
}
impl Patch {
	pub fn get_data_hash(&self) -> Hash256 {
		compute_data_hash(&self.new_data)
	}
	pub fn get_data_cid(&self) -> Vec<u8> {
		compute_data_cid(&self.new_data)
	}
}

pub struct Metadata {
	pub proto_fragment: ProtoFragment,
	pub metadata_key: Vec<u8>,
	pub data: Vec<u8>,
}
impl Metadata {
	pub fn get_data_hash(&self) -> Hash256 {
		compute_data_hash(&self.data)
	}
}

pub struct Stake {
	pub proto_fragment: ProtoFragment,
	pub lock: Lock,
}

impl Stake {
	pub fn get_stake_amount(&self) -> u64 {
		self.proto_fragment.include_cost.unwrap()
	}
}

/// NOTE: All `ProtoFragment`-type fields found in `DummyData` have no references
pub struct DummyData {
	pub proto_fragment: ProtoFragment,
	pub proto_fragment_second: ProtoFragment,
	pub proto_fragment_third: ProtoFragment,
	pub proto_fragment_fourth: ProtoFragment,
	pub proto_fragment_fifth: ProtoFragment,
	pub patch: Patch,
	pub metadata: Metadata,
	pub stake: Stake,
	pub account_id: sp_core::ed25519::Public,
	pub account_id_second: sp_core::ed25519::Public,
	pub ethereum_account_id: H160,
}

impl DummyData {
	pub fn new() -> Self {
		let proto = ProtoFragment {
			references: Vec::new(),
			category: Categories::Text(TextCategories::Plain),
			tags: Vec::new(),
			linked_asset: None,
			include_cost: Some(867),
			data: "0x111".as_bytes().to_vec(),
		};

		let proto_second = ProtoFragment {
			references: Vec::new(),
			category: Categories::Text(TextCategories::Plain),
			tags: Vec::new(),
			linked_asset: None,
			include_cost: Some(2),
			data: "0x222".as_bytes().to_vec(),
		};

		let num: [u8; 16] = [1u8; 16];
		let shard = ShardsTraitInfo {
			name: "Shards1".to_string(),
			description: "test 1".to_string(),
			id: num,
		};

		let proto_third = ProtoFragment {
			references: Vec::new(),
			category: Categories::Trait(shard),
			tags: Vec::new(),
			linked_asset: None,
			include_cost: Some(2),
			data: "ThisIsATest".as_bytes().to_vec(),
		};

		let num2: [u8; 16] = [13u8; 16];
		let shard2 = ShardsTraitInfo {
			name: "NameOfShard".to_string(),
			description: "description".to_string(),
			id: num2,
		};

		let proto_fourth = ProtoFragment {
			references: Vec::new(),
			category: Categories::Trait(shard2),
			tags: Vec::new(),
			linked_asset: None,
			include_cost: Some(2),
			data: "0x555".as_bytes().to_vec(),
		};

		let num3: [u8; 16] = [4u8; 16];
		let shard3 = ShardsTraitInfo {
			name: "NameOfShard".to_string(),
			description: "description2".to_string(),
			id: num3,
		};

		let proto_fifth = ProtoFragment {
			references: Vec::new(),
			category: Categories::Trait(shard3),
			tags: Vec::new(),
			linked_asset: None,
			include_cost: Some(2),
			data: "0x666".as_bytes().to_vec(),
		};

		let patch = Patch {
			proto_fragment: ProtoFragment {
				references: Vec::new(),
				category: Categories::Text(TextCategories::Plain),
				tags: Vec::new(),
				linked_asset: None,
				include_cost: Some(3),
				data: "0x333".as_bytes().to_vec(),
			},
			include_cost: Some(123),
			new_references: Vec::new(),
			new_data: b"<Insert Anything Here>".to_vec(),
		};

		let metadata = Metadata {
			proto_fragment: ProtoFragment {
				references: Vec::new(),
				category: Categories::Text(TextCategories::Plain),
				tags: Vec::new(),
				linked_asset: None,
				include_cost: Some(4),
				data: "0x444".as_bytes().to_vec(),
			},
			metadata_key: b"json_description".to_vec(),
			data: b"{\"name\": \"ram\"}".to_vec(),
		};

		let stake = Stake {
			proto_fragment: ProtoFragment {
				references: Vec::new(),
				category: Categories::Text(TextCategories::Plain),
				tags: Vec::new(),
				linked_asset: None,
				include_cost: Some(5),
				data: "0x555".as_bytes().to_vec(),
			},
			lock: Lock {
				data: pallet_accounts::EthLockUpdate {
					public: sp_core::ed25519::Public([69u8; 32]),
					amount: U256::from(69u32),
					sender: get_ethereum_account_id_from_ecdsa_public_struct(
						&sp_core::ecdsa::Pair::from_seed(&[1u8; 32]).public(),
					),
					signature: create_lock_signature(
						sp_core::ecdsa::Pair::from_seed(&[1u8; 32]),
						U256::from(69u32),
					),
					lock: true, // yes, please lock it!
					block_number: 69,
				},
				link: Link {
					clamor_account_id: sp_core::ed25519::Public::from_raw([255u8; 32]),
					link_signature: create_link_signature(
						sp_core::ed25519::Public::from_raw([3u8; 32]),
						sp_core::ecdsa::Pair::from_seed(&[1u8; 32]),
					),
				},
				ethereum_account_pair: sp_core::ecdsa::Pair::from_seed(&[1u8; 32]),
			},
		};

		Self {
			proto_fragment: proto,
			proto_fragment_second: proto_second,
			proto_fragment_third: proto_third,
			proto_fragment_fourth: proto_fourth,
			proto_fragment_fifth: proto_fifth,
			patch,
			metadata,
			stake,
			account_id: sp_core::ed25519::Public::from_raw([1u8; 32]),
			account_id_second: sp_core::ed25519::Public::from_raw([2u8; 32]),
			ethereum_account_id: H160::random(),
		}
	}
}