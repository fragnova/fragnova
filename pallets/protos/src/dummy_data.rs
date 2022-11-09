use std::str::FromStr;
use ethabi::ethereum_types::Address;
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

use protos::categories::{Categories, ShardsFormat, ShardsScriptInfo, TextCategories};

use protos::traits::{RecordInfo, Trait, VariableType, VariableTypeInfo};

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

/// NOTE: All `ProtoFragment`-type fields found in `DummyData` have no references
pub struct DummyData {
	pub proto_fragment: ProtoFragment,
	pub proto_fragment_second: ProtoFragment,
	pub proto_fragment_third: ProtoFragment,
	pub proto_fragment_fourth: ProtoFragment,
	pub proto_fragment_fifth: ProtoFragment,
	pub proto_shard_script: ProtoFragment,
	pub proto_shard_script_2: ProtoFragment,
	pub proto_shard_script_3: ProtoFragment,
	pub proto_shard_script_4: ProtoFragment,
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

		let records1 = vec![(
			"int1".to_string(),
			RecordInfo::SingleType(VariableTypeInfo {
				type_: VariableType::Int,
				default: Some(Vec::new()),
			}),
		)];
		let trait1 = Trait { name: "Trait1".to_string(), records: records1 };

		let data_trait = twox_64(&trait1.encode());

		let proto_third = ProtoFragment {
			references: Vec::new(),
			category: Categories::Trait(Some(data_trait)),
			tags: Vec::new(),
			linked_asset: None,
			include_cost: Some(2),
			data: trait1.encode(),
		};

		let records2 = vec![(
			"int2".to_string(),
			RecordInfo::SingleType(VariableTypeInfo {
				type_: VariableType::Int,
				default: Some(Vec::new()),
			}),
		)];

		let trait2 = Trait { name: "Trait2".to_string(), records: records2 };

		let data_trait_2 = twox_64(&trait2.encode());

		let proto_fourth = ProtoFragment {
			references: Vec::new(),
			category: Categories::Trait(Some(data_trait_2)),
			tags: Vec::new(),
			linked_asset: None,
			include_cost: Some(2),
			data: trait2.encode(),
		};

		let records3 = vec![(
			"int3".to_string(),
			RecordInfo::SingleType(VariableTypeInfo {
				type_: VariableType::Int,
				default: Some(Vec::new()),
			}),
		)];

		let trait3 = Trait { name: "Trait3".to_string(), records: records3 };
		let data_trait_3 = twox_64(&trait3.encode());

		let proto_fifth = ProtoFragment {
			references: Vec::new(),
			category: Categories::Trait(Some(data_trait_3)),
			tags: Vec::new(),
			linked_asset: None,
			include_cost: Some(2),
			data: trait3.encode(),
		};

		let shard_script_num_1: [u8; 8] = [4u8; 8];
		let shard_script_num_2: [u8; 8] = [5u8; 8];
		let shard_script_1 = ShardsScriptInfo {
			format: ShardsFormat::Edn,
			requiring: vec![shard_script_num_1],
			implementing: vec![shard_script_num_2],
		};

		let proto_shard_script = ProtoFragment {
			references: Vec::new(),
			category: Categories::Shards(shard_script_1),
			tags: Vec::new(),
			linked_asset: None,
			include_cost: Some(2),
			data: "0x661".as_bytes().to_vec(),
		};

		let shard_script_num_3: [u8; 8] = [9u8; 8];
		let shard_script_2 = ShardsScriptInfo {
			format: ShardsFormat::Edn,
			requiring: vec![shard_script_num_1],
			implementing: vec![shard_script_num_2, shard_script_num_3],
		};

		let proto_shard_script_2 = ProtoFragment {
			references: Vec::new(),
			category: Categories::Shards(shard_script_2),
			tags: Vec::new(),
			linked_asset: None,
			include_cost: Some(2),
			data: "0x667".as_bytes().to_vec(),
		};

		let shard_script_num_4: [u8; 8] = [1u8; 8];
		let shard_script_num_5: [u8; 8] = [7u8; 8];
		let shard_script_3 = ShardsScriptInfo {
			format: ShardsFormat::Edn,
			requiring: vec![shard_script_num_4],
			implementing: vec![shard_script_num_5],
		};

		let proto_shard_script_3 = ProtoFragment {
			references: Vec::new(),
			category: Categories::Shards(shard_script_3),
			tags: Vec::new(),
			linked_asset: None,
			include_cost: Some(2),
			data: "0x669".as_bytes().to_vec(),
		};

		let shard_script_4 = ShardsScriptInfo {
			format: ShardsFormat::Binary,
			requiring: vec![shard_script_num_4],
			implementing: vec![shard_script_num_5],
		};

		let proto_shard_script_4 = ProtoFragment {
			references: Vec::new(),
			category: Categories::Shards(shard_script_4),
			tags: Vec::new(),
			linked_asset: None,
			include_cost: Some(2),
			data: "0x670".as_bytes().to_vec(),
		};

		let patch = Patch {
			proto_fragment: proto.clone(),
			include_cost: Some(123),
			new_references: Vec::new(),
			new_data: b"<Insert Anything Here>".to_vec(),
		};

		let metadata = Metadata {
			proto_fragment: proto.clone(),
			metadata_key: b"json_description".to_vec(),
			data: b"{\"name\": \"ram\"}".to_vec(),
		};

		let contracts = vec![String::from("0x8a819F380ff18240B5c11010285dF63419bdb2d5")];
		let contract = Address::from_str(&contracts[0].as_str()[2..]).map_err(|_| "Invalid response - invalid sender").unwrap();
		let stake = Stake {
			proto_fragment: proto.clone(),
			lock: Lock {
				data: pallet_accounts::EthLockUpdate {
					public: sp_core::ed25519::Public([69u8; 32]),
					amount: U256::from(69u32),
					lock_period: 1,
					sender: get_ethereum_account_id_from_ecdsa_public_struct(
						&sp_core::ecdsa::Pair::from_seed(&[1u8; 32]).public(),
					),
					signature: create_lock_signature(
						sp_core::ecdsa::Pair::from_seed(&[1u8; 32]),
						U256::from(69u32),
						1,
						get_ethereum_account_id_from_ecdsa_public_struct(
							&sp_core::ecdsa::Pair::from_seed(&[1u8; 32]).public(),
						),
						contract,
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
			proto_shard_script,
			proto_shard_script_2,
			proto_shard_script_3,
			proto_shard_script_4,
			patch,
			metadata,
			stake,
			account_id: sp_core::ed25519::Public::from_raw([1u8; 32]),
			account_id_second: sp_core::ed25519::Public::from_raw([2u8; 32]),
			ethereum_account_id: H160::random(),
		}
	}
}
