//! Helper Functions that can be used in other packages of this workspace

use codec::{Decode, Encode, Error as CodecError};
use sp_core::offchain::{HttpRequestStatus, Timestamp};
use sp_io::{hashing::blake2_256, offchain};
use sp_std::{
	vec::Vec
};

/// Make an HTTP POST Request with data `body` to the URL `url`
pub fn http_json_post(
	url: &str,
	body: &[u8],
	wait: Option<Timestamp>,
) -> Result<Vec<u8>, &'static str> {
	log::debug!("sp_fragnova http_request called...");

	let request =
		offchain::http_request_start("POST", url, &[]).map_err(|_| "Failed to start request")?;

	offchain::http_request_add_header(request, "Content-Type", "application/json")
		.map_err(|_| "Failed to add header")?;

	offchain::http_request_write_body(request, body, None).map_err(|_| "Failed to write body")?;

	// send off the request
	offchain::http_request_write_body(request, &[], None).unwrap();

	let results = offchain::http_response_wait(&[request], wait);
	let status = results[0];

	match status {
		HttpRequestStatus::Finished(status) => match status {
			200 => {
				let mut response_body: Vec<u8> = Vec::new();
				loop {
					let mut buffer = Vec::new();
					buffer.resize(1024, 0);
					let len =
						offchain::http_response_read_body(request, &mut buffer, None).unwrap();
					if len == 0 {
						break
					}
					response_body.extend_from_slice(&buffer[..len as usize]);
				}
				Ok(response_body)
			},
			_ => {
				log::error!("request had unexpected status: {}", status);
				Err("request had unexpected status")
			},
		},
		HttpRequestStatus::DeadlineReached => {
			log::error!("request failed for reached timeout");
			Err("timeout reached")
		},
		_ => {
			log::error!("request failed with status: {:?}", status);
			Err("request failed")
		},
	}
}

/// Returns an account ID that can stake FRAG tokens.
/// This returned account ID is determinstically computed from the given account ID (`who`).
pub fn get_locked_frag_account<TAccountId: Encode + Decode>(
	who: &TAccountId,
) -> Result<TAccountId, CodecError> {
	// the idea is to use an account that users cannot access
	let mut who = who.encode();
	who.append(&mut b"frag-locked-account".to_vec());
	let who = blake2_256(&who);
	TAccountId::decode(&mut &who[..])
}

/// **Get** an **Account ID** deterministically computed from an input `hash` and a `prefix`.
pub fn get_account_id<TAccountId: Encode + Decode>(prefix: &[u8], hash: &[u8]) -> TAccountId {
	let hash = blake2_256(&[prefix, hash].concat());
	TAccountId::decode(&mut &hash[..]).expect("T::AccountId should decode")
}
