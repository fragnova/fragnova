//! This package declares the Runtime APIs related to Pallet Fragments.
//!
//! A Runtime API facilitates this kind of communication between the outer node and the runtime
//!
//! # Background:
//!
//! Each Substrate node contains a runtime.
//! The runtime contains the business logic of the chain.
//! It defines what transactions are valid and invalid and determines how the chain's state changes in response to transactions.
//! The runtime is compiled to Wasm to facilitate runtime upgrades. The "outer node", everything other than the runtime,
//! does not compile to Wasm, only to native.
//! The outer node is responsible for handling peer discovery, transaction pooling, block and transaction gossiping, consensus,
//! and answering RPC calls from the outside world. While performing these tasks, the outer node sometimes needs to query the runtime for information,
//! or provide information to the runtime.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;

use pallet_fragments::{GetDefinitionsParams, GetInstanceOwnerParams, GetInstancesParams};
use sp_std::vec::Vec;

// Declares given traits as runtime apis
//
// For more information, read: https://docs.rs/sp-api/latest/sp_api/macro.decl_runtime_apis.html
sp_api::decl_runtime_apis! {
	/// The trait `FragmentsRuntimeApi` is declared to be a Runtime API
	pub trait FragmentsRuntimeApi<AccountId>
	where
		AccountId: Codec
	{
		/// **Query** and **Return** **Fragmnent Definition(s)** based on **`params`**
		fn get_definitions(params: GetDefinitionsParams<AccountId, Vec<u8>>) -> Result<Vec<u8>, Vec<u8>>;

		/// **Query** and **Return** **Fragmnent Instance(s)** based on **`params`**
		fn get_instances(params: GetInstancesParams<AccountId, Vec<u8>>) -> Result<Vec<u8>, Vec<u8>>;

		/// Query the owner of a Fragment Instance. The return type is a String
		fn get_instance_owner(params: GetInstanceOwnerParams<Vec<u8>>) -> Result<Vec<u8>, Vec<u8>>;
	}
}
