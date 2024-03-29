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

//! Autogenerated weights for pallet_accounts
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
// --pallet=pallet_accounts
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./pallets/accounts/src/weights.rs
// --template=./.maintain/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_accounts.
pub trait WeightInfo {
	fn add_key() -> Weight;
	fn del_key() -> Weight;
	fn link() -> Weight;
	fn unlink() -> Weight;
	fn internal_lock_update() -> Weight;
	fn sponsor_account() -> Weight;
	fn add_sponsor() -> Weight;
	fn remove_sponsor() -> Weight;
}

/// Weights for pallet_accounts using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	// Storage: Accounts FragKeys (r:1 w:1)
	fn add_key() -> Weight {
		Weight::from_ref_time(6_401_000 as u64)
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Accounts FragKeys (r:1 w:1)
	fn del_key() -> Weight {
		Weight::from_ref_time(6_890_000 as u64)
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Accounts EVMLinks (r:1 w:1)
	// Storage: Accounts EVMLinksReverse (r:1 w:1)
	// Storage: Accounts EthReservedNova (r:1 w:0)
	fn link() -> Weight {
		Weight::from_ref_time(58_668_000 as u64)
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	// Storage: Accounts EVMLinks (r:1 w:1)
	// Storage: Accounts EVMLinksReverse (r:1 w:1)
	// Storage: Accounts PendingUnlinks (r:1 w:1)
	fn unlink() -> Weight {
		Weight::from_ref_time(23_631_000 as u64)
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	// Storage: Accounts EVMLinkVotingClosed (r:1 w:1)
	// Storage: Accounts EVMLinksReverse (r:1 w:0)
	// Storage: Accounts EthReservedNova (r:0 w:1)
	// Storage: Accounts EthLockedFrag (r:0 w:1)
	fn internal_lock_update() -> Weight {
		Weight::from_ref_time(67_671_000 as u64)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	// Storage: Accounts ExternalAuthorities (r:1 w:0)
	// Storage: Proxy Proxies (r:1 w:1)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: Accounts ExternalID2Account (r:0 w:1)
	fn sponsor_account() -> Weight {
		Weight::from_ref_time(37_290_000 as u64)
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	// Storage: Accounts ExternalAuthorities (r:1 w:1)
	fn add_sponsor() -> Weight {
		Weight::from_ref_time(6_491_000 as u64)
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Accounts ExternalAuthorities (r:1 w:1)
	fn remove_sponsor() -> Weight {
		Weight::from_ref_time(7_589_000 as u64)
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: Accounts FragKeys (r:1 w:1)
	fn add_key() -> Weight {
		Weight::from_ref_time(6_401_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(1 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: Accounts FragKeys (r:1 w:1)
	fn del_key() -> Weight {
		Weight::from_ref_time(6_890_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(1 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: Accounts EVMLinks (r:1 w:1)
	// Storage: Accounts EVMLinksReverse (r:1 w:1)
	// Storage: Accounts EthReservedNova (r:1 w:0)
	fn link() -> Weight {
		Weight::from_ref_time(58_668_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(4 as u64))
			.saturating_add(RocksDbWeight::get().writes(2 as u64))
	}
	// Storage: Accounts EVMLinks (r:1 w:1)
	// Storage: Accounts EVMLinksReverse (r:1 w:1)
	// Storage: Accounts PendingUnlinks (r:1 w:1)
	fn unlink() -> Weight {
		Weight::from_ref_time(23_631_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(3 as u64))
			.saturating_add(RocksDbWeight::get().writes(3 as u64))
	}
	// Storage: Accounts EVMLinkVotingClosed (r:1 w:1)
	// Storage: Accounts EVMLinksReverse (r:1 w:0)
	// Storage: Accounts EthReservedNova (r:0 w:1)
	// Storage: Accounts EthLockedFrag (r:0 w:1)
	fn internal_lock_update() -> Weight {
		Weight::from_ref_time(67_671_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(2 as u64))
			.saturating_add(RocksDbWeight::get().writes(4 as u64))
	}
	// Storage: Accounts ExternalAuthorities (r:1 w:0)
	// Storage: Proxy Proxies (r:1 w:1)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: Accounts ExternalID2Account (r:0 w:1)
	fn sponsor_account() -> Weight {
		Weight::from_ref_time(37_290_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(3 as u64))
			.saturating_add(RocksDbWeight::get().writes(2 as u64))
	}
	// Storage: Accounts ExternalAuthorities (r:1 w:1)
	fn add_sponsor() -> Weight {
		Weight::from_ref_time(6_491_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(1 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: Accounts ExternalAuthorities (r:1 w:1)
	fn remove_sponsor() -> Weight {
		Weight::from_ref_time(7_589_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(1 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
}
