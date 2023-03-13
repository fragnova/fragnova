//! TODO - Documentation

use crate::Hash256;

use sp_std::vec::Vec;

/// A runtime interface for the Fragnova Blockchain
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
pub trait Fragnova {
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
