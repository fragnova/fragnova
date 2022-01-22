//! Benchmarking setup for pallet-entities

use super::*;
#[allow(unused)]
use crate::Pallet as Entities;
use fragments_pallet::{AuthData, IncludeInfo};
use frame_benchmarking::{benchmarks, vec, whitelisted_caller};
use frame_system::RawOrigin;
use sp_io::hashing::blake2_256;

const FRAGMENT_HASH: Hash256 = [
	30, 138, 136, 186, 232, 46, 112, 65, 122, 54, 110, 89, 123, 195, 7, 150, 12, 134, 10, 179, 245,
	51, 83, 227, 72, 251, 5, 148, 207, 251, 119, 59,
];

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {

	create {
		let caller: T::AccountId = whitelisted_caller();
		let immutable_data = vec![0u8; 1 as usize];
		let fragment_hash = blake2_256(immutable_data.as_slice());
		let references = vec![IncludeInfo {
			fragment_hash: FRAGMENT_HASH,
			mutable_index: Some(Compact(1)),
			staked_amount: Compact(1),
		}];

		let public: [u8; 33] = [3, 137, 65, 23, 149, 81, 74, 241, 98, 119, 101, 236, 239, 252, 189, 0, 39, 25, 240, 49, 96, 79, 173, 215, 209, 136, 226, 220, 88, 91, 78, 26, 251];

		 let signature: [u8; 65] = [29, 83, 249, 100, 228, 85, 48, 71, 56, 134, 254, 85, 188, 199, 241, 160, 149, 99, 4, 236, 47, 249, 66, 140, 5, 123, 161, 152, 76, 152, 92, 89, 32, 85, 113, 187, 51, 6, 13, 223, 25, 225, 100, 38, 60, 46, 94, 71, 221, 149, 171, 23, 85, 228, 34, 227, 244, 85, 56, 171, 103, 111, 119, 61, 1];

		let auth_data = AuthData{
			signature: sp_core::ecdsa::Signature::from_raw(signature),
			block: 1
		};

		fragments_pallet::Pallet::<T>::add_upload_auth(RawOrigin::Root.into(), sp_core::ecdsa::Public::from_raw(public))?;
		fragments_pallet::Pallet::<T>::upload(RawOrigin::Signed(caller.clone()).into(), references , None, None, auth_data, immutable_data.clone())?;
		let entity_data = EntityMetadata {
			name: "name".as_bytes().to_vec(),
			external_url: "external_url".as_bytes().to_vec(),
		};

	}: _(RawOrigin::Signed(caller.clone()), fragment_hash , entity_data, true, true, None)
	verify {
		assert_last_event::<T>(Event::<T>::EntityAdded(caller).into())
	}

	impl_benchmark_test_suite!(Entities, crate::mock::new_test_ext(), crate::mock::Test);
}
