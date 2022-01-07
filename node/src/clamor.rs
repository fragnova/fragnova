use core::{
	future::Future,
	pin::Pin,
	task::{Context, Poll},
};

use std::sync::{
	mpsc::{channel, Receiver, Sender},
	Arc,
};

use sc_client_api::client::BlockBackend;
use sp_runtime::traits::Block as BlockT;

