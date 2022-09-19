use sc_cli::RunCmd;

/// An overarching CLI command definition.
#[derive(Debug, clap::Parser)]
pub struct Cli {
	#[clap(subcommand)]
	pub subcommand: Option<Subcommand>,

	#[clap(flatten)]
	pub run: RunCmd,

	#[clap(short, long)]
	pub geth_url: Option<String>,
}

/// Possible subcommands of the main binary (i.e of the binary `/target/debug/clamor`).
///
/// The basic syntax for running a command:
///
// ```sh
// clamor [subcommand] [flags] [options]
// ```
///
/// # Example
///
///  Here's an example of the subcommands of the node-template program: https://docs.substrate.io/reference/command-line-tools/node-template/#subcommands
///
/// Note: The node-template program provides a working Substrate node with FRAME system pallets
/// and a subset of additional pallets for working with common blockchain functional operations.
/// With its baseline of functional pallets, the node-template serves as a starter kit for building your own blockchain
/// and developing a custom runtime. You can use the node-template program to start a Substrate node and
/// to perform the tasks listed in Subcommands.
/// (https://docs.substrate.io/reference/command-line-tools/node-template)
#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
	/// Key management cli utilities
	#[clap(subcommand)]
	Key(sc_cli::KeySubcommand),
	/// Build a chain specification.
	BuildSpec(sc_cli::BuildSpecCmd),

	/// Validate blocks.
	CheckBlock(sc_cli::CheckBlockCmd),

	/// Export blocks.
	ExportBlocks(sc_cli::ExportBlocksCmd),

	/// Export the state of a given block into a chain spec.
	ExportState(sc_cli::ExportStateCmd),

	/// Import blocks.
	ImportBlocks(sc_cli::ImportBlocksCmd),

	/// Remove the whole chain.
	PurgeChain(sc_cli::PurgeChainCmd),

	/// Revert the chain to a previous state.
	Revert(sc_cli::RevertCmd),

	/// Sub-commands concerned with benchmarking.
	#[clap(subcommand)]
	Benchmark(frame_benchmarking_cli::BenchmarkCmd),
}
