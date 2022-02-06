//! Benchmarking setup for pallet-fragments

use super::*;
#[allow(unused)]
use crate::Pallet as Protos;
use frame_benchmarking::{account, benchmarks, vec, whitelisted_caller};
use frame_system::RawOrigin;
use pallet_detach::Pallet as Detach;
use sp_io::hashing::blake2_256;

const PROTO_HASH: Hash256 = [
	30, 138, 136, 186, 232, 46, 112, 65, 122, 54, 110, 89, 123, 195, 7, 150, 12, 134, 10, 179, 245,
	51, 83, 227, 72, 251, 5, 148, 207, 251, 119, 59,
];

const PUBLIC: [u8; 33] = [
	3, 137, 65, 23, 149, 81, 74, 241, 98, 119, 101, 236, 239, 252, 189, 0, 39, 25, 240, 49, 96, 79,
	173, 215, 209, 136, 226, 220, 88, 91, 78, 26, 251,
];
const SEED: u32 = 0;

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
	add_upload_auth {
		let validator: sp_core::ecdsa::Public = sp_core::ecdsa::Public::from_raw(PUBLIC);
	}: _(RawOrigin::Root, validator.clone())
	verify {
		assert!(UploadAuthorities::<T>::get().contains(&validator));
	}

	del_upload_auth {
		let validator: sp_core::ecdsa::Public = sp_core::ecdsa::Public::from_raw(PUBLIC);
		Protos::<T>::add_upload_auth(RawOrigin::Root.into(), validator.clone())?;
		assert!(UploadAuthorities::<T>::get().contains(&validator));
	}: _(RawOrigin::Root, validator.clone())
	verify {
		assert!(!UploadAuthorities::<T>::get().contains(&validator));
	}

	upload {
		let caller: T::AccountId = whitelisted_caller();
		let mut immutable_data: [u8; 9] = [0; 9];
		hex::decode_to_slice("010000000b00803103", &mut immutable_data).unwrap();
		let immutable_data = immutable_data.to_vec();
		let proto_hash = blake2_256(immutable_data.as_slice());
		let references = vec![];
		let tags = vec![Tags::Code];

		let mut signature: [u8; 65] = [0; 65];
		hex::decode_to_slice("97a6c44d476f4a3944217d679642c60dac98dc3b2857d6e762e532361ea8185423fa376afc201a36834c57399050a391e3d9d2046790bdd0b49d4c2b307c1ee801", &mut signature).unwrap();
		let mut public: [u8; 33] = [0; 33];
		hex::decode_to_slice("020a1091341fe5664bfa1782d5e04779689068c916b04cb365ec3153755684d9a1", &mut public).unwrap();

		let auth_data = AuthData {
			signature: sp_core::ecdsa::Signature::from_raw(signature),
			block: 0
		};

		Protos::<T>::add_upload_auth(RawOrigin::Root.into(), sp_core::ecdsa::Public::from_raw(public))?;
	}: _(RawOrigin::Signed(caller), auth_data, references, tags, None, None, immutable_data)
	verify {
		assert_last_event::<T>(Event::<T>::Uploaded(proto_hash).into())
	}

	patch {
		let caller: T::AccountId = whitelisted_caller();

		let mut immutable_data: [u8; 9] = [0; 9];
		hex::decode_to_slice("010000000b00803103", &mut immutable_data).unwrap();
		let immutable_data = immutable_data.to_vec();
		let proto_hash = blake2_256(immutable_data.as_slice());
		let references = vec![];
		let tags = vec![Tags::Code];

		let mut signature: [u8; 65] = [0; 65];
		hex::decode_to_slice("97a6c44d476f4a3944217d679642c60dac98dc3b2857d6e762e532361ea8185423fa376afc201a36834c57399050a391e3d9d2046790bdd0b49d4c2b307c1ee801", &mut signature).unwrap();
		let mut public: [u8; 33] = [0; 33];
		hex::decode_to_slice("020a1091341fe5664bfa1782d5e04779689068c916b04cb365ec3153755684d9a1", &mut public).unwrap();

		let auth_data = AuthData {
			signature: sp_core::ecdsa::Signature::from_raw(signature),
			block: 0
		};

		Protos::<T>::add_upload_auth(RawOrigin::Root.into(), sp_core::ecdsa::Public::from_raw(public))?;
		Protos::<T>::upload(RawOrigin::Signed(caller.clone()).into(), auth_data, references, tags, None, None, immutable_data.clone())?;

		let public: [u8; 33] = [2, 10, 16, 145, 52, 31, 229, 102, 75, 250, 23, 130, 213, 224, 71, 121, 104, 144, 104, 201, 22, 176, 76, 179, 101, 236, 49, 83, 117, 86, 132, 217, 161];
		let signature: [u8; 65] = [163, 192, 46, 155, 7, 30, 102, 0, 188, 241, 106, 73, 76, 37, 190, 132, 170, 178, 123, 175, 0, 93, 24, 151, 12, 223, 89, 208, 107, 246, 11, 183, 45, 161, 145, 125, 117, 245, 79, 5, 186, 41, 12, 100, 251, 0, 231, 13, 133, 82, 207, 51, 40, 180, 83, 26, 98, 112, 102, 134, 72, 89, 111, 187, 1];

		let auth_data = AuthData{
			signature: sp_core::ecdsa::Signature::from_raw(signature),
			block: 1
		};

		Protos::<T>::add_upload_auth(RawOrigin::Root.into(), sp_core::ecdsa::Public::from_raw(public))?;
	}: _(RawOrigin::Signed(caller), auth_data, proto_hash , Some(Compact(123)), Some(immutable_data))
	verify {
		assert_last_event::<T>(Event::<T>::Patched(proto_hash).into())
	}

	detach {
		let caller: T::AccountId = whitelisted_caller();
		let immutable_data = "0x0155a0e40220".as_bytes().to_vec();
		let references = vec![PROTO_HASH];

		let public: [u8; 33] = [3, 137, 65, 23, 149, 81, 74, 241, 98, 119, 101, 236, 239, 252, 189, 0, 39, 25, 240, 49, 96, 79, 173, 215, 209, 136, 226, 220, 88, 91, 78, 26, 251];
		 let signature: [u8; 65] = [132, 58, 255, 255, 79, 60, 138, 156, 240, 245, 176, 220, 10, 248, 217, 156, 60, 180, 61, 210, 74, 231, 141, 45, 165, 102, 165, 251, 130, 229, 229, 144, 75, 26, 6, 42, 15, 121, 152, 88, 5, 120, 95, 20, 183, 221, 44, 110, 72, 245, 228, 200, 232, 253, 209, 69, 10, 197, 75, 108, 56, 196, 190, 182, 0];

		let auth_data = AuthData{
			signature: sp_core::ecdsa::Signature::from_raw(signature),
			block: 1
		};

		Protos::<T>::add_upload_auth(RawOrigin::Root.into(), sp_core::ecdsa::Public::from_raw(public))?;
		Protos::<T>::upload(RawOrigin::Signed(caller.clone()).into(),auth_data, references, Vec::new() , None, None, immutable_data.clone())?;

		let public: [u8; 33] = [2, 44, 133, 69, 18, 57, 0, 152, 97, 145, 160, 85, 122, 14, 119, 232, 88, 169, 142, 77, 139, 133, 214, 67, 188, 128, 137, 28, 23, 247, 242, 193, 104];

		let target_account: Vec<u8> = [203, 109, 249, 222, 30, 252, 167, 163, 153, 138, 142, 173, 78, 2, 21, 157, 95, 169, 156, 62, 13, 79, 214, 67, 38, 103, 57, 11, 180, 114, 104, 84].to_vec();
		Detach::<T>::add_eth_auth(RawOrigin::Root.into(), sp_core::ecdsa::Public::from_raw(public))?;

		let pre_len: usize = <pallet_detach::DetachRequests<T>>::get().len();
	}: _(RawOrigin::Signed(caller), PROTO_HASH, pallet_detach::SupportedChains::EthereumMainnet, target_account)
	verify {
		assert_eq!(<pallet_detach::DetachRequests<T>>::get().len(), pre_len + 1 as usize);
	}

	transfer {
		let caller: T::AccountId = whitelisted_caller();
		let new_owner: T::AccountId = account("Sample", 100, SEED);
		let immutable_data = "0x0155a0e40220".as_bytes().to_vec();
		let references = vec![PROTO_HASH];

		let public: [u8; 33] = [3, 137, 65, 23, 149, 81, 74, 241, 98, 119, 101, 236, 239, 252, 189, 0, 39, 25, 240, 49, 96, 79, 173, 215, 209, 136, 226, 220, 88, 91, 78, 26, 251];
		 let signature: [u8; 65] = [132, 58, 255, 255, 79, 60, 138, 156, 240, 245, 176, 220, 10, 248, 217, 156, 60, 180, 61, 210, 74, 231, 141, 45, 165, 102, 165, 251, 130, 229, 229, 144, 75, 26, 6, 42, 15, 121, 152, 88, 5, 120, 95, 20, 183, 221, 44, 110, 72, 245, 228, 200, 232, 253, 209, 69, 10, 197, 75, 108, 56, 196, 190, 182, 0];

		let auth_data = AuthData{
			signature: sp_core::ecdsa::Signature::from_raw(signature),
			block: 1
		};

		Protos::<T>::add_upload_auth(RawOrigin::Root.into(), sp_core::ecdsa::Public::from_raw(public))?;
		Protos::<T>::upload(RawOrigin::Signed(caller.clone()).into(), auth_data, references, Vec::new(), None, None, immutable_data.clone())?;


	}: _(RawOrigin::Signed(caller), PROTO_HASH, new_owner.clone())
	verify {
		assert_last_event::<T>(Event::<T>::Transferred(PROTO_HASH, new_owner).into())
	}

	impl_benchmark_test_suite!(Protos, crate::mock::new_test_ext(), crate::mock::Test);
}
