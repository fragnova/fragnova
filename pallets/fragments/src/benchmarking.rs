//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused)]
use crate::Pallet as Template;

use frame_benchmarking::{benchmarks, vec, whitelisted_caller};
use frame_system::RawOrigin;

benchmarks! {
	upload {
		let immutable_data = vec![1, 2, 3, 4];
		let mutable_data = vec![1, 2, 3, 4];
		let caller: T::AccountId = whitelisted_caller();
	}: _(RawOrigin::Signed(caller), immutable_data, mutable_data, None, None)
	verify {
		assert_eq!(UnverifiedFragments::<T>::get().len(), 1);
	}

	impl_benchmark_test_suite!(Template, crate::mock::new_test_ext(), crate::mock::Test);
}
