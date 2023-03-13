//! A collection of node-specific RPC methods.
//!
//! Since `substrate` core functionality makes no assumptions
//! about the modules used inside the runtime, so do
//! RPC methods defined in `sc-rpc` crate.
//! It means that `client/rpc` can't have any methods that
//! need some strong assumptions about the particular runtime.
//!
//! The RPCs available in this crate however can make some assumptions
//! about how the runtime is constructed and what FRAME pallets
//! are part of it. Therefore all node-runtime-specific RPCs can
//! be placed here or imported from corresponding FRAME RPC definitions.

#![warn(missing_docs)]

use std::sync::Arc;

use fragnova_runtime::{opaque::Block, AccountId, Balance, Index};
use jsonrpsee::RpcModule;
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};

pub use sc_rpc_api::DenyUnsafe;

/// Full client dependencies.
pub struct FullDeps<C, P> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// Whether to deny unsafe calls
	pub deny_unsafe: DenyUnsafe,
}

/// Instantiate all Full RPC extensions.
pub fn create_full<C, P>(
	deps: FullDeps<C, P>,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
	C: Send + Sync + 'static,
	C: sc_client_api::BlockBackend<Block>, // used for in the RPC method `protos_getData`

	C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
	C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
	C::Api: pallet_protos_rpc::ProtosRuntimeApi<Block, AccountId>,
	C::Api: pallet_fragments_rpc::FragmentsRuntimeApi<Block, AccountId>,
	C::Api: BlockBuilder<Block>,
	P: TransactionPool + 'static,
{
	use pallet_fragments_rpc::{FragmentsRpcServer, FragmentsRpcServerImpl};
	use pallet_protos_rpc::{ProtosRpcServer, ProtosRpcServerImpl};
	use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
	// use sc_rpc::dev::{Dev, DevApiServer};
	use substrate_frame_rpc_system::{System, SystemApiServer};

	let mut module = RpcModule::new(());
	let FullDeps { client, pool, deny_unsafe } = deps;

	module.merge(System::new(client.clone(), pool.clone(), deny_unsafe).into_rpc())?;
	// Making synchronous calls in light client freezes the browser currently,
	// more context: https://github.com/paritytech/substrate/pull/3480
	// These RPCs should use an asynchronous caller instead.
	module.merge(TransactionPayment::new(client.clone()).into_rpc())?;
	module.merge(ProtosRpcServerImpl::new(client.clone()).into_rpc())?;
	module.merge(FragmentsRpcServerImpl::new(client).into_rpc())?;

	// TODO Review - This line is not executed in the `node-template` (https://github.com/paritytech/substrate/blob/polkadot-v0.9.37/bin/node-template/node/src/rpc.rs)
	// (but it's executed in `node` (https://github.com/paritytech/substrate/blob/polkadot-v0.9.37/bin/node/rpc/src/lib.rs)), that's why I've commented it out
	// module.merge(Dev::new(client.clone(), deny_unsafe).into_rpc())?;

	Ok(module)
}
