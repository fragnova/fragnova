use crate as pallet_fragments;
use crate::mock;

use crate::*;

use crate::dummy_data::*;

use crate::mock::*;

use codec::Encode;
use frame_support::{assert_noop, assert_ok};
use protos::categories::{Categories, TextCategories};
use protos::permissions::FragmentPerms;
use sp_io::hashing::blake2_128;


use copied_from_pallet_protos::upload as upload;
mod copied_from_pallet_protos {
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
}



use create_tests::create as create;
mod create_tests {
	use super::*;

	pub fn create(
		signer: <Test as frame_system::Config>::AccountId,
		definition: &Definition,
	) -> DispatchResult {
		FragmentsPallet::create(
			Origin::signed(signer),
			definition.proto_fragment.get_proto_hash(),
			definition.metadata.clone(),
			definition.permissions,
			definition.unique.clone(),
			definition.max_supply,
		)
	}


	#[test]
	fn create_should_work() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let current_block_number = System::block_number(); 

			let definition = dd.definition;

			upload(dd.account_id, &definition.proto_fragment);

			assert_ok!(create(dd.account_id, &definition));

			// TODO - what does `T::Currency::reserve()` do in the `create` extrinsic

			let correct_definition_struct = FragmentDefinition {
				proto_hash: definition.proto_fragment.get_proto_hash(),
				metadata: definition.metadata.clone(),
				permissions: definition.permissions.clone(),
				unique: definition.unique.clone(),
				max_supply: definition.max_supply.map(|max_supply| Compact::from(max_supply)),
				creator: dd.account_id,
				created_at: current_block_number
			};

			assert_eq!(
				<Definitions<Test>>::get(&definition.get_definition_id()).unwrap(), 
				correct_definition_struct
			);

			assert!(
				<Proto2Fragments<Test>>::get(&definition.proto_fragment.get_proto_hash()).unwrap()
				.contains(&definition.get_definition_id())
			);

			let event = System::events().pop().expect("Expected at least one EventRecord to be found").event;
        	assert_eq!(
				event, 
				mock::Event::from(
					pallet_fragments::Event::DefinitionCreated(definition.get_definition_id())
				)
			);

		});
	}


	#[test]
	fn create_should_not_work_if_fragment_definition_already_exists() {
		
		new_test_ext().execute_with(|| {

			let dd = DummyData::new();

			let definition = dd.definition;

			upload(dd.account_id, &definition.proto_fragment);

			create(dd.account_id, &definition);

			assert_noop!(
				create(dd.account_id, &definition),
				Error::<Test>::AlreadyExist
			);

		});

	}

	#[test]
	fn create_should_not_work_if_proto_does_not_exist() {

		new_test_ext().execute_with(|| {

			let dd = DummyData::new();

			let definition = dd.definition;

			assert_noop!(
				create(dd.account_id, &definition),
				Error::<Test>::ProtoNotFound
			);

		});

	}

	#[test]
	fn create_should_not_work_if_user_does_not_own_proto() {

		new_test_ext().execute_with(|| {

			let dd = DummyData::new();

			let definition = dd.definition;

			upload(dd.account_id, &definition.proto_fragment);

			assert_noop!(
				create(dd.account_id_second, &definition),
				Error::<Test>::NoPermission
			);

		});

	}

	#[test]
	fn create_should_not_work_if_proto_is_detached() {
		todo!()
	}


}

use publish_tests::publish_ as publish_;
mod publish_tests {
	use super::*;

	pub fn publish_(
		signer: <Test as frame_system::Config>::AccountId,
		publish: &Publish,
	) -> DispatchResult {
		FragmentsPallet::publish(
			Origin::signed(signer),
			publish.definition.get_definition_id(),
			publish.price,
			publish.quantity,
			publish.expires,
			publish.amount,
		)
	}

	#[test]
	fn publish_should_work() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let publish = dd.publish;

			upload(dd.account_id, &publish.definition.proto_fragment);
			create(dd.account_id, &publish.definition);

			assert_ok!(publish_(dd.account_id, &publish));

			let correct_publishing_data_struct = PublishingData {
				price: Compact::from(publish.price),
				units_left: publish.quantity.map(|quantity| Compact::from(quantity)),
				expiration: publish.expires,
				amount: publish.amount.map(|amount| Compact::from(amount)),
			};

			assert_eq!(
				<Publishing<Test>>::get(&publish.definition.get_definition_id()).unwrap(), 
				correct_publishing_data_struct
			);

			let event = System::events().pop().expect("Expected at least one EventRecord to be found").event;
        	assert_eq!(
				event, 
				mock::Event::from(
					pallet_fragments::Event::Publishing(publish.definition.get_definition_id())
				)
			);

		});

	}

	#[test]
	fn publish_should_not_work_if_user_does_not_own_the_fragment_definition() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let publish = dd.publish;

			upload(dd.account_id, &publish.definition.proto_fragment);
			create(dd.account_id, &publish.definition);

			assert_noop!(
				publish_(dd.account_id_second, &publish),
				Error::<Test>::NoPermission
			);

		});
	}

	#[test]
	fn publish_should_not_work_if_fragment_definition_does_not_exist() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let publish = dd.publish;

			assert_noop!(
				publish_(dd.account_id, &publish),
				Error::<Test>::NotFound
			);

		});
	}

	#[test]
	fn publish_should_not_work_if_fragment_definition_is_currently_published() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let publish = dd.publish;

			upload(dd.account_id, &publish.definition.proto_fragment);
			create(dd.account_id, &publish.definition);
			publish_(dd.account_id, &publish);

			assert_noop!(
				publish_(dd.account_id, &publish),
				Error::<Test>::SaleAlreadyOpen
			);

		});
	}

	#[test]
	fn publish_should_not_work_if_the_quantity_parameter_is_none_but_the_fragment_definition_has_a_max_supply() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let publish_with_max_supply = dd.publish_with_max_supply;

			upload(dd.account_id, &publish_with_max_supply.definition.proto_fragment);
			create(dd.account_id, &publish_with_max_supply.definition);

			assert_noop!(
				publish_(
					dd.account_id, 
					&Publish { 
						quantity: None,
						..publish_with_max_supply
					}
				),
				Error::<Test>::ParamsNotValid
			);

		});	
	}

	#[test]
	fn publish_should_work_if_the_quantity_to_publish_is_lesser_than_or_equal_to_the_max_supply_of_the_fragment_definition() {

		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let publish_with_max_supply = dd.publish_with_max_supply;

			upload(dd.account_id, &publish_with_max_supply.definition.proto_fragment);
			create(dd.account_id, &publish_with_max_supply.definition);

			assert_ok!(
				publish_(
					dd.account_id, 
					&Publish { 
						quantity: publish_with_max_supply.definition.max_supply,
						..publish_with_max_supply
					}
				)
			);

		});		

	}

	#[test]
	fn publish_should_not_work_if_the_quantity_to_publish_is_greater_than_the_max_supply_of_the_fragment_definition() {

		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let publish_with_max_supply = dd.publish_with_max_supply;

			upload(dd.account_id, &publish_with_max_supply.definition.proto_fragment);
			create(dd.account_id, &publish_with_max_supply.definition);

			assert_noop!(
				publish_(
					dd.account_id, 
					&Publish { 
						quantity: publish_with_max_supply.definition.max_supply.map(|max_supply| max_supply + 1),
						..publish_with_max_supply
					}
				),
				Error::<Test>::MaxSupplyReached
			);

		});		

	}

	#[test]
	fn publish_should_not_work_if_proto_is_detached() {
		todo!()
	}


}

mod unpublish_tests {
	
	use super::*;

	pub fn unpublish_(
		signer: <Test as frame_system::Config>::AccountId,
		definition: &Definition,
	) -> DispatchResult {
		FragmentsPallet::unpublish(
			Origin::signed(signer), 
			definition.get_definition_id()
		)
	}

	#[test]
	fn unpublish_should_work() {

		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let publish = dd.publish;

			upload(dd.account_id, &publish.definition.proto_fragment);
			create(dd.account_id, &publish.definition);
			publish_(dd.account_id, &publish);

			assert_ok!(unpublish_(dd.account_id, &publish.definition));

			assert_eq!(
				<Publishing<Test>>::contains_key(&publish.definition.get_definition_id()),
				false
			);

			let event = System::events().pop().expect("Expected at least one EventRecord to be found").event;
        	assert_eq!(
				event, 
				mock::Event::from(
					pallet_fragments::Event::Unpublishing(publish.definition.get_definition_id())
				)
			);

		});

	}

	#[test]
	fn upublish_should_not_work_if_user_does_not_own_the_fragment_definition() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let publish = dd.publish;

			upload(dd.account_id, &publish.definition.proto_fragment);
			create(dd.account_id, &publish.definition);
			publish_(dd.account_id, &publish);

			assert_noop!(
				unpublish_(dd.account_id_second, &publish.definition),
				Error::<Test>::NoPermission
			);

		});
		
	}

	#[test]
	fn unpublish_should_not_work_if_fragment_definition_does_not_exist() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let publish = dd.publish;

			assert_noop!(
				unpublish_(dd.account_id, &publish.definition),
				Error::<Test>::NotFound
			);

		});

	}

	#[test]
	fn unpublish_should_not_work_if_fragment_definition_is_not_published() {

		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let publish = dd.publish;

			upload(dd.account_id, &publish.definition.proto_fragment);
			create(dd.account_id, &publish.definition);

			assert_noop!(
				unpublish_(dd.account_id_second, &publish.definition),

				// this error does not exist yet @sinkingsugar ???
				Error::<Test>::SystematicFailure
			);

		});
	}

	#[test]
	fn unpublish_should_not_work_if_proto_is_detached() {
		todo!()
	}

}



use mint_tests::mint_ as mint_;
mod mint_tests {
	use super::*;

	// pub fn mint_(
	// 	signer: <Test as frame_system::Config>::AccountId,
	// 	definition: &Definition,
	// 	options: &FragmentBuyOptions,
	// 	amount: &Option<u64>,
	// ) -> DispatchResult {
	// 	FragmentsPallet::mint(
	// 		Origin::signed(signer),
	// 		definition.get_definition_id(), 
	// 		options.clone(), 
	// 		amount.clone()
	// 	)
	// }

	pub fn mint_(
		signer: <Test as frame_system::Config>::AccountId,
		mint: &Mint,
	) -> DispatchResult {
		FragmentsPallet::mint(
			Origin::signed(signer), 
			mint.definition.get_definition_id(), 
			mint.buy_options.clone(),
			mint.amount.clone()
		)
	}

	#[test]
	fn mint_should_work() {
		new_test_ext().execute_with(|| {

			let dd = DummyData::new();

			let current_block_number = System::block_number(); 

			let mint = dd.mint_non_unique;

			upload(dd.account_id, &mint.definition.proto_fragment);
			create(dd.account_id, &mint.definition);

			assert_ok!(mint_(dd.account_id, &mint));

			let correct_fragment_instance_struct = FragmentInstance {
				permissions: mint.definition.permissions,
				created_at: current_block_number,
				custom_data: None,
				expiring_at: None,
				amount: mint.amount.map(|amount| Compact::from(amount)), 
			};

			let quantity = match mint.buy_options {
				FragmentBuyOptions::Quantity(amount) => u64::from(amount),
				_ => 1u64,
			};
			

			for edition_id in 0..quantity {

				assert_eq!(
					<Fragments<Test>>::get(
						(mint.definition.get_definition_id(), edition_id, 1)
					).unwrap(),
					correct_fragment_instance_struct
				);

				assert_eq!(
					<CopiesCount<Test>>::get(
						(mint.definition.get_definition_id(), edition_id)
					).unwrap(),
					Compact(1)
				);

				assert!(
					<Inventory<Test>>::get(
						dd.account_id, mint.definition.get_definition_id()
					).unwrap()
					.contains(&(Compact(edition_id), Compact(1)))
				);

				assert!(
					<Owners<Test>>::get(
						mint.definition.get_definition_id(), dd.account_id
					)
					.unwrap()
					.contains(&(Compact(edition_id), Compact(1)))
				);

				let event = System::events()[edition_id as usize].event.clone();
				assert_eq!(
					event, 
					mock::Event::from(
						pallet_fragments::Event::InventoryAdded(
							dd.account_id,
							mint.definition.get_definition_id(),
							(edition_id, 1)
						)
					)
				);


			}

			assert_eq!(
				<EditionsCount<Test>>::get(mint.definition.get_definition_id()).unwrap(), 
				Compact(quantity)
			);


		});
	}

	#[test]
	fn mint_should_not_work_if_the_user_is_not_the_owner_of_the_fragment_definition() {

		new_test_ext().execute_with(|| {

			let dd = DummyData::new();

			let mint = dd.mint_non_unique;

			upload(dd.account_id, &mint.definition.proto_fragment);
			create(dd.account_id, &mint.definition);

			assert_noop!(
				mint_(dd.account_id_second, &mint),
				Error::<Test>::NoPermission
			);

		});
		
	}

	#[test]
	fn mint_should_not_work_if_fragment_definition_does_not_exist() {
		new_test_ext().execute_with(|| {

			let dd = DummyData::new();

			let mint = dd.mint_non_unique;

			upload(dd.account_id, &mint.definition.proto_fragment);

			assert_noop!(
				mint_(dd.account_id, &mint),
				Error::<Test>::NotFound
			);

		});
	}

	#[test]
	fn mint_should_work_if_the_quantity_to_create_is_lesser_than_or_equal_to_the_max_supply_of_the_fragment_definition() {

		new_test_ext().execute_with(|| {

			let dd = DummyData::new();

			let mint = dd.mint_non_unique;

			assert!(mint.definition.unique.is_none()); // we use non-unique because we can create multiple instances in a single extrinsic call (basically we're lazy)
			assert!(mint.definition.max_supply.is_some()); // max supply exists

			upload(dd.account_id, &mint.definition.proto_fragment);
			create(dd.account_id, &mint.definition);

			assert_ok!(
				mint_(
					dd.account_id, 
					&Mint {
						buy_options: FragmentBuyOptions::Quantity(mint.definition.max_supply.unwrap()),
						..mint
					}
				)
			);		


		});
		
	}

	// #[test] // TODO Uncomment this later
	fn mint_should_not_work_if_the_quantity_to_create_is_greater_than_the_max_supply_of_the_fragment_definition() {
		new_test_ext().execute_with(|| {

			let dd = DummyData::new();

			let mint = dd.mint_non_unique;

			assert!(mint.definition.unique.is_none()); // we use non-unique because we can create multiple instances in a single extrinsic call (basically we're lazy)
			assert!(mint.definition.max_supply.is_some()); // max supply exists

			upload(dd.account_id, &mint.definition.proto_fragment);
			create(dd.account_id, &mint.definition);

			assert_noop!(
				mint_(
					dd.account_id, 
					&Mint {
						buy_options: FragmentBuyOptions::Quantity(mint.definition.max_supply.unwrap() + 1),
						..mint
					}
				),
				Error::<Test>::MaxSupplyReached
			);	


		});

	}

	#[test]
	fn mint_should_not_work_if_the_options_parameter_is_unique_but_the_the_fragment_definition_is_not_unique() {

		new_test_ext().execute_with(|| {

			let dd = DummyData::new();

			let mint_non_unique = dd.mint_non_unique; // fragment definition is not unique

			upload(dd.account_id, &mint_non_unique.definition.proto_fragment);
			create(dd.account_id, &mint_non_unique.definition);

			assert_noop!(
				mint_(
					dd.account_id, 
					&Mint {
						buy_options: FragmentBuyOptions::UniqueData(b"I dati".to_vec()), // options parameter is unique
						..mint_non_unique
					}
				),
				// this error does not exist yet @sinkingsugar ???
				Error::<Test>::SystematicFailure
			);


		});
		

	}

	#[test]
	fn mint_should_not_work_if_the_options_parameter_is_not_unique_but_the_the_fragment_definition_is_unique() {

		new_test_ext().execute_with(|| {

			let dd = DummyData::new();

			let mint_unique = dd.mint_unique; // fragment definition is unique

			upload(dd.account_id, &mint_unique.definition.proto_fragment);
			create(dd.account_id, &mint_unique.definition);

			assert_noop!(
				mint_(
					dd.account_id_second, 
					&Mint {
						buy_options: FragmentBuyOptions::Quantity(123456789), // options parameter is not unique
						..mint_unique
					}
				),
				// this error does not exist yet @sinkingsugar ???
				Error::<Test>::SystematicFailure
			);

		});

	}


	#[test]
	fn mint_should_not_work_if_fragment_instance_was_already_created_with_the_same_unique_data() {

		new_test_ext().execute_with(|| {

			let dd = DummyData::new();

			let mint_unique = dd.mint_unique; 

			upload(dd.account_id, &mint_unique.definition.proto_fragment);
			create(dd.account_id, &mint_unique.definition);

			mint_(
				dd.account_id, 
				&mint_unique
			);

			assert_noop!(
				mint_(
					dd.account_id, 
					&mint_unique
				),
				// this error does not exist yet @sinkingsugar ???
				Error::<Test>::SystematicFailure
			);

		});			

	}

	#[test]
	fn mint_should_not_work_if_proto_is_detached() {
		todo!()
	}

	
}


mod buy_tests {
	use super::*;

	fn buy_(
		signer: <Test as frame_system::Config>::AccountId,
		buy: &Buy,
	) -> DispatchResult {
		FragmentsPallet::buy(
			Origin::signed(signer), 
			buy.publish.definition.get_definition_id(), 
			buy.buy_options.clone(),
		)
	}


	#[test]
	fn buy_should_work() {

		new_test_ext().execute_with(|| {

			let dd = DummyData::new();

			let current_block_number = System::block_number();

			let buy = dd.buy_non_unique;

			upload(dd.account_id, &buy.publish.definition.proto_fragment);
			create(dd.account_id, &buy.publish.definition);
			
			publish_(dd.account_id, &buy.publish);

			let quantity = match buy.buy_options {
				FragmentBuyOptions::Quantity(amount) => u64::from(amount),
				_ => 1u64,
			};

			// TODO Transfer
			<BalancesPallet as Currency<<Test as frame_system::Config>::AccountId>>::deposit_creating(
				&dd.account_id_second, 
				buy.publish.price.saturating_mul(quantity as u128)
			);

			assert_ok!(buy_(dd.account_id_second, &buy));

			let correct_fragment_instance_struct = FragmentInstance {
				permissions: buy.publish.definition.permissions,
				created_at: current_block_number,
				custom_data: None,
				expiring_at: buy.publish.expires, 
				amount: buy.publish.amount.map(|amount| Compact::from(amount)), 
			};
			
			for edition_id in 0..quantity {

				assert_eq!(
					<Fragments<Test>>::get(
						(buy.publish.definition.get_definition_id(), edition_id, 1)
					).unwrap(),
					correct_fragment_instance_struct
				);

				assert_eq!(
					<CopiesCount<Test>>::get(
						(buy.publish.definition.get_definition_id(), edition_id)
					).unwrap(),
					Compact(1)
				);

				assert!(
					<Inventory<Test>>::get(
						dd.account_id_second, buy.publish.definition.get_definition_id()
					).unwrap()
					.contains(&(Compact(edition_id), Compact(1)))
				);

				assert!(
					<Owners<Test>>::get(
						buy.publish.definition.get_definition_id(), dd.account_id_second
					)
					.unwrap()
					.contains(&(Compact(edition_id), Compact(1)))
				);

				let event = System::events()[edition_id as usize].event.clone();
				assert_eq!(
					event, 
					mock::Event::from(
						pallet_fragments::Event::InventoryAdded(
							dd.account_id,
							buy.publish.definition.get_definition_id(),
							(edition_id, 1)
						)
					)
				);


			}

			assert_eq!(
				<EditionsCount<Test>>::get(buy.publish.definition.get_definition_id()).unwrap(), 
				Compact(quantity)
			);




		});

	}

	#[test]
	fn buy_should_not_work_if_the_fragment_definition_is_not_published() {

		new_test_ext().execute_with(|| {

			let dd = DummyData::new();

			let buy = dd.buy_non_unique;

			upload(dd.account_id, &buy.publish.definition.proto_fragment);
			create(dd.account_id, &buy.publish.definition);

			assert_noop!(
				buy_(dd.account_id_second, &buy),
				Error::<Test>::NotFound,
			);
			

		});

	}

	#[test]
	fn buy_should_work_if_user_has_sufficient_balance() {

		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let buy = dd.buy_non_unique;

			upload(dd.account_id, &buy.publish.definition.proto_fragment);
			create(dd.account_id, &buy.publish.definition);
			publish_(dd.account_id, &buy.publish);

			// TODO Transfer

			assert_ok!(buy_(dd.account_id_second, &buy));



		});

	}

	#[test]
	fn buy_should_not_work_if_user_has_insufficient_balance() {

		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let buy = dd.buy_non_unique;

			upload(dd.account_id, &buy.publish.definition.proto_fragment);
			create(dd.account_id, &buy.publish.definition);
			publish_(dd.account_id, &buy.publish);

			// TODO Transfer 

			assert_noop!(
				buy_(dd.account_id_second, &buy),
				Error::<Test>::InsufficientBalance
			);



		});
	}

	#[test]
	fn buy_should_work_if_the_quantity_to_create_is_lesser_than_or_equal_to_the_max_supply_of_the_fragment_definition() {

		new_test_ext().execute_with(|| {

			let dd = DummyData::new();

			let buy = dd.buy_non_unique;

			assert!(buy.publish.definition.unique.is_none()); // we use non-unique because we can create multiple instances in a single extrinsic call (basically we're lazy)
			assert!(buy.publish.definition.max_supply.is_some()); // max supply exists

			upload(dd.account_id, &buy.publish.definition.proto_fragment);
			create(dd.account_id, &buy.publish.definition);
			publish_(dd.account_id, &buy.publish);


			// TODO Transfer

			assert_ok!(
				buy_(
					dd.account_id_second, 
					&Buy {
						buy_options: FragmentBuyOptions::Quantity(buy.publish.definition.max_supply.unwrap()),
						..buy
					}
				)
			);		


		});
	}

	#[test]
	fn buy_should_not_work_if_the_quantity_to_create_is_greater_than_the_max_supply_of_the_fragment_definition() {

		new_test_ext().execute_with(|| {

			let dd = DummyData::new();

			let buy = dd.buy_non_unique;

			assert!(buy.publish.definition.unique.is_none()); // we use non-unique because we can create multiple instances in a single extrinsic call (basically we're lazy)
			assert!(buy.publish.definition.max_supply.is_some()); // max supply exists

			upload(dd.account_id, &buy.publish.definition.proto_fragment);
			create(dd.account_id, &buy.publish.definition);
			publish_(dd.account_id, &buy.publish);


			// TODO Transfer

			assert_noop!(
				buy_(
					dd.account_id_second, 
					&Buy {
						buy_options: FragmentBuyOptions::Quantity(buy.publish.definition.max_supply.unwrap() + 1),
						..buy
					}
				),
				Error::<Test>::MaxSupplyReached
			);	
			
		});

	}

	#[test]
	fn buy_should_not_work_if_the_options_parameter_is_unique_but_the_the_fragment_definition_is_not_unique() {

		new_test_ext().execute_with(|| {

			let dd = DummyData::new();

			let buy_non_unique = dd.buy_non_unique; // fragment definition is not unique

			upload(dd.account_id, &buy_non_unique.publish.definition.proto_fragment);
			create(dd.account_id, &buy_non_unique.publish.definition);
			publish_(dd.account_id, &buy_non_unique.publish);

			// TODO Transfer

			assert_noop!(
				buy_(
					dd.account_id_second, 
					&Buy {
						buy_options: FragmentBuyOptions::UniqueData(b"I dati".to_vec()), // options parameter is unique
						..buy_non_unique
					}
				),
				// this error does not exist yet @sinkingsugar ???
				Error::<Test>::SystematicFailure
			);

		});
	}

	#[test]
	fn buy_should_not_work_if_the_options_parameter_is_not_unique_but_the_the_fragment_definition_is_unique() {

		new_test_ext().execute_with(|| {

			let dd = DummyData::new();

			let buy_unique = dd.buy_unique; // fragment definition is unique

			upload(dd.account_id, &buy_unique.publish.definition.proto_fragment);
			create(dd.account_id, &buy_unique.publish.definition);

			// TODO Transfer

			assert_noop!(
				buy_(
					dd.account_id_second, 
					&Buy {
						buy_options: FragmentBuyOptions::Quantity(123456789), // options parameter is not unique
						..buy_unique
					}
				),
				// this error does not exist yet @sinkingsugar ???
				Error::<Test>::SystematicFailure
			);

		});
	}

	// #[test] // TODO
	fn buy_should_work_if_the_sale_has_not_expired() {
		new_test_ext().execute_with(|| {
			
			let dd = DummyData::new();

			let buy = dd.buy_non_unique;

			assert!(buy.publish.expires.is_some()); // sale must have an expiration

			upload(dd.account_id, &buy.publish.definition.proto_fragment);
			create(dd.account_id, &buy.publish.definition);
			publish_(dd.account_id, &buy.publish);

			// TODO Transfer

			run_to_block(buy.publish.expires.unwrap() - 1);

			assert_ok!(buy_(dd.account_id_second, &buy));


		});
		
	}
	// #[test] // TODO
	fn buy_should_not_work_if_the_sale_has_expired() {
		new_test_ext().execute_with(|| {
			
			let dd = DummyData::new();

			let buy = dd.buy_non_unique;

			assert!(buy.publish.expires.is_some()); // sale must have an expiration

			upload(dd.account_id, &buy.publish.definition.proto_fragment);
			create(dd.account_id, &buy.publish.definition);
			publish_(dd.account_id, &buy.publish);

			// TODO Transfer

			run_to_block(buy.publish.expires.unwrap());

			assert_noop!(
				buy_(dd.account_id_second, &buy),
				Error::<Test>::Expired,
			);


		});

	}

	// #[test] // TODO
	fn buy_should_not_work_if_fragment_instance_was_already_created_with_the_same_unique_data() {

		new_test_ext().execute_with(|| {

			let dd = DummyData::new();

			let buy_unique = dd.buy_unique; 

			upload(dd.account_id, &buy_unique.publish.definition.proto_fragment);
			create(dd.account_id, &buy_unique.publish.definition);

			buy_(
				dd.account_id_second, 
				&buy_unique
			);

			assert_noop!(
				buy_(
					dd.account_id_second, 
					&buy_unique
				),
				// this error does not exist yet @sinkingsugar ???
				Error::<Test>::SystematicFailure
			);

		});		
		
	}

	#[test]
	fn buy_should_not_work_if_proto_is_detached() {
		todo!()
	}


}

mod give_tests {
	use super::*;

	pub fn give_(
		// Review whether these many parameters are appropriate/needed @karan
		signer: <Test as frame_system::Config>::AccountId,
		give: &Give
	) -> DispatchResult {
		FragmentsPallet::give(
			Origin::signed(signer),
			give.mint.definition.get_definition_id(),
			give.edition_id,
			give.copy_id,
			give.to,
			give.new_permissions,
			give.expiration,
		)
	}


	#[test]
	fn give_should_work_if_the_fragment_instance_does_not_have_copy_permissions() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let give = dd.give_no_copy_perms;

			upload(dd.account_id, &give.mint.definition.proto_fragment);
			create(dd.account_id, &give.mint.definition);
			mint_(dd.account_id, &give.mint);

			assert_ok!(give_(dd.account_id, &give));

			assert_eq!(
				<Owners<Test>>::get(
					give.mint.definition.get_definition_id(), dd.account_id
				).unwrap().contains(
					&(Compact(give.edition_id), Compact(give.copy_id))
				),
				false
			);
			assert_eq!(
				<Inventory<Test>>::get(
					dd.account_id, give.mint.definition.get_definition_id()
				).unwrap().contains(
					&(Compact(give.edition_id), Compact(give.copy_id))
				),
				false
			);

			assert_eq!(
				<Owners<Test>>::get(
					give.mint.definition.get_definition_id(), give.to
				).unwrap().contains(
					&(Compact(give.edition_id), Compact(give.copy_id))
				),
				true
			);
			assert_eq!(
				<Inventory<Test>>::get(
					give.to, give.mint.definition.get_definition_id()
				).unwrap().contains(
					&(Compact(give.edition_id), Compact(give.copy_id))
				),
				true
			);

			assert_eq!(
				<Fragments<Test>>::get(
					(give.mint.definition.get_definition_id(), give.edition_id, give.copy_id + 1)
				).unwrap()
				.permissions,
				give.new_permissions.unwrap()
			);
			
			let event = System::events().pop().expect("Expected at least one EventRecord to be found").event;
        	assert_eq!(
				event, 
				mock::Event::from(
					pallet_fragments::Event::InventoryAdded(
						give.to,
						give.mint.definition.get_definition_id(), 
						(give.edition_id, give.copy_id)
					)
				)
			);

			let event = System::events().pop().expect("Expected at least one EventRecord to be found").event;
        	assert_eq!(
				event, 
				mock::Event::from(
					pallet_fragments::Event::InventoryRemoved(
						dd.account_id,
						give.mint.definition.get_definition_id(), 
						(give.edition_id, give.copy_id)
					)
				)
			);



		});
	}


	#[test]
	fn give_should_work_if_the_fragment_instance_has_copy_permissions() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let give = dd.give_copy_perms;

			upload(dd.account_id, &give.mint.definition.proto_fragment);
			create(dd.account_id, &give.mint.definition);
			mint_(dd.account_id, &give.mint);

			assert_ok!(give_(dd.account_id, &give));

			assert_eq!(
				<Owners<Test>>::get(
					give.mint.definition.get_definition_id(), dd.account_id
				).unwrap().contains(
					&(Compact(give.edition_id), Compact(give.copy_id))
				),
				true
			);
			assert_eq!(
				<Inventory<Test>>::get(
					dd.account_id, give.mint.definition.get_definition_id()
				).unwrap().contains(
					&(Compact(give.edition_id), Compact(give.copy_id))
				),
				true
			);

			assert_eq!(
				<Owners<Test>>::get(
					give.mint.definition.get_definition_id(), give.to
				).unwrap().contains(
					&(Compact(give.edition_id), Compact(give.copy_id + 1))
				),
				true
			);
			assert_eq!(
				<Inventory<Test>>::get(
					give.to, give.mint.definition.get_definition_id()
				).unwrap().contains(
					&(Compact(give.edition_id), Compact(give.copy_id + 1))
				),
				true
			);

			assert_eq!(
				<CopiesCount<Test>>::get(
					(give.mint.definition.get_definition_id(), give.edition_id)
				).unwrap(),
				Compact(2)
			);

			assert_eq!(
				<Fragments<Test>>::get(
					(give.mint.definition.get_definition_id(), give.edition_id, give.copy_id + 1)
				).unwrap()
				.permissions,
				give.new_permissions.unwrap()
			);

			assert!(
				<Expirations<Test>>::get(&give.expiration.unwrap()).unwrap().contains(
					&(give.mint.definition.get_definition_id(), Compact(give.edition_id), Compact(give.copy_id + 1))
				)
			);


			let event = System::events().pop().expect("Expected at least one EventRecord to be found").event;
        	assert_eq!(
				event, 
				mock::Event::from(
					pallet_fragments::Event::InventoryAdded(
						give.to,
						give.mint.definition.get_definition_id(), 
						(give.edition_id, give.copy_id + 1)
					)
				)
			);



		});

	}

		#[test]
		fn give_should_not_work_if_the_user_does_not_own_the_fragment_instance() {

			new_test_ext().execute_with(|| {
				let dd = DummyData::new();

				let give = dd.give_no_copy_perms;
	
				upload(dd.account_id, &give.mint.definition.proto_fragment);
				create(dd.account_id, &give.mint.definition);
				mint_(dd.account_id, &give.mint);
	
				assert_noop!(
					give_(dd.account_id_second, &give),
					Error::<Test>::NoPermission
				);
			});
			
		}

		#[test]
		fn give_should_not_work_if_the_new_permissions_are_less_restrictive() {

			// use itertools::Itertools;

			// let all_permissions = vec![FragmentPerms::EDIT, FragmentPerms::TRANSFER, FragmentPerms::COPY];

			// for n in 1..all_permissions.len() {
			// 	for permissions in all_permissions.into_iter().combinations(n) {
			// 		let sum = permissions.iter().fold(0, |acc, x| acc + x); // https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.fold
			// 	}
			// }
			

			// new_test_ext().execute_with(|| {
			// 	let dd = DummyData::new();

			// 	let mut give = dd.give_no_copy_perms;

			// 	give.mint.definition.permissions = FragmentPerms::ALL - FragmentPerms::TRANSFER; // does not have transfer permission
	
			// 	upload(dd.account_id, &give.mint.definition.proto_fragment);
			// 	create(dd.account_id, &give.mint.definition);
			// 	mint_(dd.account_id, &give.mint);
	
			// 	assert_noop!(
			// 		give_(dd.account_id_second, &give),
			// 		Error::<Test>::NoPermission
			// 	);
			// });


		}



		#[test]
		fn give_should_not_work_if_the_fragment_instance_does_not_have_the_transfer_permission() {

			new_test_ext().execute_with(|| {
				let dd = DummyData::new();

				let mut give = dd.give_no_copy_perms;

				give.mint.definition.permissions = FragmentPerms::ALL - FragmentPerms::TRANSFER; // does not have transfer permission
	
				upload(dd.account_id, &give.mint.definition.proto_fragment);
				create(dd.account_id, &give.mint.definition);
				mint_(dd.account_id, &give.mint);
	
				assert_noop!(
					give_(dd.account_id_second, &give),
					Error::<Test>::NoPermission
				);
			});
		}

		#[test]
		fn give_should_not_work_if_the_fragment_instance_expires_at_the_current_block_number() {

			todo!() // A Fragment Instance that is initially created does not have an expiration. Is this intended @sinkingsugar ???

		}


		#[test]
		fn give_should_work_if_the_expiration_parameter_is_greater_than_the_current_block_number() {
			new_test_ext().execute_with(|| {
				let dd = DummyData::new();

				let current_block_number = System::block_number();

				let give = Give {
					expiration: Some(current_block_number + 1),
					..dd.give_no_copy_perms
				};
	
				upload(dd.account_id, &give.mint.definition.proto_fragment);
				create(dd.account_id, &give.mint.definition);
				mint_(dd.account_id, &give.mint);
	
				assert_noop!(
					give_(dd.account_id_second, &give),
					Error::<Test>::NoPermission
				);
			});
		}

		#[test]
		fn give_should_not_work_if_the_expiration_parameter_is_lesser_than_or_equal_to_the_current_block_number() {
			new_test_ext().execute_with(|| {
				let dd = DummyData::new();

				let current_block_number = System::block_number();

				let give = Give {
					expiration: Some(current_block_number),
					..dd.give_no_copy_perms
				};
	
				upload(dd.account_id, &give.mint.definition.proto_fragment);
				create(dd.account_id, &give.mint.definition);
				mint_(dd.account_id, &give.mint);
	
				assert_noop!(
					give_(dd.account_id_second, &give),
					Error::<Test>::NoPermission
				);
			});
		}

		// TODO - test to check if duplicated Instance's expirations changes
		#[test]
		fn give_should_not_change_the_expiration_of_the_copied_fragment_instance_if_the_expiration_is_() {
			todo!();
		}

		#[test]
		fn give_should_change_the_expiration_of_the_copied_fragment_instance_if_the_expiration_is_() {
			todo!();
		}
		
	
	
}



mod create_account_tests {
	use super::*;

	pub fn create_account_(
		signer: <Test as frame_system::Config>::AccountId,
		create_account: &CreateAccount
	) -> DispatchResult {
		FragmentsPallet::create_account(
			Origin::signed(signer),
			create_account.mint.definition.get_definition_id(),
			create_account.edition_id,
			create_account.copy_id,
		)
	}

	#[test]
	fn create_account_should_work() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let create_account = dd.create_account;

			upload(dd.account_id, &create_account.mint.definition.proto_fragment);
			create(dd.account_id, &create_account.mint.definition);
			mint_(dd.account_id, &create_account.mint);

			assert_ok!(create_account_(dd.account_id, &create_account));

			todo!();

		});
	}

	#[test]
	fn create_account_should_not_work_if_the_user_is_not_the_owner_of_the_fragment_instance() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let create_account = dd.create_account;

			upload(dd.account_id, &create_account.mint.definition.proto_fragment);
			create(dd.account_id, &create_account.mint.definition);
			mint_(dd.account_id, &create_account.mint);

			assert_noop!(
				create_account_(dd.account_id_second, &create_account),
				Error::<Test>::NoPermission
			);

		});
	}


}
