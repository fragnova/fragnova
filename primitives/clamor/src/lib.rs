//! Helper Functions and Types that are used in other Packages of the this Workspace

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

// #[cfg(feature = "std")]
// extern crate chainblocks;

// TODO Review - maybe rename this module to `sp_protos` since we have a crate with the same name `protos` (https://github.com/fragcolor-xyz/protos)
/// Types that will be used by the Protos pallet
pub mod protos;
// TODO Review - maybe rename this module `sp_fragments` since we have a crate with the same name `fragments` (https://github.com/fragcolor-xyz/fragments)  - although it's not being used
/// Types that will be used by the Fragments pallet
pub mod fragments;

use codec::{Decode, Encode, Error as CodecError};
use sp_core::offchain::{HttpRequestStatus, Timestamp};
use sp_io::{hashing::blake2_256, offchain};
use sp_std::{
	vec::Vec
};

/// 64 bytes u8-Array
pub type Hash64 = [u8; 8];
/// 128 bytes u8-Array
pub type Hash128 = [u8; 16];
/// 256 bytes u8-Array
pub type Hash256 = [u8; 32];

/// The IPFS CID prefix used to use to obtain any data that is stored on the Fragnova Blockchain
///
/// The format of the CID prefix is: <cid-version><multicodec><multihash> (see: https://proto.school/anatomy-of-a-cid/05)
///
/// 0x01 stands for CID v1.
/// 0x55 is the Multicodec code for raw (https://github.com/multiformats/multicodec)
/// 0xa0e402 is the Multihash code for blake2b-256 (https://github.com/multiformats/multihash)
/// 0x20 is the length of the digest
pub const CID_PREFIX: [u8; 6] = hex_literal::hex!("0155a0e40220");

#[cfg(feature = "std")]
mod details {
	use lazy_static::lazy_static;
	use std::sync::Mutex;

	lazy_static! {
		pub static ref GETH_URL: Mutex<Option<Vec<u8>>> = Mutex::new(None);
	}

	// lazy_static! {
	// 	static ref FETCH_EXTRINSIC: Mutex<Option<Box<dyn Fn(&Hash256) -> Option<Vec<u8>>>>> =
	// 		Mutex::new(None);
	// }

	// use std::{convert::TryInto, sync::Mutex};

	// use chainblocks::{
	// 	cbl_env,
	// 	core::destroyVar,
	// 	types::{ChainRef, ExternalVar, Node},
	// };

	pub fn _say_hello_world(_data: &str) {
		// lazy_static! {
		// 	static ref VAR: Mutex<ExternalVar> = Mutex::new(ExternalVar::default());
		// 	static ref NODE: Node = {
		// 		let node = Node::default();
		// 		// let mut chain_var = cbl_env!("(defloop test (Msg \"Hello\"))");
		// 		let mut chain_var = cbl_env!("(Chain \"test\" :Looped .text (ExpectString) (Log))");
		// 		let chain: ChainRef = chain_var.try_into().unwrap();
		// 		chain.set_external("text", &VAR.lock().unwrap());
		// 		node.schedule(chain);
		// 		destroyVar(&mut chain_var);
		// 		node
		// 	};
		// }
		// VAR.lock().unwrap().update(data);
		// NODE.tick();
	}

	/// Get the URL of the Fragnova-owned Geth Node
	pub fn _get_geth_url() -> Option<Vec<u8>> {
		if let Some(geth_url) = GETH_URL.lock().unwrap().as_ref() {
			// well, we are doing an allocation every time we call this function here...
			Some(geth_url.clone())
		} else {
			None
		}
	}
}

#[cfg(not(feature = "std"))]
mod details {
	use super::*;

	/// TODO: Remove
	pub fn _say_hello_world(data: &str) {}

	/// TODO
	pub fn _fetch_extrinsic(hash: &Hash256) -> Option<Vec<u8>> {
		None
	}

	/// TODO
	pub fn _get_geth_url() -> Option<Vec<u8>> {
		None
	}
}

/// A runtime interface for the Clamor Blockchain
///
/// Background:
///
/// `#[sp_runtime_interface::runtime_interface]` is an attribute macro for transforming a trait declaration into a runtime interface.
///
/// A runtime interface is a fixed interface between a Substrate compatible runtime and the native node.
/// This interface is callable from a native and a wasm runtime.
/// The **macro** will **generate** the **corresponding code for the native implementation** and the **code for calling from the wasm side to the native implementation**.
/// The macro expects the runtime interface declaration as trait declaration.
///
/// Source: https://paritytech.github.io/substrate/latest/sp_runtime_interface/attr.runtime_interface.html
#[sp_runtime_interface::runtime_interface]
pub trait Clamor {
	// these are called NATIVE from even WASM
	// that's the deal

	/// A function that can be called from native/wasm.
	///
	/// The implementation given to this function is only compiled on native.
	fn say_hello_world(data: &str) {
		details::_say_hello_world(data);
	}

	/// TODO
	fn on_new_fragment(_fragment_hash: &Hash256) -> bool {
		log::debug!("sp_fragnova on_new_fragment called...");
		true
	}

	/// Get the URL of the Fragnova-owned Geth Node
	fn get_geth_url() -> Option<Vec<u8>> {
		details::_get_geth_url()
	}
}

/// Set the Fragnova-owned Geth Node's URL
#[cfg(feature = "std")]
pub fn init(geth_url: Option<String>) {
	if let Some(geth_url) = geth_url {
		*details::GETH_URL.lock().unwrap() = Some(geth_url.into_bytes());
	}

	// use chainblocks::{cbl_env, shlog};

	// details::init(fetch_extrinsic);

	// // needs to go first!
	// chainblocks::core::init();

	// shlog!("Chainblocks initializing...");

	// // load default chains
	// let chain = cbl_env!(include_str!("validate_fragment.edn"));

	// shlog!("Chainblocks initialized!");
}

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

/// **Get** an **Account ID** deterministically computed from an input `hash`**.
pub fn get_vault_id<TAccountId: Encode + Decode>(hash: Hash128) -> TAccountId {
	let hash = blake2_256(&[&b"fragnova-vault"[..], &hash].concat());
	TAccountId::decode(&mut &hash[..]).expect("T::AccountId should decode")
}
