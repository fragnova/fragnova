use std::sync::Arc;

use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use pallet_protos::GetProtosParams;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_clamor::Hash256;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};

pub use pallet_protos_rpc_runtime_api::ProtosApi as ProtosRuntimeApi;

#[rpc]
pub trait ProtosApi<BlockHash, Tags, AccountId> {
	#[rpc(name = "protos_getProtos")]
	fn get_protos(
		&self,
		params: GetProtosParams<AccountId, String>,
		at: Option<BlockHash>,
	) -> Result<String>;
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

impl<C, Block, Tags, AccountId> ProtosApi<<Block as BlockT>::Hash, Tags, AccountId>
	for Protos<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	C::Api: ProtosRuntimeApi<Block, Tags, AccountId>,
	Tags: Codec,
	AccountId: Codec,
{
	fn get_protos(
		&self,
		params: GetProtosParams<AccountId, String>,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<String> {
		let api = self.client.runtime_api();

		// If the block hash is not supplied in `at`, use the best block's hash
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		let params_no_std = GetProtosParams::<AccountId, Vec<u8>> {
			metadata_keys: params.metadata_keys.map(|list_keys| {
				list_keys.into_iter().map(|s| s.into_bytes()).collect::<Vec<Vec<u8>>>()
			}),
			desc: params.desc,
			from: params.from,
			limit: params.limit,
			owner: params.owner,
			return_owners: params.return_owners,
			tags: params.tags,
		};

		api.get_protos(&at, params_no_std)
			.map(|list_bytes| String::from_utf8(list_bytes).unwrap_or(String::from("")))
			.map_err(|e| RpcError {
				code: ErrorCode::ServerError(Error::RuntimeError.into()),
				message: "Unable to fetch data.".into(),
				data: Some(format!("{:?}", e).into()),
			})
	}
}
