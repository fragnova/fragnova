use crate::*;

use codec::Encode;

use sp_core::{
	ecdsa,
	keccak_256,
	Pair,
	H160, // size of an Ethereum Account Address
	U256,
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

pub fn create_link_signature(
	clamor_account_id: sp_core::ed25519::Public,
	ethereum_account_pair: sp_core::ecdsa::Pair,
) -> sp_core::ecdsa::Signature {
	let mut message = b"EVM2Fragnova".to_vec();
	message.extend_from_slice(&get_ethereum_chain_id().to_be_bytes());
	message.extend_from_slice(&clamor_account_id.encode());

	let hashed_message = keccak_256(&message);

	ethereum_account_pair.sign_prehashed(&hashed_message)
}

pub fn create_lock_signature(
	ethereum_account_pair: sp_core::ecdsa::Pair,
	lock_amount: U256,
	locktime: U256
) -> sp_core::ecdsa::Signature {
	let ethereum_account_id =
		get_ethereum_account_id_from_ecdsa_public_struct(&ethereum_account_pair.public());

	let mut message = b"FragLock".to_vec();
	message.extend_from_slice(&ethereum_account_id.0[..]);
	message.extend_from_slice(&get_ethereum_chain_id().to_be_bytes());
	message.extend_from_slice(&Into::<[u8; 32]>::into(lock_amount.clone()));
	message.extend_from_slice(&Into::<[u8; 32]>::into(locktime.clone()));

	let hashed_message = keccak_256(&message);

	// Ethereum Signature is produced by signing a keccak256 hash with the following format:
	// "\x19Ethereum Signed Message\n" + len(msg) + msg
	// Note: `msg` is the hashed message
	let hashed_message =
		keccak_256(&[b"\x19Ethereum Signed Message:\n32", &hashed_message[..]].concat());

	ethereum_account_pair.sign_prehashed(&hashed_message)
}
pub fn create_unlock_signature(
	ethereum_account_pair: sp_core::ecdsa::Pair,
	unlock_amount: U256,
	locktime: U256
) -> sp_core::ecdsa::Signature {
	let ethereum_account_id =
		get_ethereum_account_id_from_ecdsa_public_struct(&ethereum_account_pair.public());

	let mut message = b"FragUnlock".to_vec();
	message.extend_from_slice(&ethereum_account_id.0[..]);
	message.extend_from_slice(&get_ethereum_chain_id().to_be_bytes());
	message.extend_from_slice(&Into::<[u8; 32]>::into(unlock_amount.clone()));
	message.extend_from_slice(&Into::<[u8; 32]>::into(locktime.clone()));

	let hashed_message = keccak_256(&message);

	// Ethereum Signature is produced by signing a keccak256 hash with the following format:
	// "\x19Ethereum Signed Message\n" + len(msg) + msg
	// Note: `msg` is the hashed message
	let hashed_message =
		keccak_256(&[b"\x19Ethereum Signed Message:\n32", &hashed_message[..]].concat());

	ethereum_account_pair.sign_prehashed(&hashed_message)
}

pub fn get_ethereum_account_id_from_ecdsa_public_struct(
	ecdsa_public_struct: &ecdsa::Public,
) -> H160 {
	let compressed_public_key = ecdsa_public_struct.0;

	let uncompressed_public_key =
		&libsecp256k1::PublicKey::parse_compressed(&compressed_public_key)
			.unwrap()
			.serialize();
	let uncompressed_public_key_without_prefix = &uncompressed_public_key[1..];
	let ethereum_account_id = &keccak_256(uncompressed_public_key_without_prefix)[12..];

	H160::from_slice(&ethereum_account_id)
}

#[derive(Clone)]
pub struct Link {
	pub clamor_account_id: sp_core::ed25519::Public,
	pub link_signature: ecdsa::Signature,
}

impl Link {
	pub fn get_recovered_ethereum_account_id(&self) -> H160 {
		let mut message = b"EVM2Fragnova".to_vec();
		message.extend_from_slice(&get_ethereum_chain_id().to_be_bytes());
		message.extend_from_slice(&self.clamor_account_id.encode());

		let hashed_message = keccak_256(&message);

		let uncompressed_public_key_without_prefix =
			Crypto::secp256k1_ecdsa_recover(&self.link_signature.0, &hashed_message)
				.map_err(|_| format!("Mayday!"))
				.unwrap();

		let ethererum_account_id = keccak_256(&uncompressed_public_key_without_prefix[..]);
		let ethererum_account_id = &ethererum_account_id[12..];
		let ethererum_account_id = H160::from_slice(&ethererum_account_id[..]);

		ethererum_account_id
	}
}

#[derive(Clone)]
pub struct Lock {
	pub data: EthLockUpdate<sp_core::ed25519::Public>,
	pub link: Link,

	pub ethereum_account_pair: sp_core::ecdsa::Pair,
}
pub struct Unlock {
	pub lock: Lock,
	pub data: EthLockUpdate<sp_core::ed25519::Public>,
}
pub struct DummyData {
	pub link: Link,
	pub link_second: Link,
	pub lock: Lock,
	pub unlock: Unlock,
	pub account_id: sp_core::ed25519::Public,
	pub account_id_second: sp_core::ed25519::Public,
	pub ethereum_account_pair: sp_core::ecdsa::Pair,
	pub link_signature: sp_core::ecdsa::Signature,
	pub lock_signature: sp_core::ecdsa::Signature,
}

impl DummyData {
	pub fn new() -> Self {
		let link = Link {
			clamor_account_id: sp_core::ed25519::Public::from_raw([1u8; 32]),
			link_signature: create_link_signature(
				sp_core::ed25519::Public::from_raw([1u8; 32]),
				sp_core::ecdsa::Pair::from_seed(&[1u8; 32]),
			),
		};

		let link_second = Link {
			clamor_account_id: sp_core::ed25519::Public::from_raw([2u8; 32]),
			link_signature: create_link_signature(
				sp_core::ed25519::Public::from_raw([2u8; 32]),
				sp_core::ecdsa::Pair::from_seed(&[2u8; 32]),
			),
		};

		let lock = Lock {
			data: EthLockUpdate {
				public: sp_core::ed25519::Public([69u8; 32]),
				amount: U256::from(69u32),
				locktime: U256::from(1234567890),
				sender: get_ethereum_account_id_from_ecdsa_public_struct(
					&sp_core::ecdsa::Pair::from_seed(&[3u8; 32]).public(),
				),
				signature: create_lock_signature(
					sp_core::ecdsa::Pair::from_seed(&[3u8; 32]),
					U256::from(69u32),
					U256::from(1234567890)
				),
				lock: true, // yes, please lock it!
				block_number: 69,
			},
			link: Link {
				clamor_account_id: sp_core::ed25519::Public::from_raw([3u8; 32]),
				link_signature: create_link_signature(
					sp_core::ed25519::Public::from_raw([3u8; 32]),
					sp_core::ecdsa::Pair::from_seed(&[3u8; 32]),
				),
			},
			ethereum_account_pair: sp_core::ecdsa::Pair::from_seed(&[3u8; 32]),
		};

		let unlock = Unlock {
			lock: Lock {
				data: EthLockUpdate {
					public: sp_core::ed25519::Public([69u8; 32]),
					amount: U256::from(69u32),
					locktime: U256::from(1234567890),
					sender: get_ethereum_account_id_from_ecdsa_public_struct(
						&sp_core::ecdsa::Pair::from_seed(&[4u8; 32]).public(),
					),
					signature: create_lock_signature(
						sp_core::ecdsa::Pair::from_seed(&[4u8; 32]),
						U256::from(69u32),
						U256::from(1234567890)
					),
					lock: true, // yes, please lock it!
					block_number: 69,
				},
				link: Link {
					clamor_account_id: sp_core::ed25519::Public::from_raw([4u8; 32]),
					link_signature: create_link_signature(
						sp_core::ed25519::Public::from_raw([4u8; 32]),
						sp_core::ecdsa::Pair::from_seed(&[4u8; 32]),
					),
				},
				ethereum_account_pair: sp_core::ecdsa::Pair::from_seed(&[4u8; 32]),
			},
			data: EthLockUpdate {
				public: sp_core::ed25519::Public([69u8; 32]),
				amount: U256::from(0u32), // when unlocking, amount must be 0u32
				locktime: U256::from(1234567890),
				sender: get_ethereum_account_id_from_ecdsa_public_struct(
					&sp_core::ecdsa::Pair::from_seed(&[4u8; 32]).public(),
				),
				signature: create_unlock_signature(
					sp_core::ecdsa::Pair::from_seed(&[4u8; 32]),
					U256::from(0u32), // when unlocking, amount must be 0u32
					U256::from(1234567890)
				),
				lock: false, // yes, please unlock it!
				block_number: 69 + 69,
			},
		};

		Self {
			link,
			link_second,
			lock,
			unlock,
			account_id: sp_core::ed25519::Public::from_raw([111u8; 32]),
			account_id_second: sp_core::ed25519::Public::from_raw([222u8; 32]),
			ethereum_account_pair: sp_core::ecdsa::Pair::from_seed(&[111u8; 32]),
			link_signature: sp_core::ecdsa::Signature::from_raw([111u8; 65]),
			lock_signature: sp_core::ecdsa::Signature::from_raw([111u8; 65]),
		}
	}
}
