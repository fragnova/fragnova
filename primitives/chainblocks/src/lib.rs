#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
fn _say_hello_world(data: &str) {
	println!("Hello world STD from: {}", data);
}

#[cfg(not(feature = "std"))]
fn _say_hello_world(data: &str) {
}

#[sp_runtime_interface::runtime_interface]
pub trait MyInterface {
	fn say_hello_world(data: &str) {
		_say_hello_world(data);
	}
}