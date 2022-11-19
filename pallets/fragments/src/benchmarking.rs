//! Benchmarking setup for pallet-fragments

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{account, benchmarks, vec, whitelisted_caller};
use frame_system::RawOrigin;
use pallet_protos::UsageLicense;
use protos::{
	categories::{Categories, TextCategories},
	permissions::FragmentPerms,
};
use sp_core::crypto::UncheckedFrom;
use sp_io::hashing::blake2_128;

use crate::Pallet as Fragments;
use pallet_assets::Pallet as Assets;
use pallet_balances::Pallet as Balances;
use pallet_protos::Pallet as Protos;

const SEED: u32 = 0;

const MAX_DATA_LENGTH: u32 = 1_000_000; // 1 MegaByte
const MAX_METADATA_NAME_LENGTH: u32 = 100;
const MAX_QUANTITY_TO_MINT: u32 = 100;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}
fn assert_has_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_has_event(generic_event.into());
}

benchmarks! {
	where_clause { where
		T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>
	}

	create { // Benchmark setup phase
		let n in 1 .. MAX_METADATA_NAME_LENGTH; // `metadata.name` length

		// `whitelisted_caller()`'s DB operations will not be counted when we run the extrinsic
		let caller: T::AccountId = whitelisted_caller();

		Assets::<T>::force_create(
			RawOrigin::Root.into(),
			T::AssetId::default(),
			T::Lookup::unlookup(caller.clone()),
			true,
			7u128.saturated_into::<<T as pallet_assets::Config>::Balance>(),
			true
		)?;

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
			// By making currency Some, we enter an extra if-statement and also do an extra DB read operation
			currency: Some(T::AssetId::default()),
		};
		let permissions: FragmentPerms = FragmentPerms::EDIT | FragmentPerms::TRANSFER;
		let unique: Option<UniqueOptions> = Some(UniqueOptions { mutable: false});
		let max_supply: Option<InstanceUnit> = Some(7);

	}: create(RawOrigin::Signed(caller), proto_hash, metadata.clone(), permissions, unique, max_supply) // Execution phase
	verify { // Optional verification phase
		let definition_hash = blake2_128(
			&[&proto_hash[..], &metadata.name.encode(), &metadata.currency.encode()].concat(),
		);
		assert_last_event::<T>(Event::<T>::DefinitionCreated { definition_hash: definition_hash}.into())
	}

	publish { // Benchmark setup phase
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
			RawOrigin::Signed(caller.clone()).into(),
			proto_hash,
			metadata.clone(),
			FragmentPerms::EDIT | FragmentPerms::TRANSFER,
			None,
			// we make the Definition's `max_supply` Some,
			// because this causes `publish()` to check if `max_supply` is exceeded
			Some(7)
		)?;
		let definition_hash = blake2_128(
			&[&proto_hash[..], &metadata.name.encode(), &metadata.currency.encode()].concat(),
		);

		let price = 7u128;
		let quantity = Some(7); // making `quantity` Some causes an if condition to execute
		let expires: Option<T::BlockNumber> = Some(T::BlockNumber::from(7u32));
		let amount: Option<InstanceUnit> = Some(7);

	}: publish(RawOrigin::Signed(caller), definition_hash, price, quantity, expires, amount) // Execution phase
	verify { // Optional verification phase
		assert_last_event::<T>(Event::<T>::Publishing { definition_hash: definition_hash}.into())
	}

	unpublish { // Benchmark setup phase
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
			RawOrigin::Signed(caller.clone()).into(),
			proto_hash,
			metadata.clone(),
			FragmentPerms::EDIT | FragmentPerms::TRANSFER,
			None,
			None
		)?;
		let definition_hash = blake2_128(
			&[&proto_hash[..], &metadata.name.encode(), &metadata.currency.encode()].concat(),
		);
		Fragments::<T>::publish(
			RawOrigin::Signed(caller.clone()).into(),
			definition_hash,
			7, // price
			None,
			None,
			None
		)?;

	}: unpublish(RawOrigin::Signed(caller), definition_hash) // Execution phase
	verify { // Optional verification phase
		assert_last_event::<T>(Event::<T>::Unpublishing { definition_hash: definition_hash}.into())
	}

	mint_definition_that_has_non_unique_capability { // Benchmark setup phase
		let q in 1 .. MAX_QUANTITY_TO_MINT; // `FragmentBuyOptions::Quantity(quantity)`'s `quantity's` length
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
			RawOrigin::Signed(caller.clone()).into(),
			proto_hash,
			metadata.clone(),
			FragmentPerms::EDIT | FragmentPerms::TRANSFER,
			None, // non-unique
			// we make the Definition's `max_supply` Some,
			// because this causes `mint()` to check if `max_supply` is exceeded
			Some(q + 1).map(|ms| ms.into())
		)?;
		let definition_hash = blake2_128(
			&[&proto_hash[..], &metadata.name.encode(), &metadata.currency.encode()].concat(),
		);

		let options = FragmentBuyOptions::Quantity(q.into());
		let amount: Option<InstanceUnit> = Some(7);

	}: mint(RawOrigin::Signed(caller.clone()), definition_hash, options, amount) // Execution phase
	verify { // Optional verification phase
		for edition_id in 1..=q {
			assert_has_event::<T>(
				Event::<T>::InventoryAdded {
					account_id: caller.clone(),
					definition_hash: definition_hash,
					fragment_id: (edition_id.into(), 1)
				}.into()
			)
		}
	}

	mint_definition_that_has_unique_capability { // Benchmark setup phase
		let d in 1 .. MAX_DATA_LENGTH; // `FragmentBuyOptions::UniqueData(data)`'s `data's` length
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
			RawOrigin::Signed(caller.clone()).into(),
			proto_hash,
			metadata.clone(),
			FragmentPerms::EDIT | FragmentPerms::TRANSFER,
			Some(UniqueOptions {mutable: false}), // unique
			// we make the Definition's `max_supply` Some,
			// because this causes `mint()` to check if `max_supply` is exceeded
			Some(7)
		)?;
		let definition_hash = blake2_128(
			&[&proto_hash[..], &metadata.name.encode(), &metadata.currency.encode()].concat(),
		);

		let options = FragmentBuyOptions::UniqueData(vec![7u8; d as usize]);
		let amount: Option<InstanceUnit> = Some(7);

	}: mint(RawOrigin::Signed(caller.clone()), definition_hash, options, amount) // Execution phase
	verify { // Optional verification phase
		assert_last_event::<T>(
			Event::<T>::InventoryAdded {
				account_id: caller,
				definition_hash: definition_hash,
				fragment_id: (1, 1)
			}.into()
		)
	}

	buy_definition_that_has_non_unique_capability { // Benchmark setup phase
		let q in 1 .. MAX_QUANTITY_TO_MINT; // `FragmentBuyOptions::Quantity(quantity)`'s `quantity's` length
		let caller: T::AccountId = whitelisted_caller();
		let definition_owner: T::AccountId = account("Sample", 100, SEED);

		let proto_data = b"Je suis Data".to_vec();
		Protos::<T>::upload(
			RawOrigin::Signed(definition_owner.clone()).into(),
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
			RawOrigin::Signed(definition_owner.clone()).into(),
			proto_hash,
			metadata.clone(),
			FragmentPerms::EDIT | FragmentPerms::TRANSFER,
			None, // non-unique
			None
		)?;
		let definition_hash = blake2_128(
			&[&proto_hash[..], &metadata.name.encode(), &metadata.currency.encode()].concat(),
		);

		let price = 7u32;
		Fragments::<T>::publish(
			RawOrigin::Signed(definition_owner).into(),
			definition_hash,
			price as u128, // price
			// We make the `quantity` Some, since this will cause an extra DB write operation
			// when we call `buy()`.
			// The aforementioned DB write operation is the modification of the value `units_left` for the Publishing Struct in the `Publishing` StorageMap
			Some(q + 1).map(|ms| ms.into()),
			None,
			None
		)?;
		_ = <Balances::<T> as fungible::Mutate<T::AccountId>>::mint_into(
			&caller.clone(),
			<T as pallet_balances::Config>::Balance::from(price.saturating_mul(q))
			+ <Balances::<T> as fungible::Inspect<T::AccountId>>::minimum_balance(),
		);

		let options = FragmentBuyOptions::Quantity(q.into());

	}: buy(RawOrigin::Signed(caller.clone()), definition_hash, options) // Execution phase
	verify { // Optional verification phase
		for edition_id in 1..=q {
			assert_has_event::<T>(
				Event::<T>::InventoryAdded {
					account_id: caller.clone(),
					definition_hash: definition_hash,
					fragment_id: (edition_id.into(), 1)
				}.into()
			)
		}
	}

	buy_definition_that_has_unique_capability { // Benchmark setup phase
		let d in 1 .. MAX_DATA_LENGTH; // `FragmentBuyOptions::UniqueData(data)`'s `data's` length
		let caller: T::AccountId = whitelisted_caller();
		let definition_owner: T::AccountId = account("Sample", 100, SEED);

		let proto_data = b"Je suis Data".to_vec();
		Protos::<T>::upload(
			RawOrigin::Signed(definition_owner.clone()).into(),
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
			RawOrigin::Signed(definition_owner.clone()).into(),
			proto_hash,
			metadata.clone(),
			FragmentPerms::EDIT | FragmentPerms::TRANSFER,
			Some(UniqueOptions {mutable: false}), // unique
			None
		)?;
		let definition_hash = blake2_128(
			&[&proto_hash[..], &metadata.name.encode(), &metadata.currency.encode()].concat(),
		);

		let price = 7u32;
		Fragments::<T>::publish(
			RawOrigin::Signed(definition_owner).into(),
			definition_hash,
			price as u128, // price
			// We make the `quantity` Some, since this will cause an extra DB write operation
			// when we call `buy()`.
			// The aforementioned DB write operation is the modification of the value `units_left` for the Publishing Struct in the `Publishing` StorageMap
			Some(7),
			None,
			None
		)?;

		_ = <Balances::<T> as fungible::Mutate<T::AccountId>>::mint_into(
			&caller,
			<T as pallet_balances::Config>::Balance::from(price)
			+ <Balances::<T> as fungible::Inspect<T::AccountId>>::minimum_balance(),
		);

		let options = FragmentBuyOptions::UniqueData(vec![7u8; d as usize]);

	}: buy(RawOrigin::Signed(caller.clone()), definition_hash, options) // Execution phase
	verify { // Optional verification phase
		assert_has_event::<T>(
			Event::<T>::InventoryAdded {
				account_id: caller,
				definition_hash: definition_hash,
				fragment_id: (1, 1)
			}.into()
		)
	}


	benchmark_give_instance_that_does_not_have_copy_perms { // Benchmark setup phase
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
			RawOrigin::Signed(caller.clone()).into(),
			proto_hash,
			metadata.clone(),
			FragmentPerms::EDIT | FragmentPerms::TRANSFER, // no copy permission
			None,
			None
		)?;
		let definition_hash = blake2_128(
			&[&proto_hash[..], &metadata.name.encode(), &metadata.currency.encode()].concat(),
		);

		Fragments::<T>::mint(
			RawOrigin::Signed(caller.clone()).into(),
			definition_hash,
			FragmentBuyOptions::Quantity(1), // only mint 1 FI
			None
		)?;

		let edition = 1;
		let copy = 1;
		let to = T::Lookup::unlookup(account("Sample", 100, SEED));
		// by making `new_permissions` Some, it executes an extra if-statement block
		let new_permissions = Some(FragmentPerms::TRANSFER);
		// whether `expiration` is Some or None doesn't make a difference to the extrinsic time, if the instance doesn't have the copy permission
		let expiration = Some(T::BlockNumber::from(7u32));

	}: give(RawOrigin::Signed(caller.clone()), definition_hash, edition, copy, to.clone(), new_permissions, expiration) // Execution phase
	verify { // Optional verification phase
		assert_has_event::<T>(
			Event::<T>::InventoryRemoved {
				account_id: caller,
				definition_hash: definition_hash,
				fragment_id: (1, 1)
			}.into()
		);
		assert_has_event::<T>(
			Event::<T>::InventoryAdded {
				account_id: T::Lookup::lookup(to).unwrap(),
				definition_hash: definition_hash,
				fragment_id: (1, 1)
			}.into()
		)
	}


	benchmark_give_instance_that_has_copy_perms { // Benchmark setup phase
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
			RawOrigin::Signed(caller.clone()).into(),
			proto_hash,
			metadata.clone(),
			FragmentPerms::EDIT | FragmentPerms::TRANSFER | FragmentPerms::COPY, // has copy permission
			None,
			None
		)?;
		let definition_hash = blake2_128(
			&[&proto_hash[..], &metadata.name.encode(), &metadata.currency.encode()].concat(),
		);

		Fragments::<T>::mint(
			RawOrigin::Signed(caller.clone()).into(),
			definition_hash,
			FragmentBuyOptions::Quantity(1), // only mint 1 FI
			None
		)?;

		let edition = 1;
		let copy = 1;
		let to = T::Lookup::unlookup(account("Sample", 100, SEED));
		// by making `new_permissions` Some, it executes an extra if-statement block
		let new_permissions = Some(FragmentPerms::TRANSFER);
		// by making `expiration` Some, it causes an extra DB write operation (only if instance has copy perms).
		// This aforementioned DB write operation adds a key to the StorageMap `Expirations`
		let expiration = Some(T::BlockNumber::from(7u32));

	}: give(RawOrigin::Signed(caller.clone()), definition_hash, edition, copy, to.clone(), new_permissions, expiration) // Execution phase
	verify { // Optional verification phase
		assert_has_event::<T>(
			Event::<T>::InventoryAdded {
				account_id: T::Lookup::lookup(to).unwrap(),
				definition_hash: definition_hash,
				fragment_id: (1, 2)
			}.into()
		)
	}


	impl_benchmark_test_suite!(Fragments, crate::mock::new_test_ext(), crate::mock::Test);
}
