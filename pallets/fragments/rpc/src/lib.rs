//! Implementation of the RPC functions related to Pallet Fragments

use std::sync::Arc;

use codec::Codec;
use jsonrpsee::{
	core::{Error as JsonRpseeError, RpcResult},
	proc_macros::rpc,
	types::error::{CallError, ErrorObject},
};
use pallet_fragments::{GetDefinitionsParams, GetInstanceOwnerParams, GetInstancesParams};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::{Block as BlockT};

pub use pallet_fragments_rpc_runtime_api::FragmentsRuntimeApi;

const RUNTIME_ERROR: i32 = 1;

// Generate both server and client implementations, prepend all the methods with `fragments_` prefix.
// Read more: https://docs.rs/jsonrpsee-proc-macros/0.15.1/jsonrpsee_proc_macros/attr.rpc.html
//
// Note: Do not name any parameter as `params` in any of your RPC Methods, otherwise it won't compile!
#[rpc(client, server, namespace = "fragments")]
pub trait FragmentsRpc<BlockHash, AccountId> {
	/// **Query** and **Return** **Fragment Definition(s)** based on **`params`**
	#[method(name = "getDefinitions")]
	fn get_definitions(
		&self,
		param: GetDefinitionsParams<AccountId, String>,
		at: Option<BlockHash>,
	) -> RpcResult<String>;
	/// **Query** and **Return** **Fragment Instance(s)** based on **`params`**
	#[method(name = "getInstances")]
	fn get_instances(
		&self,
		param: GetInstancesParams<AccountId, String>,
		at: Option<BlockHash>,
	) -> RpcResult<String>;
	/// Query the owner of a Fragment Instance. The return type is a String
	#[method(name = "getInstanceOwner")]
	fn get_instance_owner(
		&self,
		param: GetInstanceOwnerParams<String>,
		at: Option<BlockHash>,
	) -> RpcResult<String>;
}

// Structure that will implement the `FragmentsRpcServer` trait.
// It can have fields, if required, as long as it's still `Send + Sync + 'static`.
// Read More: https://docs.rs/jsonrpsee-proc-macros/0.15.1/jsonrpsee_proc_macros/attr.rpc.html
/// A struct that implements all the RPC functions related to Pallet Fragments (since it implements the trait `FragmentsRpc`)
pub struct FragmentsRpcServerImpl<C, P> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<P>,
}

impl<C, P> FragmentsRpcServerImpl<C, P> {
	/// Create new `Fragments` with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		FragmentsRpcServerImpl { client, _marker: Default::default() }
	}
}

// Note that the trait name we use is `FragmentsRpcServer`, not `FragmentsRpc`!
impl<C, Block, AccountId> FragmentsRpcServer<<Block as BlockT>::Hash, AccountId>
	for FragmentsRpcServerImpl<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	C::Api: FragmentsRuntimeApi<Block, AccountId>,
	AccountId: Codec,
{
	/// **Query** and **Return** **Fragment Definition(s)** based on **`params`**
	fn get_definitions(
		&self,
		param: GetDefinitionsParams<AccountId, String>,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<String> {
		let api = self.client.runtime_api();

		// If the block hash is not supplied in `at`, use the best block's hash
		let at_hash = at.unwrap_or_else(|| self.client.info().best_hash);

		let param_no_std = GetDefinitionsParams::<AccountId, Vec<u8>> {
			metadata_keys: param.metadata_keys.into_iter().map(|s| s.into_bytes()).collect(),
			desc: param.desc,
			from: param.from,
			limit: param.limit,
			owner: param.owner,
			return_owners: param.return_owners,
		};

		let result_outer = api
			.get_definitions(at_hash, param_no_std)
			.map(|bytes| bytes.map(|bytes| String::from_utf8(bytes).unwrap_or_default()));
		match result_outer {
			Err(e) => Err(runtime_error_into_rpc_err(e)),
			Ok(result_outer) => match result_outer {
				Err(e) => Err(runtime_error_into_rpc_err(e)),
				Ok(result_inner) => Ok(result_inner),
			},
		}
	}

	/// **Query** and **Return** **Fragment Instance(s)** based on **`params`**
	fn get_instances(
		&self,
		param: GetInstancesParams<AccountId, String>,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<String> {
		let api = self.client.runtime_api();

		// If the block hash is not supplied in `at`, use the best block's hash
		let at_hash = at.unwrap_or_else(|| self.client.info().best_hash);

		let param_no_std = GetInstancesParams::<AccountId, Vec<u8>> {
			metadata_keys: param.metadata_keys.into_iter().map(|s| s.into_bytes()).collect(),
			desc: param.desc,
			from: param.from,
			limit: param.limit,
			definition_hash: param.definition_hash.into_bytes(),
			owner: param.owner,
			only_return_first_copies: param.only_return_first_copies,
		};

		let result_outer = api
			.get_instances(at_hash, param_no_std)
			.map(|bytes| bytes.map(|bytes| String::from_utf8(bytes).unwrap_or_default()));
		match result_outer {
			Err(e) => Err(runtime_error_into_rpc_err(e)),
			Ok(result_outer) => match result_outer {
				Err(e) => Err(runtime_error_into_rpc_err(e)),
				Ok(result_inner) => Ok(result_inner),
			},
		}
	}

	/// Query the owner of a Fragment Instance. The return type is a String
	fn get_instance_owner(
		&self,
		param: GetInstanceOwnerParams<String>,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<String> {
		let api = self.client.runtime_api();

		// If the block hash is not supplied in `at`, use the best block's hash
		let at_hash = at.unwrap_or_else(|| self.client.info().best_hash);

		let param_no_std = GetInstanceOwnerParams::<Vec<u8>> {
			definition_hash: param.definition_hash.into_bytes(),
			edition_id: param.edition_id,
			copy_id: param.copy_id,
		};

		let result_outer = api
			.get_instance_owner(at_hash, param_no_std)
			.map(|bytes| bytes.map(|bytes| String::from_utf8(bytes).unwrap_or_default()));

		match result_outer {
			Err(e) => Err(runtime_error_into_rpc_err(e)),
			Ok(result_outer) => match result_outer {
				Err(e) => Err(runtime_error_into_rpc_err(e)),
				Ok(result_inner) => Ok(result_inner),
			},
		}
	}
}

fn runtime_error_into_rpc_err(err: impl std::fmt::Debug) -> JsonRpseeError {
	CallError::Custom(ErrorObject::owned(
		RUNTIME_ERROR,
		"Runtime error",
		Some(format!("{:?}", err)),
	))
	.into()
}
