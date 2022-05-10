//! Benchmarking setup for pallet-fragments

use super::*;
#[allow(unused)]
use crate::Pallet as Fragments;
use frame_benchmarking::{benchmarks, vec, whitelisted_caller};
use frame_system::RawOrigin;
use pallet_protos::categories::{Categories, ChainCategories};
use pallet_protos::AuthData;
use sp_io::hashing::blake2_256;
use sp_std::collections::btree_set::BTreeSet;

const PROTO_HASH: Hash256 = [
	30, 138, 136, 186, 232, 46, 112, 65, 122, 54, 110, 89, 123, 195, 7, 150, 12, 134, 10, 179, 245,
	51, 83, 227, 72, 251, 5, 148, 207, 251, 119, 59,
];

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
	where_clause { where
		T::AccountId: AsRef<[u8]>
	}

	create {
		let caller: T::AccountId = whitelisted_caller();
		let immutable_data = vec![0u8; 1 as usize];
		let proto_hash = blake2_256(immutable_data.as_slice());
		let references = vec![PROTO_HASH];

		let public: [u8; 33] = [3, 137, 65, 23, 149, 81, 74, 241, 98, 119, 101, 236, 239, 252, 189, 0, 39, 25, 240, 49, 96, 79, 173, 215, 209, 136, 226, 220, 88, 91, 78, 26, 251];
		let signature: [u8; 65] = [29, 83, 249, 100, 228, 85, 48, 71, 56, 134, 254, 85, 188, 199, 241, 160, 149, 99, 4, 236, 47, 249, 66, 140, 5, 123, 161, 152, 76, 152, 92, 89, 32, 85, 113, 187, 51, 6, 13, 223, 25, 225, 100, 38, 60, 46, 94, 71, 221, 149, 171, 23, 85, 228, 34, 227, 244, 85, 56, 171, 103, 111, 119, 61, 1];

		let auth_data = AuthData{
			signature: sp_core::ecdsa::Signature::from_raw(signature),
			block: 1
		};

		pallet_protos::Pallet::<T>::add_upload_auth(RawOrigin::Root.into(), sp_core::ecdsa::Public::from_raw(public))?;
		pallet_protos::Pallet::<T>::upload(RawOrigin::Signed(caller.clone()).into(), auth_data, references, (Categories::Chain(ChainCategories::Generic), <BTreeSet<Vec<u8>>>::new()), None, None, immutable_data.clone())?;
		let fragment_data = FragmentMetadata {
			name: "name".as_bytes().to_vec(),
			external_url: "external_url".as_bytes().to_vec(),
		};

	}: _(RawOrigin::Signed(caller.clone()), proto_hash , fragment_data, true, true, None)
	verify {
		assert_last_event::<T>(Event::<T>::FragmentAdded(caller).into())
	}

	impl_benchmark_test_suite!(Fragments, crate::mock::new_test_ext(), crate::mock::Test);
}
