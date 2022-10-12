//! Benchmarking setup for pallet-detach

#![cfg(feature = "runtime-benchmarks")]

use super::*;
#[allow(unused)]
use crate::Pallet as Detach;
use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;

const PUBLIC: [u8; 33] = [
	3, 137, 65, 23, 149, 81, 74, 241, 98, 119, 101, 236, 239, 252, 189, 0, 39, 25, 240, 49, 96, 79,
	173, 215, 209, 136, 226, 220, 88, 91, 78, 26, 251,
];

benchmarks! {

	add_eth_auth {
		let validator: sp_core::ecdsa::Public = sp_core::ecdsa::Public::from_raw(PUBLIC);
	}: _(RawOrigin::Root, validator.clone())
	verify {
		assert!(EthereumAuthorities::<T>::get().contains(&validator));
	}

	del_eth_auth {
		let validator: sp_core::ecdsa::Public = sp_core::ecdsa::Public::from_raw(PUBLIC);
		Detach::<T>::add_eth_auth(RawOrigin::Root.into(), validator.clone())?;
		assert!(EthereumAuthorities::<T>::get().contains(&validator));
	}: _(RawOrigin::Root, validator.clone())
	verify {
		assert!(!EthereumAuthorities::<T>::get().contains(&validator));
	}

	impl_benchmark_test_suite!(Detach, crate::mock::new_test_ext(), crate::mock::Test);
}
