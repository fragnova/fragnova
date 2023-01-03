use sp_core::ed25519::Public;
use crate::{mock::*, *};

pub struct DummyData {
	pub account_id: Public,
	pub account_id_2: Public,
}

impl DummyData {
	pub fn new() -> Self {
		let account_id = sp_core::ed25519::Public::from_raw([1u8; 32]);
		let account_id_2 = sp_core::ed25519::Public::from_raw([2u8; 32]);

		Self {
			account_id,
			account_id_2,
		}
	}
}
