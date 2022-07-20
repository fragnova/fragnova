use crate::*;

pub use pallet_accounts::dummy_data::{
	Lock, Link,
};


use sp_core::{

	Pair,

	U256,

	H160, // Ethereum Account Addresses use this type
	
};

use sp_clamor::{
	Hash256, CID_PREFIX,
};

use protos::categories::{
	Categories, TextCategories,
};


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
	pub fn get_data_cid(&self) -> Vec<u8> {
		compute_data_cid(&self.data)
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
	pub patch: Patch,
	pub metadata: Metadata,
	// This is a stake that will succeed
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
			data:  "0x0155a0e40220".as_bytes().to_vec(),
		};

		let proto_second = ProtoFragment { 
			references: Vec::new(), 
			category: Categories::Text(TextCategories::Plain), 
			tags: Vec::new(), 
			linked_asset: None, 
			include_cost: Some(2), 
			data:  "0x222".as_bytes().to_vec(),
		};

		let patch = Patch { 
			proto_fragment: ProtoFragment { 
				references: Vec::new(), 
				category: Categories::Text(TextCategories::Plain), 
				tags: Vec::new(), 
				linked_asset: None, 
				include_cost: Some(3), 
				data:  "0x333".as_bytes().to_vec(),
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
				data:  "0x444".as_bytes().to_vec(),
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
				data:  "0x555".as_bytes().to_vec(),
			},

			lock: Lock {
				ethereum_account_pair: sp_core::ecdsa::Pair::from_seed(&[1u8; 32]),
				lock_amount: U256::from(69u32),
				block_number: 69,

				linked_clamor_account_id: sp_core::ed25519::Public::from_raw([1u8; 32])
			}
		};

        Self {
            proto_fragment: proto,
			proto_fragment_second: proto_second,
			patch: patch,
			metadata: metadata,
			stake: stake,

			account_id: sp_core::ed25519::Public::from_raw([1u8; 32]),

			account_id_second: sp_core::ed25519::Public::from_raw([2u8; 32]),
			
			ethereum_account_id: H160::random(),
        }
    }
}