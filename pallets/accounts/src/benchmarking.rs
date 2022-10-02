//! Benchmarking setup for pallet-accounts

use super::*;
#[allow(unused)]
use crate::Pallet as Accounts;
use frame_benchmarking::{account, benchmarks, vec, whitelisted_caller};
use frame_system::RawOrigin;
use pallet_protos::UsageLicense;
use protos::{
	categories::{Categories, TextCategories},
	permissions::FragmentPerms,
};
use sp_core::crypto::UncheckedFrom;
use sp_io::hashing::blake2_128;

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
	where_clause { where
		T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>
	}

	add_key_benchmark {
		let caller: T::AccountId = whitelisted_caller();
		Keys::<T>::put(caller.clone());

		let public: T::AccountId = account("Sample", 100, SEED);

	}: add_key(RawOrigin::Signed(caller), public)
	verify {
		assert_last_event::<T>(Event::<T>::DefinitionCreated { definition_hash: hash }.into())
	}

	del_key_benchmark {
		let caller: T::AccountId = whitelisted_caller();
		Keys::<T>::put(caller.clone());

		let public: T::AccountId = account("Sample", 100, SEED);

		Accounts::<T>::add_key(
			RawOrigin::Signed(caller.clone()).into(),
			public
		)?;

	}: del_key(RawOrigin::Signed(caller), public)
	verify {
		assert_last_event::<T>(Event::<T>::DefinitionCreated { definition_hash: hash }.into())
	}

	link_benchmark {
		let caller: T::AccountId = whitelisted_caller();

		let mut message = b"EVM2Fragnova".to_vec();
		message.extend_from_slice(&get_ethereum_chain_id().to_be_bytes());
		message.extend_from_slice(&clamor_account_id.encode());
		let hashed_message = keccak_256(&message);
		ethereum_account_pair.sign_prehashed(&hashed_message)

	}: link(RawOrigin::Signed(caller), signature)
	verify {
		assert_last_event::<T>(Event::<T>::DefinitionCreated { definition_hash: hash }.into())
	}

	unlink_benchmark {
		let caller: T::AccountId = whitelisted_caller();
	}: unlink(RawOrigin::Signed(caller), public)
	verify {
		assert_last_event::<T>(Event::<T>::DefinitionCreated { definition_hash: hash }.into())
	}

	internal_lock_update_benchmark {
		let caller: T::AccountId = whitelisted_caller();
	}: internal_lock_update(origin, public)
	verify {
		assert_last_event::<T>(Event::<T>::DefinitionCreated { definition_hash: hash }.into())
	}

	sponser_account_benchmark {
		let caller: T::AccountId = whitelisted_caller();
	}: sponser_account(origin, external_id)
	verify {
		assert_last_event::<T>(Event::<T>::DefinitionCreated { definition_hash: hash }.into())
	}

	add_sponsor_benchmark {
		let caller: T::AccountId = whitelisted_caller();
	}: add_sponsor(origin, account)
	verify {
		assert_last_event::<T>(Event::<T>::DefinitionCreated { definition_hash: hash }.into())
	}

	remove_sponsor_benchmark {
		let caller: T::AccountId = whitelisted_caller();
	}: remove_sponsor(origin, account)

	verify {
		assert_last_event::<T>(Event::<T>::DefinitionCreated { definition_hash: hash }.into())
	}

	impl_benchmark_test_suite!(Fragments, crate::mock::new_test_ext(), crate::mock::Test);
}
