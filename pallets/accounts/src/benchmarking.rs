//! Benchmarking setup for pallet-accounts

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::traits::{Currency, Get};
use frame_system::RawOrigin;
use sp_io::hashing::keccak_256;
use sp_runtime::traits::Zero;
use sp_std::collections::btree_set::BTreeSet;

use crate::Pallet as Accounts;

const SEED: u32 = 0;

fn sign(
	message_struct: &libsecp256k1::Message,
	ethereum_secret_key_struct: &libsecp256k1::SecretKey,
) -> ecdsa::Signature {
	let (signature_struct, recovery_id_struct) =
		libsecp256k1::sign(message_struct, &ethereum_secret_key_struct);
	let signature = ecdsa::Signature({
		let mut sixty_five_bit_signature: [u8; 65] = [0; 65];
		let (one, two) = sixty_five_bit_signature.split_at_mut(signature_struct.serialize().len());
		one.copy_from_slice(&signature_struct.serialize());
		two.copy_from_slice(&[recovery_id_struct.serialize()]);
		sixty_five_bit_signature
	});

	return signature
}

fn get_ethereum_public_key(secret_key_struct: &libsecp256k1::SecretKey) -> H160 {
	let public_key_struct = libsecp256k1::PublicKey::from_secret_key(secret_key_struct);

	let uncompressed_public_key = public_key_struct.serialize();

	let uncompressed_public_key_without_prefix = &uncompressed_public_key[1..];
	let ethereum_account_id = &keccak_256(uncompressed_public_key_without_prefix)[12..];

	H160::from_slice(&ethereum_account_id)
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {

	// I need these trait bounds so that I can create a dummy `T::Public` value and a dummy `T::Signature` value in some of the benchmark tests
	where_clause {
		where
			T::Public: From<sp_core::ed25519::Public>,
			T::Signature: From<sp_core::ed25519::Signature>
	}

	add_key {
		let mut frag_keys = FragKeys::<T>::get();

		let public = sp_core::ed25519::Public::from_raw([7u8; 32]);
	}: add_key(RawOrigin::Root, public.clone())
	verify {
		frag_keys.insert(public); // log::info!("FragKeys' length are: {}", FragKeys::<T>::get().len());
		assert_eq!(FragKeys::<T>::get(), frag_keys);
	}

	del_key {
		let frag_keys = FragKeys::<T>::get();

		let public = sp_core::ed25519::Public::from_raw([7u8; 32]);
		Accounts::<T>::add_key(
			RawOrigin::Root.into(),
			public
		)?;

	}: del_key(RawOrigin::Root, public)
	verify {
		assert_eq!(FragKeys::<T>::get(), frag_keys);
	}

	link {
		let caller: T::AccountId = whitelisted_caller();

		// let ethereum_account_pair: ecdsa::Pair = sp_core::ecdsa::Pair::from_seed(&[7u8; 32]);
		let ethereum_secret_key_struct: libsecp256k1::SecretKey = libsecp256k1::SecretKey::parse(&[7u8; 32]).unwrap();
		let genesis_hash_string = format!("0x{}", hex::encode(<frame_system::Pallet<T>>::block_hash(T::BlockNumber::zero())));
		let sender_string = format!("0x{}", hex::encode(caller.encode()));
		let signature: ecdsa::Signature = sign(
			&libsecp256k1::Message::parse(
				&keccak_256(&[
					&[0x19, 0x01],
					&keccak_256(
						&ethabi::encode(
							&vec![
								Token::Uint(
									U256::from(keccak_256(b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"))
								),
								Token::Uint(U256::from(keccak_256(b"Fragnova Network"))),
								Token::Uint(U256::from(keccak_256(b"1"))),
								Token::Uint(U256::from(T::EthChainId::get())),
								Token::Address(H160::from(TryInto::<[u8; 20]>::try_into(hex::decode(LINK_VERIFYING_CONTRACT).unwrap()).unwrap())),
							]
						)
					)[..],
					&keccak_256(
						&ethabi::encode(
							&vec![
								Token::Uint(
									U256::from(keccak_256(b"Msg(string fragnovaGenesis,string op,string sender)"))
								),
								Token::Uint(U256::from(keccak_256(&genesis_hash_string.into_bytes()))),
								Token::Uint(U256::from(keccak_256(b"link"))),
								Token::Uint(U256::from(keccak_256(&sender_string.into_bytes()))),
							]
						)
					)[..]
				].concat())
			),
			&ethereum_secret_key_struct
		);

	}: link(RawOrigin::Signed(caller.clone()), signature)
	verify {
		assert_last_event::<T>(
			Event::<T>::Linked {
				sender: caller,
				eth_key: get_ethereum_public_key(&ethereum_secret_key_struct)
			}.into()
		)
	}

	unlink {
		let caller: T::AccountId = whitelisted_caller();

		// let ethereum_account_pair: ecdsa::Pair = sp_core::ecdsa::Pair::from_seed(&[7u8; 32]);
		let ethereum_secret_key_struct: libsecp256k1::SecretKey = libsecp256k1::SecretKey::parse(&[7u8; 32]).unwrap();
		let genesis_hash_string = format!("0x{}", hex::encode(<frame_system::Pallet<T>>::block_hash(T::BlockNumber::zero())));
		let sender_string = format!("0x{}", hex::encode(caller.encode()));
		Accounts::<T>::link(
			RawOrigin::Signed(caller.clone()).into(),
			sign(
				&libsecp256k1::Message::parse(
					&keccak_256(&[
						&[0x19, 0x01],
						&keccak_256(
							&ethabi::encode(
								&vec![
									Token::Uint(
										U256::from(keccak_256(b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"))
									),
									Token::Uint(U256::from(keccak_256(b"Fragnova Network"))),
									Token::Uint(U256::from(keccak_256(b"1"))),
									Token::Uint(U256::from(T::EthChainId::get())),
									Token::Address(H160::from(TryInto::<[u8; 20]>::try_into(hex::decode(LINK_VERIFYING_CONTRACT).unwrap()).unwrap())),
								]
							)
						)[..],
						&keccak_256(
							&ethabi::encode(
								&vec![
									Token::Uint(
										U256::from(keccak_256(b"Msg(string fragnovaGenesis,string op,string sender)"))
									),
									Token::Uint(U256::from(keccak_256(&genesis_hash_string.into_bytes()))),
									Token::Uint(U256::from(keccak_256(b"link"))),
									Token::Uint(U256::from(keccak_256(&sender_string.into_bytes()))),
								]
							)
						)[..]
					].concat())
				),
				&ethereum_secret_key_struct
			)
		)?;

	}: unlink(RawOrigin::Signed(caller.clone()), get_ethereum_public_key(&ethereum_secret_key_struct))
	verify {
		assert_last_event::<T>(
			Event::<T>::Unlinked {
				sender: caller,
				eth_key: get_ethereum_public_key(&ethereum_secret_key_struct)
			}.into()
		)
	}

/*	internal_lock_update {
		// let ethereum_account_pair: ecdsa::Pair = sp_core::ecdsa::Pair::from_seed(&[7u8; 32]);
		let ethereum_secret_key_struct: libsecp256k1::SecretKey = libsecp256k1::SecretKey::parse(&[7u8; 32]).unwrap();
		let data = EthLockUpdate::<T::Public> {
			public: sp_core::ed25519::Public([7u8; 32]).into(),
			amount: U256::from(100),
			lock_period: 1,
			sender: get_ethereum_public_key(&ethereum_secret_key_struct),
			signature: sign(
				&libsecp256k1::Message::parse(
					&keccak_256(
						&[
							b"\x19Ethereum Signed Message:\n32",
							&keccak_256(
								&[
									&b"FragLock"[..],
									&get_ethereum_public_key(&ethereum_secret_key_struct).0[..],
									&T::EthChainId::get().to_be_bytes(),
									&Into::<[u8; 32]>::into(U256::from(100u32)), // same as `data.amount`
									&Into::<[u8; 32]>::into(U256::from(1u32)) // same as `data.lock_period`
								].concat()
							)[..]
						].concat()
					),
				),
				&ethereum_secret_key_struct
			),
			lock: true, // yes, please lock it!
			block_number: 7,
		};
		let signature: T::Signature = sp_core::ed25519::Signature([69u8; 64]).into(); // this can be anything and it will still work
	}: internal_lock_update(RawOrigin::None, data.clone(), signature)
	verify {
		let events = <frame_system::Pallet<T>>::events();
		assert_eq!(events.len(), 3);
	}*/

	sponsor_account {
		let caller: T::AccountId = whitelisted_caller();

		Accounts::<T>::add_sponsor(
			RawOrigin::Root.into(),
			caller.clone()
		)?;

		_ = <T as pallet_proxy::Config>::Currency::deposit_creating(
			&caller.clone(),
			<T as pallet_proxy::Config>::ProxyDepositBase::get() + <T as pallet_proxy::Config>::ProxyDepositFactor::get() + <T as pallet_proxy::Config>::Currency::minimum_balance(),
		);

		let external_id = ExternalID::Discord(7u64);

	}: sponsor_account(RawOrigin::Signed(caller.clone()), external_id.clone())
	verify {
		assert_last_event::<T>(
			Event::<T>::SponsoredAccount {
				sponsor: caller.clone(),
				sponsored: T::AccountId::decode(
					&mut &blake2_256(
						&[&b"fragnova-sponsor-account"[..], &caller.encode(), &external_id.encode()]
						.concat(),
					)[..]
				).unwrap(),
				external_id: external_id,
			}.into()
		)
	}

	add_sponsor {
		let account: T::AccountId = account("Sample", 100, SEED);

	}: add_sponsor(RawOrigin::Root, account.clone())
	verify {
		assert_eq!(ExternalAuthorities::<T>::get(), BTreeSet::from([account]));
	}

	remove_sponsor {
		let account: T::AccountId = account("Sample", 100, SEED);

		Accounts::<T>::add_sponsor(
			RawOrigin::Root.into(),
			account.clone()
		)?;

	}: remove_sponsor(RawOrigin::Root, account.clone())
	verify {
		assert_eq!(ExternalAuthorities::<T>::get(), BTreeSet::new());
	}

	impl_benchmark_test_suite!(Accounts, crate::mock::new_test_ext(), crate::mock::Test);
}
