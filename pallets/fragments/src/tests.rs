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

#[test]
fn add_validator_should_works() {
	sp_io::TestExternalities::default().execute_with(|| {
		let validator = Default::default();
		assert_ok!(FragmentsPallet::add_validator(Origin::root(), validator));
		assert!(FragmentValidators::<Test>::get().contains(&validator));
	});
}

#[test]
fn remove_validator_should_works() {
	sp_io::TestExternalities::default().execute_with(|| {
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

	let mut t = sp_io::TestExternalities::default();
	t.register_extension(OffchainWorkerExt::new(offchain));
	t.register_extension(TransactionPoolExt::new(pool));
	t.register_extension(KeystoreExt(Arc::new(keystore)));

	let payload = FragmentValidation {
		block_number: 1,
		fragment_hash: [
			74, 25, 49, 128, 53, 97, 244, 49, 222, 202, 176, 2, 231, 66, 95, 10, 133, 49, 213, 228, 86,
			161, 164, 127, 217, 153, 138, 37, 48, 192, 248, 0,
		],
		public: <Test as SigningTypes>::Public::from(public_key),
		result: true,
	};

	t.execute_with(|| {
		let tx = pool_state.write().transactions.pop().unwrap();
		let tx = Extrinsic::decode(&mut &*tx).unwrap();
		assert_eq!(tx.signature, None);
		if let Call::FragmentsPallet(crate::Call::internal_confirm_upload {
								 fragment_data: body,
								 signature,
							 }) = tx.call
		{
			assert_eq!(body, payload);

			let signature_valid =
				<FragmentValidation<
					<Test as SigningTypes>::Public,
					<Test as frame_system::Config>::BlockNumber,
				> as frame_system::offchain::SignedPayload<Test>>::verify::<fragments_pallet::crypto::FragmentsAuthId>(&payload, signature);

			assert!(signature_valid);
		}

	});
}

