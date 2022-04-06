pub use crate::{mock::*, AuthData, Error, LinkedAsset, ProtoOwner, Protos, Tags, UploadAuthorities};
pub use codec::{Compact, Encode};
pub use frame_support::{assert_noop, assert_ok};
pub use pallet_detach::{
	DetachInternalData, DetachedHashes, EthereumAuthorities, SupportedChains, KEY_TYPE,
};
pub use sp_chainblocks::Hash256;
pub use sp_core::Pair;
pub use sp_io::hashing::blake2_256;
pub use sp_keystore::{testing::KeyStore, KeystoreExt};
pub use std::sync::Arc;


pub const DATA: &str = "0x0155a0e40220";
pub const PROTO_HASH: Hash256 = [
	30, 138, 136, 186, 232, 46, 112, 65, 122, 54, 110, 89, 123, 195, 7, 150, 12, 134, 10, 179, 245,
	51, 83, 227, 72, 251, 5, 148, 207, 251, 119, 59,
];
pub const PUBLIC: [u8; 33] = [
	3, 137, 65, 23, 149, 81, 74, 241, 98, 119, 101, 236, 239, 252, 189, 0, 39, 25, 240, 49, 96, 79,
	173, 215, 209, 136, 226, 220, 88, 91, 78, 26, 251,
];
pub const PUBLIC1: [u8; 32] = [
	137, 65, 23, 149, 81, 74, 241, 98, 119, 101, 236, 239, 252, 189, 0, 39, 25, 240, 49, 96, 79,
	173, 215, 209, 136, 226, 220, 88, 91, 78, 26, 251,
];

pub const PUBLIC2: [u8; 32] = [1u8; 32];

// This is equal to `Vec::<u8>::from(r#"{"name": "dummy", "description": "hello"}"#)`
pub const METADATA_DATA : [u8; 41] = [123, 34, 110, 97, 109, 101, 34, 58, 32, 34, 100, 117, 109, 109, 121, 34, 44, 32, 34, 100, 101, 115, 99, 114, 105, 112, 116, 105, 111, 110, 34, 58, 32, 34, 104, 101, 108, 108, 111, 34, 125];


pub fn generate_signature(suri: &str) -> sp_core::ecdsa::Signature {
	let pair = sp_core::ecdsa::Pair::from_string(suri, None).unwrap();
	let msg = sp_core::keccak_256(b"this should be a hashed message");

	pair.sign(&msg)
}

pub fn initial_set_up_and_get_signature(
	data: Vec<u8>,
	references: Vec<Hash256>,
	nonce: u64,
	tags: Vec<Tags>
) -> sp_core::ecdsa::Signature {
	let pair = sp_core::ecdsa::Pair::from_string("//Charlie", None).unwrap();

	let proto_hash = blake2_256(&data);
	let linked_asset: Option<LinkedAsset> = None;
	let signature: sp_core::ecdsa::Signature = pair.sign(
		&[
			&proto_hash[..],
			&references.encode(),
			&tags.encode(),
			&linked_asset.encode(),
			&nonce.encode(),
			&1.encode(),
		]
		.concat(),
	);
	assert_ok!(ProtosPallet::add_upload_auth(Origin::root(), pair.public()));
	signature
}

pub fn initial_upload_and_get_signature() -> AuthData {
	let data = DATA.as_bytes().to_vec();
	let references = vec![PROTO_HASH];
	let signature = initial_set_up_and_get_signature(data.clone(), references.clone(), 0, vec![]);
	let auth_data = AuthData { signature, block: 1 };

	assert_ok!(ProtosPallet::upload(
		Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC1)),
		auth_data.clone(),
		references,
		Vec::new(),
		None,
		None,
		data,
	));
	auth_data
}



pub fn set_authority(authority_string : &str) {
	let pair = sp_core::ecdsa::Pair::from_string(authority_string, None).unwrap();
	assert_ok!(ProtosPallet::add_upload_auth(Origin::root(), pair.public()));
}

pub fn get_ecdsa_signature_for_metadata(authority_string : &str, proto_hash: Hash256, metadata_data: Vec<u8>, nonce: u64, block_number: u32) -> sp_core::ecdsa::Signature {

	let pair = sp_core::ecdsa::Pair::from_string(authority_string, None).unwrap();

	let data_hash = blake2_256(&metadata_data);

	let signature: sp_core::ecdsa::Signature = pair.sign(
		&[
			&proto_hash[..],
			&data_hash.encode(),
			&nonce.encode(),
			&block_number.encode(),
		]
		.concat(),
	);

	signature

}