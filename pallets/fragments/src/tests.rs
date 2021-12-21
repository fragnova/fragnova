use crate::{mock::*, Error, FragmentValidation, FragmentValidators, Fragments, SupportedChains, EthereumAuthorities, KEY_TYPE};
use codec::Decode;
use frame_support::{assert_noop, assert_ok};
use frame_system::offchain::SigningTypes;
use sp_chainblocks::FragmentHash;
use sp_core::{
	ecdsa,
	offchain::{testing, OffchainWorkerExt, TransactionPoolExt},
	Pair,
	Public
};
use sp_io::hashing::blake2_256;
use sp_keystore::{testing::KeyStore, KeystoreExt, SyncCryptoStore};
use sp_runtime::RuntimeAppPublic;
use std::sync::Arc;

#[test]
fn add_validator_should_works() {
	new_test_ext().execute_with(|| {
		let validator = Default::default();
		assert_ok!(FragmentsPallet::add_validator(Origin::root(), validator));
		assert!(FragmentValidators::<Test>::get().contains(&validator));
	});
}

#[test]
fn remove_validator_should_works() {
	new_test_ext().execute_with(|| {
		let validator = Default::default();
		assert_ok!(FragmentsPallet::add_validator(Origin::root(), validator));
		assert!(FragmentValidators::<Test>::get().contains(&validator));
		assert_ok!(FragmentsPallet::remove_validator(Origin::root(), validator));
		assert!(!FragmentValidators::<Test>::get().contains(&validator));
	});
}

#[test]
fn internal_confirm_should_upload_works() {
	const PHRASE: &str =
		"news slush supreme milk chapter athlete soap sausage put clutch what kitten";
	let (offchain, _) = testing::TestOffchainExt::new();
	let (pool, pool_state) = testing::TestTransactionPoolExt::new();

	let keystore = KeyStore::new();

	SyncCryptoStore::sr25519_generate_new(
		&keystore,
		crate::crypto::Public::ID,
		Some(&format!("{}/hunter1", PHRASE)),
	)
	.unwrap();

	let public_key = SyncCryptoStore::sr25519_public_keys(&keystore, crate::crypto::Public::ID)
		.get(0)
		.unwrap()
		.clone();

	let mut t = new_test_ext();
	t.register_extension(OffchainWorkerExt::new(offchain));
	t.register_extension(TransactionPoolExt::new(pool));
	t.register_extension(KeystoreExt(Arc::new(keystore)));

	let hash: FragmentHash = [
		30, 138, 136, 186, 232, 46, 112, 65, 122, 54, 110, 89, 123, 195, 7, 150, 12, 134, 10, 179,
		245, 51, 83, 227, 72, 251, 5, 148, 207, 251, 119, 59,
	];

	let fragment_data = FragmentValidation {
		block_number: 101,
		fragment_hash: hash,
		public: <Test as SigningTypes>::Public::from(public_key),
		result: true,
	};

	t.execute_with(|| {
		System::set_block_number(15000);
		FragmentsPallet::upload(
			Origin::signed(Default::default()),
			"0x0155a0e40220".as_bytes().to_vec(),
			"0x0155a0e40220".as_bytes().to_vec(),
			Some(vec![hash]),
			None,
		)
		.unwrap();
		FragmentsPallet::process_unverified_fragments(101);

		let tx = pool_state.write().transactions.first().unwrap().clone();
		let tx = Extrinsic::decode(&mut &*tx).unwrap();
		assert_eq!(tx.signature, None);
		if let Call::FragmentsPallet(crate::Call::internal_confirm_upload {
			fragment_data: body,
			signature,
		}) = tx.call
		{
			assert_eq!(body, fragment_data);

			let signature_valid = <FragmentValidation<
				<Test as SigningTypes>::Public,
				<Test as frame_system::Config>::BlockNumber,
			> as frame_system::offchain::SignedPayload<Test>>::verify::<
				fragments_pallet::crypto::FragmentsAuthId,
			>(&fragment_data, signature);

			assert!(signature_valid);
		}
	});
}

#[test]
fn upload_should_works() {
	new_test_ext().execute_with(|| {
		let hash: FragmentHash = [
			30, 138, 136, 186, 232, 46, 112, 65, 122, 54, 110, 89, 123, 195, 7, 150, 12, 134, 10,
			179, 245, 51, 83, 227, 72, 251, 5, 148, 207, 251, 119, 59,
		];
		let immutable_data = "0x0155a0e40220".as_bytes().to_vec();
		assert_ok!(FragmentsPallet::upload(
			Origin::signed(Default::default()),
			immutable_data.clone(),
			"0x0155a0e40220".as_bytes().to_vec(),
			Some(vec![hash]),
			None
		));

		let fragment_hash = blake2_256(immutable_data.as_slice());

		assert!(<Fragments<Test>>::contains_key(fragment_hash));
	});
}

#[test]
fn upload_should_not_works_if_fragment_hash_exists() {
	new_test_ext().execute_with(|| {
		let hash: FragmentHash = [
			30, 138, 136, 186, 232, 46, 112, 65, 122, 54, 110, 89, 123, 195, 7, 150, 12, 134, 10,
			179, 245, 51, 83, 227, 72, 251, 5, 148, 207, 251, 119, 59,
		];
		let immutable_data = "0x0155a0e40220".as_bytes().to_vec();
		assert_ok!(FragmentsPallet::upload(
			Origin::signed(Default::default()),
			immutable_data.clone(),
			"0x0155a0e40220".as_bytes().to_vec(),
			Some(vec![hash]),
			None
		));

		let fragment_hash = blake2_256(immutable_data.clone().as_slice());

		assert!(<Fragments<Test>>::contains_key(fragment_hash));

		assert_noop!(
			FragmentsPallet::upload(
				Origin::signed(Default::default()),
				immutable_data,
				"0x0155a0e40220".as_bytes().to_vec(),
				Some(vec![hash]),
				None
			),
			Error::<Test>::FragmentExists
		);
	});
}

#[test]
fn update_fragment_should_work() {
	new_test_ext().execute_with(|| {
		let hash: FragmentHash = [
			30, 138, 136, 186, 232, 46, 112, 65, 122, 54, 110, 89, 123, 195, 7, 150, 12, 134, 10,
			179, 245, 51, 83, 227, 72, 251, 5, 148, 207, 251, 119, 59,
		];
		let immutable_data = "0x0155a0e40220".as_bytes().to_vec();
		let who: sp_core::sr25519::Public = Default::default();
		assert_ok!(FragmentsPallet::upload(
			Origin::signed(who.clone()),
			immutable_data.clone(),
			"0x0155a0e40220".as_bytes().to_vec(),
			Some(vec![hash]),
			None
		));

		let fragment_hash = blake2_256(immutable_data.as_slice());

		assert_ok!(FragmentsPallet::update(
			Origin::signed(who),
			fragment_hash,
			Some("0x0155a0e40220".as_bytes().to_vec()),
			None
		));
	});
}

#[test]
fn update_fragment_should_not_work_if_user_is_unauthorized() {
	new_test_ext().execute_with(|| {
		let hash: FragmentHash = [
			30, 138, 136, 186, 232, 46, 112, 65, 122, 54, 110, 89, 123, 195, 7, 150, 12, 134, 10,
			179, 245, 51, 83, 227, 72, 251, 5, 148, 207, 251, 119, 59,
		];
		let immutable_data = "0x0155a0e40220".as_bytes().to_vec();
		let who: sp_core::sr25519::Public = Default::default();
		assert_ok!(FragmentsPallet::upload(
			Origin::signed(who.clone()),
			immutable_data.clone(),
			"0x0155a0e40220".as_bytes().to_vec(),
			Some(vec![hash]),
			None
		));

		let fragment_hash = blake2_256(immutable_data.as_slice());

		let (pair, _) = sp_core::sr25519::Pair::generate();

		assert_noop!(
			FragmentsPallet::update(
				Origin::signed(pair.public()),
				fragment_hash,
				Some("0x0155a0e40220".as_bytes().to_vec()),
				None
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

		let (pair, _) = sp_core::sr25519::Pair::generate();

		assert_noop!(
			FragmentsPallet::update(
				Origin::signed(pair.public()),
				fragment_hash,
				Some("0x0155a0e40220".as_bytes().to_vec()),
				None
			),
			Error::<Test>::FragmentNotFound
		);
	});
}

#[test]
fn detach_fragment_should_not_work_if_user_is_unauthorized() {
	new_test_ext().execute_with(|| {
		let hash: FragmentHash = [
			30, 138, 136, 186, 232, 46, 112, 65, 122, 54, 110, 89, 123, 195, 7, 150, 12, 134, 10,
			179, 245, 51, 83, 227, 72, 251, 5, 148, 207, 251, 119, 59,
		];
		let immutable_data = "0x0155a0e40220".as_bytes().to_vec();
		let who: sp_core::sr25519::Public = Default::default();
		assert_ok!(FragmentsPallet::upload(
			Origin::signed(who.clone()),
			immutable_data.clone(),
			"0x0155a0e40220".as_bytes().to_vec(),
			Some(vec![hash]),
			None
		));

		let fragment_hash = blake2_256(immutable_data.as_slice());

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
fn detach_fragment_should_work() {
	let keystore = KeyStore::new();
	let mut t = new_test_ext();

	t.register_extension(KeystoreExt(Arc::new(keystore)));
	t.execute_with(|| {
		let hash: FragmentHash = [
			30, 138, 136, 186, 232, 46, 112, 65, 122, 54, 110, 89, 123, 195, 7, 150, 12, 134, 10,
			179, 245, 51, 83, 227, 72, 251, 5, 148, 207, 251, 119, 59,
		];
		let immutable_data = "0x0155a0e40220".as_bytes().to_vec();
		let who: sp_core::sr25519::Public = Default::default();
		assert_ok!(FragmentsPallet::upload(
			Origin::signed(who.clone()),
			immutable_data.clone(),
			"0x0155a0e40220".as_bytes().to_vec(),
			Some(vec![hash]),
			None
		));

		let fragment_hash = blake2_256(immutable_data.as_slice());
		let (pair, _) = sp_core::sr25519::Pair::generate();

		sp_io::crypto::ecdsa_generate(KEY_TYPE, None);
		let keys = sp_io::crypto::ecdsa_public_keys(KEY_TYPE);

		<EthereumAuthorities<Test>>::mutate(|authorities| {
			authorities.insert(keys.get(0).unwrap().clone());
		});
		assert_ok!(FragmentsPallet::detach(
			Origin::signed(who),
			fragment_hash,
			SupportedChains::EthereumMainnet,
			pair.to_raw_vec()
		));
	});
}
