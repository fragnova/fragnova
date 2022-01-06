//! Benchmarking setup for pallet-template

use super::*;
#[allow(unused)]
use crate::Pallet as Fragments;
use frame_benchmarking::{account, benchmarks, vec, whitelisted_caller};
use frame_system::RawOrigin;
use sp_io::hashing::blake2_256;
use frame_support::traits::Get;

const KEY_TYPE: KeyTypeId = KeyTypeId(*b"frag");

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {

	add_eth_auth {
		let validator: sp_core::ecdsa::Public = Default::default();
	}: _(RawOrigin::Root, validator.clone())
	verify {
		assert!(EthereumAuthorities::<T>::get().contains(&validator));
	}

	del_eth_auth {
		let validator: sp_core::ecdsa::Public = Default::default();
		Fragments::<T>::add_eth_auth(RawOrigin::Root.into(), validator.clone())?;
		assert!(EthereumAuthorities::<T>::get().contains(&validator));
	}: _(RawOrigin::Root, validator.clone())
	verify {
		assert!(!EthereumAuthorities::<T>::get().contains(&validator));
	}

	add_upload_auth {
		let validator: sp_core::ecdsa::Public = Default::default();
	}: _(RawOrigin::Root, validator.clone())
	verify {
		assert!(UploadAuthorities::<T>::get().contains(&validator));
	}

	del_upload_auth {
		let validator: sp_core::ecdsa::Public = Default::default();
		Fragments::<T>::add_eth_auth(RawOrigin::Root.into(), validator.clone())?;
		assert!(EthereumAuthorities::<T>::get().contains(&validator));
	}: _(RawOrigin::Root, validator.clone())
	verify {
		assert!(!UploadAuthorities::<T>::get().contains(&validator));
	}

	upload {
		let l in 1 .. T::MaxDataLength::get();
		let caller: T::AccountId = whitelisted_caller();
		let immutable_data = vec![0u8; l as usize];
		let fragment_hash = blake2_256(immutable_data.as_slice());
		let nonce: u64 = 0;
		let references = vec![IncludeInfo {
			fragment_hash,
			mutable_index: Some(Compact(1)),
			staked_amount: Compact(1),
		}];

		let key: sp_core::ecdsa::Public = Default::default();
		let linked_asset: Option<LinkedAsset> = None;
		let signature =
		Crypto::ecdsa_sign(KEY_TYPE, &key, &[&fragment_hash[..], &references.encode(), &linked_asset.encode(), &nonce.encode(),&1.encode()].concat());
		//let signature = Crypto::ecdsa_sign(KEY_TYPE, key, &msg[..]);

		let auth_data = AuthData{
			signature: signature.unwrap(),
			block: 1
		};
	}: _(RawOrigin::Signed(caller), references , linked_asset, None, auth_data, immutable_data)
	verify {
		assert_last_event::<T>(Event::<T>::Upload(fragment_hash).into())
	}

	// update {
	// 	let immutable_data = vec![1, 2, 3, 4];
	// 	let mutable_data = vec![1, 2, 3, 4];
	// 	let caller: T::AccountId = whitelisted_caller();
	// 	Fragments::<T>::upload(RawOrigin::Signed(caller.clone()).into(), immutable_data.clone(), mutable_data.clone(), None, None)?;
	// 	let fragment_hash = blake2_256(immutable_data.as_slice());
	// }: _(RawOrigin::Signed(caller), fragment_hash.clone(), Some(mutable_data), None)
	// verify {
	// 	assert_last_event::<T>(Event::<T>::Update(fragment_hash).into())
	// }

	// detach {
	// 	let immutable_data = vec![1, 2, 3, 4];
	// 	let mutable_data = vec![1, 2, 3, 4];
	// 	let caller: T::AccountId = whitelisted_caller();
	// 	Fragments::<T>::upload(RawOrigin::Signed(caller.clone()).into(), immutable_data.clone(), mutable_data.clone(), None, None)?;
	// 	let fragment_hash = blake2_256(immutable_data.as_slice());
	// 	let (pair, _) = sp_core::sr25519::Pair::generate();
	//
	// 	sp_io::crypto::ecdsa_generate(KEY_TYPE, None);
	// 	let keys = sp_io::crypto::ecdsa_public_keys(KEY_TYPE);
	//
	// 	<EthereumAuthorities<T>>::mutate(|authorities| {
	// 		authorities.insert(keys.get(0).unwrap().clone());
	// 	});
	//
	// }: _(RawOrigin::Signed(caller), fragment_hash.clone(), SupportedChains::EthereumMainnet, pair.to_raw_vec())
	// verify {
	// 	assert!(DetachedFragments::<T>::contains_key(fragment_hash));
	// }

	impl_benchmark_test_suite!(Fragments, crate::mock::new_test_ext(), crate::mock::Test);
}
