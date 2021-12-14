use core::{
	future::Future,
	pin::Pin,
	task::{Context, Poll},
};

use std::sync::{
	mpsc::{channel, Receiver, Sender},
	Arc,
};

use sp_chainblocks::Hash;

use sc_client_api::client::BlockBackend;
use sp_runtime::traits::Block as BlockT;

pub struct BlockDataFetcher<Client, Block> {
	client: Arc<Client>,
	query_sender: Sender<Hash>,
	query_receiver: Receiver<Hash>,
	result_sender: Sender<Option<Vec<u8>>>,
	result_receiver: Receiver<Option<Vec<u8>>>,
	_marker: std::marker::PhantomData<Block>,
}

impl<Client, Block> BlockDataFetcher<Client, Block>
where
	Client: BlockBackend<Block>,
	Block: BlockT,
{
	pub fn new(client: Arc<Client>) -> Self {
		let (query_sender, query_receiver) = channel();
		let (result_sender, result_receiver) = channel();
		BlockDataFetcher {
			client: client.clone(),
			query_sender,
			query_receiver,
			result_sender,
			result_receiver,
			_marker: Default::default(),
		}
	}
}

impl<Client, Block> Future for BlockDataFetcher<Client, Block>
where
	Client: BlockBackend<Block>,
	Block: BlockT,
{
	type Output = ();

	fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
		Poll::Pending
	}
}
