#![cfg(test)]

use crate as pallet_accounts;
use crate::{dummy_data::*, mock, mock::*, *};
use codec::Encode;
use ethabi::Token;
use frame_support::{assert_noop, assert_ok, dispatch::DispatchResult};
use frame_system::offchain::{SignedPayload, SigningTypes};
use sp_core::H256;
use sp_runtime::SaturatedConversion;

pub use internal_lock_update_tests::lock_;
pub use link_tests::link_;
use pallet_oracle::OraclePrice;

fn apply_percent(amount: u128, percent: u8) -> u128 {
	if amount == 0 {
		return 0;
	}
	sp_runtime::Percent::from_percent(percent).mul_ceil(amount) as u128
}

fn get_oracle_price() -> u128 {
	1 // Assume the current price of 1 FRAG = 1 USD
}

pub fn store_price_() -> DispatchResult {
	let oracle_price = OraclePrice {
		price: U256::from(1000000),
		block_number: System::block_number(),
		public: sp_core::ed25519::Public([69u8; 32]),
	};
	Oracle::store_price(
		Origin::none(),
		oracle_price,
		sp_core::ed25519::Signature([69u8; 64]), // this can be anything
	)
}

mod link_tests {
	use super::*;

	pub fn link_(link: &Link) -> DispatchResult {
		Accounts::link(Origin::signed(link.clamor_account_id), link.link_signature.clone())
	}

	#[test]
	fn link_should_work() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let link = dd.link;

			assert_ok!(link_(&link));

			assert_eq!(<EVMLinks<Test>>::get(&link.clamor_account_id).unwrap(), link.get_ethereum_public_address_of_signer());
			assert_eq!(<EVMLinksReverse<Test>>::get(&link.get_ethereum_public_address_of_signer()).unwrap(), link.clamor_account_id);

			let event = <frame_system::Pallet<Test>>::events()
				.pop()
				.expect("Expected at least one EventRecord to be found")
				.event;
			assert_eq!(
				event,
				mock::Event::from(pallet_accounts::Event::Linked {
					sender: link.clamor_account_id,
					eth_key: link.get_ethereum_public_address_of_signer()
				})
			);
		});
	}

	#[test]
	fn link_should_not_work_if_signature_parameter_is_invalid() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let link = Link { link_signature: dd.link_signature, ..dd.link };

			assert_noop!(link_(&link), Error::<Test>::VerificationFailed,);
		});
	}

	#[test]
	fn link_should_not_work_if_clamor_account_is_already_linked() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let link = dd.link;

			assert_ok!(link_(&link));

			let link_diff_ethereum_account_id = Link {
				clamor_account_id: link.clamor_account_id,
				link_signature: create_link_signature(
					link.clamor_account_id,
					dd.ethereum_account_pair.clone(),
				),
				_ethereum_account_pair: dd.ethereum_account_pair,
			};

			assert_noop!(
				link_(&link_diff_ethereum_account_id),
				Error::<Test>::AccountAlreadyLinked
			);
		});
	}

	#[test]
	fn link_should_not_work_if_ethereum_account_is_already_linked() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let link = Link {
				clamor_account_id: dd.account_id,
				link_signature: create_link_signature(
					dd.account_id,
					dd.ethereum_account_pair.clone(),
				),
				_ethereum_account_pair: dd.ethereum_account_pair.clone(),
			};

			assert_ok!(link_(&link));

			let link_diff_clamor_account_id = Link {
				clamor_account_id: dd.account_id_second,
				link_signature: create_link_signature(
					dd.account_id_second,
					dd.ethereum_account_pair.clone(),
				),
				_ethereum_account_pair: dd.ethereum_account_pair,
			};

			assert_noop!(link_(&link_diff_clamor_account_id), Error::<Test>::AccountAlreadyLinked);
		});
	}
}

mod unlink_tests {
	use super::*;

	#[test]
	fn unlink_should_work() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();
			let link = dd.link;

			assert_ok!(link_(&link));

			assert_ok!(Accounts::unlink(
				Origin::signed(link.clamor_account_id),
				link.get_ethereum_public_address_of_signer()
			));

			assert_eq!(<EVMLinks<Test>>::contains_key(&link.clamor_account_id), false);
			assert_eq!(<EVMLinksReverse<Test>>::contains_key(&link.get_ethereum_public_address_of_signer()), false);

			assert!(<PendingUnlinks<Test>>::get().contains(&link.clamor_account_id));

			let event = <frame_system::Pallet<Test>>::events()
				.pop()
				.expect("Expected at least one EventRecord to be found")
				.event;
			assert_eq!(
				event,
				mock::Event::from(pallet_accounts::Event::Unlinked {
					sender: link.clamor_account_id,
					eth_key: link.get_ethereum_public_address_of_signer(),
				})
			);
		});
	}

	#[test]
	fn unlink_should_not_work_if_link_does_not_exist() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();
			let link = dd.link;

			assert_noop!(
				Accounts::unlink(
					Origin::signed(link.clamor_account_id),
					link.get_ethereum_public_address_of_signer()
				),
				Error::<Test>::AccountNotLinked
			);
		});
	}

	#[test]
	fn unlink_should_not_work_if_origin_parameter_and_account_paramter_are_linked_but_not_with_each_other(
	) {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let link = dd.link;

			let link_second = dd.link_second;

			assert_ok!(link_(&link));
			assert_ok!(link_(&link_second));

			assert_noop!(
				Accounts::unlink(
					Origin::signed(link.clamor_account_id),
					link_second.get_ethereum_public_address_of_signer()
				),
				Error::<Test>::DifferentAccountLinked
			);
		});
	}
}

mod sync_partner_contracts_tests {
	use super::*;

	#[test] #[ignore]
	fn sync_frag_locks_should_work() {
		let (mut t, pool_state, _offchain_state, ed25519_public_key) = new_test_ext_with_ocw();

		let dd = DummyData::new();
		let lock = dd.lock;

		let expected_data = EthLockUpdate {
			public: <Test as SigningTypes>::Public::from(ed25519_public_key),
			..lock.data
		};

		t.execute_with(|| {
			Accounts::sync_partner_contracts(1);

			let tx = pool_state.write().transactions.pop().unwrap();
			let tx = <Extrinsic as codec::Decode>::decode(&mut &*tx).unwrap();
			assert_eq!(tx.signature, None); // Because it's an **unsigned transaction** with a signed payload

			if let Call::Accounts(crate::Call::internal_lock_update { data, signature }) =
				tx.call
			{
				assert_eq!(data, expected_data);

				let signature_valid =
					<EthLockUpdate<<Test as SigningTypes>::Public> as SignedPayload<Test>>::verify::<
						crypto::FragAuthId,
					>(&data, signature); // Notice in `pallet_accounts` that `EthLockUpdate<T::Public>` implements the trait `SignedPayload`

				assert!(signature_valid); // If `signature_valid` is true, it means `payload` and `signature` recovered the public address `data.public`
			}
		});
	}
}

mod internal_lock_update_tests {
	use core::str::FromStr;
	use ethabi::Address;
	use sp_core::keccak_256;
	use super::*;

	pub fn lock_(lock: &Lock) -> DispatchResult {
		Accounts::internal_lock_update(
			Origin::none(),
			lock.data.clone(),
			sp_core::ed25519::Signature([69u8; 64]), // this can be anything and it will still work
		)
	}

	pub fn unlock_(unlock: &Unlock) -> DispatchResult {
		Accounts::internal_lock_update(
			Origin::none(),
			unlock.data.clone(),
			sp_core::ed25519::Signature([69u8; 64]), // this can be anything
		)
	}

	#[test]
	fn test_eip_712_hash(){
		new_test_ext().execute_with(|| {
			let message = b"FragLock".to_vec();
			let contract ="0x3AEEE3a4952C7d27917eA9dF70669cf5a7bD20df";
			let sender = "0x90f8bf6a479f320ead074411a4b0e7944ea8c9c1";
			let contract = Address::from_str(&contract[2..]).map_err(|_| "Invalid response - invalid sender").unwrap();
			let sender = Address::from_str(&sender[2..]).map_err(|_| "Invalid response - invalid sender").unwrap();
			let lock_amount = U256::from(1000);
			let lock_period = U256::from(0);
			//let lock_amount: [u8; 32] = lock_amount.into();
			let message: Vec<u8> = [&[0x19, 0x01],
				// This is the `domainSeparator` (https://eips.ethereum.org/EIPS/eip-712#definition-of-domainseparator)
				&keccak_256(
					// We use the ABI encoding Rust library since it encodes each token as 32-bytes
					&ethabi::encode(
						&vec![
							Token::Uint(
								U256::from(keccak_256(b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"))
							),
							Token::Uint(U256::from(keccak_256(b"Fragnova Network Token"))), // The dynamic values bytes and string are encoded as a keccak_256 hash of their contents.
							Token::Uint(U256::from(keccak_256(b"1"))), // The dynamic values bytes and string are encoded as a keccak_256 hash of their contents.
							Token::Uint(U256::from(5)),
							Token::Address(contract),
						]
					)
				)[..],
				// This is the `hashStruct(message)`. Note: `hashStruct(message : 𝕊) = keccak_256(typeHash ‖ encodeData(message))`, where `typeHash = keccak_256(encodeType(typeOf(message)))`.
				&keccak_256(
					// We use the ABI encoding Rust library since it encodes each token as 32-bytes
					&ethabi::encode(
						&vec![
							// This is the `typeHash`
							Token::Uint(
								U256::from(keccak_256(b"Msg(string name,address sender,uint256 amount,uint8 lock_period)"))
							),
							// This is the `encodeData(message)`. (https://eips.ethereum.org/EIPS/eip-712#definition-of-encodedata)
							Token::Uint(U256::from(keccak_256(&message))),
							Token::Address(sender),
							Token::Uint(U256::from(lock_amount)),
							Token::Uint(U256::from(lock_period)),
						]
					)
				)[..]
			].concat();

			let hashed_message = keccak_256(&message);
			// hash taken from JS unit tests in hasten-contracts where the same EIP-712 typed message is composed with these same data
			assert_eq!("22fcb86fdede97797990263fa68e980fb61c8c4edfcee544b96a721ace81edbb", hex::encode(hashed_message));
		});
	}

	#[test]
	fn lock_by_unlinked_account_should_lock_frag_internally_and_reserve_nova() {
		new_test_ext().execute_with(|| {
			let _ = store_price_();
			let dd = DummyData::new();

			let current_block_number = System::block_number();

			let lock = dd.lock;

			assert_ok!(lock_(&lock));

			let mut events = <frame_system::Pallet<Test>>::events();
			assert_eq!(events.clone().len(), 3);
			let event = events.pop().expect("Expected at least one EventRecord to be found").event;
			assert_eq!(
				event,
				mock::Event::from(pallet_accounts::Event::Locked {
					eth_key: lock.data.sender.clone(),
					balance: SaturatedConversion::saturated_into::<
						<Test as pallet_assets::Config>::Balance,
					>(lock.data.amount),
					lock_period: lock.data.lock_period.clone()
				})
			);

			assert_eq!(
				<EthLockedFrag<Test>>::get(&lock.data.sender, current_block_number).unwrap(),
				EthLock {
					amount: SaturatedConversion::saturated_into::<
						<Test as pallet_assets::Config>::Balance,
					>(lock.data.amount.clone()),
					block_number: current_block_number,
					lock_period: 1,
					last_withdraw: 0,
				}
			);
			let initial_nova_amount =
				apply_percent(lock.data.amount.clone().as_u128(), get_initial_percentage_nova())
					* get_usd_equivalent_amount()
					* get_oracle_price();

			assert_eq!(
				<EthReservedNova<Test>>::get(&lock.data.sender).unwrap(),
				SaturatedConversion::saturated_into::<<Test as pallet_balances::Config>::Balance>(
					initial_nova_amount
				)
			);

			let data_tuple = (
				lock.data.amount,
				lock.data.lock_period,
				lock.data.sender,
				lock.data.signature,
				true,
				lock.data.block_number,
			);

			let data_hash: H256 = data_tuple.using_encoded(sp_io::hashing::blake2_256).into();

			assert_eq!(<EVMLinkVotingClosed<Test>>::get(&data_hash).unwrap(), current_block_number);

			let mut events = <frame_system::Pallet<Test>>::events();
			assert_eq!(events.clone().len(), 3);

			let event = events.pop().expect("Expected at least one EventRecord to be found").event;
			assert_eq!(
				event,
				mock::Event::from(pallet_accounts::Event::Locked {
					eth_key: lock.data.sender,
					balance: SaturatedConversion::saturated_into::<
						<Test as pallet_assets::Config>::Balance,
					>(lock.data.amount),
					lock_period: lock.data.lock_period.clone()
				})
			);

			let event = events.pop().expect("Expected at least one EventRecord to be found").event;
			assert_eq!(
				event,
				mock::Event::from(pallet_accounts::Event::NOVAReserved {
					eth_key: lock.data.sender.clone(),
					balance: SaturatedConversion::saturated_into::<
						<Test as pallet_balances::Config>::Balance,
					>(
						apply_percent(lock.data.amount.as_u128(), get_initial_percentage_nova())
							* get_usd_equivalent_amount()
							* get_oracle_price()
					)
				})
			);
		});
	}

	#[test]
	fn lock_by_linked_account_should_lock_frag_and_mint_nova() {
		new_test_ext().execute_with(|| {
			let _ = store_price_();
			let dd = DummyData::new();
			let lock = dd.lock;
			let link = lock.link.clone();
			let current_block_number = System::block_number();

			assert_ok!(link_(&link));
			assert_ok!(lock_(&lock));
			// assert that Frag is locked in Clamor
			assert_eq!(
				<EthLockedFrag<Test>>::get(&lock.data.sender, current_block_number).unwrap(),
				EthLock {
					amount: SaturatedConversion::saturated_into::<
						<Test as pallet_assets::Config>::Balance,
					>(lock.data.amount.clone()),
					block_number: current_block_number,
					lock_period: 1,
					last_withdraw: 0,
				}
			);
			let initial_nova_amount =
				apply_percent(lock.data.amount.clone().as_u128(), get_initial_percentage_nova())
					* get_usd_equivalent_amount()
					* get_oracle_price();

			let nova = pallet_balances::Pallet::<Test>::free_balance(&link.clamor_account_id);
			assert_eq!(U256::from(nova), U256::from(initial_nova_amount));
		});
	}

	#[test]
	fn link_an_account_with_reserved_nova_should_mint_and_increase_balance() {
		new_test_ext().execute_with(|| {
			let _ = store_price_();
			let dd = DummyData::new();
			let lock = dd.lock;
			let link = lock.link.clone();
			let current_block_number = System::block_number();

			assert_ok!(lock_(&lock));
			// assert that Frag is locked in Clamor
			assert_eq!(
				<EthLockedFrag<Test>>::get(&lock.data.sender, current_block_number).unwrap(),
				EthLock {
					amount: SaturatedConversion::saturated_into::<
						<Test as pallet_assets::Config>::Balance,
					>(lock.data.amount.clone()),
					block_number: current_block_number,
					lock_period: 1,
					last_withdraw: 0,
				}
			);

			let initial_nova_amount =
				apply_percent(lock.data.amount.clone().as_u128(), get_initial_percentage_nova())
					* get_usd_equivalent_amount()
					* get_oracle_price();

			assert_eq!(
				<EthReservedNova<Test>>::get(&lock.data.sender).unwrap(),
				SaturatedConversion::saturated_into::<<Test as pallet_balances::Config>::Balance>(
					initial_nova_amount
				)
			);

			let nova = pallet_balances::Pallet::<Test>::free_balance(&link.clamor_account_id);
			assert_eq!(nova, 0);

			// now link the account to have the reserved nova minted and put in balance
			// of Clamor account
			assert_ok!(link_(&link));

			let nova_linked =
				pallet_balances::Pallet::<Test>::free_balance(&link.clamor_account_id);

			assert_eq!(nova_linked as u128, initial_nova_amount);
			assert_eq!(<EthReservedNova<Test>>::contains_key(&lock.data.sender), false);
		});
	}

	#[test]
	fn lock_should_not_work_if_locked_amount_is_zero() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let contracts = <Test as Config>::EthFragContract::get_partner_contracts();
			let contract = Address::from_str(&contracts[0].as_str()[2..]).map_err(|_| "Invalid response - invalid sender").unwrap();
			let mut lock = dd.lock;
			lock.data.amount = U256::from(0u32);
			lock.data.lock_period = 1;
			lock.data.signature = create_lock_signature(
				lock.ethereum_account_pair.clone(),
				lock.data.amount.clone(),
				lock.data.lock_period.clone(),
				lock.data.sender.clone(),
				contract,
			);

			assert_noop!(lock_(&lock), Error::<Test>::SystematicFailure);
		});
	}

	#[test]
	fn lock_should_not_work_if_the_sender_is_not_recovered_from_the_signature() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let mut lock = dd.lock;
			lock.data.signature = dd.lock_signature;

			assert_noop!(lock_(&lock), Error::<Test>::VerificationFailed);
		});
	}

	#[test]
	fn block_number_of_first_lock_event_should_be_correct() {
		new_test_ext().execute_with(|| {
			let _ = store_price_();

			let dd = DummyData::new();
			let unlock = dd.unlock;
			let link = unlock.lock.link.clone();
			let current_block_number = System::block_number();

			assert_ok!(link_(&link));
			assert_ok!(lock_(&unlock.lock));
			assert_ok!(unlock_(&unlock));

			// assert that Frag is locked in Clamor
			assert_eq!(
				<EthLockedFrag<Test>>::get(&unlock.data.sender, current_block_number).unwrap(),
				EthLock {
					amount: SaturatedConversion::saturated_into::<
						<Test as pallet_assets::Config>::Balance,
					>(unlock.lock.data.amount.clone()),
					block_number: current_block_number,
					lock_period: 255,
					last_withdraw: 0,
				}
			);
		});
	}

	#[test]
	fn unlock_should_work() {
		new_test_ext().execute_with(|| {
			let _ = store_price_();
			let dd = DummyData::new();
			let unlock = dd.unlock;
			//let lock = dd.lock;
			let link = unlock.lock.link.clone();

			let current_block_number = System::block_number();

			assert_ok!(lock_(&unlock.lock));
			assert_ok!(unlock_(&unlock));

			assert_eq!(
				<EthLockedFrag<Test>>::get(&unlock.data.sender, current_block_number).unwrap(),
				EthLock {
					amount: SaturatedConversion::saturated_into::<
						<Test as pallet_assets::Config>::Balance,
					>(unlock.lock.data.amount),
					block_number: current_block_number,
					lock_period: 255,
					last_withdraw: 0,
				}
			);

			let initial_nova_amount = apply_percent(
				unlock.lock.data.amount.clone().as_u128(),
				get_initial_percentage_nova(),
			) * get_usd_equivalent_amount()
				* get_oracle_price();

			assert_eq!(
				<EthReservedNova<Test>>::get(&unlock.lock.data.sender).unwrap(),
				SaturatedConversion::saturated_into::<<Test as pallet_balances::Config>::Balance>(
					initial_nova_amount
				)
			);

			let nova = pallet_balances::Pallet::<Test>::free_balance(&link.clamor_account_id);
			assert_eq!(nova, 0);

			let data_tuple = (
				unlock.data.amount,
				unlock.data.lock_period,
				unlock.data.sender,
				unlock.data.signature,
				false,
				unlock.data.block_number,
			);

			let data_hash: H256 = data_tuple.using_encoded(sp_io::hashing::blake2_256).into();

			assert_eq!(<EVMLinkVotingClosed<Test>>::get(&data_hash).unwrap(), current_block_number);

			let event = <frame_system::Pallet<Test>>::events()
				.pop()
				.expect("Expected at least one EventRecord to be found")
				.event;
			assert_eq!(
				event,
				mock::Event::from(pallet_accounts::Event::Unlocked {
					eth_key: unlock.data.sender,
					balance: SaturatedConversion::saturated_into::<
						<Test as pallet_assets::Config>::Balance,
					>(0)
				})
			);
		});
	}

	#[test]
	fn unlock_should_not_work_if_unlocked_amount_is_greater_than_zero() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();
			let contracts = <Test as Config>::EthFragContract::get_partner_contracts();
			let contract = Address::from_str(&contracts[0].as_str()[2..]).map_err(|_| "Invalid response - invalid sender").unwrap();
			let mut unlock = dd.unlock;
			unlock.data.amount = U256::from(69u32); // greater than zero
			unlock.data.signature = create_unlock_signature(
				unlock.lock.ethereum_account_pair.clone(),
				U256::from(69u32),
				unlock.lock.data.sender,
				contract,
			);

			assert_ok!(lock_(&unlock.lock));

			assert_noop!(unlock_(&unlock), Error::<Test>::SystematicFailure);
		});
	}
}

mod withdraw_tests {
	use super::*;

	fn withdraw_(lock: &Lock) -> DispatchResult {
		Accounts::withdraw(Origin::signed(lock.link.clamor_account_id))
	}

	pub fn get_initial_amounts(lock: &Lock) -> u128 {
			apply_percent(lock.data.amount.clone().as_u128(), get_initial_percentage_nova())
				* get_usd_equivalent_amount()
				* get_oracle_price()

	}

	pub fn expected_nova_amount(week_num: u64, lock_period: u64, data_amount: u128) -> u128 {
		let nova_per_week = apply_percent(data_amount, 100 - get_initial_percentage_nova())
			/ u128::try_from(lock_period).unwrap();
		let expected_amount = nova_per_week
			* get_usd_equivalent_amount() // NOVA per week and 1 FRAG = 100 NOVA. 80% of 100 / 4 weeks
			* u128::try_from(week_num).unwrap()
			* get_oracle_price(); //
		expected_amount
	}

	#[test]
	fn withdraw_should_increase_nova_balance() {
		new_test_ext().execute_with(|| {
			let _ = store_price_();
			let dd = DummyData::new();
			let lock = dd.lock;
			let link = lock.link.clone();
			let current_block = System::block_number();

			assert_ok!(link_(&link));
			assert_ok!(lock_(&lock));

			// check the balance of the Clamor account
			let nova_balance =
				pallet_balances::Pallet::<Test>::free_balance(&link.clamor_account_id);

			// initial amounts of NOVA = 20%
			let initial_nova_amount = get_initial_amounts(&lock);
			assert_eq!(nova_balance as u128, initial_nova_amount);

			let lock_period = <EthLockedFrag<Test>>::get(&lock.data.sender, current_block)
				.unwrap()
				.lock_period;

			let lock_period_in_weeks =
				Accounts::eth_lock_period_to_weeks(lock_period).ok().unwrap();
			// fast forward to week 3
			let week_num = (lock_period_in_weeks - 1) as u64;
			System::set_block_number((60 * 60 * 24 * 7 * week_num / 6).into());

			assert_ok!(withdraw_(&lock)); // withdraw at week 3

			let data_amount: u128 = lock.data.amount.try_into().ok().unwrap();

			let expected_amount = expected_nova_amount(
				week_num.into(),
				lock_period_in_weeks.into(),
				data_amount.clone(),
			);
			let nova_new_balance =
				pallet_balances::Pallet::<Test>::free_balance(&link.clamor_account_id);
			assert_eq!(nova_new_balance as u128, expected_amount + initial_nova_amount);
		});
	}

	#[test]
	fn withdraw_after_lock_period_is_over_gives_correct_amounts() {
		new_test_ext().execute_with(|| {
			let _ = store_price_();
			let dd = DummyData::new();
			let lock = dd.lock;
			let link = lock.link.clone();
			let current_block = System::block_number();

			assert_ok!(link_(&link));
			assert_ok!(lock_(&lock));

			let initial_nova_amount= get_initial_amounts(&lock);

			let nova = pallet_balances::Pallet::<Test>::free_balance(&link.clamor_account_id);
			assert_eq!(nova as u128, initial_nova_amount);

			let lock_period = <EthLockedFrag<Test>>::get(&lock.data.sender, current_block)
				.unwrap()
				.lock_period;
			let lock_period_in_weeks =
				Accounts::eth_lock_period_to_weeks(lock_period).ok().unwrap();
			let exceeding_week_num = (lock_period_in_weeks + 1) as u64; //
			System::set_block_number((60 * 60 * 24 * 7 * exceeding_week_num / 6).into());

			assert_ok!(withdraw_(&lock));

			let data_amount: u128 = lock.data.amount.try_into().ok().unwrap();
			let expected_amount = expected_nova_amount(
				lock_period_in_weeks.into(),
				lock_period_in_weeks.into(),
				data_amount.clone(),
			);
			let nova_new_balance =
				pallet_balances::Pallet::<Test>::free_balance(&link.clamor_account_id);
			assert_eq!(nova_new_balance as u128, expected_amount + initial_nova_amount);

			assert_eq!(<EthLockedFrag<Test>>::get(&lock.data.sender, current_block), None);
		});
	}

	#[test]
	fn subsequent_withdraws_after_one_lock_mint_correct_amounts() {
		new_test_ext().execute_with(|| {
			let _ = store_price_();
			let dd = DummyData::new();
			let lock = dd.lock;
			let link = lock.link.clone();

			assert_ok!(link_(&link));
			assert_ok!(lock_(&lock));

			let current_block = System::block_number();
			assert_eq!(current_block, 1);

			assert_eq!(
				<EthLockedFrag<Test>>::get(&lock.data.sender, current_block).unwrap(),
				EthLock {
					amount: SaturatedConversion::saturated_into::<
						<Test as pallet_assets::Config>::Balance,
					>(lock.data.amount.clone()),
					block_number: current_block,
					lock_period: <EthLockedFrag<Test>>::get(&lock.data.sender, current_block).unwrap().lock_period,
					last_withdraw: 0,
				}
			);

			let initial_nova_amount = get_initial_amounts(&lock);

			let nova = pallet_balances::Pallet::<Test>::free_balance(&link.clamor_account_id);
			assert_eq!(nova as u128, initial_nova_amount);

			let lock_period = <EthLockedFrag<Test>>::get(&lock.data.sender, current_block).unwrap().lock_period;
			let lock_period_in_weeks = Accounts::eth_lock_period_to_weeks(lock_period).ok().unwrap();

			// Go to week 1 after lock
			let weeks_after_first_lock = 1;
			let future_block_number = (60 * 60 * 24 * 7 * weeks_after_first_lock)/ 6;
			assert_eq!(future_block_number, 100800);

			System::set_block_number(future_block_number);

			assert_ok!(withdraw_(&lock)); // withdraw at week 1

			// check that we registered to correct info about lock and that last_withdraw field is updated
			assert_eq!(
				<EthLockedFrag<Test>>::get(&lock.data.sender, current_block).unwrap(),
				EthLock {
					amount: SaturatedConversion::saturated_into::<
						<Test as pallet_assets::Config>::Balance,
					>(lock.data.amount.clone()),
					block_number: current_block,
					lock_period: <EthLockedFrag<Test>>::get(&lock.data.sender, current_block).unwrap().lock_period,
					last_withdraw: u128::try_from(weeks_after_first_lock.clone()).unwrap(),
				}
			);

			let data_amount: u128 = lock.data.amount.try_into().ok().unwrap();
			let expected_amount = expected_nova_amount(weeks_after_first_lock.clone(), lock_period_in_weeks.into(), data_amount.clone());
			let nova_new_balance =
				pallet_balances::Pallet::<Test>::free_balance(&link.clamor_account_id);
			assert_eq!(nova_new_balance as u128, expected_amount + initial_nova_amount);

			System::set_block_number(60 * 60 * 24 * 7 * lock_period_in_weeks as u64/ 6);

			assert_ok!(withdraw_(&lock)); // withdraw at week 4
			assert_eq!(
				<EthLockedFrag<Test>>::get(&lock.data.sender, future_block_number.clone()), None,
				"EthLockedFrag should have been removed for this lock event since all the due amount is yielded and withdrawn."
			);

			let expected_amount = expected_nova_amount(lock_period_in_weeks.into(), lock_period_in_weeks.into(), data_amount.clone());
			let nova_new_balance =
				pallet_balances::Pallet::<Test>::free_balance(&link.clamor_account_id);
			assert_eq!(nova_new_balance as u128, expected_amount + initial_nova_amount.clone());
		});
	}

	#[test]
	fn subsequent_withdraws_after_multiple_locks_produces_correct_lock_registrations() {
		new_test_ext().execute_with(|| {
			let _ = store_price_();
			let dd = DummyData::new();
			let lock = dd.lock;
			let lock2 = dd.lock2;
			let link = lock.link.clone();
			let lock_period = lock.data.lock_period;
			let lock_period_in_weeks = Accounts::eth_lock_period_to_weeks(lock_period).ok().unwrap();

			assert_ok!(link_(&link));
			assert_ok!(lock_(&lock));

			let current_block = System::block_number();
			assert_eq!(current_block, 1);

			assert_eq!(
				<EthLockedFrag<Test>>::get(&lock.data.sender, current_block).unwrap(),
				EthLock {
					amount: SaturatedConversion::saturated_into::<
						<Test as pallet_assets::Config>::Balance,
					>(lock.data.amount.clone()),
					block_number: current_block,
					lock_period: <EthLockedFrag<Test>>::get(&lock.data.sender, current_block).unwrap().lock_period,
					last_withdraw: 0,
				}
			);

			// GO TWO WEEKS LATER FOR A WITHDRAW
			let weeks_later_from_first_lock = 2;
			let future_block_number = (60 * 60 * 24 * 7 * weeks_later_from_first_lock.clone())/ 6;
			assert_eq!(future_block_number, 201600);

			System::set_block_number(future_block_number);
			assert_ok!(lock_(&lock2));

			assert_eq!(
				<EthLockedFrag<Test>>::get(&lock.data.sender, current_block).unwrap(),
				EthLock {
					amount: SaturatedConversion::saturated_into::<
						<Test as pallet_assets::Config>::Balance,
					>(lock.data.amount.clone()),
					block_number: current_block,
					lock_period: <EthLockedFrag<Test>>::get(&lock.data.sender, current_block).unwrap().lock_period,
					last_withdraw: 0,
				}
			);
			assert_eq!(
				<EthLockedFrag<Test>>::get(&lock2.data.sender, future_block_number).unwrap(),
				EthLock {
					amount: SaturatedConversion::saturated_into::<
						<Test as pallet_assets::Config>::Balance,
					>(lock2.data.amount.clone()),
					block_number: future_block_number,
					lock_period: lock2.data.lock_period,
					last_withdraw: 0,
				}
			);

			assert_ok!(withdraw_(&lock)); // withdraw at week 2

			let last_withdraw = (future_block_number - current_block) * 6 / (60 * 60 * 24 * 7);

			assert_eq!(
				<EthLockedFrag<Test>>::get(&lock.data.sender, current_block.clone()).unwrap(),
				EthLock {
					amount: SaturatedConversion::saturated_into::<
						<Test as pallet_assets::Config>::Balance,
					>(lock.data.amount.clone()),
					block_number: current_block.clone(),
					lock_period: lock.data.lock_period.clone(),
					last_withdraw: (last_withdraw +1) as u128,
				}
			);
			assert_eq!(
				<EthLockedFrag<Test>>::get(&lock2.data.sender, future_block_number).unwrap(),
				EthLock {
					amount: SaturatedConversion::saturated_into::<
						<Test as pallet_assets::Config>::Balance,
					>(lock2.data.amount.clone()),
					block_number: future_block_number,
					lock_period: lock2.data.lock_period,
					last_withdraw: last_withdraw as u128,
				}
			);

			let next_week = (60 * 60 * 24 * 7 * lock_period_in_weeks as u64)/ 6;
			System::set_block_number(next_week);
			let last_withdraw = (next_week - future_block_number) * 6 / (60 * 60 * 24 * 7) + 1;

			assert_ok!(withdraw_(&lock)); // withdraw at week 1
			assert_eq!(
				<EthLockedFrag<Test>>::get(&lock.data.sender, current_block.clone()), None,
				"EthLockedFrag should have been removed for this account since there is nothing more to possibly withdraw"
			);

			assert_eq!(
				<EthLockedFrag<Test>>::get(&lock2.data.sender, future_block_number).unwrap(),
				EthLock {
					amount: SaturatedConversion::saturated_into::<
						<Test as pallet_assets::Config>::Balance,
					>(lock2.data.amount.clone()),
					block_number: future_block_number,
					lock_period: lock2.data.lock_period,
					last_withdraw: last_withdraw as u128,
				}
			);

		});
	}

	#[test]
	fn subsequent_withdraws_after_multiple_locks_produces_correct_balances() {
		new_test_ext().execute_with(|| {
			let _ = store_price_();
			let dd = DummyData::new();
			let lock = dd.lock;
			let lock2 = dd.lock2;
			let link = lock.link.clone();
			let link2 = lock2.link.clone();
			let lock_period = lock.data.lock_period;
			let lock_period2 = lock2.data.lock_period;
			let lock_period_in_weeks =
				Accounts::eth_lock_period_to_weeks(lock_period).ok().unwrap();
			let lock_period_in_weeks2 =
				Accounts::eth_lock_period_to_weeks(lock_period2).ok().unwrap();
			let data_amount: u128 = lock.data.amount.try_into().ok().unwrap();
			let data_amount2: u128 = lock2.data.amount.try_into().ok().unwrap();

			assert_ok!(link_(&link));
			assert_ok!(lock_(&lock));

			let current_block = System::block_number();
			assert_eq!(current_block, 1);

			let initial_nova_amount= get_initial_amounts(&lock);
			let nova = pallet_balances::Pallet::<Test>::free_balance(&link.clamor_account_id);
			assert_eq!(nova as u128, initial_nova_amount);

			// GO TWO WEEKS LATER FOR A WITHDRAW
			let weeks_later_from_first_lock = 2;
			let future_block_number = (60 * 60 * 24 * 7 * weeks_later_from_first_lock.clone()) / 6;
			System::set_block_number(future_block_number);

			assert_ok!(lock_(&lock2));

			let initial_nova_amount2= get_initial_amounts(&lock2);
			let nova2 = pallet_balances::Pallet::<Test>::free_balance(&link2.clamor_account_id);
			assert_eq!(nova2 as u128, initial_nova_amount2 + initial_nova_amount);

			assert_ok!(withdraw_(&lock)); // withdraw at week 2

			let expected_amount = expected_nova_amount(
				weeks_later_from_first_lock.clone(),
				lock_period_in_weeks.into(),
				data_amount.clone(),
			);
			let expected_amount2 = expected_nova_amount(
				weeks_later_from_first_lock.clone() - 1,
				lock_period_in_weeks2.into(),
				data_amount2.clone(),
			);

			let nova_new_balance =
				pallet_balances::Pallet::<Test>::free_balance(&link.clamor_account_id);
			assert_eq!(
				nova_new_balance as u128,
				expected_amount
					+ expected_amount2 + initial_nova_amount.clone()
					+ initial_nova_amount2.clone()
			);

			let next_week = (60 * 60 * 24 * 7 * lock_period_in_weeks as u64) / 6;
			System::set_block_number(next_week);

			assert_ok!(withdraw_(&lock)); // withdraw at week 1

			let expected_amount = expected_nova_amount(
				lock_period_in_weeks.into(),
				lock_period_in_weeks.into(),
				data_amount.clone(),
			);
			let expected_amount2 = expected_nova_amount(
				(lock_period_in_weeks - 1).into(),
				lock_period_in_weeks2.into(),
				data_amount2.clone(),
			);
			let nova_new_balance =
				pallet_balances::Pallet::<Test>::free_balance(&link.clamor_account_id);
			assert_eq!(
				nova_new_balance as u128,
				expected_amount
					+ expected_amount2 + initial_nova_amount.clone()
					+ initial_nova_amount2.clone()
			);
		});
	}
}
