#![cfg(test)]

use crate as pallet_detach;
use crate::mock;
use crate::*;
use crate::dummy_data::*;
use crate::mock::*;

use frame_support::{assert_ok};

mod process_detach_requests_tests {
	use sp_core::Pair;
	use sp_io::TestExternalities;
	use super::*;

	fn eth_sign_detach_request(key_type: KeyTypeId, ecdsa_key: ecdsa::Public, detach_request: DetachRequest, nonce: u64) -> Vec<u8> {

		let mut chain_id_be: [u8; 32] = [0u8; 32]; // "be" stands for big-endian
		match detach_request.target_chain {
			SupportedChains::EthereumMainnet => U256::from(1),
			SupportedChains::EthereumRinkeby => U256::from(4),
			SupportedChains::EthereumGoerli => U256::from(5),
		}.to_big_endian(&mut chain_id_be);

		let mut eth_signature = Crypto::ecdsa_sign_prehashed(
			key_type,
			&ecdsa_key,
			&keccak_256(
				&[
					b"\x19Ethereum Signed Message:\n32",
					&keccak_256(
						&[
							&detach_request.hash.get_signable_hash()[..],
							&chain_id_be[..],
							&detach_request.target_account.clone()[..],
							&nonce.to_be_bytes()
						].concat()
					)[..]
				].concat()
			)
		).unwrap().0.to_vec();
		eth_signature[64] += 27u8;
		eth_signature
	}

	/// Deterministically compute ECDSA Key from Ed25519 Public key
	fn deterministically_compute_ecdsa_key(key_type: KeyTypeId, ed25519_key: ed25519::Public) -> ecdsa::Public {
		let signature = Crypto::ed25519_sign(
			key_type,
			&ed25519_key,
			&[
				b"detach-ecdsa-keys",
				&ed25519_key.to_vec()[..]
			].concat()
		).unwrap();

		let ecdsa_seed = keccak_256(&signature.0[..]);

		let ecdsa_pair = ecdsa::Pair::from_seed_slice(&ecdsa_seed[..]).unwrap();

		ecdsa_pair.public()
	}

	fn add_detach_requests_to_local_storage(ext: &mut TestExternalities, process_detach_requests: &ProcessDetachRequests) {
		ext.execute_with(|| {
			offchain_index::set(b"detach-requests", &process_detach_requests.detach_requests.encode());
		});

		// Logic copied from "frame/merkle-mountain-range/src/tests.rs"
		ext.persist_offchain_overlay();
		register_offchain_ext(ext);
	}

	#[test]
	fn process_detach_requests_should_work() {

		let (mut ext, pool_state, _offchain_state, ed25519_public_key) = new_test_ext_with_ocw();

		let dd = DummyData::new();
		let process_detach_requests = dd.process_detach_requests;

		add_detach_requests_to_local_storage(&mut ext, &process_detach_requests);

		ext.execute_with(|| {

			let computed_ecdsa_key = deterministically_compute_ecdsa_key(KEY_TYPE, ed25519_public_key);

			assert_ok!(DetachPallet::add_key(Origin::root(), ed25519_public_key));
			assert_ok!(DetachPallet::add_eth_auth(Origin::root(), computed_ecdsa_key));

			DetachPallet::process_detach_requests();

			assert!(Crypto::ecdsa_public_keys(KEY_TYPE).contains(&computed_ecdsa_key));
			assert_eq!(
				StorageValueRef::persistent(b"detach-ecdsa-keys").get::<Vec<ed25519::Public>>().unwrap().unwrap(),
				vec![ed25519_public_key]
			);
			assert_eq!(
				StorageValueRef::persistent(b"detach-requests").get::<Vec<DetachRequest>>().unwrap().unwrap(),
				Vec::<DetachRequest>::new()
			);

			let tx = pool_state.write().transactions.pop().unwrap();
			let tx = <Extrinsic as codec::Decode>::decode(&mut &*tx).unwrap();
			assert_eq!(tx.signature, None); // Because `DetachPallet::process_detach_requests()` sends an unsigned transaction with a signed payload

			let Call::DetachPallet(crate::Call::internal_finalize_detach { data, signature }) = tx.call else {
				panic!("The unsigned transaction that was sent is incorrect!");
			};

			let nonce = 1u64;
			assert_eq!(
				data,
				DetachInternalData {
					public: MultiSigner::Ed25519(ed25519_public_key),
					hash: process_detach_requests.detach_requests[0].hash.clone(),
					target_chain: process_detach_requests.detach_requests[0].target_chain,
					target_account: process_detach_requests.detach_requests[0].target_account.clone(),
					remote_signature: eth_sign_detach_request(
						KEY_TYPE,
						computed_ecdsa_key,
						process_detach_requests.detach_requests[0].clone(),
						nonce
					),
					nonce,
				}
			);
			// Verify signature `signature` against SignedPayload object `data`. Returns a bool indicating whether the signature is valid or not.
			assert!(
				<DetachInternalData::<<Test as SigningTypes>::Public> as SignedPayload::<Test>>::verify::<
					crypto::DetachAuthId,
				>(&data, signature)
			);

		})
	}

	#[test]
	fn process_detach_requests_should_not_send_unsigned_transaction_if_no_ecdsa_key_in_the_keystore_under_key_type_is_an_ethereum_authority() { // `key_type` mentioned in the test function name here is the constant `KEY_TYPE`

		let (mut ext, pool_state, _offchain_state, ed25519_public_key) = new_test_ext_with_ocw();

		let dd = DummyData::new();
		let process_detach_requests = dd.process_detach_requests;

		add_detach_requests_to_local_storage(&mut ext, &process_detach_requests);

		ext.execute_with(|| {

			let computed_ecdsa_key = deterministically_compute_ecdsa_key(KEY_TYPE, ed25519_public_key);

			assert_ok!(DetachPallet::add_key(Origin::root(), ed25519_public_key));
			// assert_ok!(DetachPallet::add_eth_auth(Origin::root(), computed_ecdsa_key)); // no ECDSA key in the keystore under `KEY_TYPE` is an Ethereum Authority

			DetachPallet::process_detach_requests();

			assert!(Crypto::ecdsa_public_keys(KEY_TYPE).contains(&computed_ecdsa_key));
			assert_eq!(
				StorageValueRef::persistent(b"detach-ecdsa-keys").get::<Vec<ed25519::Public>>().unwrap().unwrap(),
				vec![ed25519_public_key]
			);
			assert_eq!(
				StorageValueRef::persistent(b"detach-requests").get::<Vec<DetachRequest>>().unwrap().unwrap(),
				Vec::<DetachRequest>::new()
			);

			assert_eq!(pool_state.write().transactions.len(), 0); // should not send unsigned transaction

		})
	}

}

mod validate_unsigned_tests {
	use sp_runtime::MultiSignature;
	use sp_runtime::transaction_validity::{
		TransactionValidity, InvalidTransaction,
	};
	use super::*;

	fn validate_unsigned_(validate_unsigned: &ValidateUnsigned) -> TransactionValidity {
		<DetachPallet as sp_runtime::traits::ValidateUnsigned>::validate_unsigned(
			validate_unsigned.source,
			&validate_unsigned.call
		)
	}

	#[test]
	fn validate_unsigned_should_work() {
		new_test_ext().execute_with(|| {

			let dd = DummyData::new();

			let validate_unsigned = dd.validate_unsigned;

			let crate::Call::internal_finalize_detach {ref data, signature: _} = validate_unsigned.call else { panic!() };

			assert_ok!(DetachPallet::add_key(Origin::root(), TryInto::<ed25519::Public>::try_into(data.public.clone()).unwrap()));
			assert!(validate_unsigned_(&validate_unsigned).is_ok());
		})
	}

	#[test]
	fn validate_unsigned_should_not_work_if_call_parameter_is_not_internal_finalize_detach() {
		new_test_ext().execute_with(|| {

			let dd = DummyData::new();

			let validate_unsigned = ValidateUnsigned {
				call: crate::Call::add_eth_auth {public: ecdsa::Public([0u8; 33])},
				..dd.validate_unsigned
			};

			assert_eq!(validate_unsigned_(&validate_unsigned), InvalidTransaction::Call.into());
		})
	}

	#[test]
	fn validate_unsigned_should_not_work_if_signer_is_not_a_detach_key() {
		new_test_ext().execute_with(|| {

			let dd = DummyData::new();

			let validate_unsigned = dd.validate_unsigned;

			assert_eq!(validate_unsigned_(&validate_unsigned), InvalidTransaction::BadSigner.into());
		})
	}

	#[test]
	fn validate_unsigned_should_not_work_if_signature_parameter_does_not_match_data_parameter() {
		new_test_ext().execute_with(|| {

			let dd = DummyData::new();

			let crate::Call::internal_finalize_detach {ref data, signature: _} = dd.validate_unsigned.call else { panic!() };

			let validate_unsigned = ValidateUnsigned {
				call: crate::Call::internal_finalize_detach {
					data: data.clone(), // `data` parameter
					signature: MultiSignature::Ed25519(sp_core::ed25519::Signature::from_raw([0u8; 64])) // `signature` parameter does not match `data` parameter
				},
				..dd.validate_unsigned
			};

			assert_ok!(DetachPallet::add_key(Origin::root(), TryInto::<ed25519::Public>::try_into(data.public.clone()).unwrap()));
			assert_eq!(
				<DetachPallet as sp_runtime::traits::ValidateUnsigned>::validate_unsigned(
					validate_unsigned.source,
					&crate::Call::internal_finalize_detach {
						data: data.clone(),
						signature: MultiSignature::Ed25519(sp_core::ed25519::Signature::from_raw([0u8; 64]))
					},
				),
				InvalidTransaction::BadProof.into(),
			);
		})
	}
}




