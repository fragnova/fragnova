//! Builds a WASM binary of the clamor project
//!
//! # What's a build script (build.rs)?
//!
//! The build script is simply another Rust file that will be compiled and invoked prior to compiling anything else in the package.
//! Hence it can be used to fulfil pre-requisites of your crate.
//!
//! All the lines printed to stdout by a build script are written to a file like target/debug/build/<pkg>/output (the precise location may depend on your configuration).
//! If you would like to see such output directly in your terminal, invoke cargo as 'very verbose' with the -vv flag.

/// Wasm builder is a utility for building a project as a Wasm binary
///
/// See to learn more: https://docs.rs/wasm-builder/latest/wasm_builder/
use substrate_wasm_builder::WasmBuilder;

fn main() {
	WasmBuilder::new()
		.with_current_project()
		.export_heap_base()
		.import_memory()
		.build()
}
