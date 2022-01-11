#![cfg_attr(not(feature = "std"), no_std)]

// #[cfg(feature = "std")]
// extern crate chainblocks;

#[cfg(feature = "std")]
extern crate lazy_static;

use sp_std::vec::Vec;

pub type Hash256 = [u8; 32];

pub enum AudioFormats {
	Ogg,
	Mp3,
	Wav,
}

pub struct AudioData {
	pub format: AudioFormats,
	pub data: Vec<u8>,
}

pub enum FragmentData {
	Chain(Vec<u8>),
	Audio(AudioData),
}

#[cfg(feature = "std")]
mod details {
	// use super::*;

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
}

#[cfg(not(feature = "std"))]
mod details {
	use super::*;

	pub fn _say_hello_world(data: &str) {}
	pub fn _fetch_extrinsic(hash: &Hash256) -> Option<Vec<u8>> {
		None
	}
}

#[sp_runtime_interface::runtime_interface]
pub trait OffchainFragments {
	fn say_hello_world(data: &str) {
		details::_say_hello_world(data);
	}

	fn on_new_fragment(_fragment_hash: &Hash256) -> bool {
		log::debug!("sp_chainblocks on_new_fragment called...");
		true
	}
}

#[cfg(feature = "std")]
pub fn init() {
	// use chainblocks::{cbl_env, cblog};

	// details::init(fetch_extrinsic);

	// // needs to go first!
	// chainblocks::core::init();

	// cblog!("Chainblocks initializing...");

	// // load default chains
	// let chain = cbl_env!(include_str!("validate_fragment.edn"));

	// cblog!("Chainblocks initialized!");
}
