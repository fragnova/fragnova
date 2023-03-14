//! Implementation of the RPC functions related to Pallet Protos

use std::sync::Arc;

use base64::{engine::general_purpose::STANDARD, Engine};
use codec::Codec;
use jsonrpsee::{
	core::{Error as JsonRpseeError, RpcResult},
	proc_macros::rpc,
	types::error::{CallError, ErrorObject},
};
use pallet_protos::{GetGenealogyParams, GetProtosParams};
use sc_client_api::BlockBackend;
use sp_api::{ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_runtime::{traits::Block as BlockT};

pub use pallet_protos_rpc_runtime_api::ProtosRuntimeApi;

const RUNTIME_ERROR: i32 = 1;

// Note: Do not name any parameter as "params" in any of your RPC Methods, otherwise it won't compile!
#[rpc(client, server, namespace = "protos")]
pub trait ProtosRpc<BlockHash, AccountId> {
	/// **Query** and **Return** **Proto-Fragment(s)** based on **`params`**.
	/// The **return type** is a **JSON string**.
	#[method(name = "getProtos")]
	fn get_protos(
		&self,
		param: GetProtosParams<AccountId, String>,
		at: Option<BlockHash>,
	) -> RpcResult<String>;

	/// **Query** the Genealogy of a Proto-Fragment based on **`params`**.
	/// The **return type** is a **JSON string** that represents an Adjacency List.
	#[method(name = "getGenealogy")]
	fn get_genealogy(
		&self,
		param: GetGenealogyParams<String>,
		at: Option<BlockHash>,
	) -> RpcResult<String>;

	/// **Query** and **Return** **Proto-Fragment** data based on **`proto_hash`**.
	/// The **return type** is base64 encoded **bytes**.
	#[method(name = "getData")]
	fn get_data(&self, proto_hash: BlockHash, at: Option<BlockHash>) -> RpcResult<String>;
}

/// An implementation of protos specific RPC methods.
pub struct ProtosRpcServerImpl<C, M> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, P> ProtosRpcServerImpl<C, P> {
	/// Create new `ProtosRpcServerImpl` with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

impl<C, Block, AccountId> ProtosRpcServer<<Block as BlockT>::Hash, AccountId>
	for ProtosRpcServerImpl<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	C: BlockBackend<Block>, // used to call the function `BlockBackend::indexed_transaction()` in the RPC method `protos_getData`
	C::Api: ProtosRuntimeApi<Block, AccountId>,
	AccountId: Codec,
{
	/// **Query** and **Return** **Proto-Fragment(s)** based on **`params`**
	fn get_protos(
		&self,
		param: GetProtosParams<AccountId, String>,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<String> {
		let api = self.client.runtime_api();

		// If the block hash is not supplied in `at`, use the best block's hash
		let at_hash = at.unwrap_or_else(|| self.client.info().best_hash);

		let param_no_std = GetProtosParams::<AccountId, Vec<u8>> {
			metadata_keys: param.metadata_keys.into_iter().map(|s| s.into_bytes()).collect(),
			desc: param.desc,
			from: param.from,
			limit: param.limit,
			owner: param.owner,
			return_owners: param.return_owners,
			categories: param.categories,
			tags: param.tags.into_iter().map(|s| s.into_bytes()).collect(),
			exclude_tags: param.exclude_tags.into_iter().map(|s| s.into_bytes()).collect(),
			available: param.available,
		};

		let result_outer = api.get_protos(at_hash, param_no_std).map(|list_bytes| {
			list_bytes.map(|list_bytes| String::from_utf8(list_bytes).unwrap_or(String::from("")))
		});
		match result_outer {
			Err(e) => Err(runtime_error_into_rpc_err(e)),
			Ok(result_outer) => match result_outer {
				Err(e) => Err(runtime_error_into_rpc_err(e)),
				Ok(result_inner) => Ok(result_inner),
			},
		}
	}

	fn get_genealogy(
		&self,
		param: GetGenealogyParams<String>,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<String> {
		let api = self.client.runtime_api();

		// If the block hash is not supplied in `at`, use the best block's hash
		let at_hash = at.unwrap_or_else(|| self.client.info().best_hash);

		let param_no_std = GetGenealogyParams::<Vec<u8>> {
			proto_hash: param.proto_hash.into_bytes(),
			get_ancestors: param.get_ancestors,
		};

		let result = api.get_genealogy(at_hash, param_no_std).map(|list_bytes| {
			list_bytes.map(|list_bytes| String::from_utf8(list_bytes).unwrap_or(String::from("")))
		});
		match result {
			Err(e) => Err(runtime_error_into_rpc_err(e)),
			Ok(result) => match result {
				Err(e) => Err(runtime_error_into_rpc_err(e)),
				Ok(result) => Ok(result),
			},
		}
	}

	fn get_data(
		&self,
		proto_hash: <Block as BlockT>::Hash,
		_at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<String> {
		let tx = self.client.indexed_transaction(proto_hash);
		match tx {
			Ok(tx) => match tx {
				Some(data) => {
					let data_str = STANDARD.encode(data);
					Ok(data_str)
				},
				None => Err(runtime_error_into_rpc_err("No indexed transaction found")),
			},
			Err(e) => Err(runtime_error_into_rpc_err(e)),
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
