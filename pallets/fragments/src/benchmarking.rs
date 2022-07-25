//! Benchmarking setup for pallet-fragments

use super::*;
#[allow(unused)]
use crate::Pallet as Fragments;
use frame_benchmarking::{benchmarks, vec, whitelisted_caller};
use frame_system::RawOrigin;
use protos::{
	categories::{Categories, TextCategories},
	permissions::FragmentPerms,
};
use sp_io::hashing::blake2_128;

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
		pallet_protos::Pallet::<T>::upload(RawOrigin::Signed(caller.clone()).into(), references, Categories::Text(TextCategories::Plain), <Vec<Vec<u8>>>::new(), None, None, immutable_data.clone())?;
		let fragment_data = FragmentMetadata {
			name: "name".as_bytes().to_vec(),
			currency: None,
		};

		let hash = blake2_128(
			&[&proto_hash[..], &fragment_data.name.encode(), &fragment_data.currency.encode()].concat(),
		);

	}: _(RawOrigin::Signed(caller.clone()), proto_hash, fragment_data, FragmentPerms::NONE, None, None)
	verify {
		assert_last_event::<T>(Event::<T>::DefinitionCreated { fragment_hash: hash }.into())
	}

	impl_benchmark_test_suite!(Fragments, crate::mock::new_test_ext(), crate::mock::Test);
}
