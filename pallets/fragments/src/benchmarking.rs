//! Benchmarking setup for pallet-fragments

use super::*;
#[allow(unused)]
use crate::Pallet as Fragments;
use frame_benchmarking::{benchmarks, vec, whitelisted_caller};
use frame_system::RawOrigin;
use pallet_protos::UsageLicense;
use protos::{
	categories::{Categories, TextCategories},
	permissions::FragmentPerms,
};
use sp_core::crypto::UncheckedFrom;
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
		T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>
	}

	create {
		let caller: T::AccountId = whitelisted_caller();
		let immutable_data = vec![0u8; 1 as usize];
		let proto_hash = blake2_256(immutable_data.as_slice());
		let references = vec![PROTO_HASH];
		pallet_protos::Pallet::<T>::upload(RawOrigin::Signed(caller.clone()).into(), references, Categories::Text(TextCategories::Plain), <Vec<Vec<u8>>>::new(), None, UsageLicense::Closed, immutable_data.clone())?;
		let fragment_data = FragmentMetadata {
			name: "name".as_bytes().to_vec(),
			currency: None,
		};

		let hash = blake2_128(
			&[&proto_hash[..], &fragment_data.name.encode(), &fragment_data.currency.encode()].concat(),
		);

	}: _(RawOrigin::Signed(caller.clone()), proto_hash, fragment_data, FragmentPerms::NONE, None, None)
	verify {
		assert_last_event::<T>(Event::<T>::DefinitionCreated { definition_hash: hash }.into())
	}


	create_benchmark { // Benchmark setup phase
		let n in 1 .. 100; // `metadata.name` length
		let c in 1 .. 1_000_000; // `metadata.currency`'s Asset ID number

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

		let metadata = FragmentMetadata {
			name: vec![7u8; n as usize],
			currency: c,
		};
		let permissions: FragmentPerms = FragmentPerms::EDIT | FragmentPerms::TRANSFER;
		let unique: Option<UniqueOptions> = Some(UniqueOptions { mutable: false});
		let max_supply: Option<Unit> = Some(7);

	}: create(RawOrigin::Signed(caller), proto_hash, metadata, permissions, unique, max_supply) // Execution phase
	verify { // Optional verification phase
		let definition_hash = blake2_128(
			&[&proto_hash[..], &metadata.name.encode(), &metadata.currency.encode()].concat(),
		);
		assert_last_event::<T>(Event::<T>::DefinitionCreated { definition_hash: definition_hash}.into())
	}

	publish_benchmark { // Benchmark setup phase
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

		let metadata = FragmentMetadata {
			name: b"Je suis un Nom".to_vec(),
			currency: None,
		};
		Fragments::<T>::create(
			RawOrigin::Signed(caller),
			proto_hash,
			metadata
			FragmentPerms::EDIT | FragmentPerms::TRANSFER,
			None,
			// we make the Definition's `max_supply` Some,
			// because this causes `publish()` to check if `max_supply` is exceeded
			Some(7)
		)
		let definition_hash = blake2_128(
			&[&proto_hash[..], &metadata.name.encode(), &metadata.currency.encode()].concat(),
		);

		let price = 7u8;
		let quantity = Some(7); // making `quantity` Some causes an if condition to execute
		let expires: Option<T::BlockNumber> = Some(7);
		let amount: Option<Unit> = Some(7);

	}: publish(RawOrigin::Signed(caller), definition_hash, price, quantity, expires, amount) // Execution phase
	verify { // Optional verification phase
		assert_last_event::<T>(Event::<T>::Publishing { definition_hash: definition_hash}.into())
	}

	unpublish_benchmark { // Benchmark setup phase
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

		let metadata = FragmentMetadata {
			name: b"Je suis un Nom".to_vec(),
			currency: None,
		};
		Fragments::<T>::create(
			RawOrigin::Signed(caller),
			proto_hash,
			metadata
			FragmentPerms::EDIT | FragmentPerms::TRANSFER,
			None,
			None
		);
		let definition_hash = blake2_128(
			&[&proto_hash[..], &metadata.name.encode(), &metadata.currency.encode()].concat(),
		);
		Fragments::<T>::publish(
			RawOrigin::Signed(caller),
			definition_hash,
			7,
			None,
			None,
			None
		);

	}: unpublish(RawOrigin::Signed(caller), definition_hash) // Execution phase
	verify { // Optional verification phase
		assert_last_event::<T>(Event::<T>::Unpublishing { definition_hash: definition_hash}.into())
	}

	mint_benchmark { // Benchmark setup phase
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

		let metadata = FragmentMetadata {
			name: b"Je suis un Nom".to_vec(),
			currency: None,
		};
		Fragments::<T>::create(
			RawOrigin::Signed(caller),
			proto_hash,
			metadata
			FragmentPerms::EDIT | FragmentPerms::TRANSFER,
			None,
			// we make the Definition's `max_supply` Some,
			// because this causes `publish()` to check if `max_supply` is exceeded
			Some(7)
		)
		let definition_hash = blake2_128(
			&[&proto_hash[..], &metadata.name.encode(), &metadata.currency.encode()].concat(),
		);

		// TODO - create variable "options"
		/// TODO - create variable "amount"


	}: mint(RawOrigin::Signed(caller), definition_hash, options, amount) // Execution phase
	verify { // Optional verification phase
		assert_last_event::<T>(Event::<T>::Publishing { definition_hash: definition_hash}.into())
	}



	impl_benchmark_test_suite!(Fragments, crate::mock::new_test_ext(), crate::mock::Test);
}
