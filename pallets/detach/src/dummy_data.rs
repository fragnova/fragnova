use sp_runtime::transaction_validity::TransactionSource;
use crate::*;

use crate::mock::Test;

use sp_core::Pair;
use sp_runtime::MultiSignature;

pub struct ProcessDetachRequests {
	pub detach_requests: Vec<DetachRequest>,
}

pub struct FinalizeDetach {
	pub data: DetachInternalData<sp_core::ed25519::Public>,
}

pub struct ValidateUnsigned {
	pub source: TransactionSource,
	pub call: crate::Call<Test>,
}

pub struct DummyData {
	pub process_detach_requests: ProcessDetachRequests,
	pub validate_unsigned: ValidateUnsigned,
	pub finalize_detach: FinalizeDetach,
}

impl DummyData {
	pub fn new() -> Self {

		let process_detach_requests = ProcessDetachRequests {
			detach_requests: vec![
				DetachRequest {
					hashes: vec![DetachHash::Proto([7u8; 32]), DetachHash::Proto([77u8; 32])],
					target_chain: SupportedChains::EthereumMainnet,
					target_account: [7u8; 20].to_vec(),
				}
			],
		};

		let data = DetachInternalData::<MultiSigner> {
			public: MultiSigner::Ed25519(ed25519::Pair::from_seed_slice(&[7u8; 32]).unwrap().public()), // ed25519::Pair::from_seed_slice(&[7u8; 32]).unwrap().public(),
			hashes: process_detach_requests.detach_requests[0].hashes.clone(),
			merkle_root: merkle_root::<Keccak256, _, _>(
				process_detach_requests.detach_requests[0].hashes.iter().map(|detach_hash| detach_hash.get_signable_hash()).collect::<Vec<Vec<u8>>>()
			),
			target_chain: process_detach_requests.detach_requests[0].target_chain,
			target_account: process_detach_requests.detach_requests[0].target_account.clone(),
			remote_signature: [7u8; 65].to_vec(), // REVIEW - this doesn't match the `hash`, `nonce`, `target_account` and `target_chain`
			nonce: 1,
		};
		let validate_unsigned = ValidateUnsigned {
			source: TransactionSource::Local,
			call: crate::Call::internal_finalize_detach {
				data: data.clone(),
				signature: MultiSignature::Ed25519(ed25519::Pair::from_seed_slice(&[7u8; 32]).unwrap().sign(&data.encode())),
			}
		};

		let finalize_detach = FinalizeDetach {
			data: DetachInternalData {
				public: sp_core::ed25519::Public([7u8; 32]),
				hashes: process_detach_requests.detach_requests[0].hashes.clone(),
				merkle_root: merkle_root::<Keccak256, _, _>(
					process_detach_requests.detach_requests[0].hashes.iter().map(|detach_hash| detach_hash.get_signable_hash()).collect::<Vec<Vec<u8>>>()
				),
				target_chain: process_detach_requests.detach_requests[0].target_chain,
				target_account: process_detach_requests.detach_requests[0].target_account.clone(),
				remote_signature:  [7u8; 65].to_vec(), // REVIEW - this doesn't match the `hash`, `nonce`, `target_account` and `target_chain`
				nonce: 1,
			}
		};

		Self {
			process_detach_requests,
			validate_unsigned,
			finalize_detach,
		}
	}
}
