use crate::{
	mock::*, Error, EthereumAuthorities, Fragments, IncludeInfo, SupportedChains,
	UploadAuthorities, KEY_TYPE,
};
use codec::{Compact, Encode};
use frame_support::{assert_noop, assert_ok};
use sp_chainblocks::Hash256;
use sp_core::Pair;
use sp_io::{crypto as Crypto, hashing::blake2_256};
use sp_keystore::{testing::KeyStore, KeystoreExt};
use std::sync::Arc;

#[test]
fn add_eth_auth_should_works() {
	new_test_ext().execute_with(|| {
		let validator: sp_core::ecdsa::Public = Default::default();
		assert_ok!(FragmentsPallet::add_eth_auth(Origin::root(), validator.clone()));
		assert!(EthereumAuthorities::<Test>::get().contains(&validator));
	});
}

#[test]
fn del_eth_auth_should_works() {
	new_test_ext().execute_with(|| {
		let validator: sp_core::ecdsa::Public = Default::default();
		assert_ok!(FragmentsPallet::del_eth_auth(Origin::root(), validator.clone()));
		assert!(!EthereumAuthorities::<Test>::get().contains(&validator));
	});
}

#[test]
fn add_upload_auth_should_works() {
	new_test_ext().execute_with(|| {
		let who: sp_core::sr25519::Public = Default::default();
		let validator: sp_core::ecdsa::Public = Default::default();
		assert_ok!(FragmentsPallet::add_upload_auth(Origin::root(), validator.clone(), who));
		assert!(UploadAuthorities::<Test>::contains_key(&validator));
	});
}

#[test]
fn del_upload_auth_should_works() {
	new_test_ext().execute_with(|| {
		let validator: sp_core::ecdsa::Public = Default::default();
		assert_ok!(FragmentsPallet::del_upload_auth(Origin::root(), validator.clone()));
		assert!(!UploadAuthorities::<Test>::contains_key(&validator));
	});
}

#[test]
fn upload_should_works() {
	new_test_ext().execute_with(|| {
		let fragment_hash: Hash256 = [
			30, 138, 136, 186, 232, 46, 112, 65, 122, 54, 110, 89, 123, 195, 7, 150, 12, 134, 10,
			179, 245, 51, 83, 227, 72, 251, 5, 148, 207, 251, 119, 59,
		];
		let data = "0x0155a0e40220".as_bytes().to_vec();
		let include_info =
			IncludeInfo { fragment_hash, mutable_index: Compact(1), staked_amount: Compact(1) };
		let references = vec![include_info];

		let suri = "//Alice";
		let pair = sp_core::ecdsa::Pair::from_string(suri, None).unwrap();

		let fragment_hash = blake2_256(&data);
		let signature_hash = blake2_256(&[&fragment_hash[..], &references.encode()].concat());

		let who: sp_core::sr25519::Public = Default::default();
		let signature: sp_core::ecdsa::Signature = pair.sign(&signature_hash);

		let recover = Crypto::secp256k1_ecdsa_recover_compressed(&signature.0, &signature_hash)
			.ok()
			.unwrap();
		let recover = sp_core::ecdsa::Public(recover);

		assert_ok!(FragmentsPallet::add_upload_auth(Origin::root(), recover, who));

		assert_ok!(FragmentsPallet::upload(
			Origin::signed(Default::default()),
			references,
			None,
			signature,
			data,
		));

		assert!(<Fragments<Test>>::contains_key(fragment_hash));
	});
}

#[test]
fn upload_should_not_works_if_fragment_hash_exists() {
	new_test_ext().execute_with(|| {
		let fragment_hash: Hash256 = [
			30, 138, 136, 186, 232, 46, 112, 65, 122, 54, 110, 89, 123, 195, 7, 150, 12, 134, 10,
			179, 245, 51, 83, 227, 72, 251, 5, 148, 207, 251, 119, 59,
		];
		let immutable_data = "0x0155a0e40220".as_bytes().to_vec();
		let include_info =
			IncludeInfo { fragment_hash, mutable_index: Compact(1), staked_amount: Compact(1) };
		let references = vec![include_info];

		let suri = "//Alice";
		let pair = sp_core::ecdsa::Pair::from_string(suri, None).unwrap();
		let who: sp_core::sr25519::Public = Default::default();

		let fragment_hash = blake2_256(&immutable_data);
		let signature_hash = blake2_256(&[&fragment_hash[..], &references.encode()].concat());

		let signature: sp_core::ecdsa::Signature = pair.sign(&signature_hash);

		let recover = Crypto::secp256k1_ecdsa_recover_compressed(&signature.0, &signature_hash)
			.ok()
			.unwrap();
		let recover = sp_core::ecdsa::Public(recover);

		assert_ok!(FragmentsPallet::add_upload_auth(Origin::root(), recover, who));

		assert_ok!(FragmentsPallet::upload(
			Origin::signed(Default::default()),
			references.clone(),
			None,
			signature.clone(),
			immutable_data.clone(),
		));

		assert!(<Fragments<Test>>::contains_key(fragment_hash));

		assert_noop!(
			FragmentsPallet::upload(
				Origin::signed(Default::default()),
				references,
				None,
				signature,
				immutable_data,
			),
			Error::<Test>::FragmentExists
		);
	});
}

#[test]
fn upload_fragment_should_not_work_if_not_verified() {
	new_test_ext().execute_with(|| {
		let fragment_hash: Hash256 = [
			30, 138, 136, 186, 232, 46, 112, 65, 122, 54, 110, 89, 123, 195, 7, 150, 12, 134, 10,
			179, 245, 51, 83, 227, 72, 251, 5, 148, 207, 251, 119, 59,
		];
		let immutable_data = "0x0155a0e40220".as_bytes().to_vec();
		let include_info =
			IncludeInfo { fragment_hash, mutable_index: Compact(1), staked_amount: Compact(1) };
		let references = vec![include_info];

		let suri = "//Alice";
		let pair = sp_core::ecdsa::Pair::from_string(suri, None).unwrap();
		let msg = sp_core::keccak_256(b"this should be a hashed message");

		let signature: sp_core::ecdsa::Signature = pair.sign(&msg);

		assert_noop!(
			FragmentsPallet::upload(
				Origin::signed(Default::default()),
				references,
				None,
				signature,
				immutable_data,
			),
			Error::<Test>::SignatureVerificationFailed
		);
	});
}

#[test]
fn update_should_works() {
	new_test_ext().execute_with(|| {
		let fragment_hash: Hash256 = [
			30, 138, 136, 186, 232, 46, 112, 65, 122, 54, 110, 89, 123, 195, 7, 150, 12, 134, 10,
			179, 245, 51, 83, 227, 72, 251, 5, 148, 207, 251, 119, 59,
		];
		let immutable_data = "0x0155a0e40220".as_bytes().to_vec();
		let include_info =
			IncludeInfo { fragment_hash, mutable_index: Compact(1), staked_amount: Compact(1) };
		let references = vec![include_info];

		let suri = "//Alice";
		let pair = sp_core::ecdsa::Pair::from_string(suri, None).unwrap();
		let who: sp_core::sr25519::Public = Default::default();

		let fragment_hash = blake2_256(&immutable_data);
		let signature_hash = blake2_256(&[&fragment_hash[..], &references.encode()].concat());

		let signature: sp_core::ecdsa::Signature = pair.sign(&signature_hash);

		let recover = Crypto::secp256k1_ecdsa_recover_compressed(&signature.0, &signature_hash)
			.ok()
			.unwrap();
		let recover = sp_core::ecdsa::Public(recover);

		assert_ok!(FragmentsPallet::add_upload_auth(Origin::root(), recover, who));

		assert_ok!(FragmentsPallet::upload(
			Origin::signed(Default::default()),
			references,
			None,
			signature,
			immutable_data,
		));

		let signature: sp_core::ecdsa::Signature = pair.sign(&fragment_hash);

		assert_ok!(FragmentsPallet::update(
			Origin::signed(Default::default()),
			fragment_hash,
			Some(Compact(123)),
			signature,
			Some("0x0155a0e40220".as_bytes().to_vec()),
		));

		assert_eq!(<Fragments<Test>>::get(fragment_hash).unwrap().include_cost, Some(Compact(123)))
	});
}

#[test]
fn update_fragment_should_not_work_if_user_is_unauthorized() {
	new_test_ext().execute_with(|| {
		let fragment_hash: Hash256 = [
			30, 138, 136, 186, 232, 46, 112, 65, 122, 54, 110, 89, 123, 195, 7, 150, 12, 134, 10,
			179, 245, 51, 83, 227, 72, 251, 5, 148, 207, 251, 119, 59,
		];
		let immutable_data = "0x0155a0e40220".as_bytes().to_vec();
		let include_info =
			IncludeInfo { fragment_hash, mutable_index: Compact(1), staked_amount: Compact(1) };
		let references = vec![include_info];

		let suri = "//Alice";
		let pair = sp_core::ecdsa::Pair::from_string(suri, None).unwrap();
		let msg = sp_core::keccak_256(b"this should be a hashed message");

		let who: sp_core::sr25519::Public = Default::default();
		let signature: sp_core::ecdsa::Signature = pair.sign(&msg);
		let payload = [immutable_data.clone(), references.encode()].concat();

		let fragment_hash = blake2_256(payload.as_slice());

		let recover = Crypto::secp256k1_ecdsa_recover_compressed(&signature.0, &fragment_hash)
			.ok()
			.unwrap();
		let recover = sp_core::ecdsa::Public(recover);

		assert_ok!(FragmentsPallet::add_upload_auth(Origin::root(), recover, who));

		assert_ok!(FragmentsPallet::upload(
			Origin::signed(Default::default()),
			references,
			None,
			signature.clone(),
			immutable_data,
		));

		let (pair, _) = sp_core::sr25519::Pair::generate();

		assert_noop!(
			FragmentsPallet::update(
				Origin::signed(pair.public()),
				fragment_hash,
				Some(Compact(123)),
				signature,
				Some("0x0155a0e40220".as_bytes().to_vec()),
			),
			Error::<Test>::Unauthorized
		);
	});
}

#[test]
fn update_fragment_should_not_work_if_fragment_not_found() {
	new_test_ext().execute_with(|| {
		let immutable_data = "0x0155a0e40220".as_bytes().to_vec();

		let fragment_hash = blake2_256(immutable_data.as_slice());
		let suri = "//Alice";
		let pair = sp_core::ecdsa::Pair::from_string(suri, None).unwrap();
		let msg = sp_core::keccak_256(b"this should be a hashed message");

		let signature: sp_core::ecdsa::Signature = pair.sign(&msg);
		assert_noop!(
			FragmentsPallet::update(
				Origin::signed(Default::default()),
				fragment_hash,
				Some(Compact(123)),
				signature,
				Some("0x0155a0e40220".as_bytes().to_vec()),
			),
			Error::<Test>::FragmentNotFound
		);
	});
}

#[test]
fn update_should_not_work_if_not_verified() {
	new_test_ext().execute_with(|| {
		let fragment_hash: Hash256 = [
			30, 138, 136, 186, 232, 46, 112, 65, 122, 54, 110, 89, 123, 195, 7, 150, 12, 134, 10,
			179, 245, 51, 83, 227, 72, 251, 5, 148, 207, 251, 119, 59,
		];
		let immutable_data = "0x0155a0e40220".as_bytes().to_vec();
		let include_info =
			IncludeInfo { fragment_hash, mutable_index: Compact(1), staked_amount: Compact(1) };
		let references = vec![include_info];

		let suri = "//Alice";
		let pair = sp_core::ecdsa::Pair::from_string(suri, None).unwrap();
		let msg = sp_core::keccak_256(b"this should be a hashed message");

		let who: sp_core::sr25519::Public = Default::default();
		let signature: sp_core::ecdsa::Signature = pair.sign(&msg);
		let payload = [immutable_data.clone(), references.encode()].concat();

		let fragment_hash = blake2_256(payload.as_slice());

		let recover = Crypto::secp256k1_ecdsa_recover_compressed(&signature.0, &fragment_hash)
			.ok()
			.unwrap();
		let recover = sp_core::ecdsa::Public(recover);

		assert_ok!(FragmentsPallet::add_upload_auth(Origin::root(), recover, who));

		assert_ok!(FragmentsPallet::upload(
			Origin::signed(Default::default()),
			references,
			None,
			signature.clone(),
			immutable_data,
		));
		let suri = "//Bob";
		let pair = sp_core::ecdsa::Pair::from_string(suri, None).unwrap();
		let msg = sp_core::keccak_256(b"this should be a hashed message");

		let signature: sp_core::ecdsa::Signature = pair.sign(&msg);

		assert_noop!(
			FragmentsPallet::update(
				Origin::signed(Default::default()),
				fragment_hash,
				Some(Compact(123)),
				signature,
				Some("0x0155a0e40220".as_bytes().to_vec()),
			),
			Error::<Test>::SignatureVerificationFailed
		);
	});
}

#[test]
fn update_should_not_work_if_detached() {
	let keystore = KeyStore::new();
	let mut t = new_test_ext();

	t.register_extension(KeystoreExt(Arc::new(keystore)));
	t.execute_with(|| {
		let fragment_hash: Hash256 = [
			30, 138, 136, 186, 232, 46, 112, 65, 122, 54, 110, 89, 123, 195, 7, 150, 12, 134, 10,
			179, 245, 51, 83, 227, 72, 251, 5, 148, 207, 251, 119, 59,
		];
		let immutable_data = "0x0155a0e40220".as_bytes().to_vec();
		let include_info =
			IncludeInfo { fragment_hash, mutable_index: Compact(1), staked_amount: Compact(1) };
		let references = vec![include_info];

		let suri = "//Alice";
		let pair = sp_core::ecdsa::Pair::from_string(suri, None).unwrap();
		let msg = sp_core::keccak_256(b"this should be a hashed message");

		let who: sp_core::sr25519::Public = Default::default();
		let signature: sp_core::ecdsa::Signature = pair.sign(&msg);
		let payload = [immutable_data.clone(), references.encode()].concat();

		let fragment_hash = blake2_256(payload.as_slice());

		let recover = Crypto::secp256k1_ecdsa_recover_compressed(&signature.0, &fragment_hash)
			.ok()
			.unwrap();
		let recover = sp_core::ecdsa::Public(recover);

		assert_ok!(FragmentsPallet::add_upload_auth(Origin::root(), recover, who));

		assert_ok!(FragmentsPallet::upload(
			Origin::signed(Default::default()),
			references,
			None,
			signature.clone(),
			immutable_data,
		));

		sp_io::crypto::ecdsa_generate(KEY_TYPE, None);
		let keys = sp_io::crypto::ecdsa_public_keys(KEY_TYPE);

		<EthereumAuthorities<Test>>::mutate(|authorities| {
			authorities.insert(keys.get(0).unwrap().clone());
		});
		assert_ok!(FragmentsPallet::detach(
			Origin::signed(Default::default()),
			fragment_hash,
			SupportedChains::EthereumMainnet,
			pair.to_raw_vec()
		));

		assert_noop!(
			FragmentsPallet::update(
				Origin::signed(Default::default()),
				fragment_hash,
				Some(Compact(123)),
				signature,
				Some("0x0155a0e40220".as_bytes().to_vec()),
			),
			Error::<Test>::FragmentDetached
		);
	});
}

#[test]
fn detach_should_not_work_if_no_validator() {
	let keystore = KeyStore::new();
	let mut t = new_test_ext();

	t.register_extension(KeystoreExt(Arc::new(keystore)));
	t.execute_with(|| {
		let fragment_hash: Hash256 = [
			30, 138, 136, 186, 232, 46, 112, 65, 122, 54, 110, 89, 123, 195, 7, 150, 12, 134, 10,
			179, 245, 51, 83, 227, 72, 251, 5, 148, 207, 251, 119, 59,
		];
		let immutable_data = "0x0155a0e40220".as_bytes().to_vec();
		let include_info =
			IncludeInfo { fragment_hash, mutable_index: Compact(1), staked_amount: Compact(1) };
		let references = vec![include_info];

		let suri = "//Alice";
		let pair = sp_core::ecdsa::Pair::from_string(suri, None).unwrap();
		let msg = sp_core::keccak_256(b"this should be a hashed message");

		let who: sp_core::sr25519::Public = Default::default();
		let signature: sp_core::ecdsa::Signature = pair.sign(&msg);
		let payload = [immutable_data.clone(), references.encode()].concat();

		let fragment_hash = blake2_256(payload.as_slice());

		let recover = Crypto::secp256k1_ecdsa_recover_compressed(&signature.0, &fragment_hash)
			.ok()
			.unwrap();
		let recover = sp_core::ecdsa::Public(recover);

		assert_ok!(FragmentsPallet::add_upload_auth(Origin::root(), recover, who));

		assert_ok!(FragmentsPallet::upload(
			Origin::signed(Default::default()),
			references,
			None,
			signature.clone(),
			immutable_data,
		));
		assert_noop!(
			FragmentsPallet::detach(
				Origin::signed(Default::default()),
				fragment_hash,
				SupportedChains::EthereumMainnet,
				pair.to_raw_vec()
			),
			Error::<Test>::NoValidator
		);
	});
}

#[test]
fn detach_fragment_should_not_work_if_user_is_unauthorized() {
	new_test_ext().execute_with(|| {
		let fragment_hash: Hash256 = [
			30, 138, 136, 186, 232, 46, 112, 65, 122, 54, 110, 89, 123, 195, 7, 150, 12, 134, 10,
			179, 245, 51, 83, 227, 72, 251, 5, 148, 207, 251, 119, 59,
		];
		let immutable_data = "0x0155a0e40220".as_bytes().to_vec();
		let include_info =
			IncludeInfo { fragment_hash, mutable_index: Compact(1), staked_amount: Compact(1) };
		let references = vec![include_info];

		let suri = "//Alice";
		let pair = sp_core::ecdsa::Pair::from_string(suri, None).unwrap();
		let msg = sp_core::keccak_256(b"this should be a hashed message");

		let who: sp_core::sr25519::Public = Default::default();
		let signature: sp_core::ecdsa::Signature = pair.sign(&msg);
		let payload = [immutable_data.clone(), references.encode()].concat();

		let fragment_hash = blake2_256(payload.as_slice());

		let recover = Crypto::secp256k1_ecdsa_recover_compressed(&signature.0, &fragment_hash)
			.ok()
			.unwrap();
		let recover = sp_core::ecdsa::Public(recover);

		assert_ok!(FragmentsPallet::add_upload_auth(Origin::root(), recover, who));

		assert_ok!(FragmentsPallet::upload(
			Origin::signed(Default::default()),
			references,
			None,
			signature.clone(),
			immutable_data,
		));

		let (pair, _) = sp_core::sr25519::Pair::generate();

		assert_noop!(
			FragmentsPallet::detach(
				Origin::signed(pair.public()),
				fragment_hash,
				SupportedChains::EthereumMainnet,
				pair.to_raw_vec()
			),
			Error::<Test>::Unauthorized
		);
	});
}

#[test]
fn detach_fragment_should_not_work_if_fragment_not_found() {
	new_test_ext().execute_with(|| {
		let immutable_data = "0x0155a0e40220".as_bytes().to_vec();
		let who: sp_core::sr25519::Public = Default::default();

		let fragment_hash = blake2_256(immutable_data.as_slice());
		let (pair, _) = sp_core::sr25519::Pair::generate();

		assert_noop!(
			FragmentsPallet::detach(
				Origin::signed(who),
				fragment_hash,
				SupportedChains::EthereumMainnet,
				pair.to_raw_vec()
			),
			Error::<Test>::FragmentNotFound
		);
	});
}

#[test]
fn detach_should_work() {
	let keystore = KeyStore::new();
	let mut t = new_test_ext();

	t.register_extension(KeystoreExt(Arc::new(keystore)));
	t.execute_with(|| {
		let fragment_hash: Hash256 = [
			30, 138, 136, 186, 232, 46, 112, 65, 122, 54, 110, 89, 123, 195, 7, 150, 12, 134, 10,
			179, 245, 51, 83, 227, 72, 251, 5, 148, 207, 251, 119, 59,
		];
		let immutable_data = "0x0155a0e40220".as_bytes().to_vec();
		let include_info =
			IncludeInfo { fragment_hash, mutable_index: Compact(1), staked_amount: Compact(1) };
		let references = vec![include_info];

		let suri = "//Alice";
		let pair = sp_core::ecdsa::Pair::from_string(suri, None).unwrap();
		let msg = sp_core::keccak_256(b"this should be a hashed message");

		let who: sp_core::sr25519::Public = Default::default();
		let signature: sp_core::ecdsa::Signature = pair.sign(&msg);
		let payload = [immutable_data.clone(), references.encode()].concat();

		let fragment_hash = blake2_256(payload.as_slice());

		let recover = Crypto::secp256k1_ecdsa_recover_compressed(&signature.0, &fragment_hash)
			.ok()
			.unwrap();
		let recover = sp_core::ecdsa::Public(recover);

		assert_ok!(FragmentsPallet::add_upload_auth(Origin::root(), recover, who));

		assert_ok!(FragmentsPallet::upload(
			Origin::signed(Default::default()),
			references,
			None,
			signature.clone(),
			immutable_data,
		));

		sp_io::crypto::ecdsa_generate(KEY_TYPE, None);
		let keys = sp_io::crypto::ecdsa_public_keys(KEY_TYPE);

		<EthereumAuthorities<Test>>::mutate(|authorities| {
			authorities.insert(keys.get(0).unwrap().clone());
		});
		assert_ok!(FragmentsPallet::detach(
			Origin::signed(Default::default()),
			fragment_hash,
			SupportedChains::EthereumMainnet,
			pair.to_raw_vec()
		));
	});
}

#[test]
fn transfer_should_works() {
	new_test_ext().execute_with(|| {
		let fragment_hash: Hash256 = [
			30, 138, 136, 186, 232, 46, 112, 65, 122, 54, 110, 89, 123, 195, 7, 150, 12, 134, 10,
			179, 245, 51, 83, 227, 72, 251, 5, 148, 207, 251, 119, 59,
		];
		let immutable_data = "0x0155a0e40220".as_bytes().to_vec();
		let include_info =
			IncludeInfo { fragment_hash, mutable_index: Compact(1), staked_amount: Compact(1) };
		let references = vec![include_info];

		let suri = "//Alice";
		let pair = sp_core::ecdsa::Pair::from_string(suri, None).unwrap();
		let msg = sp_core::keccak_256(b"this should be a hashed message");

		let who: sp_core::sr25519::Public = Default::default();
		let signature: sp_core::ecdsa::Signature = pair.sign(&msg);
		let payload = [immutable_data.clone(), references.encode()].concat();

		let fragment_hash = blake2_256(payload.as_slice());

		let recover = Crypto::secp256k1_ecdsa_recover_compressed(&signature.0, &fragment_hash)
			.ok()
			.unwrap();
		let recover = sp_core::ecdsa::Public(recover);

		assert_ok!(FragmentsPallet::add_upload_auth(Origin::root(), recover, who));

		assert_ok!(FragmentsPallet::upload(
			Origin::signed(Default::default()),
			references,
			None,
			signature,
			immutable_data,
		));

		let (pair, _) = sp_core::sr25519::Pair::generate();
		assert_ok!(FragmentsPallet::transfer(
			Origin::signed(Default::default()),
			fragment_hash,
			pair.public()
		));

		assert_eq!(<Fragments<Test>>::get(fragment_hash).unwrap().owner, pair.public())
	});
}

#[test]
fn transfer_should_not_work_if_fragment_not_found() {
	new_test_ext().execute_with(|| {
		let immutable_data = "0x0155a0e40220".as_bytes().to_vec();

		let fragment_hash = blake2_256(immutable_data.as_slice());
		let (pair, _) = sp_core::sr25519::Pair::generate();

		assert_noop!(
			FragmentsPallet::transfer(
				Origin::signed(Default::default()),
				fragment_hash,
				pair.public()
			),
			Error::<Test>::FragmentNotFound
		);
	});
}

#[test]
fn transfer_should_not_work_if_user_is_unauthorized() {
	new_test_ext().execute_with(|| {
		let fragment_hash: Hash256 = [
			30, 138, 136, 186, 232, 46, 112, 65, 122, 54, 110, 89, 123, 195, 7, 150, 12, 134, 10,
			179, 245, 51, 83, 227, 72, 251, 5, 148, 207, 251, 119, 59,
		];
		let immutable_data = "0x0155a0e40220".as_bytes().to_vec();
		let include_info =
			IncludeInfo { fragment_hash, mutable_index: Compact(1), staked_amount: Compact(1) };
		let references = vec![include_info];

		let suri = "//Alice";
		let pair = sp_core::ecdsa::Pair::from_string(suri, None).unwrap();
		let msg = sp_core::keccak_256(b"this should be a hashed message");

		let who: sp_core::sr25519::Public = Default::default();
		let signature: sp_core::ecdsa::Signature = pair.sign(&msg);
		let payload = [immutable_data.clone(), references.encode()].concat();

		let fragment_hash = blake2_256(payload.as_slice());

		let recover = Crypto::secp256k1_ecdsa_recover_compressed(&signature.0, &fragment_hash)
			.ok()
			.unwrap();
		let recover = sp_core::ecdsa::Public(recover);

		assert_ok!(FragmentsPallet::add_upload_auth(Origin::root(), recover, who));

		assert_ok!(FragmentsPallet::upload(
			Origin::signed(Default::default()),
			references,
			None,
			signature.clone(),
			immutable_data,
		));

		let (pair, _) = sp_core::sr25519::Pair::generate();

		assert_noop!(
			FragmentsPallet::transfer(Origin::signed(pair.public()), fragment_hash, pair.public()),
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
		let fragment_hash: Hash256 = [
			30, 138, 136, 186, 232, 46, 112, 65, 122, 54, 110, 89, 123, 195, 7, 150, 12, 134, 10,
			179, 245, 51, 83, 227, 72, 251, 5, 148, 207, 251, 119, 59,
		];
		let immutable_data = "0x0155a0e40220".as_bytes().to_vec();
		let include_info =
			IncludeInfo { fragment_hash, mutable_index: Compact(1), staked_amount: Compact(1) };
		let references = vec![include_info];

		let suri = "//Alice";
		let pair = sp_core::ecdsa::Pair::from_string(suri, None).unwrap();
		let msg = sp_core::keccak_256(b"this should be a hashed message");

		let who: sp_core::sr25519::Public = Default::default();
		let signature: sp_core::ecdsa::Signature = pair.sign(&msg);
		let payload = [immutable_data.clone(), references.encode()].concat();

		let fragment_hash = blake2_256(payload.as_slice());

		let recover = Crypto::secp256k1_ecdsa_recover_compressed(&signature.0, &fragment_hash)
			.ok()
			.unwrap();
		let recover = sp_core::ecdsa::Public(recover);

		assert_ok!(FragmentsPallet::add_upload_auth(Origin::root(), recover, who));

		assert_ok!(FragmentsPallet::upload(
			Origin::signed(Default::default()),
			references,
			None,
			signature.clone(),
			immutable_data,
		));

		sp_io::crypto::ecdsa_generate(KEY_TYPE, None);
		let keys = sp_io::crypto::ecdsa_public_keys(KEY_TYPE);

		<EthereumAuthorities<Test>>::mutate(|authorities| {
			authorities.insert(keys.get(0).unwrap().clone());
		});
		assert_ok!(FragmentsPallet::detach(
			Origin::signed(Default::default()),
			fragment_hash,
			SupportedChains::EthereumMainnet,
			pair.to_raw_vec()
		));

		let (pair, _) = sp_core::sr25519::Pair::generate();
		assert_noop!(
			FragmentsPallet::transfer(
				Origin::signed(Default::default()),
				fragment_hash,
				pair.public()
			),
			Error::<Test>::FragmentDetached
		);
	});
}
