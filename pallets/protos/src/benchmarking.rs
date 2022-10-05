//! Benchmarking setup for pallet-fragments
//!
//! We want to simulate the worst-case scenario for each extrinsic

use super::*;
#[allow(unused)]
use frame_benchmarking::{account, benchmarks, vec, whitelisted_caller};
use frame_system::RawOrigin;
use pallet_detach::Pallet as Detach;
use protos::categories::{Categories, TextCategories};
use sp_clamor::CID_PREFIX;
use sp_io::hashing::blake2_256;

use crate::Pallet as Protos;

const SEED: u32 = 0;

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

const MAX_REFERENCES_LENGTH: u32 = 100;
const MAX_TAGS_LENGTH: u32 = 100;
const MAX_DATA_LENGTH: u32 = 1_000_000; // 1 MegaByte

benchmarks! {

	where_clause { where
		T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>
	}

	upload_benchmark {
		let r in 1 .. MAX_REFERENCES_LENGTH; // `references` length
		let t in 1 .. MAX_TAGS_LENGTH; // `tags` length
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
				Vec::<Vec<u8>>::new(),
				None,
				UsageLicense::Closed,
				proto_data.clone()
			)?;
			let proto_hash = blake2_256(&proto_data);
			Ok(proto_hash)
		}).collect::<Result::<Vec<Hash256>, _>>()?;
		let category = Categories::Text(TextCategories::Plain);
		let tags = (0 .. t).into_iter().map(|i| {
			format!("{}", i).into_bytes().to_vec()
		}).collect::<Vec<Vec<u8>>>();
		let linked_asset: Option<LinkedAsset> = None;
		let license = UsageLicense::Closed;
		let data = vec![7u8; d as usize];

	}: upload(RawOrigin::Signed(caller), references, category, tags, linked_asset, license, data.clone())
	verify {
		let proto_hash = blake2_256(&data);
		let cid = [&CID_PREFIX[..], &proto_hash[..]].concat();
		let cid = cid.to_base58();
		let cid = [&b"z"[..], cid.as_bytes()].concat();
		assert_last_event::<T>(Event::<T>::Uploaded { proto_hash: proto_hash, cid: cid }.into())
	}

	patch_benchmark {
		let r in 1 .. MAX_REFERENCES_LENGTH; // `new_references` length
		let t in 1 .. MAX_TAGS_LENGTH; // `new_tags` length
		let d in 1 .. MAX_DATA_LENGTH; // `data` length
		let caller: T::AccountId = whitelisted_caller();

		let proto_data = b"Je suis Data".to_vec();
		Protos::<T>::upload(
			RawOrigin::Signed(caller.clone()).into(),
			Vec::<Hash256>::new(),
			Categories::Text(TextCategories::Plain),
			Vec::<Vec<u8>>::new(),
			None,
			UsageLicense::Closed,
			proto_data.clone()
		)?;
		let proto_hash = blake2_256(&proto_data);

		let new_references: Vec<Hash256> = (0 .. r).into_iter().map(|i| -> Result<Hash256, sp_runtime::DispatchError> {
			let proto_data = format!("{}", i).into_bytes();
			Protos::<T>::upload(
				RawOrigin::Signed(caller.clone()).into(),
				Vec::<Hash256>::new(),
				Categories::Text(TextCategories::Plain),
				Vec::<Vec<u8>>::new(),
				None,
				UsageLicense::Closed,
				proto_data.clone()
			)?;
			let proto_hash = blake2_256(&proto_data);
			Ok(proto_hash)
		}).collect::<Result::<Vec<Hash256>, _>>()?;
		let new_tags: Option<Vec<Vec<u8>>> = Some(
			(0 .. t).into_iter().map(|i| {
				format!("{}", i).into_bytes().to_vec()
			}).collect::<Vec<Vec<u8>>>()
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

	detach {
		let caller: T::AccountId = whitelisted_caller();

		let mut immutable_data: [u8; 9] = [0; 9];
		hex::decode_to_slice("010000000b00803103", &mut immutable_data).unwrap();
		let immutable_data = immutable_data.to_vec();
		let proto_hash = blake2_256(immutable_data.as_slice());
		let references = vec![];

		Protos::<T>::upload(RawOrigin::Signed(caller.clone()).into(), references, Categories::Text(TextCategories::Plain), <Vec<Vec<u8>>>::new(), None, UsageLicense::Closed, immutable_data.clone())?;

		let public: [u8; 33] = [2, 44, 133, 69, 18, 57, 0, 152, 97, 145, 160, 85, 122, 14, 119, 232, 88, 169, 142, 77, 139, 133, 214, 67, 188, 128, 137, 28, 23, 247, 242, 193, 104];
		let target_account: Vec<u8> = [203, 109, 249, 222, 30, 252, 167, 163, 153, 138, 142, 173, 78, 2, 21, 157, 95, 169, 156, 62, 13, 79, 214, 67, 38, 103, 57, 11, 180, 114, 104, 84].to_vec();
		Detach::<T>::add_eth_auth(RawOrigin::Root.into(), sp_core::ecdsa::Public::from_raw(public))?;

		let pre_len: usize = <pallet_detach::DetachRequests<T>>::get().len();
	}: _(RawOrigin::Signed(caller), proto_hash, pallet_detach::SupportedChains::EthereumMainnet, target_account)
	verify {
		assert_eq!(<pallet_detach::DetachRequests<T>>::get().len(), pre_len + 1 as usize);
	}

	transfer_benchmark {
		let caller: T::AccountId = whitelisted_caller();
		let new_owner: T::AccountId = account("Sample", 100, SEED);

		let proto_data = b"Je suis Data".to_vec();
		Protos::<T>::upload(
			RawOrigin::Signed(caller.clone()).into(),
			Vec::<Hash256>::new(),
			Categories::Text(TextCategories::Plain),
			Vec::<Vec<u8>>::new(),
			None,
			UsageLicense::Closed,
			proto_data.clone()
		)?;
		let proto_hash = blake2_256(&proto_data);

	}: transfer(RawOrigin::Signed(caller), proto_hash, new_owner.clone())
	verify {
		assert_last_event::<T>(Event::<T>::Transferred { proto_hash: proto_hash, owner_id: new_owner }.into())
	}

	set_metadata_benchmark { // Benchmark setup phase
		let m in 1 .. 100; // `metadata_key` length
		let d in 1 .. MAX_DATA_LENGTH; // 1 byte to 1 Megabyte (I tried 1 byte to 1 Gigabyte, but I got the error: Thread 'main' panicked at 'Failed to allocate memory: "Requested allocation size is too large"', /Users/home/.cargo/git/checkouts/substrate-c784a31f8dac2358/401804d/primitives/io/src/lib.rs:1382)

		// `whitelisted_caller()`'s DB operations will not be counted when we run the extrinsic
		let caller: T::AccountId = whitelisted_caller();

		let proto_data = b"Je suis Data".to_vec();
		Protos::<T>::upload(
			RawOrigin::Signed(caller.clone()).into(),
			Vec::<Hash256>::new(),
			Categories::Text(TextCategories::Plain),
			Vec::<Vec<u8>>::new(),
			None,
			UsageLicense::Closed,
			proto_data.clone()
		)?;
		let proto_hash = blake2_256(&proto_data);

		let metadata_key: Vec<u8> = vec![7u8; m as usize];
		let data: Vec<u8> = vec![7u8; d as usize];

	}: set_metadata(RawOrigin::Signed(caller), proto_hash.clone(), metadata_key.clone(), data) // Execution phase
	verify { // Optional verification phase
		assert_last_event::<T>(Event::<T>::MetadataChanged { proto_hash: proto_hash, cid: metadata_key }.into())
	}

	impl_benchmark_test_suite!(Protos, crate::mock::new_test_ext(), crate::mock::Test);
}
