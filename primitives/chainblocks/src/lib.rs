#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate chainblocks;

#[cfg(feature = "std")]
#[macro_use]
extern crate lazy_static;

use codec::{Compact, Decode, Encode};
use sp_std::vec::Vec;

pub type Hash = sp_core::H256;

pub type FragmentHash = [u8; 32];
pub type MutableDataHash = [u8; 32];

#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug)]
pub struct Fragment {
	/// Plain hash of indexed data.
	pub mutable_hash: MutableDataHash,
	/// Include price of the fragment.
	pub include_cost: Option<Compact<u128>>,
	/// The original creator of the fragment.
	pub creator: Vec<u8>,
	/// The current owner of the fragment.
	pub owner: Vec<u8>,
	/// Immutable data of the fragment.
	pub immutable_block: u32,
	/// Mutable data of the fragment.
	pub mutable_block: u32,
	/// References to other fragments.
	pub references: Option<Vec<FragmentHash>>,
	/// If the fragment has been verified and is passed validation
	pub verified: bool,
}

#[cfg(feature = "std")]
mod details {
	use super::*;

	use sp_runtime::traits::Block as BlockT;

	// lazy_static! {
	// 	static ref FETCH_EXTRINSIC: Mutex<Option<Box<dyn Fn(&Hash) -> Option<Vec<u8>>>>> =
	// 		Mutex::new(None);
	// }

	use std::{convert::TryInto, sync::Mutex};

	use chainblocks::{
		cbl_env,
		core::destroyVar,
		types::{ChainRef, ExternalVar, Node, Var},
		CBVAR_FLAGS_EXTERNAL,
	};

	pub fn init<F>(fetch_extrinsic: F)
	where
		F: Fn(&Hash) -> Option<Vec<u8>>,
	{
		// *FETCH_EXTRINSIC.lock().unwrap() = Some(fetch_extrinsic);
	}

	pub fn _say_hello_world(data: &str) {
		lazy_static! {
			static ref VAR: Mutex<ExternalVar> = Mutex::new(ExternalVar::default());
			static ref NODE: Node = {
				let node = Node::default();
				// let mut chain_var = cbl_env!("(defloop test (Msg \"Hello\"))");
				let mut chain_var = cbl_env!("(Chain \"test\" :Looped .text (ExpectString) (Log))");
				let chain: ChainRef = chain_var.try_into().unwrap();
				chain.set_external("text", &VAR.lock().unwrap());
				node.schedule(chain);
				destroyVar(&mut chain_var);
				node
			};
		}
		VAR.lock().unwrap().update(data);
		NODE.tick();
	}

	pub fn _fetch_extrinsic(hash: &Hash) -> Option<Vec<u8>> {
		// FETCH_EXTRINSIC.lock().unwrap().unwrap()(hash)
		None
	}
}

#[cfg(not(feature = "std"))]
mod details {
	use super::*;

	pub fn _say_hello_world(data: &str) {}
	pub fn _fetch_extrinsic(hash: &Hash) -> Option<Vec<u8>> {
		None
	}
}

#[sp_runtime_interface::runtime_interface]
pub trait OffchainFragments {
	fn say_hello_world(data: &str) {
		details::_say_hello_world(data);
	}

	fn fetch_extrinsic(hash: &Hash) -> Option<Vec<u8>> {
		details::_fetch_extrinsic(hash)
	}

	fn on_new_fragment(_immutable_data: &[u8], _mutable_data: &[u8]) -> Result<(), ()> {
		log::debug!("sp_chainblocks on_new_fragment called...");
		Ok(())
	}
}

#[cfg(feature = "std")]
pub fn init<F>(fetch_extrinsic: F)
where
	F: Fn(&Hash) -> Option<Vec<u8>>,
{
	use chainblocks::{cbl_env, cblog};

	details::init(fetch_extrinsic);

	// needs to go first!
	chainblocks::core::init();

	cblog!("Chainblocks initializing...");

	// load default chains
	let chain = cbl_env!(include_str!("validate_fragment.edn"));

	cblog!("Chainblocks initialized!");
}
