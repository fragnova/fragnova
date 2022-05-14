use crate::{
	categories::{Categories, ChainCategories},
	mock::*,
	AuthData, Error, LinkedAsset, ProtoOwner, Protos, UploadAuthorities,
};
use codec::{Compact, Encode};
use frame_support::{assert_noop, assert_ok};
use pallet_detach::{
	DetachInternalData, DetachedHashes, EthereumAuthorities, SupportedChains, KEY_TYPE,
};
use sp_clamor::Hash256;
use sp_core::Pair;
use sp_io::hashing::blake2_256;
use sp_keystore::{testing::KeyStore, KeystoreExt};
use std::sync::Arc;

fn generate_signature(suri: &str) -> sp_core::ecdsa::Signature {
	let pair = sp_core::ecdsa::Pair::from_string(suri, None).unwrap();
	let msg = sp_core::keccak_256(b"this should be a hashed message");

	pair.sign(&msg)
}

fn initial_set_up_and_get_signature(
	data: Vec<u8>,
	references: Vec<Hash256>,
	nonce: u64,
) -> sp_core::ecdsa::Signature {
	let pair = sp_core::ecdsa::Pair::from_string("//Charlie", None).unwrap();
	let categories = (Categories::Chain(ChainCategories::Generic), <Vec<Vec<u8>>>::new());

	let proto_hash = blake2_256(&data);
	let linked_asset: Option<LinkedAsset> = None;
	let signature: sp_core::ecdsa::Signature = pair.sign(
		&[
			&proto_hash[..],
			&references.encode(),
			&categories.encode(),
			&linked_asset.encode(),
			&nonce.encode(),
			&1.encode(),
		]
		.concat(),
	);
	assert_ok!(ProtosPallet::add_upload_auth(Origin::root(), pair.public()));
	signature
}

fn initial_upload_and_get_signature() -> AuthData {
	let data = DATA.as_bytes().to_vec();
	let references = vec![];
	let signature = initial_set_up_and_get_signature(data.clone(), references.clone(), 0);
	let auth_data = AuthData { signature, block: 1 };

	assert_ok!(ProtosPallet::upload(
		Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC1)),
		auth_data.clone(),
		references,
		(Categories::Chain(ChainCategories::Generic), Vec::new()),
		None,
		None,
		data,
	));
	auth_data
}

#[test]
fn add_eth_auth_should_works() {
	new_test_ext().execute_with(|| {
		let validator: sp_core::ecdsa::Public = sp_core::ecdsa::Public::from_raw(PUBLIC);
		assert_ok!(DetachPallet::add_eth_auth(Origin::root(), validator.clone()));
		assert!(EthereumAuthorities::<Test>::get().contains(&validator));
	});
}

#[test]
fn del_eth_auth_should_works() {
	new_test_ext().execute_with(|| {
		let validator: sp_core::ecdsa::Public = sp_core::ecdsa::Public::from_raw(PUBLIC);
		assert_ok!(DetachPallet::del_eth_auth(Origin::root(), validator.clone()));
		assert!(!EthereumAuthorities::<Test>::get().contains(&validator));
	});
}

#[test]
fn add_upload_auth_should_works() {
	new_test_ext().execute_with(|| {
		let validator: sp_core::ecdsa::Public = sp_core::ecdsa::Public::from_raw(PUBLIC);
		assert_ok!(ProtosPallet::add_upload_auth(Origin::root(), validator.clone()));
		assert!(UploadAuthorities::<Test>::get().contains(&validator));
	});
}

#[test]
fn del_upload_auth_should_works() {
	new_test_ext().execute_with(|| {
		let validator: sp_core::ecdsa::Public = sp_core::ecdsa::Public::from_raw(PUBLIC);
		assert_ok!(ProtosPallet::del_upload_auth(Origin::root(), validator.clone()));
		assert!(!UploadAuthorities::<Test>::get().contains(&validator));
	});
}

#[test]
fn upload_should_works() {
	new_test_ext().execute_with(|| {
		let data = DATA.as_bytes().to_vec();
		let references = vec![];

		let signature = initial_set_up_and_get_signature(data.clone(), references.clone(), 0);

		let auth_data = AuthData { signature, block: 1 };

		assert_ok!(ProtosPallet::upload(
			Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC1)),
			auth_data,
			references,
			(Categories::Chain(ChainCategories::Generic), Vec::new()),
			None,
			None,
			data,
		));

		assert!(<Protos<Test>>::contains_key(PROTO_HASH));
	});
}

#[test]
fn upload_should_not_works_if_proto_hash_exists() {
	new_test_ext().execute_with(|| {
		let data = DATA.as_bytes().to_vec();
		initial_upload_and_get_signature();
		let references = vec![];

		let signature = initial_set_up_and_get_signature(data.clone(), references.clone(), 1);
		let auth_data = AuthData { signature, block: 1 };
		assert_noop!(
			ProtosPallet::upload(
				Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC1)),
				auth_data,
				references,
				(Categories::Chain(ChainCategories::Generic), Vec::new()),
				None,
				None,
				data,
			),
			Error::<Test>::ProtoExists
		);
	});
}

#[test]
fn upload_proto_should_not_work_if_not_verified() {
	new_test_ext().execute_with(|| {
		let immutable_data = "0x0155a0e40220".as_bytes().to_vec();
		let references = vec![];
		let signature: sp_core::ecdsa::Signature = generate_signature("//Alice");
		let auth_data = AuthData { signature, block: 1 };

		assert_noop!(
			ProtosPallet::upload(
				Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC1)),
				auth_data,
				references,
				(Categories::Chain(ChainCategories::Generic), Vec::new()),
				None,
				None,
				immutable_data,
			),
			Error::<Test>::VerificationFailed
		);
	});
}

#[test]
fn patch_should_works() {
	new_test_ext().execute_with(|| {
		let pair = sp_core::ecdsa::Pair::from_string("//Alice", None).unwrap();
		let immutable_data = DATA.as_bytes().to_vec();
		initial_upload_and_get_signature();

		let data = immutable_data.clone();
		let proto_hash = blake2_256(&immutable_data);
		let data_hash = blake2_256(&data);
		let nonce: u64 = 1;
		let signature: sp_core::ecdsa::Signature =
			pair.sign(&[&proto_hash[..], &data_hash[..], &nonce.encode(), &1.encode()].concat());
		assert_ok!(ProtosPallet::add_upload_auth(Origin::root(), pair.public()));

		let auth_data = AuthData { signature, block: 1 };
		assert_ok!(ProtosPallet::patch(
			Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC1)),
			auth_data,
			proto_hash,
			Some(Compact(123)),
			data,
		));

		assert_eq!(<Protos<Test>>::get(proto_hash).unwrap().include_cost, Some(Compact(123)))
	});
}

#[test]
fn patch_proto_should_not_work_if_user_is_unauthorized() {
	new_test_ext().execute_with(|| {
		let data = DATA.as_bytes().to_vec();
		let auth_data = initial_upload_and_get_signature();

		let (pair, _) = sp_core::ed25519::Pair::generate();

		assert_noop!(
			ProtosPallet::patch(
				Origin::signed(pair.public()),
				auth_data,
				PROTO_HASH,
				Some(Compact(123)),
				data,
			),
			Error::<Test>::Unauthorized
		);
	});
}

#[test]
fn patch_proto_should_not_work_if_proto_not_found() {
	new_test_ext().execute_with(|| {
		let immutable_data = DATA.as_bytes().to_vec();

		let proto_hash = blake2_256(immutable_data.as_slice());
		let signature: sp_core::ecdsa::Signature = generate_signature("//Alice");
		let auth_data = AuthData { signature, block: 1 };
		assert_noop!(
			ProtosPallet::patch(
				Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC1)),
				auth_data,
				proto_hash,
				Some(Compact(123)),
				immutable_data,
			),
			Error::<Test>::ProtoNotFound
		);
	});
}

#[test]
fn patch_should_not_work_if_not_verified() {
	new_test_ext().execute_with(|| {
		let data = DATA.as_bytes().to_vec();
		initial_upload_and_get_signature();

		let signature: sp_core::ecdsa::Signature = generate_signature("//Bob");
		let auth_data = AuthData { signature, block: 1 };

		assert_noop!(
			ProtosPallet::patch(
				Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC1)),
				auth_data,
				PROTO_HASH,
				Some(Compact(123)),
				data,
			),
			Error::<Test>::VerificationFailed
		);
	});
}

#[test]
fn patch_should_not_work_if_detached() {
	let keystore = KeyStore::new();
	let mut t = new_test_ext();

	t.register_extension(KeystoreExt(Arc::new(keystore)));
	t.execute_with(|| {
		let pair = sp_core::ed25519::Pair::from_string("//Alice", None).unwrap();
		let data = DATA.as_bytes().to_vec();
		let auth_data = initial_upload_and_get_signature();

		sp_io::crypto::ecdsa_generate(KEY_TYPE, None);
		let keys = sp_io::crypto::ecdsa_public_keys(KEY_TYPE);

		<EthereumAuthorities<Test>>::mutate(|authorities| {
			authorities.insert(keys.get(0).unwrap().clone());
		});
		assert_ok!(ProtosPallet::detach(
			Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC1)),
			PROTO_HASH,
			SupportedChains::EthereumMainnet,
			pair.to_raw_vec()
		));

		let detach_data = DetachInternalData {
			public: sp_core::ed25519::Public::from_raw(PUBLIC1),
			hash: PROTO_HASH,
			remote_signature: vec![],
			target_account: vec![],
			target_chain: SupportedChains::EthereumGoerli,
			nonce: 1,
		};

		assert_ok!(DetachPallet::internal_finalize_detach(
			Origin::none(),
			detach_data,
			pair.sign(DATA.as_bytes())
		));

		assert_noop!(
			ProtosPallet::patch(
				Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC1)),
				auth_data,
				PROTO_HASH,
				Some(Compact(123)),
				data,
			),
			Error::<Test>::Detached
		);
	});
}

#[test]
fn detach_proto_should_not_work_if_user_is_unauthorized() {
	new_test_ext().execute_with(|| {
		initial_upload_and_get_signature();

		let (pair, _) = sp_core::ed25519::Pair::generate();

		assert_noop!(
			ProtosPallet::detach(
				Origin::signed(pair.public()),
				PROTO_HASH,
				SupportedChains::EthereumMainnet,
				pair.to_raw_vec()
			),
			Error::<Test>::Unauthorized
		);
	});
}

#[test]
fn detach_proto_should_not_work_if_proto_not_found() {
	new_test_ext().execute_with(|| {
		let who: sp_core::ed25519::Public = sp_core::ed25519::Public::from_raw(PUBLIC1);
		let (pair, _) = sp_core::ed25519::Pair::generate();

		assert_noop!(
			ProtosPallet::detach(
				Origin::signed(who),
				PROTO_HASH,
				SupportedChains::EthereumMainnet,
				pair.to_raw_vec()
			),
			Error::<Test>::ProtoNotFound
		);
	});
}

#[test]
fn detach_should_work() {
	let keystore = KeyStore::new();
	let mut t = new_test_ext();

	t.register_extension(KeystoreExt(Arc::new(keystore)));
	t.execute_with(|| {
		let pair = sp_core::ecdsa::Pair::from_string("//Alice", None).unwrap();
		initial_upload_and_get_signature();

		sp_io::crypto::ecdsa_generate(KEY_TYPE, None);
		let keys = sp_io::crypto::ecdsa_public_keys(KEY_TYPE);

		<EthereumAuthorities<Test>>::mutate(|authorities| {
			authorities.insert(keys.get(0).unwrap().clone());
		});
		assert_ok!(ProtosPallet::detach(
			Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC1)),
			PROTO_HASH,
			SupportedChains::EthereumMainnet,
			pair.to_raw_vec()
		));
	});
}

#[test]
fn transfer_should_works() {
	new_test_ext().execute_with(|| {
		initial_upload_and_get_signature();

		let (pair, _) = sp_core::ed25519::Pair::generate();
		assert_ok!(ProtosPallet::transfer(
			Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC1)),
			PROTO_HASH,
			pair.public()
		));

		assert_eq!(<Protos<Test>>::get(PROTO_HASH).unwrap().owner, ProtoOwner::User(pair.public()));
	});
}

#[test]
fn transfer_should_not_work_if_proto_not_found() {
	new_test_ext().execute_with(|| {
		let (pair, _) = sp_core::ed25519::Pair::generate();

		assert_noop!(
			ProtosPallet::transfer(
				Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC1)),
				PROTO_HASH,
				pair.public()
			),
			Error::<Test>::ProtoNotFound
		);
	});
}

#[test]
fn transfer_should_not_work_if_user_is_unauthorized() {
	new_test_ext().execute_with(|| {
		initial_upload_and_get_signature();

		let (pair, _) = sp_core::ed25519::Pair::generate();

		assert_noop!(
			ProtosPallet::transfer(Origin::signed(pair.public()), PROTO_HASH, pair.public()),
			Error::<Test>::Unauthorized
		);
	});
}

#[test]
fn transfer_should_not_work_if_detached() {
	let keystore = KeyStore::new();
	let mut t = new_test_ext();

	t.register_extension(KeystoreExt(Arc::new(keystore)));
	t.execute_with(|| {
		let pair = sp_core::ed25519::Pair::from_string("//Alice", None).unwrap();
		initial_upload_and_get_signature();
		sp_io::crypto::ecdsa_generate(KEY_TYPE, None);
		let keys = sp_io::crypto::ecdsa_public_keys(KEY_TYPE);

		<EthereumAuthorities<Test>>::mutate(|authorities| {
			authorities.insert(keys.get(0).unwrap().clone());
		});
		let detach_data = DetachInternalData {
			public: sp_core::ed25519::Public::from_raw(PUBLIC1),
			hash: PROTO_HASH,
			remote_signature: vec![],
			target_account: vec![],
			target_chain: SupportedChains::EthereumGoerli,
			nonce: 1,
		};

		assert_ok!(DetachPallet::internal_finalize_detach(
			Origin::none(),
			detach_data,
			pair.sign(DATA.as_bytes())
		));

		let (pair, _) = sp_core::ed25519::Pair::generate();
		assert_noop!(
			ProtosPallet::transfer(
				Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC1)),
				PROTO_HASH,
				pair.public()
			),
			Error::<Test>::Detached
		);
	});
}

#[test]
fn internal_finalize_detach_should_works() {
	new_test_ext().execute_with(|| {
		let pair = sp_core::ed25519::Pair::from_string("//Alice", None).unwrap();

		let detach_data = DetachInternalData {
			public: sp_core::ed25519::Public::from_raw(PUBLIC1),
			hash: PROTO_HASH,
			remote_signature: vec![],
			target_account: vec![],
			target_chain: SupportedChains::EthereumGoerli,
			nonce: 1,
		};

		assert_ok!(DetachPallet::internal_finalize_detach(
			Origin::none(),
			detach_data,
			pair.sign(DATA.as_bytes())
		));

		assert!(<DetachedHashes<Test>>::contains_key(PROTO_HASH));
	});
}
