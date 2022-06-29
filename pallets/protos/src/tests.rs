use crate::mock;
use crate as pallet_protos;

use crate::*;

use crate::dummy_data::*;

use crate::mock::*;



use frame_support::{
	assert_noop, assert_ok,
	traits::{
		Get,
	}
};
use codec::{Compact};


use std::collections::BTreeMap;

use frame_support::dispatch::DispatchResult;


use upload_tests::upload as upload;
use stake_tests::stake_ as stake_;


use pallet_frag::EthLockUpdate;
use frame_system::offchain::SigningTypes;
use copied_from_pallet_frag::lock_ as lock_;
use copied_from_pallet_frag::link_ as link_;


mod copied_from_pallet_frag {

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

        TicketsPallet::internal_lock_update(Origin::none(), payload, sp_core::ed25519::Signature([69u8; 64]))
    }

	pub fn link_(signer: <Test as frame_system::Config>::AccountId, link_signature: &sp_core::ecdsa::Signature) -> DispatchResult {
        TicketsPallet::link(
            Origin::signed(signer),
            link_signature.clone()
        )
    }
}


mod upload_tests {

	use super::*;


	pub fn upload(signer: <Test as frame_system::Config>::AccountId, proto: &ProtoFragment) -> DispatchResult {
		ProtosPallet::upload(
			Origin::signed(signer), 
			proto.references.clone(), 
			proto.category.clone(), 
			proto.tags.clone(), 
			proto.linked_asset.clone(), 
			proto.include_cost.map(|cost| Compact::from(cost)), 
			proto.data.clone(),
		)
	}


	#[test]
	fn upload_should_work() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let block_number = System::block_number(); //@sinkingsugar

			let proto = dd.proto_fragment;

			assert_ok!(upload(dd.account_id, &proto));

			assert!(<Protos<Test>>::contains_key(proto.get_proto_hash())); 
			
			let proto_struct = <Protos<Test>>::get(proto.get_proto_hash()).unwrap();

			// I am using `match` to ensure that this test case fails if a new field is ever added to the `Proto` struct
			// match proto_struct {
			// 	Proto {
			// 		block: 1, 
			// 		patches: Vec::new(), 
			// 		include_cost: proto.include_cost.map(|cost| Compact::from(cost)),  
			// 		creator: dd.account_id, 
			// 		owner: dd.account_id, 
			// 		references: proto.references, 
			// 		category: proto.category,
			// 		tags: proto.tags,
			// 		metadata: BTreeMap::new(),
			// 	} => (),
			// 	// _ => println!("Time to panic!!! Mayday"),
			// }

			let correct_proto_struct = Proto {
				block: block_number, 
				patches: Vec::new(), 
				include_cost: proto.include_cost.map(|cost| Compact::from(cost)),  
				creator: dd.account_id, 
				owner: ProtoOwner::User(dd.account_id), 
				references: proto.references.clone(), 
				category: proto.category.clone(),
				tags: Vec::new(), // proto.tags,
				metadata: BTreeMap::new(),
			};

			// Ensure that this test case fails if a new field is ever added to the `Proto` struct
			match proto_struct {
				s if s == correct_proto_struct => (),
				_ => panic!("The correct `Proto` struct was not saved in the StorageMap `Protos`"),
			}


			assert!(<ProtosByCategory<Test>>::get(&proto.category).unwrap().contains(&proto.get_proto_hash()));
			assert!(<ProtosByOwner<Test>>::get(ProtoOwner::User(dd.account_id)).unwrap().contains(&proto.get_proto_hash()));

			let event = <frame_system::Pallet<Test>>::events().pop().expect("Expected at least one EventRecord to be found").event;
        	assert_eq!(event, mock::Event::from(pallet_protos::Event::Uploaded(proto.get_proto_hash(), proto.get_proto_cid())));
		});
	}

	#[test]
	fn upload_should_not_works_if_proto_hash_exists() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();
			let proto = dd.proto_fragment;
			upload(dd.account_id, &proto);
			assert_noop!(
				upload(dd.account_id, &proto), 
				Error::<Test>::ProtoExists
			);
		});
	}

	#[test]
	fn upload_should_work_if_user_staked_enough() {

		new_test_ext().execute_with(|| {
			
			let dd = DummyData::new();

			let stake = dd.stake;
	
			upload(dd.account_id, &stake.proto_fragment);

			lock_(&stake.lock);
			link_(stake.lock.get_link().clamor_account_id, &stake.lock.get_link().get_link_signature());

			stake_(
				stake.lock.get_link().clamor_account_id, 
				&stake.proto_fragment, 
				&stake.get_stake_amount()
			);
	
			let proto_with_refs = ProtoFragment {
				references: vec![stake.proto_fragment.get_proto_hash()], 
				..dd.proto_fragment
			};
			
			assert_ok!(upload(stake.lock.get_link().clamor_account_id, &proto_with_refs));

		});

	}
	
	#[test]
	fn upload_should_not_work_if_user_did_not_stake_enough() {

		new_test_ext().execute_with(|| {

			let dd = DummyData::new();

			let stake = dd.stake;
	
			upload(dd.account_id, &stake.proto_fragment);

			lock_(&stake.lock);
			link_(stake.lock.get_link().clamor_account_id, &stake.lock.get_link().get_link_signature());

			stake_(
				stake.lock.get_link().clamor_account_id, 
				&stake.proto_fragment, 
				&(stake.get_stake_amount() - 1)
			);
	
			let proto_with_refs = ProtoFragment {
				references: vec![stake.proto_fragment.get_proto_hash()], 
				..dd.proto_fragment
			};
			
		
			assert_noop!(
				upload(stake.lock.get_link().clamor_account_id, &proto_with_refs),
				Error::<Test>::NotEnoughStaked
			);

		});



	}

	#[test]
	fn upload_should_not_work_if_user_did_not_stake_() {

		new_test_ext().execute_with(|| {

			let dd = DummyData::new();

			let stake = dd.stake;
	
			upload(dd.account_id, &stake.proto_fragment);

			lock_(&stake.lock);
			link_(stake.lock.get_link().clamor_account_id, &stake.lock.get_link().get_link_signature());

			let proto_with_refs = ProtoFragment {
				references: vec![stake.proto_fragment.get_proto_hash()], 
				..dd.proto_fragment
			};
					
			assert_noop!(
				upload(stake.lock.get_link().clamor_account_id, &proto_with_refs), 
				Error::<Test>::StakeNotFound
			);

		});

	}

}


mod patch_tests {

	use super::*;

	fn patch_(signer: <Test as frame_system::Config>::AccountId, patch: &Patch) -> DispatchResult {
		ProtosPallet::patch(
			Origin::signed(signer), 
			patch.proto_fragment.clone().get_proto_hash(), 
			patch.include_cost.map(|cost| Compact::from(cost)), 
			patch.new_references.clone(), 
			patch.new_data.clone()
		)
	}

	#[test]
	fn patch_should_work() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let block_number = System::block_number(); //@sinkingsugar

			let patch = dd.patch;

			upload(dd.account_id, &patch.proto_fragment);

			
			assert_ok!(patch_(dd.account_id, &patch));
			let proto_struct = <Protos<Test>>::get(patch.proto_fragment.get_proto_hash()).unwrap();
			assert_eq!(proto_struct.include_cost, patch.include_cost.map(|cost| Compact::from(cost)));
			assert!(proto_struct.patches.contains(&ProtoPatch {block: block_number, data_hash: patch.get_data_hash(), references: patch.new_references.clone()}));

			let event = <frame_system::Pallet<Test>>::events().pop().expect("Expected at least one EventRecord to be found").event;
        	assert_eq!(
				event, 
				mock::Event::from(pallet_protos::Event::Patched(patch.proto_fragment.get_proto_hash(), patch.get_data_cid()))
			);
				

				
		});
	}

	#[test]
	fn patch_should_not_work_if_user_is_unauthorized() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let patch = dd.patch;

			upload(dd.account_id, &patch.proto_fragment);

			assert_noop!(
				patch_(dd.account_id_second, &patch), 
				Error::<Test>::Unauthorized
			);
		});

			
	}

	#[test]
	fn patch_should_not_work_if_proto_not_found() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();
			let patch = dd.patch;
			assert_noop!(
				patch_(dd.account_id, &patch), 
				Error::<Test>::ProtoNotFound
			);
		});
	}


	#[test]
	fn patch_should_work_if_user_staked_enough() {

		new_test_ext().execute_with(|| {

			let dd = DummyData::new();

			let stake = dd.stake;
	
			upload(dd.account_id, &stake.proto_fragment);

			lock_(&stake.lock);
			link_(stake.lock.get_link().clamor_account_id, &stake.lock.get_link().get_link_signature());

			stake_(
				stake.lock.get_link().clamor_account_id, 
				&stake.proto_fragment,
				&stake.get_stake_amount()
			);


			upload(stake.lock.get_link().clamor_account_id, &dd.proto_fragment);
			
			let patch_with_refs = Patch {
				proto_fragment: dd.proto_fragment,
				include_cost: None,
				new_references: vec![stake.proto_fragment.get_proto_hash()],
				new_data: b"<insert anything here>".to_vec(),
			};
			
			assert_ok!(patch_(stake.lock.get_link().clamor_account_id, &patch_with_refs));

		});




	}
	
	#[test]
	fn patch_should_not_work_if_user_did_not_stake_enough() {

		new_test_ext().execute_with(|| {

			let dd = DummyData::new();

			let stake = dd.stake;
			
	
			upload(dd.account_id, &stake.proto_fragment);

			lock_(&stake.lock);
			link_(stake.lock.get_link().clamor_account_id, &stake.lock.get_link().get_link_signature());

			stake_(
				stake.lock.get_link().clamor_account_id, 
				&stake.proto_fragment,
				&(stake.get_stake_amount() - 1)
			);


			upload(stake.lock.get_link().clamor_account_id, &dd.proto_fragment);
			
			let patch_with_refs = Patch {
				proto_fragment: dd.proto_fragment,
				include_cost: None,
				new_references: vec![stake.proto_fragment.get_proto_hash()],
				new_data: b"<insert anything here>".to_vec(),
			};
			
			assert_noop!(
				patch_(stake.lock.get_link().clamor_account_id, &patch_with_refs),
				Error::<Test>::NotEnoughStaked
			);

		});



	}

	#[test]
	fn patch_should_not_work_if_user_did_not_stake_() {


		new_test_ext().execute_with(|| {

			let dd = DummyData::new();

			let stake = dd.stake;
	
			upload(dd.account_id, &stake.proto_fragment);

			lock_(&stake.lock);
			link_(stake.lock.get_link().clamor_account_id, &stake.lock.get_link().get_link_signature());


			upload(stake.lock.get_link().clamor_account_id, &dd.proto_fragment);
			
			let patch_with_refs = Patch {
				proto_fragment: dd.proto_fragment,
				include_cost: None,
				new_references: vec![stake.proto_fragment.get_proto_hash()],
				new_data: b"<insert anything here>".to_vec(),
			};
				
			assert_noop!(
				patch_(stake.lock.get_link().clamor_account_id, &patch_with_refs),
				Error::<Test>::StakeNotFound
			);

		});

	}

	

	// #[test]
	// fn patch_should_not_work_if_detached() {
	// 	let keystore = KeyStore::new();
	// 	let mut t = new_test_ext();

	// 	t.register_extension(KeystoreExt(Arc::new(keystore)));
	// 	t.execute_with(|| {
	// 		let pair = sp_core::ed25519::Pair::from_string("//Alice", None).unwrap();
	// 		let data = DATA.as_bytes().to_vec();
	// 		initial_upload();

	// 		sp_io::crypto::ecdsa_generate(KEY_TYPE, None);
	// 		let keys = sp_io::crypto::ecdsa_public_keys(KEY_TYPE);

	// 		<EthereumAuthorities<Test>>::mutate(|authorities| {
	// 			authorities.insert(keys.get(0).unwrap().clone());
	// 		});
	// 		assert_ok!(ProtosPallet::detach(
	// 			Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC1)),
	// 			PROTO_HASH,
	// 			SupportedChains::EthereumMainnet,
	// 			pair.to_raw_vec()
	// 		));

	// 		let detach_data = DetachInternalData {
	// 			public: sp_core::ed25519::Public::from_raw(PUBLIC1),
	// 			hash: PROTO_HASH,
	// 			remote_signature: vec![],
	// 			target_account: vec![],
	// 			target_chain: SupportedChains::EthereumGoerli,
	// 			nonce: 1,
	// 		};

	// 		assert_ok!(DetachPallet::internal_finalize_detach(
	// 			Origin::none(),
	// 			detach_data,
	// 			pair.sign(DATA.as_bytes())
	// 		));

	// 		assert_noop!(
	// 			ProtosPallet::patch_(
	// 				Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC1)),
	// 				PROTO_HASH,
	// 				Some(Compact(123)),
	// 				vec![],
	// 				data,
	// 			),
	// 			Error::<Test>::Detached
	// 		);


	// 	});
	// }


}


mod transfer_tests {

	use super::*;

	fn transfer(signer: <Test as frame_system::Config>::AccountId, proto: &ProtoFragment, new_owner: <Test as frame_system::Config>::AccountId) -> DispatchResult {
		ProtosPallet::transfer(Origin::signed(signer), proto.get_proto_hash(), new_owner)
	}

	#[test]
	fn transfer_should_work() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let proto = dd.proto_fragment;

			upload(dd.account_id, &proto);

			assert_ok!(transfer(dd.account_id, &proto, dd.account_id_second));

			assert_eq!(<Protos<Test>>::get(proto.get_proto_hash()).unwrap().owner, ProtoOwner::User(dd.account_id_second));

			assert!(<ProtosByOwner<Test>>::get(ProtoOwner::User(dd.account_id)).unwrap().contains(&proto.get_proto_hash()) == false);
			assert!(<ProtosByOwner<Test>>::get(ProtoOwner::User(dd.account_id_second)).unwrap().contains(&proto.get_proto_hash()));	

			let event = <frame_system::Pallet<Test>>::events().pop().expect("Expected at least one EventRecord to be found").event;
			assert_eq!(event, mock::Event::from(pallet_protos::Event::Transferred(proto.get_proto_hash(), dd.account_id_second)));
			
		});
	}

	#[test]
	fn transfer_should_not_work_if_user_is_unauthorized() {
		new_test_ext().execute_with(|| {

			let dd = DummyData::new();

			let proto = dd.proto_fragment;

			upload(dd.account_id, &proto);

			assert_noop!(
				transfer(dd.account_id_second, &proto, dd.account_id), 
				Error::<Test>::Unauthorized
			);


		});
	}

	#[test]
	fn transfer_should_not_work_if_proto_not_found() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let proto = dd.proto_fragment;

			assert_noop!(
				transfer(dd.account_id, &proto, dd.account_id_second), 
				Error::<Test>::ProtoNotFound
			);
		});
	}



}



mod set_metadata_tests {

	use super::*;


	fn set_metadata(signer: <Test as frame_system::Config>::AccountId, metadata: &Metadata) -> DispatchResult {
		ProtosPallet::set_metadata(
			Origin::signed(signer), 
			metadata.proto_fragment.get_proto_hash(), 
			metadata.metadata_key.clone(), 
			metadata.data.clone()
		)
	}

	#[test]
	fn set_metadata_should_work() {

		new_test_ext().execute_with(|| {
			let key_count = <MetaKeysIndex<Test>>::try_get().unwrap_or_default(); // @sinkingsugar

			let dd = DummyData::new();

			let metadata = dd.metadata;
	
			upload(dd.account_id, &metadata.proto_fragment);
	
			assert_ok!(set_metadata(dd.account_id, &metadata));
			
			let metadata_map = <Protos<Test>>::get(metadata.proto_fragment.get_proto_hash()).unwrap().metadata;
			let metadata_key_index = <MetaKeys<Test>>::get(&metadata.metadata_key).unwrap();
			assert_eq!(metadata_map[&<Compact<u64>>::from(metadata_key_index)], metadata.get_data_hash());
	
			assert_eq!(<MetaKeysIndex<Test>>::get(), key_count + 1); // @sinkingsugar

			let event = <frame_system::Pallet<Test>>::events().pop().expect("Expected at least one EventRecord to be found").event;
			assert_eq!(event, mock::Event::from(pallet_protos::Event::MetadataChanged(metadata.proto_fragment.get_proto_hash(), metadata.metadata_key.clone())));
		});

	}

	#[test]
	fn set_metadata_should_not_work_if_user_is_unauthorized() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();
			let metadata = dd.metadata;
			upload(dd.account_id, &metadata.proto_fragment);
			assert_noop!(
				set_metadata(dd.account_id_second, &metadata), 
				Error::<Test>::Unauthorized
			);
		});
	}

	#[test]
	fn set_metadata_should_not_work_if_proto_not_found() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();
			let metadata = dd.metadata;
			assert_noop!(
				set_metadata(dd.account_id, &metadata), 
				Error::<Test>::ProtoNotFound
			);
		});
	}


}


mod stake_tests {

	use super::*;

	pub fn stake_(signer: <Test as frame_system::Config>::AccountId, proto: &ProtoFragment, stake_amount: &u64) -> DispatchResult {
		ProtosPallet::stake(
			Origin::signed(signer), 
			proto.get_proto_hash(), 
			stake_amount.clone()
		)
	}

	#[test]
	fn stake_should_work() {

		new_test_ext().execute_with(|| {

			let dd = DummyData::new();

			let stake = dd.stake;
	
			upload(dd.account_id, &stake.proto_fragment);

			let frag_staked = <pallet_frag::FragUsage<Test>>::get(stake.lock.get_link().clamor_account_id).unwrap_or_default();

			let current_block_number = System::block_number(); //@sinkingsugar

			lock_(&stake.lock);
			link_(stake.lock.get_link().clamor_account_id, &stake.lock.get_link().get_link_signature());	
			assert_ok!(
				stake_(
					stake.lock.get_link().clamor_account_id, 
					&stake.proto_fragment, 
					&stake.get_stake_amount()
				)
			);

			assert_eq!(
				<pallet_frag::FragUsage<Test>>::get(stake.lock.get_link().clamor_account_id).unwrap(), 
				frag_staked.saturating_add(stake.get_stake_amount())
			);

			assert_eq!(
				<ProtoStakes<Test>>::get(stake.proto_fragment.get_proto_hash(), dd.account_id).unwrap(), 
				(stake.get_stake_amount(), current_block_number)
			);
			assert!(<AccountStakes<Test>>::get(dd.account_id).unwrap().contains(&stake.proto_fragment.get_proto_hash()));

			let event = <frame_system::Pallet<Test>>::events().pop().expect("Expected at least one EventRecord to be found").event;
			assert_eq!(
				event, 
				mock::Event::from(
					pallet_protos::Event::Staked(
						stake.proto_fragment.get_proto_hash(), 
						dd.account_id, 
						stake.get_stake_amount()
					)
				)
			);

		});

	}

	#[test]
	fn stake_should_not_work_if_proto_not_found() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();
			let stake = dd.stake;

			lock_(&stake.lock);
			link_(stake.lock.get_link().clamor_account_id, &stake.lock.get_link().get_link_signature());

			assert_noop!(
				stake_(
					stake.lock.get_link().clamor_account_id, 
					&stake.proto_fragment, 
					&stake.get_stake_amount()
				), 
				Error::<Test>::ProtoNotFound
			);
		});
	}


	#[test]
	fn stake_should_work_if_user_has_sufficient_balance() {

		new_test_ext().execute_with(|| {

			let dd = DummyData::new();

			let stake = dd.stake;

			upload(dd.account_id, &stake.proto_fragment);

			lock_(&stake.lock);
			link_(stake.lock.get_link().clamor_account_id, &stake.lock.get_link().get_link_signature());

			let frag_locked = <pallet_frag::EthLockedFrag<Test>>::get(stake.lock.get_link().get_ethereum_account_id()).unwrap()
							.amount;
			let frag_staked = <pallet_frag::FragUsage<Test>>::get(stake.lock.get_link().clamor_account_id).unwrap_or_default();
			let balance = frag_locked - frag_staked;

			assert_ok!(
				stake_(
					stake.lock.get_link().clamor_account_id, 
					&stake.proto_fragment, 
					&balance
				)
			);

		});

	}

	#[test]
	fn stake_should_not_work_if_user_does_has_insufficient_balance() {

		new_test_ext().execute_with(|| {

			let dd = DummyData::new();

			let stake = dd.stake;

			upload(dd.account_id, &stake.proto_fragment);

			lock_(&stake.lock);
			link_(stake.lock.get_link().clamor_account_id, &stake.lock.get_link().get_link_signature());

			let frag_locked = <pallet_frag::EthLockedFrag<Test>>::get(stake.lock.get_link().get_ethereum_account_id()).unwrap()
							.amount;
			let frag_staked = <pallet_frag::FragUsage<Test>>::get(stake.lock.get_link().clamor_account_id).unwrap_or_default();
			let balance = frag_locked - frag_staked;

			assert_noop!(
				stake_(
					stake.lock.get_link().clamor_account_id, 
					&stake.proto_fragment,
					&(balance - 1)
				),
				Error::<Test>::InsufficientBalance
			);

		});

	}

}



mod unstake_tests {

	use super::*;

	fn unstake_(signer: <Test as frame_system::Config>::AccountId, proto: &ProtoFragment) -> DispatchResult {
		ProtosPallet::unstake(Origin::signed(signer), proto.get_proto_hash())
	}

	#[test]
	fn unstake_should_work() {
		new_test_ext().execute_with(|| {

			let dd = DummyData::new();

			let stake = dd.stake;
	
			upload(dd.account_id, &stake.proto_fragment);

			let frag_staked = <pallet_frag::FragUsage<Test>>::get(stake.lock.get_link().clamor_account_id).unwrap_or_default();

			let current_block_number = System::block_number(); //@sinkingsugar

			lock_(&stake.lock);
			link_(stake.lock.get_link().clamor_account_id, &stake.lock.get_link().get_link_signature());	
			stake_(
				stake.lock.get_link().clamor_account_id, 
				&stake.proto_fragment,
				&stake.get_stake_amount(),
			);


			let lockup_period = <<Test as pallet_protos::Config>::StakeLockupPeriod as Get<u64>>::get(); // .saturated_into(); - @sinkingsugar: what is `.saturated_into())`

			run_to_block(current_block_number + lockup_period + 1); // @sinkingsugar should I check if this works in a seperate function
			
			assert_ok!(unstake_(stake.lock.get_link().clamor_account_id, &stake.proto_fragment));

			assert_eq!(
				<pallet_frag::FragUsage<Test>>::get(stake.lock.get_link().clamor_account_id).unwrap(), 
				frag_staked
			);
	
			assert!(
				<ProtoStakes<Test>>::contains_key(
					stake.proto_fragment.get_proto_hash(), stake.lock.get_link().clamor_account_id
				) == false
			);

			assert!(
				<AccountStakes<Test>>::get(stake.lock.get_link().clamor_account_id).unwrap().contains(
					&stake.proto_fragment.get_proto_hash()
				) == false
			);
	
			let event = <frame_system::Pallet<Test>>::events().pop().expect("Expected at least one EventRecord to be found").event;
			assert_eq!(
				event, 
				mock::Event::from(
					pallet_protos::Event::Unstaked(
						stake.proto_fragment.get_proto_hash(), 
						stake.lock.get_link().clamor_account_id, 
						stake.get_stake_amount()
					)
				)
			);

		});

	}

	#[test]
	fn unstake_should_not_work_if_proto_not_found() {
		new_test_ext().execute_with(|| {

			let dd = DummyData::new();
			let stake = dd.stake;

			lock_(&stake.lock);
			link_(stake.lock.get_link().clamor_account_id, &stake.lock.get_link().get_link_signature());

			
			assert_noop!(
				unstake_(stake.lock.get_link().clamor_account_id, &stake.proto_fragment), 
				Error::<Test>::ProtoNotFound
			);

		});
	}

	fn unstake_should_not_work_if_lockup_period_has_not_ended() {
		new_test_ext().execute_with(|| {

			let dd = DummyData::new();

			let stake = dd.stake;
	
			upload(dd.account_id, &stake.proto_fragment);

			let current_block_number = System::block_number(); //@sinkingsugar

			lock_(&stake.lock);
			link_(stake.lock.get_link().clamor_account_id, &stake.lock.get_link().get_link_signature());		
			stake_(
				stake.lock.get_link().clamor_account_id, 
				&stake.proto_fragment,
				&stake.get_stake_amount()
			);

			let lockup_period = <<Test as pallet_protos::Config>::StakeLockupPeriod as Get<u64>>::get(); // .saturated_into(); - @sinkingsugar: what is `.saturated_into())`

			run_to_block(current_block_number + lockup_period);
			
			assert_noop!(
				unstake_(stake.lock.get_link().clamor_account_id, &stake.proto_fragment),
				Error::<Test>::StakeLocked
			);

		});

	}



	



}



