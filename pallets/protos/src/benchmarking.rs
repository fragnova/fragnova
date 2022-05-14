//! Benchmarking setup for pallet-fragments

use super::*;
#[allow(unused)]
use crate::Pallet as Protos;
use categories::{Categories, ChainCategories};
use frame_benchmarking::{account, benchmarks, vec, whitelisted_caller};
use frame_system::RawOrigin;
use pallet_detach::Pallet as Detach;
use sp_io::hashing::blake2_256;

const PUBLIC: [u8; 33] = [
	3, 137, 65, 23, 149, 81, 74, 241, 98, 119, 101, 236, 239, 252, 189, 0, 39, 25, 240, 49, 96, 79,
	173, 215, 209, 136, 226, 220, 88, 91, 78, 26, 251,
];

const SEED: u32 = 0;

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {

	where_clause { where
		T::AccountId: AsRef<[u8]>
	}

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
		let categories = (Categories::Chain(ChainCategories::Generic), <Vec<Vec<u8>>>::new());

		let mut signature: [u8; 65] = [0; 65];
		hex::decode_to_slice("97a6c44d476f4a3944217d679642c60dac98dc3b2857d6e762e532361ea8185423fa376afc201a36834c57399050a391e3d9d2046790bdd0b49d4c2b307c1ee801", &mut signature).unwrap();
		let mut public: [u8; 33] = [0; 33];
		hex::decode_to_slice("020a1091341fe5664bfa1782d5e04779689068c916b04cb365ec3153755684d9a1", &mut public).unwrap();

		let auth_data = AuthData {
			signature: sp_core::ecdsa::Signature::from_raw(signature),
			block: 0
		};

		Protos::<T>::add_upload_auth(RawOrigin::Root.into(), sp_core::ecdsa::Public::from_raw(public))?;
	}: _(RawOrigin::Signed(caller), auth_data, references, categories, None, None, immutable_data)
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
		let categories = (Categories::Chain(ChainCategories::Generic), <Vec<Vec<u8>>>::new());

		let mut signature: [u8; 65] = [0; 65];
		hex::decode_to_slice("97a6c44d476f4a3944217d679642c60dac98dc3b2857d6e762e532361ea8185423fa376afc201a36834c57399050a391e3d9d2046790bdd0b49d4c2b307c1ee801", &mut signature).unwrap();
		let mut public: [u8; 33] = [0; 33];
		hex::decode_to_slice("020a1091341fe5664bfa1782d5e04779689068c916b04cb365ec3153755684d9a1", &mut public).unwrap();

		let auth_data = AuthData {
			signature: sp_core::ecdsa::Signature::from_raw(signature),
			block: 0
		};

		Protos::<T>::add_upload_auth(RawOrigin::Root.into(), sp_core::ecdsa::Public::from_raw(public))?;
		Protos::<T>::upload(RawOrigin::Signed(caller.clone()).into(), auth_data, references, categories, None, None, immutable_data.clone())?;

		let mut signature: [u8; 65] = [0; 65];
		hex::decode_to_slice("9b68ce0f7274e50a75196f819f72e41ebdce74f5f0632b6d97f85b09831debc31fc9b82a47fde245349c971ddd148991c1b3f3813ea7d7f64972e5698f029eb401", &mut signature).unwrap();

		let auth_data = AuthData {
			signature: sp_core::ecdsa::Signature::from_raw(signature),
			block: 0
		};

		let mut data: [u8; 3] = [0; 3];
		hex::decode_to_slice("C0FFEE", &mut data).unwrap();

		Protos::<T>::add_upload_auth(RawOrigin::Root.into(), sp_core::ecdsa::Public::from_raw(public))?;
	}: _(RawOrigin::Signed(caller), auth_data, proto_hash , Some(Compact(123)), data.to_vec())
	verify {
		assert_last_event::<T>(Event::<T>::Patched(proto_hash).into())
	}

	detach {
		let caller: T::AccountId = whitelisted_caller();

		let mut immutable_data: [u8; 9] = [0; 9];
		hex::decode_to_slice("010000000b00803103", &mut immutable_data).unwrap();
		let immutable_data = immutable_data.to_vec();
		let proto_hash = blake2_256(immutable_data.as_slice());
		let references = vec![];
		let categories = (Categories::Chain(ChainCategories::Generic), <Vec<Vec<u8>>>::new());

		let mut signature: [u8; 65] = [0; 65];
		hex::decode_to_slice("97a6c44d476f4a3944217d679642c60dac98dc3b2857d6e762e532361ea8185423fa376afc201a36834c57399050a391e3d9d2046790bdd0b49d4c2b307c1ee801", &mut signature).unwrap();
		let mut public: [u8; 33] = [0; 33];
		hex::decode_to_slice("020a1091341fe5664bfa1782d5e04779689068c916b04cb365ec3153755684d9a1", &mut public).unwrap();

		let auth_data = AuthData {
			signature: sp_core::ecdsa::Signature::from_raw(signature),
			block: 0
		};

		Protos::<T>::add_upload_auth(RawOrigin::Root.into(), sp_core::ecdsa::Public::from_raw(public))?;
		Protos::<T>::upload(RawOrigin::Signed(caller.clone()).into(), auth_data, references, categories, None, None, immutable_data.clone())?;

		let public: [u8; 33] = [2, 44, 133, 69, 18, 57, 0, 152, 97, 145, 160, 85, 122, 14, 119, 232, 88, 169, 142, 77, 139, 133, 214, 67, 188, 128, 137, 28, 23, 247, 242, 193, 104];
		let target_account: Vec<u8> = [203, 109, 249, 222, 30, 252, 167, 163, 153, 138, 142, 173, 78, 2, 21, 157, 95, 169, 156, 62, 13, 79, 214, 67, 38, 103, 57, 11, 180, 114, 104, 84].to_vec();
		Detach::<T>::add_eth_auth(RawOrigin::Root.into(), sp_core::ecdsa::Public::from_raw(public))?;

		let pre_len: usize = <pallet_detach::DetachRequests<T>>::get().len();
	}: _(RawOrigin::Signed(caller), proto_hash, pallet_detach::SupportedChains::EthereumMainnet, target_account)
	verify {
		assert_eq!(<pallet_detach::DetachRequests<T>>::get().len(), pre_len + 1 as usize);
	}

	transfer {
		let caller: T::AccountId = whitelisted_caller();
		let new_owner: T::AccountId = account("Sample", 100, SEED);

		let mut immutable_data: [u8; 9] = [0; 9];
		hex::decode_to_slice("010000000b00803103", &mut immutable_data).unwrap();
		let immutable_data = immutable_data.to_vec();
		let proto_hash = blake2_256(immutable_data.as_slice());
		let references = vec![];
		let categories = (Categories::Chain(ChainCategories::Generic), <Vec<Vec<u8>>>::new());

		let mut signature: [u8; 65] = [0; 65];
		hex::decode_to_slice("97a6c44d476f4a3944217d679642c60dac98dc3b2857d6e762e532361ea8185423fa376afc201a36834c57399050a391e3d9d2046790bdd0b49d4c2b307c1ee801", &mut signature).unwrap();
		let mut public: [u8; 33] = [0; 33];
		hex::decode_to_slice("020a1091341fe5664bfa1782d5e04779689068c916b04cb365ec3153755684d9a1", &mut public).unwrap();

		let auth_data = AuthData {
			signature: sp_core::ecdsa::Signature::from_raw(signature),
			block: 0
		};

		Protos::<T>::add_upload_auth(RawOrigin::Root.into(), sp_core::ecdsa::Public::from_raw(public))?;
		Protos::<T>::upload(RawOrigin::Signed(caller.clone()).into(), auth_data, references, categories, None, None, immutable_data.clone())?;
	}: _(RawOrigin::Signed(caller), proto_hash, new_owner.clone())
	verify {
		assert_last_event::<T>(Event::<T>::Transferred(proto_hash, new_owner).into())
	}

	impl_benchmark_test_suite!(Protos, crate::mock::new_test_ext(), crate::mock::Test);
}
