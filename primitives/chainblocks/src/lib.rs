#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate chainblocks;

#[cfg(feature = "std")]
mod details {
	use chainblocks::core::{log, init};

	pub fn _say_hello_world(data: &str) {
		init();
		println!("Hello world STD from: {}", data);
		log("Hello from chainblocks");
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