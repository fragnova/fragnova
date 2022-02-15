use std::sync::Arc;

use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_chainblocks::Hash256;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};

pub use pallet_protos_rpc_runtime_api::ProtosApi as ProtosRuntimeApi;

#[rpc]
pub trait ProtosApi<BlockHash, Tags> {
	#[rpc(name = "get_proto_by_tags")]
	fn get_proto_by_tags(&self, tags: Tags, at: Option<BlockHash>) -> Result<Option<Vec<Hash256>>>;
}

/// An implementation of protos specific RPC methods.
pub struct Protos<C, M> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, P> Protos<C, P> {
	/// Create new `Protos` with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Protos { client, _marker: Default::default() }
	}
}

/// Error type of this RPC api.
pub enum Error {
	/// The transaction was not decoded.
	DecodeError,
	/// The call to runtime failed.
	RuntimeError,
}

impl From<Error> for i64 {
	fn from(e: Error) -> i64 {
		match e {
			Error::RuntimeError => 1,
			Error::DecodeError => 2,
		}
	}
}

impl<C, Block, Tags> ProtosApi<<Block as BlockT>::Hash, Tags> for Protos<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	C::Api: ProtosRuntimeApi<Block, Tags>,
	Tags: Codec,
{
	fn get_proto_by_tags(
		&self,
		tags: Tags,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Option<Vec<Hash256>>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		api.get_proto_by_tags(&at, tags).map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "Unable to fetch data.".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}
}
