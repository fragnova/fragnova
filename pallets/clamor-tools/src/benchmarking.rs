//! Benchmarking setup for pallet-entities

use super::*;
#[allow(unused)]
use crate::Pallet as ClamorTools;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
use sp_chainblocks::FragmentOwner;

const FRAGMENT_HASH: Hash256 = [
	30, 138, 136, 186, 232, 46, 112, 65, 122, 54, 110, 89, 123, 195, 7, 150, 12, 134, 10, 179, 245,
	51, 83, 227, 72, 251, 5, 148, 207, 251, 119, 59,
];

const PUBLIC: [u8; 33] = [
	3, 137, 65, 23, 149, 81, 74, 241, 98, 119, 101, 236, 239, 252, 189, 0, 39, 25, 240, 49, 96, 79,
	173, 215, 209, 136, 226, 220, 88, 91, 78, 26, 251,
];

const PUBLIC1: [u8; 32] = [
	137, 65, 23, 149, 81, 74, 241, 98, 119, 101, 236, 239, 252, 189, 0, 39, 25, 240, 49, 96, 79,
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
		ClamorTools::<T>::add_eth_auth(RawOrigin::Root.into(), validator.clone())?;
		assert!(EthereumAuthorities::<T>::get().contains(&validator));
	}: _(RawOrigin::Root, validator.clone())
	verify {
		assert!(!EthereumAuthorities::<T>::get().contains(&validator));
	}

	add_key {
		let validator: sp_core::ed25519::Public = sp_core::ed25519::Public::from_raw(PUBLIC1);
	}: _(RawOrigin::Root, validator.clone())
	verify {
		assert!(FragKeys::<T>::get().contains(&validator));
	}

	del_key {
		let validator: sp_core::ed25519::Public = sp_core::ed25519::Public::from_raw(PUBLIC1);
		ClamorTools::<T>::add_key(RawOrigin::Root.into(), validator.clone())?;
		assert!(FragKeys::<T>::get().contains(&validator));
	}: _(RawOrigin::Root, validator.clone())
	verify {
		assert!(!FragKeys::<T>::get().contains(&validator));
	}

	detach {
		let caller: T::AccountId = whitelisted_caller();

		let public: [u8; 33] = [2, 44, 133, 69, 18, 57, 0, 152, 97, 145, 160, 85, 122, 14, 119, 232, 88, 169, 142, 77, 139, 133, 214, 67, 188, 128, 137, 28, 23, 247, 242, 193, 104];

		let target_account: Vec<u8> = [203, 109, 249, 222, 30, 252, 167, 163, 153, 138, 142, 173, 78, 2, 21, 157, 95, 169, 156, 62, 13, 79, 214, 67, 38, 103, 57, 11, 180, 114, 104, 84].to_vec();
		ClamorTools::<T>::add_eth_auth(RawOrigin::Root.into(), sp_core::ecdsa::Public::from_raw(public))?;

		let owner = FragmentOwner::User(caller.clone());
		let pre_len: usize = <DetachRequests<T>>::get().len();
	}: _(RawOrigin::Signed(caller), FRAGMENT_HASH, SupportedChains::EthereumMainnet, target_account, owner)
	verify {
		assert_eq!(<DetachRequests<T>>::get().len(), pre_len + 1 as usize);
	}

	// internal_finalize_detach {
	// 	let caller: T::AccountId = whitelisted_caller();
	//
	// 	let public: [u8; 32] = [137, 65, 23, 149, 81, 74, 241, 98, 119, 101, 236, 239, 252, 189, 0, 39, 25, 240, 49, 96, 79, 173, 215, 209, 136, 226, 220, 88, 91, 78, 26, 251];
	// 	let signature: [u8; 64] = [ 58, 255, 255, 79, 60, 138, 156, 240, 245, 176, 220, 10, 248, 217, 156, 60, 180, 61, 210, 74, 231, 141, 45, 165, 102, 165, 251, 130, 229, 229, 144, 75, 26, 6, 42, 15, 121, 152, 88, 5, 120, 95, 20, 183, 221, 44, 110, 72, 245, 228, 200, 232, 253, 209, 69, 10, 197, 75, 108, 56, 196, 190, 182, 0];
	//
	// 	let p = sp_core::sr25519::Public::from_raw(public);
	// 	//let public_key: T::Public = <T as SigningTypes>::Public::from(sp_core::ecdsa::Public::from_raw(public));
	// 	let detach_data = DetachInternalData {
	// 		public: <T as SigningTypes>::Public::from(p),
	// 		fragment_hash: FRAGMENT_HASH,
	// 		remote_signature: vec![],
	// 		target_account: vec![],
	// 		target_chain: SupportedChains::EthereumGoerli,
	// 		nonce: 1
	// 	};
	//
	//
	// }: _(RawOrigin::None, detach_data, sp_core::ed25519::Signature::from_raw(signature))
	// verify {
	// 	assert!(<DetachedFragments<T>>::contains_key(FRAGMENT_HASH));
	// }

	impl_benchmark_test_suite!(ClamorTools, crate::mock::new_test_ext(), crate::mock::Test);
}
