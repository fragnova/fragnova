//! Benchmarking setup for pallet-template

use super::*;
#[allow(unused)]
use crate::Pallet as Fragments;
use frame_benchmarking::{account, benchmarks, vec, whitelisted_caller};
use frame_system::RawOrigin;

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {

	add_validator {
		let validator: T::AccountId = account("fragments", 0, 0);
	}: _(RawOrigin::Root, validator.clone())
	verify {
		assert!(FragmentValidators::<T>::get().contains(&validator));
	}

	remove_validator {
		let validator: T::AccountId = account("fragments", 0, 0);
		Fragments::<T>::add_validator(RawOrigin::Root.into(), validator.clone())?;
		assert!(FragmentValidators::<T>::get().contains(&validator));
	}: _(RawOrigin::Root, validator.clone())
	verify {
		assert!(!FragmentValidators::<T>::get().contains(&validator));
	}

	// internal_confirm_upload {
	// 	let caller: T::AccountId = whitelisted_caller();
	// 	let keystore = KeyStore::new();
	// 	const PHRASE: &str =
	// 	"news slush supreme milk chapter athlete soap sausage put clutch what kitten";
	//
	// 	SyncCryptoStore::sr25519_generate_new(
	// 		&keystore,
	// 		crate::crypto::Public::ID,
	// 		Some(&format!("{}/hunter1", PHRASE)),
	// 	)
	// 	.unwrap();
	//
	// 	let public_key = SyncCryptoStore::sr25519_public_keys(&keystore, crate::crypto::Public::ID)
	// 		.get(0)
	// 		.unwrap()
	// 		.clone();
	// 	let hash: FragmentHash = [
	// 		30, 138, 136, 186, 232, 46, 112, 65, 122, 54, 110, 89, 123, 195, 7, 150, 12, 134, 10,
	// 		179, 245, 51, 83, 227, 72, 251, 5, 148, 207, 251, 119, 59,
	// 	];
	//
	// 	let fragment_data:FragmentValidation<T::Public, T::BlockNumber> = FragmentValidation {
	// 		block_number: 101,
	// 		fragment_hash: hash,
	// 		public: public_key,
	// 		result: true,
	// 	};
	//
	// }: _(RawOrigin::Root, fragment_data, signature)
	// // verify {
	// // 	assert_last_event::<T>(Event::<T>::Verified{hash,true}.into())
	// // }

	upload {
		let immutable_data = vec![1, 2, 3, 4];
		let mutable_data = vec![1, 2, 3, 4];
		let caller: T::AccountId = whitelisted_caller();
	}: _(RawOrigin::Signed(caller), immutable_data, mutable_data, None, None)
	verify {
		assert_eq!(UnverifiedFragments::<T>::get().len(), 1);
	}

	impl_benchmark_test_suite!(Fragments, crate::mock::new_test_ext(), crate::mock::Test);
}
