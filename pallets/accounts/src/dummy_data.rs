use crate::*;

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
// (If pallet_accounts::dummy_data is being built in another pallet)
#[cfg(not(test))]
fn get_ethereum_chain_id() -> u64 {
	5
}

#[cfg(test)]
fn get_genesis_hash() -> sp_core::H256 {
	use crate::mock::Test;
	<frame_system::Pallet<Test>>::block_hash(<Test as frame_system::Config>::BlockNumber::zero())
}
// (If pallet_accounts::dummy_data is being built in another pallet)
#[cfg(not(test))]
fn get_genesis_hash() -> sp_core::H256 {
	H256([0u8; 32])
}

pub fn create_link_signature(
	clamor_account_id: sp_core::ed25519::Public,
	ethereum_account_pair: sp_core::ecdsa::Pair,
) -> sp_core::ecdsa::Signature {

	let sender_string = hex::encode(clamor_account_id);
	let genesis_hash_string = hex::encode(get_genesis_hash());

	let message: Vec<u8> = [
		&[0x19, 0x01],
		// This is the `domainSeparator` (https://eips.ethereum.org/EIPS/eip-712#definition-of-domainseparator)
		&keccak_256(
			// We use the ABI encoding Rust library since it encodes each token as 32-bytes
			&ethabi::encode(
				&vec![
					Token::Uint(
						U256::from(keccak_256(b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"))
					),
					Token::Uint(U256::from(keccak_256(b"Fragnova Network"))), // The dynamic values bytes and string are encoded as a keccak_256 hash of their contents.
					Token::Uint(U256::from(keccak_256(b"1"))), // The dynamic values bytes and string are encoded as a keccak_256 hash of their contents.
					Token::Uint(U256::from(get_ethereum_chain_id())),
					Token::Address(H160::from(TryInto::<[u8; 20]>::try_into(hex::decode("F5A0Af5a0AF5a0AF5a0af5A0Af5A0AF5a0AF5A0A").unwrap()).unwrap())),
				]
			)
		)[..],
		// This is the `hashStruct(message)`. Note: `hashStruct(message : 𝕊) = keccak_256(typeHash ‖ encodeData(message))`, where `typeHash = keccak_256(encodeType(typeOf(message)))`.
		&keccak_256(
			// We use the ABI encoding Rust library since it encodes each token as 32-bytes
			&ethabi::encode(
				&vec![
					// This is the `typeHash`
					Token::Uint(
						U256::from(keccak_256(b"Msg(string fragnovaGenesis,string op,string sender)"))
					),
					// This is the `encodeData(message)`. (https://eips.ethereum.org/EIPS/eip-712#definition-of-encodedata)
					Token::Uint(U256::from(keccak_256(&genesis_hash_string.into_bytes()))),
					Token::Uint(U256::from(keccak_256(b"link"))),
					Token::Uint(U256::from(keccak_256(&sender_string.into_bytes()))),
				]
			)
		)[..]
	].concat();

	let message = [
		format!("\x19Ethereum Signed Message:\n{}", message.len()).into_bytes(),
		message
	].concat();

	let hashed_message = keccak_256(&message);

	ethereum_account_pair.sign_prehashed(&hashed_message)
}

pub fn create_lock_signature(
	ethereum_account_pair: sp_core::ecdsa::Pair,
	lock_amount: U256,
	locktime: U256,
) -> sp_core::ecdsa::Signature {
	ethereum_account_pair.sign_prehashed(
		&keccak_256(
			&[
				b"\x19Ethereum Signed Message:\n32",
				&keccak_256(
					&[
						&b"FragLock"[..],
						&get_ethereum_public_address(&ethereum_account_pair).0[..],
						&get_ethereum_chain_id().to_be_bytes(),
						&Into::<[u8; 32]>::into(lock_amount.clone()),
						&Into::<[u8; 32]>::into(locktime.clone())
					].concat()
				)[..]
			].concat()
		)
	)
}
pub fn create_unlock_signature(
	ethereum_account_pair: sp_core::ecdsa::Pair,
	unlock_amount: U256,
) -> sp_core::ecdsa::Signature {
	ethereum_account_pair.sign_prehashed(
		&keccak_256(
			&[
				b"\x19Ethereum Signed Message:\n32",
				&keccak_256(
					&[
						&b"FragUnlock"[..],
						&get_ethereum_public_address(&ethereum_account_pair).0[..],
						&get_ethereum_chain_id().to_be_bytes(),
						&Into::<[u8; 32]>::into(unlock_amount.clone())
					].concat()
				)[..]
			].concat()
		)
	)
}

pub fn get_ethereum_public_address(
	ecdsa_pair_struct: &ecdsa::Pair,
) -> H160 {

	let ecdsa_public_struct = ecdsa_pair_struct.public();

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

	pub _ethereum_account_pair: sp_core::ecdsa::Pair,
}

impl Link {
	pub fn get_recovered_ethereum_account_id(&self) -> H160 {
		get_ethereum_public_address(&self._ethereum_account_pair)
	}
}

#[derive(Clone)]
pub struct Lock {
	pub data: EthLockUpdate<sp_core::ed25519::Public>,
	pub link: Link,

	pub _ethereum_account_pair: sp_core::ecdsa::Pair,
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
			_ethereum_account_pair: sp_core::ecdsa::Pair::from_seed(&[1u8; 32]),
		};

		let link_second = Link {
			clamor_account_id: sp_core::ed25519::Public::from_raw([2u8; 32]),
			link_signature: create_link_signature(
				sp_core::ed25519::Public::from_raw([2u8; 32]),
				sp_core::ecdsa::Pair::from_seed(&[2u8; 32]),
			),
			_ethereum_account_pair: sp_core::ecdsa::Pair::from_seed(&[2u8; 32]),
		};

		let lock = Lock {
			data: EthLockUpdate {
				public: sp_core::ed25519::Public([69u8; 32]),
				amount: U256::from(69u32),
				locktime: U256::from(1234567890),
				sender: get_ethereum_public_address(
					&link._ethereum_account_pair.clone(),
				),
				signature: create_lock_signature(
					link._ethereum_account_pair.clone(),
					U256::from(69u32),
					U256::from(1234567890),
				),
				lock: true, // yes, please LOCK it!
				block_number: 69,
			},
			link: link.clone(),
			_ethereum_account_pair: link._ethereum_account_pair.clone(),
		};

		let unlock = Unlock {
			lock: lock.clone(),
			data: EthLockUpdate {
				public: sp_core::ed25519::Public([69u8; 32]),
				amount: U256::from(0u32), // when unlocking, amount must be 0u32
				locktime: U256::from(0),  // can be whatever. It is not considered in case of unlock.
				sender: get_ethereum_public_address(
					&lock._ethereum_account_pair.clone(),
				),
				signature: create_unlock_signature(
					lock._ethereum_account_pair.clone(),
					U256::from(0u32), // when unlocking, amount must be 0u32
				),
				lock: false, // yes, please UNLOCK it!
				block_number: lock.data.block_number.clone() + 69,
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
