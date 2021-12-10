use crate::{mock::*, FragmentValidation};
use crate::FragmentValidators;
use frame_support::{assert_ok};
use std::sync::Arc;
use sp_core::{
	offchain::{testing, OffchainWorkerExt, TransactionPoolExt},
};
use codec::Decode;
use sp_keystore::{testing::KeyStore, KeystoreExt, SyncCryptoStore};
use sp_runtime::{
	RuntimeAppPublic,
};
use frame_system::offchain::SigningTypes;
use sp_chainblocks::FragmentHash;

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
		assert_ok!(FragmentsPallet::remove_validator(Origin::root(), validator));
		assert!(!FragmentValidators::<Test>::get().contains(&validator));
	});
}

#[test]
fn internal_confirm_should_upload_works() {
	const PHRASE: &str =
		"news slush supreme milk chapter athlete soap sausage put clutch what kitten";
	let (offchain, offchain_state) = testing::TestOffchainExt::new();
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

	let hash: FragmentHash = [30, 138, 136, 186, 232, 46, 112, 65, 122, 54, 110, 89, 123, 195, 7, 150, 12, 134, 10, 179, 245, 51, 83, 227, 72, 251, 5, 148, 207, 251, 119, 59];

	let fragment_data = FragmentValidation {
		block_number: 101,
		fragment_hash: hash,
		public: <Test as SigningTypes>::Public::from(public_key),
		result: true,
	};

	t.execute_with(|| {

		 System::set_block_number(15000);
		 FragmentsPallet::upload(Origin::signed(Default::default()), "0x0155a0e40220".as_bytes().to_vec(), "0x0155a0e40220".as_bytes().to_vec(), Some(vec![hash]), None).unwrap();
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

			let signature_valid =
				<FragmentValidation<
					<Test as SigningTypes>::Public,
					<Test as frame_system::Config>::BlockNumber,
				> as frame_system::offchain::SignedPayload<Test>>::verify::<fragments_pallet::crypto::FragmentsAuthId>(&fragment_data, signature);

			assert!(signature_valid);
		}

	});
}
