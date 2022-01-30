use crate::{mock::*, Error, KEY_TYPE, DetachInternalData, DetachedFragments, EthereumAuthorities, FragKeys};
use frame_support::{assert_noop, assert_ok};
use sp_core::Pair;
use sp_keystore::{testing::KeyStore, KeystoreExt};
use std::sync::Arc;
use sp_chainblocks::{FragmentOwner, SupportedChains};

#[test]
fn add_eth_auth_should_works() {
	new_test_ext().execute_with(|| {
		let validator: sp_core::ecdsa::Public = sp_core::ecdsa::Public::from_raw(PUBLIC);
		assert_ok!(ClamorToolsPallet::add_eth_auth(Origin::root(), validator.clone()));
		assert!(EthereumAuthorities::<Test>::get().contains(&validator));
	});
}

#[test]
fn del_eth_auth_should_works() {
	new_test_ext().execute_with(|| {
		let validator: sp_core::ecdsa::Public = sp_core::ecdsa::Public::from_raw(PUBLIC);
		assert_ok!(ClamorToolsPallet::del_eth_auth(Origin::root(), validator.clone()));
		assert!(!EthereumAuthorities::<Test>::get().contains(&validator));
	});
}

#[test]
fn add_key_should_works() {
	new_test_ext().execute_with(|| {
		let validator: sp_core::ed25519::Public = sp_core::ed25519::Public::from_raw(PUBLIC1);
		assert_ok!(ClamorToolsPallet::add_key(Origin::root(), validator.clone()));
		assert!(FragKeys::<Test>::get().contains(&validator));
	});
}

#[test]
fn del_key_should_works() {
	new_test_ext().execute_with(|| {
		let validator: sp_core::ed25519::Public = sp_core::ed25519::Public::from_raw(PUBLIC1);
		assert_ok!(ClamorToolsPallet::del_key(Origin::root(), validator.clone()));
		assert!(!FragKeys::<Test>::get().contains(&validator));
	});
}

#[test]
fn detach_fragment_should_not_work_if_user_is_unauthorized() {
	new_test_ext().execute_with(|| {

		let (pair, _) = sp_core::ed25519::Pair::generate();
		let who = sp_core::ed25519::Public::from_raw(PUBLIC1);
		let owner = FragmentOwner::User(who.clone());
		assert_noop!(
			ClamorToolsPallet::detach(
				Origin::signed(pair.public()),
				FRAGMENT_HASH,
				SupportedChains::EthereumMainnet,
				pair.to_raw_vec(),
				owner
			),
			Error::<Test>::Unauthorized
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

		sp_io::crypto::ecdsa_generate(KEY_TYPE, None);
		let keys = sp_io::crypto::ecdsa_public_keys(KEY_TYPE);

		<EthereumAuthorities<Test>>::mutate(|authorities| {
			authorities.insert(keys.get(0).unwrap().clone());
		});
		let who = sp_core::ed25519::Public::from_raw(PUBLIC1);
		let owner = FragmentOwner::User(who.clone());
		assert_ok!(ClamorToolsPallet::detach(
			Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC1)),
			FRAGMENT_HASH,
			SupportedChains::EthereumMainnet,
			pair.to_raw_vec(),
			owner
		));
	});
}

#[test]
fn internal_finalize_detach_should_works() {
	new_test_ext().execute_with(|| {
		let pair = sp_core::ed25519::Pair::from_string("//Alice", None).unwrap();

		let detach_data = DetachInternalData {
			public: sp_core::ed25519::Public::from_raw(PUBLIC1),
			fragment_hash: FRAGMENT_HASH,
			remote_signature: vec![],
			target_account: vec![],
			target_chain: SupportedChains::EthereumGoerli,
			nonce: 1
		};

		assert_ok!(ClamorToolsPallet::internal_finalize_detach(
			Origin::none(),
			detach_data,
			pair.sign(DATA.as_bytes())
		));

		assert!(<DetachedFragments<Test>>::contains_key(FRAGMENT_HASH));
	});
}
