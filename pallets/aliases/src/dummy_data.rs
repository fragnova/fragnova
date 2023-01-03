use sp_core::ed25519::Public;
use crate::{mock::*, *};

pub struct DummyData {
	pub account_id: Public,
}

impl DummyData {
	pub fn new() -> Self {
		let account_id = sp_core::ed25519::Public::from_raw([1u8; 32]);

		Self {
			account_id,
		}
	}
}
