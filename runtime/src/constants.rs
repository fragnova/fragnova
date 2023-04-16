//! A set of constant values used in substrate runtime.

/// Money matters.
pub mod currency {
	use sp_fragnova::Balance;

	pub const MILLICENTS: Balance = 1_000_000_000;
	pub const CENTS: Balance = 1_000 * MILLICENTS; // assume this is worth about a cent.
	pub const DOLLARS: Balance = 100 * CENTS;

	/// The amount of balance a caller has to pay for calling an extrinsic with `bytes` bytes and storage items `items`.
	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 15 * CENTS + (bytes as Balance) * 6 * CENTS
	}
}

/// Time.
pub mod time {
	use sp_fragnova::{BlockNumber, Moment};

	/// This determines the average expected block time that we are targeting.
	/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
	/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
	/// up by `pallet_aura` to implement `fn slot_duration()`.
	///
	/// Change this to adjust the block time.
	pub const MILLISECS_PER_BLOCK: Moment = 6000;
	pub const SECS_PER_BLOCK: Moment = MILLISECS_PER_BLOCK / 1000;

	/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
	///
	/// Note: Currently it is not possible to change the slot duration after the chain has started.
	///       Attempting to do so will brick block production.
	pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;

	// NOTE: Currently it is not possible to change the epoch duration after the chain has started.
	//       Attempting to do so will brick block production.
	pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 10 * MINUTES;
	pub const EPOCH_DURATION_IN_SLOTS: u64 = {
		const SLOT_FILL_RATE: f64 = MILLISECS_PER_BLOCK as f64 / SLOT_DURATION as f64;

		(EPOCH_DURATION_IN_BLOCKS as f64 * SLOT_FILL_RATE) as u64
	};

	// These time units are defined in number of blocks.
	/// Number of blocks that would be added to the Blockchain on average, when one minute passes
	pub const MINUTES: BlockNumber = 60 / (SECS_PER_BLOCK as BlockNumber);
	/// Number of blocks that would be added to the Blockchain on average, when one hour passes
	pub const HOURS: BlockNumber = MINUTES * 60;
	/// Number of blocks that would be added to the Blockchain on average, when one day passes
	pub const DAYS: BlockNumber = HOURS * 24;
}

/// Block matters
pub mod block {
	use sp_runtime::Perbill;
	use frame_support::weights::{constants::WEIGHT_REF_TIME_PER_SECOND, Weight};


	/// We assume that ~10% of the block weight is consumed by `on_initialize` handlers.
	/// This is used to limit the maximal weight of a single extrinsic.
	pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
	/// We allow `Normal` extrinsics to fill up the block up to 75% (i.e up to 75% of the block length and block weight of a block can be filled up by Normal extrinsics).
	/// The rest can be used by Operational extrinsics.
	pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
	/// TODO Documentation - It is actually the "target block weight", not "maximum block weight".
	/// The **maximum block weight** is the **maximum amount of computation time** (assuming no extrinsic class uses its `reserved` space - please see the type `BlockWeights` below to understand what `reserved` is)
	/// that is **allowed to be spent in constructing a block** by a Node.
	///
	/// Here, we set this to 2 seconds because we want a 6 second average block time. (since in Substrate, the **maximum block weight** should be equivalent to **one-third of the target block time** - see the crate documentation above for more information)
	pub const MAXIMUM_BLOCK_WEIGHT: Weight =
		Weight::from_parts(2u64 * WEIGHT_REF_TIME_PER_SECOND, u64::MAX);
	/// The maximum possible length (in bytes) that a Fragnova Block can be
	pub const MAXIMUM_BLOCK_LENGTH: u32 = 16 * 1024 * 1024;
}

pub mod validation_logic {
	/// Maximum length (in bytes) that the metadata data of a Proto-Fragment / Fragment Definition / Fragment Instance can be
	pub const MAXIMUM_METADATA_DATA_LENGTH: usize = 1 * 1024 * 1024;

	// /// **Maximum permitted depth level** that a **descendant call** of the **outermost call of an extrinsic** can be
	// pub const MAXIMUM_NESTED_CALL_DEPTH_LEVEL: u8 = 4;
}

/// Maximum number of iterations for balancing that will be executed in the embedded OCW
/// miner of election provider multi phase.
pub const MINER_MAX_ITERATIONS: u32 = 10;

/// Prints debug output of the `contracts` pallet to stdout if the node is
/// started with `-lruntime::contracts=debug`.
pub const CONTRACTS_DEBUG_OUTPUT: bool = true;
