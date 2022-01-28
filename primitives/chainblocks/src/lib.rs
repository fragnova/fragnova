#![cfg_attr(not(feature = "std"), no_std)]

// #[cfg(feature = "std")]
// extern crate chainblocks;

#[cfg(feature = "std")]
extern crate lazy_static;

use sp_std::vec::Vec;
use codec::{Decode, Encode};
use sp_core::{crypto::KeyTypeId, ecdsa, H160, U256};
use core::slice::Iter;

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

/// Defines application identifier for crypto keys of this module.
///
/// Every module that deals with signatures needs to declare its unique identifier for
/// its crypto keys.
/// When offchain worker is signing transactions it's going to request keys of type
/// `KeyTypeId` from the keystore and use the ones it finds to sign the transaction.
/// The keys can be inserted manually via RPC (see `author_insertKey`).
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"frag");

#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub enum SupportedChains {
	EthereumMainnet,
	EthereumRinkeby,
	EthereumGoerli,
}


impl SupportedChains {
	pub fn iterator() -> Iter<'static, SupportedChains> {
		static CHAINS: [SupportedChains; 3] = [
			SupportedChains::EthereumMainnet,
			SupportedChains::EthereumRinkeby,
			SupportedChains::EthereumGoerli,
		];
		CHAINS.iter()
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub enum FragmentOwner<TAccountId> {
	// A regular account on this chain
	User(TAccountId),
	// An external asset not on this chain
	ExternalAsset(LinkedAsset),
}


#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub enum LinkSource {
	// Generally we just store this data, we don't verify it as we assume auth service did it.
	// (Link signature, Linked block number, EIP155 Chain ID)
	Evm(ecdsa::Signature, u64, U256),
}

#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub enum LinkedAsset {
	// Ethereum (ERC721 Contract address, Token ID, Link source)
	Erc721(H160, U256, LinkSource),
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
