//! Helper Functions and Types that are used in other Packages of the this Workspace

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

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


/// Types that will be used by the Protos pallet
pub mod protos;
/// Types that will be used by the Fragments pallet
pub mod fragments;

/// Helper Functions that can be used in other packages of this workspace
#[cfg(feature = "fragnova-workspace")]
mod helper_functions;
#[cfg(feature = "fragnova-workspace")]
pub use helper_functions::{
	http_json_post,
	get_locked_frag_account,
	get_account_id,
};
/// TODO - Documentation
#[cfg(feature = "fragnova-workspace")]
mod runtime_interface;
#[cfg(feature = "fragnova-workspace")]
pub use runtime_interface::fragnova;
#[cfg(feature = "fragnova-workspace")]
#[cfg(feature = "std")]
pub use runtime_interface::init;
