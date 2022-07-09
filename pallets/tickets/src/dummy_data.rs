use crate::*;

use codec::{Decode, Encode};

use sp_core::{

	H160, // size of an Ethereum Account Address

	U256,

	Pair,

	keccak_256,
	
	ecdsa,

};


#[cfg(test)]
fn get_ethereum_chain_id() -> u64 {
	use crate::mock::Test;
	use frame_support::traits::TypedGet;
	<Test as crate::Config>::EthChainId::get()
}

#[cfg(not(test))]
fn get_ethereum_chain_id() -> u64 {
	5
}

// fn get_ethereum_chain_id() -> u64 {
//     if cfg!(test) {
//         use crate::mock::Test;
//         use frame_support::traits::TypedGet;

//         <Test as crate::Config>::EthChainId::get()
//     } else {
//         5
//     }
// }

fn get_ethereum_account_id(ecdsa_public_struct: &ecdsa::Public) -> H160 {

		let compressed_public_key = ecdsa_public_struct.0;

		
		let uncompressed_public_key = &libsecp256k1::PublicKey::parse_compressed(&compressed_public_key).unwrap().serialize();
		let uncompressed_public_key_without_prefix = &uncompressed_public_key[1..];
		let ethereum_account_id = &keccak_256(uncompressed_public_key_without_prefix)[12..];

		// println!("uncompressed_public_key is: {:?}", uncompressed_public_key);
		// println!("ethereum_account_id is: {:?}", ethereum_account_id);


		H160::from_slice(&ethereum_account_id)
}


fn create_link_hashed_message(clamor_account_id: &sp_core::ed25519::Public) -> [u8; 32] {

	let mut message = b"EVM2Fragnova".to_vec();
	message.extend_from_slice(&get_ethereum_chain_id().to_be_bytes());
	message.extend_from_slice(&clamor_account_id.encode());

	let hashed_message = keccak_256(&message);

	hashed_message

}

fn create_lock_hashed_message(ethereum_account_id: &H160, lock_amount: &U256) -> [u8; 32] {

	let mut message = b"FragLock".to_vec();
	message.extend_from_slice(&ethereum_account_id.0[..]);
	message.extend_from_slice(&get_ethereum_chain_id().to_be_bytes());
	message.extend_from_slice(&Into::<[u8; 32]>::into(lock_amount.clone()));

	let hashed_message = keccak_256(&message);

	// Ethereum Signature is produced by signing a keccak256 hash with the following format:
	// "\x19Ethereum Signed Message\n" + len(msg) + msg
	// Note: `msg` is the hashed message
	let ethereum_signed_message = [b"\x19Ethereum Signed Message:\n32", &hashed_message[..]].concat(); 
	let ethereum_signed_message_hash = keccak_256(&ethereum_signed_message);

	ethereum_signed_message_hash
}

fn create_unlock_hashed_message(ethereum_account_id: &H160, unlock_amount: &U256) -> [u8; 32] {

	let mut message = b"FragUnlock".to_vec();
	message.extend_from_slice(&ethereum_account_id.0[..]);
	message.extend_from_slice(&get_ethereum_chain_id().to_be_bytes());
	message.extend_from_slice(&Into::<[u8; 32]>::into(unlock_amount.clone()));

	let hashed_message = keccak_256(&message);

	// Ethereum Signature is produced by signing a keccak256 hash with the following format:
	// "\x19Ethereum Signed Message\n" + len(msg) + msg
	// Note: `msg` is the hashed message
	let ethereum_signed_message = [b"\x19Ethereum Signed Message:\n32", &hashed_message[..]].concat(); 
	let ethereum_signed_message_hash = keccak_256(&ethereum_signed_message);

	ethereum_signed_message_hash
}

pub struct Link {
	pub ethereum_account_pair: sp_core::ecdsa::Pair,
	pub clamor_account_id: sp_core::ed25519::Public,
}

impl Link {

	pub fn get_ethereum_account_id(&self) -> H160 {

		// The `ethereum_account_id` I get from the commented code below is wrong, and I don't understand why!!! @sinkingsugar

		get_ethereum_account_id(&self.ethereum_account_pair.public())


		// The `uncompressed_public_key` here doesn't include the 0x04 prefix
		// let uncompressed_public_key = sp_io::crypto::secp256k1_ecdsa_recover(&self.get_link_signature().0, &self.get_link_hashed_message()).map_err(|e| "Unable to recover pubkey!").unwrap();
		// let ethereum_account_id = &keccak_256(&uncompressed_public_key)[12..];
		// H160::from_slice(&ethereum_account_id)
	}

	pub fn get_link_hashed_message(&self) -> [u8; 32] {

		create_link_hashed_message(&self.clamor_account_id)
	}

	pub fn get_link_signature(&self) -> sp_core::ecdsa::Signature {
		let message_hash = self.get_link_hashed_message();

		self.ethereum_account_pair.sign_prehashed(&message_hash)
	}
}

#[derive(Clone)]
pub struct Lock {
	pub ethereum_account_pair: sp_core::ecdsa::Pair,
	pub lock_amount: U256,
	pub block_number: u64, // block.number returns `uint` in Solidity (`uint` is the same as `uint256` in Solidity)
	
	pub linked_clamor_account_id: sp_core::ed25519::Public,
}

impl Lock {

	pub fn get_ethereum_account_id(&self) -> H160 {
		get_ethereum_account_id(&self.ethereum_account_pair.public())
	}

	pub fn get_lock_hashed_message(&self) -> [u8; 32] {
		create_lock_hashed_message(&self.get_ethereum_account_id(), &self.lock_amount)
	}

	pub fn get_lock_signature(&self) -> sp_core::ecdsa::Signature {
		let hashed_message = self.get_lock_hashed_message();
		self.ethereum_account_pair.sign_prehashed(&hashed_message)
	}

	pub fn get_link(&self) -> Link {
		Link { 
			ethereum_account_pair: self.ethereum_account_pair.clone(), 
			clamor_account_id: self.linked_clamor_account_id.clone(),
		}
	}
}


pub struct Unlock {
	pub lock: Lock,

	pub unlock_amount: U256,
	pub block_number: u64,
}

impl Unlock {

	pub fn get_ethereum_account_id(&self) -> H160 {
		self.lock.get_ethereum_account_id()
	}

	pub fn get_unlock_hashed_message(&self) -> [u8; 32] {
		create_unlock_hashed_message(&self.lock.get_ethereum_account_id(), &U256::from(0u32))
	}

	pub fn get_unlock_signature(&self) -> sp_core::ecdsa::Signature {
		let hashed_message = self.get_unlock_hashed_message();
		self.lock.ethereum_account_pair.sign_prehashed(&hashed_message)
	}
}

pub struct DummyData {
	pub link: Link,

	pub link_second: Link,

	pub lock: Lock,

	pub unlock: Unlock,

	pub account_id: sp_core::ed25519::Public,

	pub ethereum_account_pair: sp_core::ecdsa::Pair, 

	pub link_signature: sp_core::ecdsa::Signature,
}

impl DummyData {
    pub fn new() -> Self {
		let link = Link {
			ethereum_account_pair: sp_core::ecdsa::Pair::from_seed(&[1u8; 32]),
			clamor_account_id: sp_core::ed25519::Public::from_raw([1u8; 32]),
		};

		let link_second = Link {
			ethereum_account_pair: sp_core::ecdsa::Pair::from_seed(&[2u8; 32]),
			clamor_account_id: sp_core::ed25519::Public::from_raw([2u8; 32]),
		};


		let lock = Lock {
			ethereum_account_pair: sp_core::ecdsa::Pair::from_seed(&[3u8; 32]),
			lock_amount: U256::from(69u32),
			block_number: 69,
			linked_clamor_account_id: sp_core::ed25519::Public::from_raw([3u8; 32]),
		};

		let unlock = Unlock {
			lock: Lock { 
				ethereum_account_pair: sp_core::ecdsa::Pair::from_seed(&[4u8; 32]), 
				lock_amount: U256::from(69u32), 
				block_number: 69, 
				linked_clamor_account_id: sp_core::ed25519::Public::from_raw([4u8; 32]),
			},

			unlock_amount: U256::from(0u32),
			block_number: 69 + 69,

		};

		Self { 
			link: link,

			link_second: link_second,

			lock: lock,

			unlock: unlock,

			account_id: sp_core::ed25519::Public::from_raw([111u8; 32]),

			ethereum_account_pair: sp_core::ecdsa::Pair::from_seed(&[111u8; 32]),

			link_signature: sp_core::ecdsa::Signature::from_raw([111u8; 65]),
		}
		
	}

}