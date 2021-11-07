#![cfg_attr(not(feature = "std"), no_std)]

#[sp_runtime_interface::runtime_interface]
pub trait MyInterface {
	fn say_hello_world(data: &str) {
		println!("Hello world from: {}", data);
	}
}