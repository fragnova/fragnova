// This file is part of Substrate.

// Copyright (C) 2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Autogenerated weights for pallet_fragments
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-10-13, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `alepc`, CPU: `11th Gen Intel(R) Core(TM) i7-1195G7 @ 2.90GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/fragnova
// benchmark
// pallet
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=pallet_fragments
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./pallets/fragments/src/weights.rs
// --template=./.maintain/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_fragments.
pub trait WeightInfo {
	fn create(n: u32, ) -> Weight;
	fn publish() -> Weight;
	fn unpublish() -> Weight;
	fn mint_definition_that_has_non_unique_capability(q: u32, ) -> Weight;
	fn mint_definition_that_has_unique_capability(d: u32, ) -> Weight;
	fn buy_definition_that_has_non_unique_capability(q: u32, ) -> Weight;
	fn buy_definition_that_has_unique_capability(d: u32, ) -> Weight;
	fn benchmark_give_instance_that_does_not_have_copy_perms() -> Weight;
	fn benchmark_give_instance_that_has_copy_perms() -> Weight;
}

/// Weights for pallet_fragments using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	// Storage: Protos Protos (r:1 w:0)
	// Storage: Detach DetachedHashes (r:1 w:0)
	// Storage: Fragments Definitions (r:1 w:1)
	// Storage: Assets Asset (r:1 w:0)
	// Storage: System Account (r:1 w:1)
	// Storage: Fragments Proto2Fragments (r:1 w:1)
	/// The range of component `n` is `[1, 100]`.
	fn create(n: u32, ) -> Weight {
		Weight::from_ref_time(53_058_000 as u64)
			// Standard Error: 9_000
			.saturating_add(Weight::from_ref_time(10_000 as u64).saturating_mul(n as u64))
			.saturating_add(T::DbWeight::get().reads(6 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	// Storage: Fragments Definitions (r:1 w:0)
	// Storage: Protos Protos (r:1 w:0)
	// Storage: Detach DetachedHashes (r:1 w:0)
	// Storage: Fragments Publishing (r:1 w:1)
	// Storage: Fragments EditionsCount (r:1 w:0)
	fn publish() -> Weight {
		Weight::from_ref_time(34_050_000 as u64)
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Fragments Definitions (r:1 w:0)
	// Storage: Protos Protos (r:1 w:0)
	// Storage: Detach DetachedHashes (r:1 w:0)
	// Storage: Fragments Publishing (r:1 w:1)
	fn unpublish() -> Weight {
		Weight::from_ref_time(32_337_000 as u64)
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Fragments Definitions (r:1 w:1)
	// Storage: Protos Protos (r:1 w:0)
	// Storage: Detach DetachedHashes (r:1 w:0)
	// Storage: unknown [0x3a65787472696e7369635f696e646578] (r:1 w:0)
	// Storage: Fragments EditionsCount (r:1 w:1)
	// Storage: Fragments Inventory (r:1 w:1)
	// Storage: Fragments Owners (r:1 w:1)
	// Storage: Fragments Fragments (r:0 w:1)
	// Storage: Fragments CopiesCount (r:0 w:1)
	/// The range of component `q` is `[1, 100]`.
	fn mint_definition_that_has_non_unique_capability(q: u32, ) -> Weight {
		Weight::from_ref_time(36_569_000 as u64)
			// Standard Error: 133_000
			.saturating_add(Weight::from_ref_time(8_255_000 as u64).saturating_mul(q as u64))
			.saturating_add(T::DbWeight::get().reads(7 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
			.saturating_add(T::DbWeight::get().writes((2 as u64).saturating_mul(q as u64)))
	}
	// Storage: Fragments Definitions (r:1 w:1)
	// Storage: Protos Protos (r:1 w:0)
	// Storage: Detach DetachedHashes (r:1 w:0)
	// Storage: unknown [0x3a65787472696e7369635f696e646578] (r:1 w:0)
	// Storage: Fragments UniqueData2Edition (r:1 w:1)
	// Storage: Fragments EditionsCount (r:1 w:1)
	// Storage: Fragments Inventory (r:1 w:1)
	// Storage: Fragments Owners (r:1 w:1)
	// Storage: Fragments Fragments (r:0 w:1)
	// Storage: Fragments CopiesCount (r:0 w:1)
	/// The range of component `d` is `[1, 1000000]`.
	fn mint_definition_that_has_unique_capability(d: u32, ) -> Weight {
		Weight::from_ref_time(77_922_000 as u64)
			// Standard Error: 0
			.saturating_add(Weight::from_ref_time(1_000 as u64).saturating_mul(d as u64))
			.saturating_add(T::DbWeight::get().reads(8 as u64))
			.saturating_add(T::DbWeight::get().writes(7 as u64))
	}
	// Storage: Fragments Publishing (r:1 w:1)
	// Storage: Fragments Definitions (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: unknown [0x3a65787472696e7369635f696e646578] (r:1 w:0)
	// Storage: Fragments EditionsCount (r:1 w:1)
	// Storage: Fragments Inventory (r:1 w:1)
	// Storage: Fragments Owners (r:1 w:1)
	// Storage: Fragments Fragments (r:0 w:1)
	// Storage: Fragments CopiesCount (r:0 w:1)
	/// The range of component `q` is `[1, 100]`.
	fn buy_definition_that_has_non_unique_capability(q: u32, ) -> Weight {
		Weight::from_ref_time(49_276_000 as u64)
			// Standard Error: 32_000
			.saturating_add(Weight::from_ref_time(7_485_000 as u64).saturating_mul(q as u64))
			.saturating_add(T::DbWeight::get().reads(7 as u64))
			.saturating_add(T::DbWeight::get().writes(6 as u64))
			.saturating_add(T::DbWeight::get().writes((2 as u64).saturating_mul(q as u64)))
	}
	// Storage: Fragments Publishing (r:1 w:1)
	// Storage: Fragments Definitions (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: unknown [0x3a65787472696e7369635f696e646578] (r:1 w:0)
	// Storage: Fragments UniqueData2Edition (r:1 w:1)
	// Storage: Fragments EditionsCount (r:1 w:1)
	// Storage: Fragments Inventory (r:1 w:1)
	// Storage: Fragments Owners (r:1 w:1)
	// Storage: Fragments Fragments (r:0 w:1)
	// Storage: Fragments CopiesCount (r:0 w:1)
	/// The range of component `d` is `[1, 1000000]`.
	fn buy_definition_that_has_unique_capability(d: u32, ) -> Weight {
		Weight::from_ref_time(33_800_000 as u64)
			// Standard Error: 0
			.saturating_add(Weight::from_ref_time(2_000 as u64).saturating_mul(d as u64))
			.saturating_add(T::DbWeight::get().reads(8 as u64))
			.saturating_add(T::DbWeight::get().writes(9 as u64))
	}
	// Storage: Fragments Fragments (r:1 w:1)
	// Storage: Fragments Inventory (r:2 w:2)
	// Storage: Fragments Owners (r:2 w:2)
	fn benchmark_give_instance_that_does_not_have_copy_perms() -> Weight {
		Weight::from_ref_time(36_669_000 as u64)
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().writes(5 as u64))
	}
	// Storage: Fragments Fragments (r:1 w:1)
	// Storage: Fragments Inventory (r:2 w:1)
	// Storage: Fragments CopiesCount (r:1 w:1)
	// Storage: Fragments Owners (r:1 w:1)
	// Storage: Fragments Expirations (r:1 w:1)
	fn benchmark_give_instance_that_has_copy_perms() -> Weight {
		Weight::from_ref_time(30_168_000 as u64)
			.saturating_add(T::DbWeight::get().reads(6 as u64))
			.saturating_add(T::DbWeight::get().writes(5 as u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: Protos Protos (r:1 w:0)
	// Storage: Detach DetachedHashes (r:1 w:0)
	// Storage: Fragments Definitions (r:1 w:1)
	// Storage: Assets Asset (r:1 w:0)
	// Storage: System Account (r:1 w:1)
	// Storage: Fragments Proto2Fragments (r:1 w:1)
	/// The range of component `n` is `[1, 100]`.
	fn create(n: u32, ) -> Weight {
		Weight::from_ref_time(53_058_000 as u64)
			// Standard Error: 9_000
			.saturating_add(Weight::from_ref_time(10_000 as u64).saturating_mul(n as u64))
			.saturating_add(RocksDbWeight::get().reads(6 as u64))
			.saturating_add(RocksDbWeight::get().writes(3 as u64))
	}
	// Storage: Fragments Definitions (r:1 w:0)
	// Storage: Protos Protos (r:1 w:0)
	// Storage: Detach DetachedHashes (r:1 w:0)
	// Storage: Fragments Publishing (r:1 w:1)
	// Storage: Fragments EditionsCount (r:1 w:0)
	fn publish() -> Weight {
		Weight::from_ref_time(34_050_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(5 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: Fragments Definitions (r:1 w:0)
	// Storage: Protos Protos (r:1 w:0)
	// Storage: Detach DetachedHashes (r:1 w:0)
	// Storage: Fragments Publishing (r:1 w:1)
	fn unpublish() -> Weight {
		Weight::from_ref_time(32_337_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(4 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: Fragments Definitions (r:1 w:1)
	// Storage: Protos Protos (r:1 w:0)
	// Storage: Detach DetachedHashes (r:1 w:0)
	// Storage: unknown [0x3a65787472696e7369635f696e646578] (r:1 w:0)
	// Storage: Fragments EditionsCount (r:1 w:1)
	// Storage: Fragments Inventory (r:1 w:1)
	// Storage: Fragments Owners (r:1 w:1)
	// Storage: Fragments Fragments (r:0 w:1)
	// Storage: Fragments CopiesCount (r:0 w:1)
	/// The range of component `q` is `[1, 100]`.
	fn mint_definition_that_has_non_unique_capability(q: u32, ) -> Weight {
		Weight::from_ref_time(36_569_000 as u64)
			// Standard Error: 133_000
			.saturating_add(Weight::from_ref_time(8_255_000 as u64).saturating_mul(q as u64))
			.saturating_add(RocksDbWeight::get().reads(7 as u64))
			.saturating_add(RocksDbWeight::get().writes(4 as u64))
			.saturating_add(RocksDbWeight::get().writes((2 as u64).saturating_mul(q as u64)))
	}
	// Storage: Fragments Definitions (r:1 w:1)
	// Storage: Protos Protos (r:1 w:0)
	// Storage: Detach DetachedHashes (r:1 w:0)
	// Storage: unknown [0x3a65787472696e7369635f696e646578] (r:1 w:0)
	// Storage: Fragments UniqueData2Edition (r:1 w:1)
	// Storage: Fragments EditionsCount (r:1 w:1)
	// Storage: Fragments Inventory (r:1 w:1)
	// Storage: Fragments Owners (r:1 w:1)
	// Storage: Fragments Fragments (r:0 w:1)
	// Storage: Fragments CopiesCount (r:0 w:1)
	/// The range of component `d` is `[1, 1000000]`.
	fn mint_definition_that_has_unique_capability(d: u32, ) -> Weight {
		Weight::from_ref_time(77_922_000 as u64)
			// Standard Error: 0
			.saturating_add(Weight::from_ref_time(1_000 as u64).saturating_mul(d as u64))
			.saturating_add(RocksDbWeight::get().reads(8 as u64))
			.saturating_add(RocksDbWeight::get().writes(7 as u64))
	}
	// Storage: Fragments Publishing (r:1 w:1)
	// Storage: Fragments Definitions (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: unknown [0x3a65787472696e7369635f696e646578] (r:1 w:0)
	// Storage: Fragments EditionsCount (r:1 w:1)
	// Storage: Fragments Inventory (r:1 w:1)
	// Storage: Fragments Owners (r:1 w:1)
	// Storage: Fragments Fragments (r:0 w:1)
	// Storage: Fragments CopiesCount (r:0 w:1)
	/// The range of component `q` is `[1, 100]`.
	fn buy_definition_that_has_non_unique_capability(q: u32, ) -> Weight {
		Weight::from_ref_time(49_276_000 as u64)
			// Standard Error: 32_000
			.saturating_add(Weight::from_ref_time(7_485_000 as u64).saturating_mul(q as u64))
			.saturating_add(RocksDbWeight::get().reads(7 as u64))
			.saturating_add(RocksDbWeight::get().writes(6 as u64))
			.saturating_add(RocksDbWeight::get().writes((2 as u64).saturating_mul(q as u64)))
	}
	// Storage: Fragments Publishing (r:1 w:1)
	// Storage: Fragments Definitions (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: unknown [0x3a65787472696e7369635f696e646578] (r:1 w:0)
	// Storage: Fragments UniqueData2Edition (r:1 w:1)
	// Storage: Fragments EditionsCount (r:1 w:1)
	// Storage: Fragments Inventory (r:1 w:1)
	// Storage: Fragments Owners (r:1 w:1)
	// Storage: Fragments Fragments (r:0 w:1)
	// Storage: Fragments CopiesCount (r:0 w:1)
	/// The range of component `d` is `[1, 1000000]`.
	fn buy_definition_that_has_unique_capability(d: u32, ) -> Weight {
		Weight::from_ref_time(33_800_000 as u64)
			// Standard Error: 0
			.saturating_add(Weight::from_ref_time(2_000 as u64).saturating_mul(d as u64))
			.saturating_add(RocksDbWeight::get().reads(8 as u64))
			.saturating_add(RocksDbWeight::get().writes(9 as u64))
	}
	// Storage: Fragments Fragments (r:1 w:1)
	// Storage: Fragments Inventory (r:2 w:2)
	// Storage: Fragments Owners (r:2 w:2)
	fn benchmark_give_instance_that_does_not_have_copy_perms() -> Weight {
		Weight::from_ref_time(36_669_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(5 as u64))
			.saturating_add(RocksDbWeight::get().writes(5 as u64))
	}
	// Storage: Fragments Fragments (r:1 w:1)
	// Storage: Fragments Inventory (r:2 w:1)
	// Storage: Fragments CopiesCount (r:1 w:1)
	// Storage: Fragments Owners (r:1 w:1)
	// Storage: Fragments Expirations (r:1 w:1)
	fn benchmark_give_instance_that_has_copy_perms() -> Weight {
		Weight::from_ref_time(30_168_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(6 as u64))
			.saturating_add(RocksDbWeight::get().writes(5 as u64))
	}
}
