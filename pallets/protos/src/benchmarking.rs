//! Benchmarking setup for pallet-protos
//!
//! We want to simulate the worst-case scenario for each extrinsic

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{account, benchmarks, vec, whitelisted_caller};
use frame_system::RawOrigin;
use pallet_detach::Pallet as Detach;
use protos::categories::{Categories, TextCategories};
use sp_clamor::CID_PREFIX;
use sp_io::hashing::blake2_256;
use frame_support::{BoundedVec, traits::Get};

use crate::Pallet as Protos;

const SEED: u32 = 0;

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

const MAX_REFERENCES_LENGTH: u32 = 100;
const MAX_DATA_LENGTH: u32 = 1_000_000; // 1 MegaByte

benchmarks! {

	where_clause { where
		T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>
	}

	upload {
		let r in 1 .. MAX_REFERENCES_LENGTH; // `references` length
		let d in 1 .. MAX_DATA_LENGTH; // `data` length
		// `whitelisted_caller()` is a special function from `frame_benchmark`
		// that returns an account whose DB operations (for e.g taking the fee from the account, or updating the nonce)
		// will not be counted for when we run the extrinsic
		let caller: T::AccountId = whitelisted_caller();

		let references: Vec<Hash256> = (0 .. r).into_iter().map(|i| -> Result<Hash256, sp_runtime::DispatchError> {
			let proto_data = format!("{}", i).into_bytes();
			Protos::<T>::upload(
				RawOrigin::Signed(caller.clone()).into(),
				Vec::<Hash256>::new(),
				Categories::Text(TextCategories::Plain),
				Vec::<BoundedVec<u8, _>>::new().try_into().unwrap(),
				None,
				UsageLicense::Closed,
				None,
				proto_data.clone(),
			)?;
			let proto_hash = blake2_256(&proto_data);
			Ok(proto_hash)
		}).collect::<Result::<Vec<Hash256>, _>>()?;
		let category = Categories::Text(TextCategories::Plain);
		let tags: BoundedVec::<BoundedVec<u8, _>, _> = (0 .. T::MaxTags::get()).into_iter().map(|i| {
			format!("{}", i).repeat(<T as pallet::Config>::StringLimit::get() as usize).into_bytes()[0..<T as pallet::Config>::StringLimit::get() as usize].to_vec().try_into().unwrap()
		}).collect::<Vec<BoundedVec<u8, _>>>().try_into().unwrap();
		let linked_asset: Option<LinkedAsset> = None;
		let license = UsageLicense::Closed;
		let data = vec![7u8; d as usize];

	}: upload(RawOrigin::Signed(caller), references, category, tags, linked_asset, license, None, data.clone())
	verify {
		let proto_hash = blake2_256(&data);
		let cid = [&CID_PREFIX[..], &proto_hash[..]].concat();
		let cid = cid.to_base58();
		let cid = [&b"z"[..], cid.as_bytes()].concat();
		assert_last_event::<T>(Event::<T>::Uploaded { proto_hash: proto_hash, cid: cid }.into())
	}

	patch {
		let r in 1 .. MAX_REFERENCES_LENGTH; // `new_references` length
		let d in 1 .. MAX_DATA_LENGTH; // `data` length
		let caller: T::AccountId = whitelisted_caller();

		let proto_data = b"Je suis Data".to_vec();
		Protos::<T>::upload(
			RawOrigin::Signed(caller.clone()).into(),
			Vec::<Hash256>::new(),
			Categories::Text(TextCategories::Plain),
			Vec::<BoundedVec<u8, _>>::new().try_into().unwrap(), // Vec::<Vec<u8>>::new(),
			None,
			UsageLicense::Closed,
			None,
			proto_data.clone(),
		)?;
		let proto_hash = blake2_256(&proto_data);

		let new_references: Vec<Hash256> = (0 .. r).into_iter().map(|i| -> Result<Hash256, sp_runtime::DispatchError> {
			let proto_data = format!("{}", i).into_bytes();
			Protos::<T>::upload(
				RawOrigin::Signed(caller.clone()).into(),
				Vec::<Hash256>::new(),
				Categories::Text(TextCategories::Plain),
				Vec::<BoundedVec<u8, _>>::new().try_into().unwrap(),
				None,
				UsageLicense::Closed,
				None,
				proto_data.clone(),
			)?;
			let proto_hash = blake2_256(&proto_data);
			Ok(proto_hash)
		}).collect::<Result::<Vec<Hash256>, _>>()?;
		let new_tags: Option::<BoundedVec::<BoundedVec<u8, _>, _>> = Some(
			(0 .. T::MaxTags::get()).into_iter().map(|i| {
				format!("{}", i).repeat(<T as pallet::Config>::StringLimit::get() as usize).into_bytes()[0..<T as pallet::Config>::StringLimit::get() as usize].to_vec().try_into().unwrap()
			}).collect::<Vec<BoundedVec<u8, _>>>().try_into().unwrap()
		);
		let license = Some(UsageLicense::Tickets(Compact(100))); // we do this since this will trigger an extra DB write (so it lets us simulate the worst-case scenario)
		let data = vec![7u8; d as usize];

	}: patch(RawOrigin::Signed(caller), proto_hash, license, new_references, new_tags, data.clone())
	verify {
		let patch_hash = blake2_256(&data);
		let cid = [&CID_PREFIX[..], &patch_hash[..]].concat();
		let cid = cid.to_base58();
		let cid = [&b"z"[..], cid.as_bytes()].concat();
		assert_last_event::<T>(Event::<T>::Patched { proto_hash: proto_hash, cid: cid }.into())
	}

	// TODO - Redo this benchmark after `detach()` is refactored to detach Fragments rather than Proto-Fragments
	detach {
		let caller: T::AccountId = whitelisted_caller();

		let proto_data = b"Je suis Data".to_vec();
		Protos::<T>::upload(
			RawOrigin::Signed(caller.clone()).into(),
			Vec::<Hash256>::new(),
			Categories::Text(TextCategories::Plain),
			Vec::<BoundedVec<u8, _>>::new().try_into().unwrap(),
			None,
			UsageLicense::Closed,
			None,
			proto_data.clone()
		)?;
		let proto_hash = blake2_256(&proto_data);

		let target_chain = pallet_detach::SupportedChains::EthereumMainnet;
		let target_account: BoundedVec<u8, _> = vec![7u8; T::DetachAccountLimit::get() as usize].try_into().unwrap();

		// Detach::<T>::add_eth_auth(RawOrigin::Root.into(), sp_core::ecdsa::Public::from_raw(public))?;

		let pre_len: usize = <pallet_detach::DetachRequests<T>>::get().len();

	}: _(RawOrigin::Signed(caller), proto_hash, target_chain, target_account)
	verify {
		assert_eq!(<pallet_detach::DetachRequests<T>>::get().len(), pre_len + 1 as usize);
	}

	transfer {
		let caller: T::AccountId = whitelisted_caller();
		let new_owner: T::AccountId = account("Sample", 100, SEED);

		let proto_data = b"Je suis Data".to_vec();
		Protos::<T>::upload(
			RawOrigin::Signed(caller.clone()).into(),
			Vec::<Hash256>::new(),
			Categories::Text(TextCategories::Plain),
			Vec::<BoundedVec<u8, _>>::new().try_into().unwrap(),
			None,
			UsageLicense::Closed,
			None,
			proto_data.clone(),
		)?;
		let proto_hash = blake2_256(&proto_data);

	}: transfer(RawOrigin::Signed(caller), proto_hash, new_owner.clone())
	verify {
		assert_last_event::<T>(Event::<T>::Transferred { proto_hash: proto_hash, owner_id: new_owner }.into())
	}

	set_metadata { // Benchmark setup phase
		let d in 1 .. MAX_DATA_LENGTH; // 1 byte to 1 Megabyte (I tried 1 byte to 1 Gigabyte, but I got the error: Thread 'main' panicked at 'Failed to allocate memory: "Requested allocation size is too large"', /Users/home/.cargo/git/checkouts/substrate-c784a31f8dac2358/401804d/primitives/io/src/lib.rs:1382)

		// `whitelisted_caller()`'s DB operations will not be counted when we run the extrinsic
		let caller: T::AccountId = whitelisted_caller();

		let proto_data = b"Je suis Data".to_vec();
		Protos::<T>::upload(
			RawOrigin::Signed(caller.clone()).into(),
			Vec::<Hash256>::new(),
			Categories::Text(TextCategories::Plain),
			Vec::<BoundedVec<u8, _>>::new().try_into().unwrap(),
			None,
			UsageLicense::Closed,
			None,
			proto_data.clone(),
		)?;
		let proto_hash = blake2_256(&proto_data);

		let metadata_key: BoundedVec<u8, _> = vec![7u8; <T as pallet::Config>::StringLimit::get() as usize].try_into().unwrap();
		let data: Vec<u8> = vec![7u8; d as usize];

	}: set_metadata(RawOrigin::Signed(caller), proto_hash.clone(), metadata_key.clone(), data) // Execution phase
	verify { // Optional verification phase
		assert_last_event::<T>(Event::<T>::MetadataChanged { proto_hash: proto_hash, metadata_key: metadata_key.into() }.into())
	}

	impl_benchmark_test_suite!(Protos, crate::mock::new_test_ext(), crate::mock::Test);
}
