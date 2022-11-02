#![cfg(test)]

use crate as pallet_detach;
use crate::mock;
use crate::*;
use crate::dummy_data::*;
use crate::mock::*;

use frame_support::{assert_noop, assert_ok};

mod process_detach_requests_tests {
	use super::*;

	fn eth_sign_detach_request(key_type: KeyTypeId, ecdsa_key: ecdsa::Public, detach_request: DetachRequest, nonce: u64) -> Vec<u8> {

		let mut chain_id_be: [u8; 32] = [0u8; 32]; // "be" stands for big-endian
		match request.target_chain {
			SupportedChains::EthereumMainnet => U256::from(1),
			SupportedChains::EthereumRinkeby => U256::from(4),
			SupportedChains::EthereumGoerli => U256::from(5),
		}.to_big_endian(&mut chain_id_be);

		let mut eth_signature = Crypto::ecdsa_sign_prehashed(
			key_type,
			ecdsa_key,
			&keccak_256(
				[
					b"\x19Ethereum Signed Message:\n32",
					&keccak_256(&[
						detach_request.hash.encode()[..],
						&chain_id_be[..],
						detach_request.target_account.clone(),
						&nonce.to_be_bytes()
					])[..]
				].concat()
			)
		).unwrap().to_vec();
		eth_signature[64] += 27u8;
		eth_signature
	}

	fn deterministically_compute_ecdsa_key_from_ed25519_key(key_type: KeyTypeId, ed25519_key: ed25519::Public) {
		let signature = Crypto::ed25519_sign(
			key_type,
			ed25519_key,
			&[
				&b"fragments-frag-ecdsa-keys"[..],
				&ed25519_key.to_vec()[..]
			]
		).unwrap();

		let ecdsa_seed = keccak_256(&signature.0[..]);
		let ecdsa_seed_hex = [
			&b"0x"[..],
			&TryInto::<[u8; 64]>::try_into(hex::encode(ecdsa_seed).into_bytes()).map_err(|_| FAILED)? // actually there's no need to throw any error I think...
		].concat();

		// TODO - generate ECDSA key and return it
	}

	#[test]
	fn process_detach_requests_should_work() {

		let (mut ext, pool_state, offchain_state, ed25519_public_key) = new_test_ext_with_ocw();

		let dd = DummyData::new();
		let process_detach_requests = dd.process_detach_requests;

		ext.execute_with(|| {
			offchain_index::set(b"fragments-detach-requests", &process_detach_requests.detach_requests.encode());
		});

		// Logic copied from "frame/merkle-mountain-range/src/tests.rs"
		ext.persist_offchain_overlay();
		register_offchain_ext(&mut ext);

		ext.execute_with(|| {
			DetachPallet::process_detach_requests();

			let tx = pool_state.write().transactions.pop().unwrap();
			let tx = <Extrinsic as codec::Decode>::decode(&mut &*tx).unwrap();
			assert_eq!(tx.signature, None); // Because `DetachPallet::process_detach_requests()` sends an unsigned transaction with a signed payload

			if let RuntimeCall::DetachPallet(crate::Call::internal_finalize_detach { data, signature }) =
				tx.call
			{

				let nonce = 1u64;

				assert_eq!(
					data,
					DetachInternalData {
						public: ed25519_public_key,
						hash: process_detach_requests.detach_requests[0].hash.clone(),
						target_chain: process_detach_requests.detach_requests[0].target_chain,
						target_account: process_detach_requests.detach_requests[0].target_account.clone(),
						remote_signature: eth_sign_detach_request(
							KEY_TYPE,
							"todo",
							process_detach_requests.detach_requests[0].clone(),
							nonce
						),
						nonce,
					}
				);

				let signature_valid =
					<EthLockUpdate::<<Test as SigningTypes>::Public> as SignedPayload::<Test>>::verify::<
						crypto::DetachAuthId,
					>(&data, signature); // Notice in `pallet_accounts` that `EthLockUpdate<T::Public>` implements the trait `SignedPayload`

				assert!(signature_valid); // If `signature_valid` is true, it means `payload` and `signature` recovered the public address `data.public`
			}

			assert_eq!(
				StorageValueRef::persistent(b"fragments-detach-requests").unwrap().unwrap(),
				Vec::<DetachRequest>::new()
			);

		})
	}
}


