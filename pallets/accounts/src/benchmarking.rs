//! Benchmarking setup for pallet-accounts

use super::*;
#[allow(unused)]
use crate::Pallet as Accounts;
use frame_benchmarking::{account, benchmarks, vec, whitelisted_caller};
use frame_system::RawOrigin;
use frame_support::traits::Get;
use sp_runtime::SaturatedConversion;

use sp_core::{
	ecdsa,
	keccak_256,
	Pair,
	H160, // size of an Ethereum Account Address
	U256,
};

const SEED: u32 = 0;

fn get_ethereum_public_key(
	ecdsa_pair_struct: &ecdsa::Pair,
) -> H160 {

	let ecdsa_public_struct = ecdsa_pair_struct.public();

	let compressed_public_key = ecdsa_public_struct.0;

	let uncompressed_public_key =
		&libsecp256k1::PublicKey::parse_compressed(&compressed_public_key)
			.unwrap()
			.serialize();
	let uncompressed_public_key_without_prefix = &uncompressed_public_key[1..];
	let ethereum_account_id = &keccak_256(uncompressed_public_key_without_prefix)[12..];

	H160::from_slice(&ethereum_account_id)
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {

	add_key_benchmark {
		let public = sp_core::ed25519::Public::from_raw([7u8; 32]);

	}: add_key(RawOrigin::Root, public)
	verify {
		todo!()
	}

	del_key_benchmark {
		let public = sp_core::ed25519::Public::from_raw([7u8; 32]);

		Accounts::<T>::add_key(
			RawOrigin::Root.into(),
			public
		)?;

	}: del_key(RawOrigin::Root, public)
	verify {
		todo!()
	}

	link_benchmark {
		let caller: T::AccountId = whitelisted_caller();

		let ethereum_account_pair: ecdsa::Pair = sp_core::ecdsa::Pair::from_seed(&[7u8; 32]);
		let signature: ecdsa::Signature = ethereum_account_pair.sign_prehashed(
			&keccak_256(&[
				&b"EVM2Fragnova"[..],
				&T::EthChainId::get().to_be_bytes(),
				&caller.encode()
			].concat())
		);

	}: link(RawOrigin::Signed(caller), signature)
	verify {
		assert_last_event::<T>(
			Event::<T>::Linked {
				sender: caller,
				eth_key: get_ethereum_public_key(&ethereum_account_pair)
			}.into()
		)
	}

	unlink_benchmark {
		let caller: T::AccountId = whitelisted_caller();

		let ethereum_account_pair: ecdsa::Pair = sp_core::ecdsa::Pair::from_seed(&[7u8; 32]);
		Accounts::<T>::link(
			RawOrigin::Signed(caller.clone()).into(),
			ethereum_account_pair.sign_prehashed(
				&keccak_256(&[
					b"EVM2Fragnova".as_slice(),
					&T::EthChainId::get().to_be_bytes(),
					&caller.encode()
				].concat())
			)
		);

	}: unlink(RawOrigin::Signed(caller), get_ethereum_public_key(&ethereum_account_pair))
	verify {
		assert_last_event::<T>(
			Event::<T>::Unlinked {
				sender: caller,
				eth_key: get_ethereum_public_key(&ethereum_account_pair)
			}.into()
		)
	}

	internal_lock_update_benchmark {
		let caller: T::AccountId = whitelisted_caller();

		let ethereum_account_pair: ecdsa::Pair = sp_core::ecdsa::Pair::from_seed(&[7u8; 32]);

		let data = EthLockUpdate::<T::Public> {
			public: Into::<T::Public>::into(sp_core::ed25519::Public([69u8; 32])),
			amount: U256::from(7),
			locktime: U256::from(7),
			sender: get_ethereum_public_key(&ethereum_account_pair),
			signature: ethereum_account_pair.sign_prehashed(
				&keccak_256(
					&[
						b"\x19Ethereum Signed Message:\n32",
						&keccak_256(
							&[
								&b"FragLock"[..],
								&get_ethereum_public_key(&ethereum_account_pair).0[..],
								&T::EthChainId::get().to_be_bytes(),
								&Into::<[u8; 32]>::into(U256::from(7u32)), // same as `data.amount`
								&Into::<[u8; 32]>::into(U256::from(7u32)) // same as `data.locktime`
							].concat()
						)[..]
					].concat()
				)
			),
			lock: true, // yes, please lock it!
			block_number: 7,
		};
		let signature: T::Signature = Into::<T::Signature>::into(sp_core::ed25519::Signature([69u8; 64])); // this can be anything and it will still work
	}: internal_lock_update(RawOrigin::Signed(caller), data, signature)
	verify {
		assert_last_event::<T>(
			Event::<T>::Locked {
				eth_key: get_ethereum_public_key(&ethereum_account_pair),
				balance: data.amount.saturated_into(),
				locktime: data.locktime.saturated_into(),
			}.into()
		)
	}

	sponser_account_benchmark {
		let caller: T::AccountId = whitelisted_caller();
	}: sponser_account(RawOrigin::Signed(caller), external_id)
	verify {
		todo!()
	}

	add_sponsor_benchmark {
		let account: T::AccountId = account("Sample", 100, SEED);

	}: add_sponsor(RawOrigin::Root, account)
	verify {
		todo!()
	}

	remove_sponsor_benchmark {
		let account: T::AccountId = account("Sample", 100, SEED);

		Accounts::<T>::add_sponsor(
			RawOrigin::Root.into(),
			account
		)?;

	}: remove_sponsor(RawOrigin::Root, account)
	verify {
		todo!()
	}

	impl_benchmark_test_suite!(Fragments, crate::mock::new_test_ext(), crate::mock::Test);
}
