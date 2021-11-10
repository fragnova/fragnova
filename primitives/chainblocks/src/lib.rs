#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate chainblocks;

#[cfg(feature = "std")]
#[macro_use]
extern crate lazy_static;

#[cfg(feature = "std")]
mod details {
	use std::sync::Mutex;
	use std::convert::TryInto;
	use chainblocks::{cbl_env, CBVAR_FLAGS_EXTERNAL};
	use chainblocks::core::{cloneVar, destroyVar, setExternalVariable};
	use chainblocks::types::{Node, Var, ClonedVar};

	pub fn _say_hello_world(data: &str) {
		lazy_static! {
			static ref VAR: Mutex<ClonedVar> = Mutex::new({
				let mut var = ClonedVar::default();
				var.0.flags |= CBVAR_FLAGS_EXTERNAL as u8;
				var
			});
			static ref NODE: Node = {
				let node = Node::default();
				// let mut chain_var = cbl_env!("(defloop test (Msg \"Hello\"))");
				let mut chain_var = cbl_env!("(Chain \"test\" :Looped .text (ExpectString) (Log))");
				let chain = chain_var.try_into().unwrap();
				setExternalVariable(chain, "text", &mut VAR.lock().unwrap().0);
				node.schedule(chain);
				destroyVar(&mut chain_var);
				node
			};
		}
		let v: Var = data.into();
		cloneVar(&mut VAR.lock().unwrap().0, &v);
		NODE.tick();
	}
}

#[cfg(not(feature = "std"))]
mod details {
	pub fn _say_hello_world(data: &str) {
	}
}

#[sp_runtime_interface::runtime_interface]
pub trait MyInterface {
	fn say_hello_world(data: &str) {
		details::_say_hello_world(data);
	}
}

#[cfg(feature = "std")]
pub fn init() {
	use chainblocks::core::{init};
	use chainblocks::{cbl_env, cblog};

	// needs to go first!
	init();

	cblog!("Chainblocks initializing...");

	// load default chains
	let chain = cbl_env!(include_str!("validate_fragment.edn"));

	cblog!("Chainblocks initialized!");
}