use crate::mock;
use crate as pallet_accounts;

use crate::*;

use crate::dummy_data::*;

use crate::mock::*;

use serde_json::json;

use frame_system::offchain::{
    SignedPayload, SigningTypes,
};

use frame_support::{
    assert_noop, assert_ok,
    traits::{
        TypedGet,
    }
};

use frame_support::dispatch::DispatchResult;

use sp_core::{

    H160,

    H256,
    
    offchain::{
        testing,
    }
};

use sp_runtime::offchain::storage::StorageValueRef;

use sp_runtime::SaturatedConversion;

use codec::Encode;

use ethabi::Token;

pub use link_tests::link_ as link_;
pub use internal_lock_update_tests::lock_ as lock_;

mod link_tests {


    use crate::FragUsage;

    use super::*;

    pub fn link_(signer: <Test as frame_system::Config>::AccountId, link_signature: &sp_core::ecdsa::Signature) -> DispatchResult {
        AccountsPallet::link(
            Origin::signed(signer),
            link_signature.clone()
        )
    }

    #[test]
    fn link_should_work() {

        new_test_ext().execute_with(|| {

			let dd = DummyData::new();

            let link = dd.link;


            assert_ok!(
                link_(link.clamor_account_id, &link.get_link_signature())
            );

            assert!(<EVMLinks<Test>>::get(&link.clamor_account_id).unwrap() == link.get_ethereum_account_id());
			assert!(<EVMLinksReverse<Test>>::get(&link.get_ethereum_account_id()).unwrap() == link.clamor_account_id);

            assert!(<FragUsage<Test>>::get(&link.clamor_account_id).unwrap() == 0);

            let event = <frame_system::Pallet<Test>>::events().pop().expect("Expected at least one EventRecord to be found").event;
        	assert_eq!(event, mock::Event::from(pallet_accounts::Event::Linked { sender: link.clamor_account_id, eth_key: link.get_ethereum_account_id() }));

        });


    }

    #[test]
    fn link_should_not_work_if_signature_is_invalid() {

        new_test_ext().execute_with(|| {

			let dd = DummyData::new();

            let link = dd.link;

            assert_noop!(
                link_(link.clamor_account_id, &dd.link_signature),
                Error::<Test>::VerificationFailed,
            );

        });

    }

    #[test]
    // #[ignore = "seems there's a bug with linking"]
    fn link_should_not_work_if_hashed_message_is_invalid() { 

        new_test_ext().execute_with(|| {

			let dd = DummyData::new();

            let link = dd.link;

            assert_noop!(
                link_(dd.account_id, &link.get_link_signature()),   
                Error::<Test>::VerificationFailed,
            );

        });

    }

    #[test]
    fn link_should_not_work_if_clamor_account_is_already_linked() {

        new_test_ext().execute_with(|| {
            
            let dd = DummyData::new();

            let link = dd.link;

            link_(link.clamor_account_id, &link.get_link_signature());

            let link_diff_ethereum_account_pair = Link {
                ethereum_account_pair: dd.ethereum_account_pair,
                ..link
            };

            assert_noop!(
                link_(link_diff_ethereum_account_pair.clamor_account_id, &link_diff_ethereum_account_pair.get_link_signature()),
                Error::<Test>::AccountAlreadyLinked
            );

        });

    }

    #[test]
    // #[ignore = "i think this is a bug!"]
    fn link_should_not_work_if_ethereum_account_is_already_linked() { 

        new_test_ext().execute_with(|| {
            
            let dd = DummyData::new();

            let link = dd.link;

            link_(link.clamor_account_id, &link.get_link_signature());

            println!("eth_key for link is: {:?}", link.get_ethereum_account_id());
            println!("eth_key for link is: {:?}", link.get_ethereum_account_id());
            println!("eth_key for link is: {:?}", link.get_ethereum_account_id());

            let link_diff_clamor_account_id = Link {
                clamor_account_id: dd.account_id,
                ..link
            };

            println!("eth_key for link_diff_clamor_account_id is: {:?}", link_diff_clamor_account_id.get_ethereum_account_id());

            assert_noop!(
                link_(link_diff_clamor_account_id.clamor_account_id, &link_diff_clamor_account_id.get_link_signature()),
                Error::<Test>::AccountAlreadyLinked
            );

        });

    }




}


mod unlink_tests {
    use super::*;

    pub fn unlink(signer: <Test as frame_system::Config>::AccountId, ethereum_accound_id: H160) -> DispatchResult {
        AccountsPallet::unlink(
            Origin::signed(signer),
            ethereum_accound_id
        )
    }

    #[test]
    fn unlink_should_work() {
        
        new_test_ext().execute_with(|| {

			let dd = DummyData::new();
            let link = dd.link;
            
            link_(link.clamor_account_id, &link.get_link_signature());

            assert_ok!(unlink(link.clamor_account_id, link.get_ethereum_account_id()));
            

            assert!(<EVMLinks<Test>>::contains_key(&link.clamor_account_id) == false);
			assert!(<EVMLinksReverse<Test>>::contains_key(&link.get_ethereum_account_id()) == false);

            assert!(<FragUsage<Test>>::contains_key(&link.clamor_account_id) == false);

            assert!(<PendingUnlinks<Test>>::get().contains(&link.clamor_account_id));

            let event = <frame_system::Pallet<Test>>::events().pop().expect("Expected at least one EventRecord to be found").event;
        	assert_eq!(event, mock::Event::from(pallet_accounts::Event::Unlinked{ sender: link.clamor_account_id, eth_key: link.get_ethereum_account_id()}));

        });

    }

    #[test]
    fn unlink_should_not_work_if_link_does_not_exist() {

        new_test_ext().execute_with(|| {

			let dd = DummyData::new();
            let link = dd.link;

            assert_noop!(
                unlink(link.clamor_account_id, link.get_ethereum_account_id()),
                Error::<Test>::AccountNotLinked
            );

        });
    }

    #[test]
    // #[ignore = "this is a known bug"]
    fn unlink_should_not_work_if_ethereum_account_id_is_linked_with_different_clamor_account_id() {

        new_test_ext().execute_with(|| {

			let dd = DummyData::new();

            let link = dd.link;
            let link_second = dd.link_second;

            link_(link.clamor_account_id, &link.get_link_signature());
            link_(link_second.clamor_account_id, &link_second.get_link_signature());

            assert_noop!(
                unlink(link.clamor_account_id, link_second.get_ethereum_account_id()),
                Error::<Test>::AccountNotLinked
            );

        });
    }
    
}



mod sync_frag_locks_tests {
    

    use super::*;

    #[test]
    fn sync_frag_locks_should_work() {

        let (mut t, pool_state, 
            offchain_state, ed25519_public_key) 
        = new_test_ext_with_ocw();

        let dd = DummyData::new();
        let lock = dd.lock;

        let geth_url = Some(String::from("https://banned.video/"));

        sp_clamor::init(geth_url);

        let latest_block_number = 
        lock.block_number // ensure that `lock.block_number` exists by making `latest_block_number` greater than or equal to it
        .saturating_add(<Test as pallet_accounts::Config>::EthConfirmations::get())
        .saturating_add(69); 

        offchain_state.write().expect_request(testing::PendingRequest {
            method: String::from("POST"),
            uri: String::from_utf8(sp_clamor::clamor::get_geth_url().unwrap()).unwrap(),
            headers: vec![(String::from("Content-Type"), String::from("application/json"))],
            body: json!({
                "jsonrpc": "2.0",
                "method": "eth_blockNumber",
                "id": 1u64, 
            }).to_string().into_bytes(),
            response: Some(json!({
                "id": 69u64,
                "jsonrpc": "2.0",
                "result": format!("0x{:x}", latest_block_number), 
            }).to_string().into_bytes()),
            sent: true,
            ..Default::default()
        });

        let from_block = 0;
        let to_block = latest_block_number.saturating_sub(<Test as pallet_accounts::Config>::EthConfirmations::get());

        offchain_state.write().expect_request(testing::PendingRequest {
            method: String::from("POST"),
            uri: String::from_utf8(sp_clamor::clamor::get_geth_url().unwrap()).unwrap(),
            headers: vec![(String::from("Content-Type"), String::from("application/json"))],
            body: json!({
                "jsonrpc": "2.0",
                "method": "eth_getLogs", // i.e get the event logs of the smart contract (more info: https://docs.alchemy.com/alchemy/guides/eth_getlogs#what-are-logs)
                "id": "0", // WHY IS THIS A STRING @sinkingsugar  MOLTO IMPORTANTE!
                "params": [{
                    "fromBlock": format!("0x{:x}", from_block), 
                    "toBlock": format!("0x{:x}", to_block), // Give us the event logs that were emitted (if any) from the block number `from_block` to the block number `to_block`, inclusive
                    "address": <<Test as pallet_accounts::Config>::EthFragContract as pallet_accounts::EthFragContract>::get_partner_contracts()[0],
                    "topics": [ 
                        // [] to OR
                        [pallet_accounts::LOCK_EVENT, pallet_accounts::UNLOCK_EVENT]
                    ],
                }]
            }).to_string().into_bytes(),
            response: Some(json!({
                "id": 69u64,
                "jsonrpc": "2.0",
                "result": [
                    {
                        "address": <<Test as pallet_accounts::Config>::EthFragContract as pallet_accounts::EthFragContract>::get_partner_contracts()[0],
                        "topics": [
                            pallet_accounts::LOCK_EVENT,
                            format!("0x{}", hex::encode(ethabi::encode(&[Token::Address(lock.get_ethereum_account_id())])))
                        ],
                        "data": format!("0x{}", hex::encode(ethabi::encode(&[Token::Bytes(lock.get_lock_signature().0.to_vec()), Token::Uint(lock.lock_amount)]))),
                        "blockNumber": format!("0x{:x}", lock.block_number),

                        // Following key-values were blindly copied from https://docs.alchemy.com/alchemy/apis/ethereum/eth-getlogs (since they won't aren't even looked at in the function `sync_frag_locks`): 
                        // So they are all wrong
                        "transactionHash": "0xab059a62e22e230fe0f56d8555340a29b2e9532360368f810595453f6fdd213b",
                        "transactionIndex": "0xac",
                        "blockHash": "0x8243343df08b9751f5ca0c5f8c9c0460d8a9b6351066fae0acbd4d3e776de8bb",
                        "logIndex": "0x56",
                        "removed": false,
                    },
                ]
            }).to_string().into_bytes()),
            sent: true,
            ..Default::default()
        });

        let expected_payload = EthLockUpdate {
            public: <Test as SigningTypes>::Public::from(ed25519_public_key),
            amount: lock.lock_amount,
            sender: lock.get_ethereum_account_id(),
            signature: lock.get_lock_signature(),
            lock: true, // yes, please lock
            block_number: lock.block_number,
        };

        
        t.execute_with(|| {               
            // when
            AccountsPallet::sync_partner_contracts(1); // THIS IS BIG DADDY!
            // then
            let tx = pool_state.write().transactions.pop().unwrap();
            let tx = <Extrinsic as codec::Decode>::decode(&mut &*tx).unwrap();
            assert_eq!(tx.signature, None); // Because it's an **unsigned transaction** with a signed payload
            

            if let Call::AccountsPallet(crate::Call::internal_lock_update { 
                data: payload,
                signature: signature,
            }) = tx.call {
                
                assert_eq!(payload, expected_payload);


                let signature_valid =
                    <EthLockUpdate<
                        <Test as SigningTypes>::Public 
                    > as SignedPayload<Test>
                    >::verify::<crypto::FragAuthId>(&payload, signature); // Notice in `pallet_accounts` that `EthLockUpdate<T::Public>` implements the trait `SignedPayload`
                    

                assert!(signature_valid); // If `signature_valid` is true, it means `payload` and `signature` recovered the public address `payload.public`
            }

            let storage = StorageValueRef::persistent(b"frag_sync_last_block");
            assert_eq!(
                storage.get::<Vec<u8>>().unwrap().unwrap(), 
                format!("0x{:x}", to_block).as_bytes().to_vec()
            );
            
        });
    }

    


    
    
}




mod internal_lock_update_tests {

    use super::*;


    pub fn lock_(lock: &Lock) -> DispatchResult {

        let payload = EthLockUpdate {
            public: <Test as SigningTypes>::Public::from(sp_core::ed25519::Public([69u8; 32])),
            amount: lock.lock_amount,
            sender: lock.get_ethereum_account_id(),
            signature: lock.get_lock_signature(),
            lock: true, // yes, please lock it! 
            block_number: lock.block_number,
        };

        AccountsPallet::internal_lock_update(Origin::none(), payload, sp_core::ed25519::Signature([69u8; 64]))
    }

    fn unlock_(unlock: &Unlock) -> DispatchResult {

        let payload = EthLockUpdate {
            public: <Test as SigningTypes>::Public::from(sp_core::ed25519::Public([69u8; 32])),
            amount: unlock.unlock_amount,
            sender: unlock.lock.get_ethereum_account_id(),
            signature: unlock.get_unlock_signature(),
            lock: false, // yes, please unlock it! 
            block_number: unlock.block_number,
        };

        AccountsPallet::internal_lock_update(Origin::none(), payload, sp_core::ed25519::Signature([69u8; 64]))
    
    }

    #[test]
    fn lock_should_work() {
        new_test_ext().execute_with(|| {

            let dd = DummyData::new();
            let lock = dd.lock;

            let current_block_number = System::block_number(); //@sinkingsugar

            assert_ok!(lock_(&lock));

            assert_eq!(
                <EthLockedFrag<Test>>::get(&lock.get_ethereum_account_id()).unwrap(), 
                EthLock {
                    amount: SaturatedConversion::saturated_into::<<Test as pallet_balances::Config>::Balance>(lock.lock_amount.clone()),
                    block_number: current_block_number,
                }
            );

            let data_tuple = 
            (lock.lock_amount, lock.get_ethereum_account_id(), lock.get_lock_signature(), true, lock.block_number);
            
            let data_hash: H256 = data_tuple.using_encoded(sp_io::hashing::blake2_256).into();

            assert_eq!(
                <EVMLinkVotingClosed<Test>>::get(&data_hash).unwrap(), 
                current_block_number
            );
            

            let event = <frame_system::Pallet<Test>>::events().pop().expect("Expected at least one EventRecord to be found").event;
            assert_eq!(
                event, 
                mock::Event::from(
                    pallet_accounts::Event::Locked { 
                        eth_key: lock.get_ethereum_account_id(), 
                        balance: SaturatedConversion::saturated_into::<<Test as pallet_balances::Config>::Balance>(lock.lock_amount)
                    }
                )
            );
        });
    }

    #[test]
    // #[ignore = "i have no idea how to import pallet-protos"]
    fn lock_should_remove_staking_information_if_linked_clamor_account_has_a_greater_used_amount_than_the_lock_amount() { // TODO
        new_test_ext().execute_with(|| {

            let dd = DummyData::new();

            // let stake = dd.stake;
            let lock = dd.lock;
            let link = lock.get_link();

            link_(link.clamor_account_id, &link.get_link_signature()); 
            lock_(&lock);

            // TODO - Stake some FRAG token

            todo!("I have no idea what to do here - because pallet-frag's Cargo.toml file doesn't use pallet-protos as a dependency!");

            // TODO - Lock more FRAG token

            
            assert!(<FragUsage<Test>>::contains_key(&link.clamor_account_id) == false);
            assert!(<PendingUnlinks<Test>>::get().contains(&link.clamor_account_id));
            

        });
    }

    

    #[test]
    fn unlock_should_work() {
        new_test_ext().execute_with(|| {

            let dd = DummyData::new();
            let unlock = dd.unlock;

            let current_block_number = System::block_number(); //@sinkingsugar

            lock_(&unlock.lock);

            assert_ok!(unlock_(&unlock));


            assert_eq!(
                <EthLockedFrag<Test>>::get(&unlock.get_ethereum_account_id()).unwrap(), 
                EthLock {
                    amount: SaturatedConversion::saturated_into::<<Test as pallet_balances::Config>::Balance>(unlock.unlock_amount.clone()),
                    block_number: current_block_number,
                }
            );

            let data_tuple = 
            (unlock.unlock_amount, unlock.get_ethereum_account_id(), unlock.get_unlock_signature(), false, unlock.block_number);
            
            let data_hash: H256 = data_tuple.using_encoded(sp_io::hashing::blake2_256).into();

            assert_eq!(
                <EVMLinkVotingClosed<Test>>::get(&data_hash).unwrap(), 
                current_block_number
            );


            let event = <frame_system::Pallet<Test>>::events().pop().expect("Expected at least one EventRecord to be found").event;
            assert_eq!(
                event, 
                mock::Event::from(
                    pallet_accounts::Event::Unlocked {
                        eth_key: unlock.get_ethereum_account_id(), 
                        balance: SaturatedConversion::saturated_into::<<Test as pallet_balances::Config>::Balance>(unlock.unlock_amount)
                    }
                )
            );


        });
    }

    #[test]
    fn unlock_should_unlink_clamor_account_if_clamor_account_is_linked() {

        new_test_ext().execute_with(|| {
            let dd = DummyData::new();

            let unlock = dd.unlock;
            let lock = unlock.lock.clone();
            let link = lock.get_link();
            
            link_(link.clamor_account_id, &link.get_link_signature()); 

            lock_(&lock);
            unlock_(&unlock);

            assert!(<EVMLinks<Test>>::contains_key(&link.clamor_account_id) == false);
			assert!(<EVMLinksReverse<Test>>::contains_key(&link.get_ethereum_account_id()) == false);

            assert!(<FragUsage<Test>>::contains_key(&link.clamor_account_id) == false);

            assert!(<PendingUnlinks<Test>>::get().contains(&link.clamor_account_id));

            let event = System::events().get(System::events().len() - 2).expect("Expected at least two EventRecords to be found").event.clone();
        	assert_eq!(event, mock::Event::from(pallet_accounts::Event::Unlinked{ sender: link.clamor_account_id, eth_key: link.get_ethereum_account_id() }));

        });

    }
    


}

