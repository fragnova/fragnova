//! Implementation of the RPC functions related to Pallet Protos

use std::sync::Arc;

use codec::Codec;
use jsonrpsee::{
	core::{async_trait, Error as JsonRpseeError, RpcResult},
	proc_macros::rpc,
	types::error::{CallError, ErrorObject},
};
use pallet_protos::{GetProtosParams, GetGenealogyParams};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};

pub use pallet_protos_rpc_runtime_api::ProtosRuntimeApi;

const RUNTIME_ERROR: i32 = 1;

#[rpc(client, server, namespace = "protos")]
pub trait ProtosRpc<BlockHash, AccountId> {
	/// **Query** and **Return** **Proto-Fragment(s)** based on **`params`**.
	/// The **return type** is a **JSON string**.
	#[method(name = "getProtos")]
	fn get_protos(
		&self,
		params: GetProtosParams<AccountId, String>,
		at: Option<BlockHash>,
	) -> RpcResult<String>;

	/// **Query** the Genealogy of a Proto-Fragment based on **`params`**.
	/// The **return type** is a **JSON string** that represents an Adjacency List.
	#[method(name = "getGenealogy")]
	fn get_genealogy(
		&self,
		params: GetGenealogyParams<String>,
		at: Option<BlockHash>,
	) -> RpcResult<String>;
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

#[async_trait]
impl<C, Block, AccountId> ProtosRpcServer<<Block as BlockT>::Hash, AccountId> for ProtosRpcServerImpl<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	C::Api: ProtosRuntimeApi<Block, AccountId>,
	AccountId: Codec,
{
	/// **Query** and **Return** **Proto-Fragment(s)** based on **`params`**
	fn get_protos(
		&self,
		params: GetProtosParams<AccountId, String>,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<String> {
		let api = self.client.runtime_api();

		// If the block hash is not supplied in `at`, use the best block's hash
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		let params_no_std = GetProtosParams::<AccountId, Vec<u8>> {
			metadata_keys: params.metadata_keys.into_iter().map(|s| s.into_bytes()).collect(),
			desc: params.desc,
			from: params.from,
			limit: params.limit,
			owner: params.owner,
			return_owners: params.return_owners,
			categories: params.categories,
			tags: params.tags.into_iter().map(|s| s.into_bytes()).collect(),
			exclude_tags: params.exclude_tags.into_iter().map(|s| s.into_bytes()).collect(),
			available: params.available,
		};

		let result_outer = api.get_protos(&at, params_no_std).map(|list_bytes| {
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
		params: GetGenealogyParams<String>,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<String> {
		let api = self.client.runtime_api();

		// If the block hash is not supplied in `at`, use the best block's hash
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		let params_no_std = GetGenealogyParams::<Vec<u8>> {
			proto_hash: params.proto_hash.into_bytes(),
			get_ancestors: params.get_ancestors,
		};

		let result = api.get_genealogy(&at, params_no_std).map(|list_bytes| {
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
}

fn runtime_error_into_rpc_err(err: impl std::fmt::Debug) -> JsonRpseeError {
	CallError::Custom(ErrorObject::owned(
		RUNTIME_ERROR,
		"Runtime error",
		Some(format!("{:?}", err)),
	))
	.into()
}
