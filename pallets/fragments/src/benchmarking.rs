//! Benchmarking setup for pallet-template

use super::*;
#[allow(unused)]
use crate::Pallet as Fragments;
use frame_benchmarking::{account, benchmarks, vec, whitelisted_caller};
use frame_system::RawOrigin;
use sp_io::hashing::blake2_256;

const FRAGMENT_HASH: Hash256 = [
	30, 138, 136, 186, 232, 46, 112, 65, 122, 54, 110, 89, 123, 195, 7, 150, 12, 134, 10, 179, 245,
	51, 83, 227, 72, 251, 5, 148, 207, 251, 119, 59,
];

const PUBLIC: [u8; 33] = [3, 137, 65, 23, 149, 81, 74, 241, 98, 119, 101, 236, 239, 252, 189, 0, 39, 25, 240, 49, 96, 79, 173, 215, 209, 136, 226, 220, 88, 91, 78, 26, 251];
const SEED: u32 = 0;

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {

	add_eth_auth {
		let validator: sp_core::ecdsa::Public = sp_core::ecdsa::Public::from_raw(PUBLIC);
	}: _(RawOrigin::Root, validator.clone())
	verify {
		assert!(EthereumAuthorities::<T>::get().contains(&validator));
	}

	del_eth_auth {
		let validator: sp_core::ecdsa::Public = sp_core::ecdsa::Public::from_raw(PUBLIC);
		Fragments::<T>::add_eth_auth(RawOrigin::Root.into(), validator.clone())?;
		assert!(EthereumAuthorities::<T>::get().contains(&validator));
	}: _(RawOrigin::Root, validator.clone())
	verify {
		assert!(!EthereumAuthorities::<T>::get().contains(&validator));
	}

	add_upload_auth {
		let validator: sp_core::ecdsa::Public = sp_core::ecdsa::Public::from_raw(PUBLIC);
	}: _(RawOrigin::Root, validator.clone())
	verify {
		assert!(UploadAuthorities::<T>::get().contains(&validator));
	}

	del_upload_auth {
		let validator: sp_core::ecdsa::Public = sp_core::ecdsa::Public::from_raw(PUBLIC);
		Fragments::<T>::add_upload_auth(RawOrigin::Root.into(), validator.clone())?;
		assert!(UploadAuthorities::<T>::get().contains(&validator));
	}: _(RawOrigin::Root, validator.clone())
	verify {
		assert!(!UploadAuthorities::<T>::get().contains(&validator));
	}

	upload {
		let caller: T::AccountId = whitelisted_caller();
		let l in 1 .. 2;
		let immutable_data = vec![0u8; l as usize];
		let fragment_hash = blake2_256(immutable_data.as_slice());
		let references = vec![IncludeInfo {
			fragment_hash: FRAGMENT_HASH,
			mutable_index: Some(Compact(1)),
			staked_amount: Compact(1),
		}];

		let public: [u8; 33] = if l == 1 {
			[3, 137, 65, 23, 149, 81, 74, 241, 98, 119, 101, 236, 239, 252, 189, 0, 39, 25, 240, 49, 96, 79, 173, 215, 209, 136, 226, 220, 88, 91, 78, 26, 251]
		} else {
			[3, 137, 65, 23, 149, 81, 74, 241, 98, 119, 101, 236, 239, 252, 189, 0, 39, 25, 240, 49, 96, 79, 173, 215, 209, 136, 226, 220, 88, 91, 78, 26, 251]
		};

 		let signature: [u8; 65] = if l == 1 {
			[29, 83, 249, 100, 228, 85, 48, 71, 56, 134, 254, 85, 188, 199, 241, 160, 149, 99, 4, 236, 47, 249, 66, 140, 5, 123, 161, 152, 76, 152, 92, 89, 32, 85, 113, 187, 51, 6, 13, 223, 25, 225, 100, 38, 60, 46, 94, 71, 221, 149, 171, 23, 85, 228, 34, 227, 244, 85, 56, 171, 103, 111, 119, 61, 1]
		} else {
			[28, 64, 136, 241, 147, 34, 162, 228, 181, 29, 74, 73, 148, 220, 75, 47, 247, 106, 95, 247, 174, 31, 157, 252, 111, 175, 221, 239, 55, 245, 128, 34, 81, 214, 149, 247, 115, 217, 252, 152, 19, 234, 232, 144, 174, 233, 39, 34, 96, 37, 112, 142, 18, 187, 129, 230, 1, 138, 233, 154, 40, 123, 7, 51, 1]
		};

		let auth_data = AuthData{
			signature: sp_core::ecdsa::Signature::from_raw(signature),
			block: 1
		};

		Fragments::<T>::add_upload_auth(RawOrigin::Root.into(), sp_core::ecdsa::Public::from_raw(public))?;
	}: _(RawOrigin::Signed(caller), references , None, None, auth_data, immutable_data)
	verify {
		assert_last_event::<T>(Event::<T>::Upload(fragment_hash).into())
	}

	update {
		let caller: T::AccountId = whitelisted_caller();
		let l in 1 .. 2;
		let immutable_data = vec![0u8; l as usize];
		let fragment_hash = blake2_256(immutable_data.as_slice());
		let references = vec![IncludeInfo {
			fragment_hash: FRAGMENT_HASH,
			mutable_index: Some(Compact(1)),
			staked_amount: Compact(1),
		}];

		let public: [u8; 33] = if l == 1 {
			[3, 137, 65, 23, 149, 81, 74, 241, 98, 119, 101, 236, 239, 252, 189, 0, 39, 25, 240, 49, 96, 79, 173, 215, 209, 136, 226, 220, 88, 91, 78, 26, 251]
		} else {
			[3, 137, 65, 23, 149, 81, 74, 241, 98, 119, 101, 236, 239, 252, 189, 0, 39, 25, 240, 49, 96, 79, 173, 215, 209, 136, 226, 220, 88, 91, 78, 26, 251]
		};

 		let signature: [u8; 65] = if l == 1 {
			[29, 83, 249, 100, 228, 85, 48, 71, 56, 134, 254, 85, 188, 199, 241, 160, 149, 99, 4, 236, 47, 249, 66, 140, 5, 123, 161, 152, 76, 152, 92, 89, 32, 85, 113, 187, 51, 6, 13, 223, 25, 225, 100, 38, 60, 46, 94, 71, 221, 149, 171, 23, 85, 228, 34, 227, 244, 85, 56, 171, 103, 111, 119, 61, 1]
		} else {
			[28, 64, 136, 241, 147, 34, 162, 228, 181, 29, 74, 73, 148, 220, 75, 47, 247, 106, 95, 247, 174, 31, 157, 252, 111, 175, 221, 239, 55, 245, 128, 34, 81, 214, 149, 247, 115, 217, 252, 152, 19, 234, 232, 144, 174, 233, 39, 34, 96, 37, 112, 142, 18, 187, 129, 230, 1, 138, 233, 154, 40, 123, 7, 51, 1]
		};

		let auth_data = AuthData{
			signature: sp_core::ecdsa::Signature::from_raw(signature),
			block: 1
		};

		Fragments::<T>::add_upload_auth(RawOrigin::Root.into(), sp_core::ecdsa::Public::from_raw(public))?;
		Fragments::<T>::upload(RawOrigin::Signed(caller.clone()).into(), references , None, None, auth_data, immutable_data.clone())?;

		let public: [u8; 33] = if l == 1 {
			[2, 10, 16, 145, 52, 31, 229, 102, 75, 250, 23, 130, 213, 224, 71, 121, 104, 144, 104, 201, 22, 176, 76, 179, 101, 236, 49, 83, 117, 86, 132, 217, 161]
		} else {
			[2, 10, 16, 145, 52, 31, 229, 102, 75, 250, 23, 130, 213, 224, 71, 121, 104, 144, 104, 201, 22, 176, 76, 179, 101, 236, 49, 83, 117, 86, 132, 217, 161]
		};

 		let signature: [u8; 65] = if l == 1 {
			[182, 126, 145, 239, 103, 243, 151, 246, 231, 244, 195, 48, 67, 37, 83, 79, 66, 229, 255, 139, 106, 33, 113, 206, 135, 22, 221, 130, 162, 206, 43, 85, 92, 8, 197, 104, 195, 51, 254, 242, 253, 21, 57, 21, 44, 79, 34, 100, 55, 47, 149, 74, 201, 50, 235, 157, 79, 201, 166, 241, 156, 191, 106, 224, 0]
		} else {
			[163, 192, 46, 155, 7, 30, 102, 0, 188, 241, 106, 73, 76, 37, 190, 132, 170, 178, 123, 175, 0, 93, 24, 151, 12, 223, 89, 208, 107, 246, 11, 183, 45, 161, 145, 125, 117, 245, 79, 5, 186, 41, 12, 100, 251, 0, 231, 13, 133, 82, 207, 51, 40, 180, 83, 26, 98, 112, 102, 134, 72, 89, 111, 187, 1]
		};


		let auth_data = AuthData{
			signature: sp_core::ecdsa::Signature::from_raw(signature),
			block: 1
		};

		Fragments::<T>::add_upload_auth(RawOrigin::Root.into(), sp_core::ecdsa::Public::from_raw(public))?;


	}: _(RawOrigin::Signed(caller), fragment_hash , Some(Compact(123)), auth_data, Some(immutable_data))
	verify {
		assert_last_event::<T>(Event::<T>::Update(fragment_hash).into())
	}

	// detach {
	// 	let caller: T::AccountId = whitelisted_caller();
	// 	let immutable_data = "0x0155a0e40220".as_bytes().to_vec();
	// 	let references = vec![IncludeInfo {
	// 		fragment_hash: FRAGMENT_HASH,
	// 		mutable_index: Some(Compact(1)),
	// 		staked_amount: Compact(1),
	// 	}];
	//
	// 	let public: [u8; 33] = [3, 137, 65, 23, 149, 81, 74, 241, 98, 119, 101, 236, 239, 252, 189, 0, 39, 25, 240, 49, 96, 79, 173, 215, 209, 136, 226, 220, 88, 91, 78, 26, 251];
 	// 	let signature: [u8; 65] = [132, 58, 255, 255, 79, 60, 138, 156, 240, 245, 176, 220, 10, 248, 217, 156, 60, 180, 61, 210, 74, 231, 141, 45, 165, 102, 165, 251, 130, 229, 229, 144, 75, 26, 6, 42, 15, 121, 152, 88, 5, 120, 95, 20, 183, 221, 44, 110, 72, 245, 228, 200, 232, 253, 209, 69, 10, 197, 75, 108, 56, 196, 190, 182, 0];
	//
	// 	let auth_data = AuthData{
	// 		signature: sp_core::ecdsa::Signature::from_raw(signature),
	// 		block: 1
	// 	};
	//
	// 	Fragments::<T>::add_upload_auth(RawOrigin::Root.into(), sp_core::ecdsa::Public::from_raw(public))?;
	// 	Fragments::<T>::upload(RawOrigin::Signed(caller.clone()).into(), references , None, None, auth_data, immutable_data.clone())?;
	//
	// 	let public: [u8; 33] = [2, 44, 133, 69, 18, 57, 0, 152, 97, 145, 160, 85, 122, 14, 119, 232, 88, 169, 142, 77, 139, 133, 214, 67, 188, 128, 137, 28, 23, 247, 242, 193, 104];
	//
	// 	let target_account: Vec<u8> = [203, 109, 249, 222, 30, 252, 167, 163, 153, 138, 142, 173, 78, 2, 21, 157, 95, 169, 156, 62, 13, 79, 214, 67, 38, 103, 57, 11, 180, 114, 104, 84].to_vec();
	// 	Fragments::<T>::add_eth_auth(RawOrigin::Root.into(), sp_core::ecdsa::Public::from_raw(public))?;
	//
	// }: _(RawOrigin::Signed(caller), FRAGMENT_HASH, SupportedChains::EthereumMainnet, target_account)
	// verify {
	// 	assert!(DetachedFragments::<T>::contains_key(&FRAGMENT_HASH));
	// }

	transfer {
		let caller: T::AccountId = whitelisted_caller();
		let new_owner: T::AccountId = account("Sample", 100, SEED);
		let immutable_data = "0x0155a0e40220".as_bytes().to_vec();
		let references = vec![IncludeInfo {
			fragment_hash: FRAGMENT_HASH,
			mutable_index: Some(Compact(1)),
			staked_amount: Compact(1),
		}];

		let public: [u8; 33] = [3, 137, 65, 23, 149, 81, 74, 241, 98, 119, 101, 236, 239, 252, 189, 0, 39, 25, 240, 49, 96, 79, 173, 215, 209, 136, 226, 220, 88, 91, 78, 26, 251];
 		let signature: [u8; 65] = [132, 58, 255, 255, 79, 60, 138, 156, 240, 245, 176, 220, 10, 248, 217, 156, 60, 180, 61, 210, 74, 231, 141, 45, 165, 102, 165, 251, 130, 229, 229, 144, 75, 26, 6, 42, 15, 121, 152, 88, 5, 120, 95, 20, 183, 221, 44, 110, 72, 245, 228, 200, 232, 253, 209, 69, 10, 197, 75, 108, 56, 196, 190, 182, 0];

		let auth_data = AuthData{
			signature: sp_core::ecdsa::Signature::from_raw(signature),
			block: 1
		};

		Fragments::<T>::add_upload_auth(RawOrigin::Root.into(), sp_core::ecdsa::Public::from_raw(public))?;
		Fragments::<T>::upload(RawOrigin::Signed(caller.clone()).into(), references , None, None, auth_data, immutable_data.clone())?;


	}: _(RawOrigin::Signed(caller), FRAGMENT_HASH, new_owner.clone())
	verify {
		assert_last_event::<T>(Event::<T>::Transfer(FRAGMENT_HASH, new_owner).into())
	}

	impl_benchmark_test_suite!(Fragments, crate::mock::new_test_ext(), crate::mock::Test);
}
