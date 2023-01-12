#![cfg(test)]

use crate as pallet_fragments;

use crate::*;
use crate::dummy_data::*;
use crate::mock::*;

use crate::Event as FragmentsEvent;

use frame_support::{assert_noop, assert_ok};
use protos::permissions::FragmentPerms;
use itertools::Itertools;

use sp_runtime::BoundedVec;

use copied_from_pallet_protos::upload;
mod copied_from_pallet_protos {
	use pallet_protos::{UsageLicense, ProtoData};

	use super::*;

	pub fn upload(
		signer: <Test as frame_system::Config>::AccountId,
		proto: &ProtoFragment,
	) -> DispatchResult {
		Protos::upload(
			Origin::signed(signer),
			proto.references.clone(),
			proto.category.clone(),
			TryInto::<BoundedVec<BoundedVec<_, _>, <Test as pallet_protos::Config>::MaxTags>>::try_into(
				proto
					.tags
					.clone()
					.into_iter()
					.map(|tag: Vec<u8>| TryInto::<BoundedVec<u8, <Test as pallet_protos::Config>::StringLimit>>::try_into(tag).unwrap())
					.collect::<Vec<BoundedVec<_, _>>>()
			).unwrap(),
			proto.linked_asset.clone(),
			proto
				.include_cost
				.map(|cost| UsageLicense::Tickets(Compact::from(cost)))
				.unwrap_or(UsageLicense::Closed),
			None,
			ProtoData::Local(proto.data.clone()),
		)
	}
}

use create_tests::create;
mod create_tests {
	use super::*;

	pub fn create(
		signer: <Test as frame_system::Config>::AccountId,
		definition: &Definition,
	) -> DispatchResult {
		FragmentsPallet::create(
			Origin::signed(signer),
			definition.proto_fragment.get_proto_hash(),
			DefinitionMetadata::<BoundedVec<_, _>, _> {
				name: definition.metadata.name.clone().try_into().unwrap(),
				currency: definition.metadata.currency,
			},
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

			assert_ok!(upload(dd.account_id, &definition.proto_fragment));
			assert_ok!(create(dd.account_id, &definition));

			let minimum_balance = <Balances as fungible::Inspect<
				<Test as frame_system::Config>::AccountId,
			>>::minimum_balance();
			System::assert_has_event(
				frame_system::Event::NewAccount {
					account: definition.get_vault_account_id(),
				}.into()
			);
			System::assert_has_event(
				pallet_balances::Event::Endowed {
					account: definition.get_vault_account_id(),
					free_balance: minimum_balance,
				}.into()
			);
			System::assert_has_event(
				pallet_balances::Event::Deposit {
					who: definition.get_vault_account_id(),
					amount: minimum_balance,
				}.into()
			);
			System::assert_last_event(
				FragmentsEvent::DefinitionCreated {
					definition_hash: definition.get_definition_id()
				}.into()
			);

			let correct_definition_struct = FragmentDefinition {
				proto_hash: definition.proto_fragment.get_proto_hash(),
				metadata: definition.metadata.clone(),
				permissions: definition.permissions.clone(),
				unique: definition.unique.clone(),
				max_supply: definition.max_supply.map(|max_supply| Compact::from(max_supply)),
				creator: dd.account_id,
				created_at: current_block_number,
				custom_metadata: BTreeMap::new(),
			};
			assert_eq!(
				<Definitions<Test>>::get(&definition.get_definition_id()).unwrap(),
				correct_definition_struct
			);
			assert!(<Proto2Fragments<Test>>::get(&definition.proto_fragment.get_proto_hash())
				.unwrap()
				.contains(&definition.get_definition_id()));

		});
	}

	#[test]
	fn create_should_not_work_if_metadata_name_is_empty() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let mut definition = dd.definition;
			definition.metadata.name = b"".to_vec();
			assert_ok!(upload(dd.account_id, &definition.proto_fragment));
			assert_noop!(create(dd.account_id, &definition), Error::<Test>::MetadataNameIsEmpty);
		});
	}

	#[test]
	fn create_should_not_work_if_fragment_definition_already_exists() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let definition = dd.definition;

			assert_ok!(upload(dd.account_id, &definition.proto_fragment));

			assert_ok!(create(dd.account_id, &definition));

			assert_noop!(create(dd.account_id, &definition), Error::<Test>::AlreadyExist);
		});
	}

	#[test]
	fn create_should_not_work_if_proto_does_not_exist() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let definition = dd.definition;

			assert_noop!(create(dd.account_id, &definition), Error::<Test>::ProtoNotFound);
		});
	}

	#[test]
	fn create_should_not_work_if_user_does_not_own_proto() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let definition = dd.definition;

			assert_ok!(upload(dd.account_id, &definition.proto_fragment));

			assert_noop!(create(dd.account_id_second, &definition), Error::<Test>::NoPermission);
		});
	}

	#[test]
	fn create_should_not_work_if_currency_does_not_exist() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let asset_id = 0;

			let mut definition = dd.definition;
			definition.metadata.currency = Currency::Custom(asset_id);

			assert_ok!(upload(dd.account_id, &definition.proto_fragment));

			assert_noop!(create(dd.account_id, &definition), Error::<Test>::CurrencyNotFound);

			assert_ok!(Assets::force_create(
				Origin::root(),
				asset_id, // The identifier of the new asset. This must not be currently in use to identify an existing asset.
				dd.account_id, // The owner of this class of assets. The owner has full superuser permissions over this asset, but may later change and configure the permissions using transfer_ownership and set_team.
				true, // Whether this asset needs users to have an existential deposit to hold this asset
				69, // The minimum balance of this new asset that any single account must have. If an account’s balance is reduced below this, then it collapses to zero.
				true // Whether the asset is transferable or not
			));

			assert_ok!(create(dd.account_id, &definition));
		});
	}

	#[test]
	#[ignore]
	fn create_should_not_work_if_proto_is_detached() {
		todo!()
	}
}

use publish_tests::publish_;
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
			publish.stack_amount,
		)
	}

	#[test]
	fn publish_should_work() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let publish = dd.publish;

			assert_ok!(upload(dd.account_id, &publish.definition.proto_fragment));
			assert_ok!(create(dd.account_id, &publish.definition));

			assert_ok!(publish_(dd.account_id, &publish));

			System::assert_last_event(
				FragmentsEvent::Publishing {
					definition_hash: publish.definition.get_definition_id()
				}.into()
			);

			let correct_publishing_data_struct = PublishingData {
				price: Compact::from(publish.price),
				units_left: publish.quantity.map(|quantity| Compact::from(quantity)),
				expiration: publish.expires,
				stack_amount: publish.stack_amount.map(|amount| Compact::from(amount)),
			};
			assert_eq!(
				<Publishing<Test>>::get(&publish.definition.get_definition_id()).unwrap(),
				correct_publishing_data_struct
			);

		});
	}

	#[test]
	fn publish_should_not_work_if_user_does_not_own_the_fragment_definition() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let publish = dd.publish;

			assert_ok!(upload(dd.account_id, &publish.definition.proto_fragment));
			assert_ok!(create(dd.account_id, &publish.definition));

			assert_noop!(publish_(dd.account_id_second, &publish), Error::<Test>::NoPermission);
		});
	}

	#[test]
	fn publish_should_not_work_if_fragment_definition_does_not_exist() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let publish = dd.publish;

			assert_noop!(publish_(dd.account_id, &publish), Error::<Test>::NotFound);
		});
	}

	#[test]
	fn publish_should_not_work_if_fragment_definition_is_currently_published() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let publish = dd.publish;

			assert_ok!(upload(dd.account_id, &publish.definition.proto_fragment));
			assert_ok!(create(dd.account_id, &publish.definition));
			assert_ok!(publish_(dd.account_id, &publish));

			assert_noop!(publish_(dd.account_id, &publish), Error::<Test>::SaleAlreadyOpen);
		});
	}

	#[test]
	fn publish_should_not_work_if_the_quantity_parameter_is_none_but_the_fragment_definition_has_a_max_supply(
	) {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let publish_with_max_supply = dd.publish_with_max_supply;

			assert_ok!(upload(dd.account_id, &publish_with_max_supply.definition.proto_fragment));
			assert_ok!(create(dd.account_id, &publish_with_max_supply.definition));

			assert_noop!(
				publish_(dd.account_id, &Publish { quantity: None, ..publish_with_max_supply }),
				Error::<Test>::ParamsNotValid
			);
		});
	}

	#[test]
	fn publish_should_not_work_if_the_quantity_to_publish_is_greater_than_the_max_supply_of_the_fragment_definition(
	) {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let publish_with_max_supply = dd.publish_with_max_supply;

			assert_ok!(upload(dd.account_id, &publish_with_max_supply.definition.proto_fragment));
			assert_ok!(create(dd.account_id, &publish_with_max_supply.definition));

			assert_noop!(
				publish_(
					dd.account_id,
					&Publish {
						quantity: publish_with_max_supply
							.definition
							.max_supply
							.map(|max_supply| max_supply + 1),
						..publish_with_max_supply.clone()
					}
				),
				Error::<Test>::MaxSupplyReached
			);
			assert_ok!(publish_(
				dd.account_id,
				&Publish {
					quantity: publish_with_max_supply.definition.max_supply,
					..publish_with_max_supply
				}
			));
		});
	}

	#[test]
	#[ignore]
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
		FragmentsPallet::unpublish(Origin::signed(signer), definition.get_definition_id())
	}

	#[test]
	fn unpublish_should_work() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let publish = dd.publish;

			assert_ok!(upload(dd.account_id, &publish.definition.proto_fragment));
			assert_ok!(create(dd.account_id, &publish.definition));
			assert_ok!(publish_(dd.account_id, &publish));

			assert_ok!(unpublish_(dd.account_id, &publish.definition));

			System::assert_last_event(
				FragmentsEvent::Unpublishing {
					definition_hash: publish.definition.get_definition_id()
				}.into()
			);

			assert_eq!(
				<Publishing<Test>>::contains_key(&publish.definition.get_definition_id()),
				false
			);

		});
	}

	#[test]
	fn upublish_should_not_work_if_user_does_not_own_the_fragment_definition() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let publish = dd.publish;

			assert_ok!(upload(dd.account_id, &publish.definition.proto_fragment));
			assert_ok!(create(dd.account_id, &publish.definition));
			assert_ok!(publish_(dd.account_id, &publish));

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

			assert_noop!(unpublish_(dd.account_id, &publish.definition), Error::<Test>::NotFound);
		});
	}

	#[test]
	fn unpublish_should_not_work_if_fragment_definition_is_not_published() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let publish = dd.publish;

			assert_ok!(upload(dd.account_id, &publish.definition.proto_fragment));
			assert_ok!(create(dd.account_id, &publish.definition));

			assert_noop!(
				unpublish_(dd.account_id, &publish.definition),
				// this error does not exist yet @sinkingsugar ???
				Error::<Test>::NotFound
			);
		});
	}

	#[test]
	#[ignore]
	fn unpublish_should_not_work_if_proto_is_detached() {
		todo!()
	}
}

use mint_tests::mint_;
mod mint_tests {
	use super::*;

	pub fn mint_(signer: <Test as frame_system::Config>::AccountId, mint: &Mint) -> DispatchResult {
		FragmentsPallet::mint(
			Origin::signed(signer),
			mint.definition.get_definition_id(),
			mint.buy_options.clone(),
			mint.amount.clone(),
		)
	}

	#[test]
	fn mint_should_work_if_the_fragment_definition_is_not_unique() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let current_block_number = System::block_number();

			let mint_non_unique = dd.mint_non_unique;

			assert_ok!(upload(dd.account_id, &mint_non_unique.definition.proto_fragment));
			assert_ok!(create(dd.account_id, &mint_non_unique.definition));

			assert_ok!(mint_(dd.account_id, &mint_non_unique));

			let correct_fragment_instance_struct = FragmentInstance {
				permissions: mint_non_unique.definition.permissions,
				created_at: current_block_number,
				custom_data: None,
				expiring_at: None,
				stack_amount: mint_non_unique.amount.map(|amount| Compact::from(amount)),
				metadata: BTreeMap::new(),
			};

			let quantity = match mint_non_unique.buy_options {
				FragmentBuyOptions::Quantity(amount) => u64::from(amount),
				_ => panic!(),
			};

			for edition_id in 1..=quantity {

				System::assert_has_event(
					FragmentsEvent::InventoryAdded {
						account_id: dd.account_id,
						definition_hash: mint_non_unique.definition.get_definition_id(),
						fragment_id: (edition_id, 1)
					}.into()
				);

				assert_eq!(
					<Fragments<Test>>::get((
						mint_non_unique.definition.get_definition_id(),
						edition_id,
						1
					))
						.unwrap(),
					correct_fragment_instance_struct
				);
				assert_eq!(
					<CopiesCount<Test>>::get((
						mint_non_unique.definition.get_definition_id(),
						edition_id
					))
						.unwrap(),
					Compact(1)
				);
				assert!(<Inventory<Test>>::get(
					dd.account_id,
					mint_non_unique.definition.get_definition_id()
				)
					.unwrap()
					.contains(&(Compact(edition_id), Compact(1))));
				assert!(<Owners<Test>>::get(
					mint_non_unique.definition.get_definition_id(),
					dd.account_id
				)
					.unwrap()
					.contains(&(Compact(edition_id), Compact(1))));
			}

			assert_eq!(
				<EditionsCount<Test>>::get(mint_non_unique.definition.get_definition_id()).unwrap(),
				Compact(quantity)
			);
		});
	}

	// TODO see if the data was indexed
	#[test]
	fn mint_should_work_if_the_fragment_definition_is_unique() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let current_block_number = System::block_number();

			let mint_unique = dd.mint_unique;

			assert_ok!(upload(dd.account_id, &mint_unique.definition.proto_fragment));
			assert_ok!(create(dd.account_id, &mint_unique.definition));

			assert_ok!(mint_(dd.account_id, &mint_unique));

			System::assert_last_event(
				FragmentsEvent::InventoryAdded {
					account_id: dd.account_id,
					definition_hash: mint_unique.definition.get_definition_id(),
					fragment_id: (1, 1)
				}.into()
			);

			let correct_fragment_instance_struct = FragmentInstance {
				permissions: mint_unique.definition.permissions,
				created_at: current_block_number,
				custom_data: match mint_unique.buy_options {
					FragmentBuyOptions::UniqueData(data) => Some(blake2_256(&data)),
					_ => panic!(),
				},
				expiring_at: None,
				stack_amount: mint_unique.amount.map(|amount| Compact::from(amount)),
				metadata: BTreeMap::new(),
			};
			assert_eq!(
				<Fragments<Test>>::get((mint_unique.definition.get_definition_id(), 1, 1)).unwrap(),
				correct_fragment_instance_struct
			);
			assert_eq!(
				<CopiesCount<Test>>::get((mint_unique.definition.get_definition_id(), 1)).unwrap(),
				Compact(1)
			);
			assert!(<Inventory<Test>>::get(
				dd.account_id,
				mint_unique.definition.get_definition_id()
			)
				.unwrap()
				.contains(&(Compact(1), Compact(1))));
			assert!(<Owners<Test>>::get(mint_unique.definition.get_definition_id(), dd.account_id)
				.unwrap()
				.contains(&(Compact(1), Compact(1))));
			assert_eq!(
				<EditionsCount<Test>>::get(mint_unique.definition.get_definition_id()).unwrap(),
				Compact(1)
			);
		});
	}

	#[test]
	fn mint_should_not_work_if_the_user_is_not_the_owner_of_the_fragment_definition() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let mint = dd.mint_non_unique;

			assert_ok!(upload(dd.account_id, &mint.definition.proto_fragment));
			assert_ok!(create(dd.account_id, &mint.definition));

			assert_noop!(mint_(dd.account_id_second, &mint), Error::<Test>::NoPermission);
		});
	}

	#[test]
	fn mint_should_not_work_if_fragment_definition_does_not_exist() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let mint = dd.mint_non_unique;

			assert_ok!(upload(dd.account_id, &mint.definition.proto_fragment));

			assert_noop!(mint_(dd.account_id, &mint), Error::<Test>::NotFound);
		});
	}

	#[test]
	fn mint_should_not_work_if_the_quantity_to_create_is_greater_than_the_max_supply_of_the_fragment_definition(
	) {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let mint = dd.mint_non_unique_with_max_supply;

			assert!(mint.definition.unique.is_none()); // we use non-unique because we can create multiple instances in a single extrinsic call (basically we're lazy)
			assert!(mint.definition.max_supply.is_some()); // max supply exists

			assert_ok!(upload(dd.account_id, &mint.definition.proto_fragment));
			assert_ok!(create(dd.account_id, &mint.definition));

			assert_noop!(
				mint_(
					dd.account_id,
					&Mint {
						buy_options: FragmentBuyOptions::Quantity(
							mint.definition.max_supply.unwrap() + 1
						),
						..mint.clone()
					}
				),
				Error::<Test>::MaxSupplyReached
			);
			assert_ok!(mint_(
				dd.account_id,
				&Mint {
					buy_options: FragmentBuyOptions::Quantity(
						mint.definition.max_supply.unwrap()
					),
					..mint
				}
			));
		});
	}

	#[test]
	fn mint_should_not_work_if_the_options_parameter_is_unique_but_the_the_fragment_definition_is_not_unique(
	) {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let mint_non_unique = dd.mint_non_unique; // fragment definition is not unique

			assert_ok!(upload(dd.account_id, &mint_non_unique.definition.proto_fragment));
			assert_ok!(create(dd.account_id, &mint_non_unique.definition));

			assert_noop!(
				mint_(
					dd.account_id,
					&Mint {
						buy_options: FragmentBuyOptions::UniqueData(b"I dati".to_vec()), // options parameter is unique
						..mint_non_unique
					}
				),
				// this error does not exist yet @sinkingsugar ???
				Error::<Test>::ParamsNotValid
			);
		});
	}

	#[test]
	fn mint_should_not_work_if_the_options_parameter_is_not_unique_but_the_the_fragment_definition_is_unique(
	) {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let mint_unique = dd.mint_unique; // fragment definition is unique

			assert_ok!(upload(dd.account_id, &mint_unique.definition.proto_fragment));
			assert_ok!(create(dd.account_id, &mint_unique.definition));

			assert_noop!(
				mint_(
					dd.account_id,
					&Mint {
						buy_options: FragmentBuyOptions::Quantity(123), // options parameter is not unique
						..mint_unique
					}
				),
				// this error does not exist yet @sinkingsugar ???
				Error::<Test>::ParamsNotValid
			);
		});
	}

	#[test]
	fn mint_should_not_work_if_fragment_instance_was_already_created_with_the_same_unique_data() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let mint_unique = dd.mint_unique;

			assert_ok!(upload(dd.account_id, &mint_unique.definition.proto_fragment));
			assert_ok!(create(dd.account_id, &mint_unique.definition));

			assert_ok!(mint_(dd.account_id, &mint_unique));

			assert_noop!(
				mint_(dd.account_id, &mint_unique),
				// this error does not exist yet @sinkingsugar ???
				Error::<Test>::UniqueDataExists
			);
		});
	}

	#[test]
	#[ignore]
	fn mint_should_not_work_if_proto_is_detached() {
		todo!()
	}
}

mod buy_tests {
	use super::*;

	fn buy_(signer: <Test as frame_system::Config>::AccountId, buy: &Buy) -> DispatchResult {
		FragmentsPallet::buy(
			Origin::signed(signer),
			buy.publish.definition.get_definition_id(),
			buy.buy_options.clone(),
		)
	}

	fn publish_definition(signer: <Test as frame_system::Config>::AccountId, buy: &Buy) {
		assert_ok!(upload(signer, &buy.publish.definition.proto_fragment));
		assert_ok!(create(signer, &buy.publish.definition));
		assert_ok!(publish_(signer, &buy.publish));
	}

	#[test]
	fn buy_should_work_if_fragment_definition_is_not_unique() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let current_block_number = System::block_number();

			let buy_non_unique = dd.buy_non_unique;

			publish_definition(dd.account_id, &buy_non_unique);

			// Deposit `quantity` to buyer's account
			let quantity = match buy_non_unique.buy_options {
				FragmentBuyOptions::Quantity(amount) => u64::from(amount),
				_ => 1u64,
			};
			let minimum_balance = <Balances as fungible::Inspect<
				<Test as frame_system::Config>::AccountId,
			>>::minimum_balance();
			_ = <Balances as fungible::Mutate<<Test as frame_system::Config>::AccountId>>::mint_into(
				&dd.account_id_second,
				buy_non_unique.publish.price.saturating_mul(quantity as u128) + minimum_balance,
			);

			assert_ok!(buy_(dd.account_id_second, &buy_non_unique));

			System::assert_has_event(
				pallet_balances::Event::Transfer {
					from: dd.account_id_second,
					to: buy_non_unique.publish.definition.get_vault_account_id(),
					amount: buy_non_unique.publish.price.saturating_mul(quantity as u128),
				}.into()
			);

			for edition_id in 1..=quantity {
				System::assert_has_event(
					FragmentsEvent::InventoryAdded {
						account_id: dd.account_id_second,
						definition_hash: buy_non_unique.publish.definition.get_definition_id(),
						fragment_id: (edition_id, 1)
					}.into()
				);

				// This `correct_fragment_instance_struct` is the same for every iteration btw!
				let correct_fragment_instance_struct = FragmentInstance {
					permissions: buy_non_unique.publish.definition.permissions,
					created_at: current_block_number,
					custom_data: None,
					expiring_at: None, // newly created Fragment Instance doesn't have an expiration date - confirm with @sinkingsugar
					stack_amount: buy_non_unique.publish.stack_amount.map(|amount| Compact::from(amount)),
					metadata: BTreeMap::new(),
				};
				assert_eq!(
					<Fragments<Test>>::get((
						buy_non_unique.publish.definition.get_definition_id(),
						edition_id,
						1
					))
						.unwrap(),
					correct_fragment_instance_struct
				);
				assert_eq!(
					<CopiesCount<Test>>::get((
						buy_non_unique.publish.definition.get_definition_id(),
						edition_id
					))
						.unwrap(),
					Compact(1)
				);
				assert!(<Inventory<Test>>::get(
					dd.account_id_second,
					buy_non_unique.publish.definition.get_definition_id()
				)
					.unwrap()
					.contains(&(Compact(edition_id), Compact(1))));

				assert!(<Owners<Test>>::get(
					buy_non_unique.publish.definition.get_definition_id(),
					dd.account_id_second
				)
					.unwrap()
					.contains(&(Compact(edition_id), Compact(1))));
			}

			assert_eq!(
				<EditionsCount<Test>>::get(buy_non_unique.publish.definition.get_definition_id())
					.unwrap(),
				Compact(quantity)
			);
		});
	}

	// TODO see if the data was indexed
	#[test]
	fn buy_should_work_if_the_fragment_definition_is_unique() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let current_block_number = System::block_number();

			let buy_unique = dd.buy_unique;

			publish_definition(dd.account_id, &buy_unique);

			// Deposit `quantity` to buyer's account
			let quantity = match buy_unique.buy_options {
				FragmentBuyOptions::Quantity(amount) => u64::from(amount),
				_ => 1u64,
			};
			let minimum_balance = <Balances as fungible::Inspect<
				<Test as frame_system::Config>::AccountId,
			>>::minimum_balance();
			_ = <Balances as fungible::Mutate<<Test as frame_system::Config>::AccountId>>::mint_into(
				&dd.account_id_second,
				buy_unique.publish.price.saturating_mul(quantity as u128) + minimum_balance,
			);

			assert_ok!(buy_(dd.account_id_second, &buy_unique));

			System::assert_has_event(
				FragmentsEvent::InventoryAdded {
					account_id: dd.account_id_second,
					definition_hash: buy_unique.publish.definition.get_definition_id(),
					fragment_id: (1, 1)
				}.into()
			);
			System::assert_has_event(
				pallet_balances::Event::Transfer {
					from: dd.account_id_second,
					to: buy_unique.publish.definition.get_vault_account_id(),
					amount: buy_unique.publish.price.saturating_mul(quantity as u128),
				}.into()
			);

			let correct_fragment_instance_struct = FragmentInstance {
				permissions: buy_unique.publish.definition.permissions,
				created_at: current_block_number,
				custom_data: match buy_unique.buy_options {
					FragmentBuyOptions::UniqueData(data) => Some(blake2_256(&data)),
					_ => panic!(),
				},
				expiring_at: None,
				stack_amount: buy_unique.publish.stack_amount.map(|amount| Compact::from(amount)),
				metadata: BTreeMap::new(),
			};
			assert_eq!(
				<Fragments<Test>>::get((buy_unique.publish.definition.get_definition_id(), 1, 1))
					.unwrap(),
				correct_fragment_instance_struct
			);
			assert!(<Inventory<Test>>::get(
				dd.account_id_second,
				buy_unique.publish.definition.get_definition_id()
			)
				.unwrap()
				.contains(&(Compact(1), Compact(1))));

			assert!(<Owners<Test>>::get(
				buy_unique.publish.definition.get_definition_id(),
				dd.account_id_second
			)
				.unwrap()
				.contains(&(Compact(1), Compact(1))));
			assert_eq!(
				<EditionsCount<Test>>::get(buy_unique.publish.definition.get_definition_id())
					.unwrap(),
				Compact(1)
			);
			assert_eq!(
				<CopiesCount<Test>>::get((buy_unique.publish.definition.get_definition_id(), 1))
					.unwrap(),
				Compact(1)
			);

		});
	}

	#[test]
	fn buy_should_not_work_if_the_fragment_definition_is_not_published() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let buy = dd.buy_non_unique;

			assert_ok!(upload(dd.account_id, &buy.publish.definition.proto_fragment));
			assert_ok!(create(dd.account_id, &buy.publish.definition));

			assert_noop!(buy_(dd.account_id_second, &buy), Error::<Test>::NotFound,);
		});
	}

	#[test]
	fn buy_should_not_work_if_user_has_insufficient_balance_in_pallet_balances() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let buy = dd.buy_non_unique;

			publish_definition(dd.account_id, &buy);

			// Deposit `quantity` to buyer's account
			let quantity = match buy.buy_options {
				FragmentBuyOptions::Quantity(amount) => u64::from(amount),
				_ => 1u64,
			};
			let minimum_balance = <Balances as fungible::Inspect<
				<Test as frame_system::Config>::AccountId,
			>>::minimum_balance();

			_ = <Balances as fungible::Mutate<<Test as frame_system::Config>::AccountId>>::mint_into(
				&dd.account_id_second,
				buy.publish.price.saturating_mul(quantity as u128) + minimum_balance - 1,
			);
			assert_noop!(buy_(dd.account_id_second, &buy), Error::<Test>::InsufficientBalance);

			_ = <Balances as fungible::Mutate<<Test as frame_system::Config>::AccountId>>::mint_into(
				&dd.account_id_third,
				buy.publish.price.saturating_mul(quantity as u128) + minimum_balance,
			);
			assert_ok!(buy_(dd.account_id_third, &buy));

		});
	}

	#[test]
	fn buy_should_not_work_if_user_has_insufficient_balance_in_pallet_assets() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let asset_id = 0;

			let mut buy = dd.buy_non_unique;
			buy.publish.definition.metadata.currency = Currency::Custom(asset_id);

			let minimum_balance = 1;
			assert_ok!(Assets::force_create(
				Origin::root(),
				asset_id, // The identifier of the new asset. This must not be currently in use to identify an existing asset.
				dd.account_id, // The owner of this class of assets. The owner has full superuser permissions over this asset, but may later change and configure the permissions using transfer_ownership and set_team.
				true,          // Whether this asset needs users to have an existential deposit to hold this asset
				minimum_balance, // The minimum balance of this new asset that any single account must have. If an account’s balance is reduced below this, then it collapses to zero.
				true
			));

			publish_definition(dd.account_id, &buy);

			// Deposit `quantity` to buyer's account
			let quantity = match buy.buy_options {
				FragmentBuyOptions::Quantity(amount) => u64::from(amount),
				_ => 1u64,
			};

			assert_ok!(Assets::mint(
				Origin::signed(dd.account_id),
				asset_id,
				dd.account_id_second,
				buy.publish.price.saturating_mul(quantity as u128) + minimum_balance - 1,
			));
			assert_noop!(buy_(dd.account_id_second, &buy), Error::<Test>::InsufficientBalance);

			assert_ok!(Assets::mint(
				Origin::signed(dd.account_id),
				asset_id,
				dd.account_id_third,
				buy.publish.price.saturating_mul(quantity as u128) + minimum_balance,
			));
			assert_ok!(buy_(dd.account_id_third, &buy));

		});
	}

	#[ignore = "will the definition vault's account ever fall below minimum balance?"]
	#[test]
	fn buy_should_work_if_the_definition_vault_id_will_have_a_minimum_balance_of_the_asset_after_transaction(
	) {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let asset_id = 0;

			let mut buy = dd.buy_non_unique;
			buy.publish.definition.metadata.currency = Currency::Custom(asset_id);

			// Deposit `quantity` to buyer's account
			let quantity = match buy.buy_options {
				FragmentBuyOptions::Quantity(amount) => u64::from(amount),
				_ => 1u64,
			};

			let minimum_balance = buy.publish.price.saturating_mul(quantity as u128); // vault ID wil have minimum balance after `buy()` transaction
			assert_ok!(Assets::force_create(
				Origin::root(),
				asset_id, // The identifier of the new asset. This must not be currently in use to identify an existing asset.
				dd.account_id, // The owner of this class of assets. The owner has full superuser permissions over this asset, but may later change and configure the permissions using transfer_ownership and set_team.
				true,          // Whether this asset needs users to have an existential deposit to hold this asset
				minimum_balance, // The minimum balance of this new asset that any single account must have. If an account’s balance is reduced below this, then it collapses to zero.
				true
			));

			publish_definition(dd.account_id, &buy);

			assert_ok!(Assets::mint(
				Origin::signed(dd.account_id),
				asset_id,
				dd.account_id_second,
				buy.publish.price.saturating_mul(quantity as u128) + minimum_balance,
			));

			assert_ok!(buy_(dd.account_id_second, &buy));
		});
	}

	#[ignore = "will the definition vault's account ever fall below minimum balance?"]
	#[test]
	fn buy_should_not_work_if_the_definition_vault_id_will_not_have_a_minimum_balance_of_the_asset_after_transaction(
	) {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let asset_id = 0;

			let mut buy = dd.buy_non_unique;
			buy.publish.definition.metadata.currency = Currency::Custom(asset_id);

			// Deposit `quantity` to buyer's account
			let quantity = match buy.buy_options {
				FragmentBuyOptions::Quantity(amount) => u64::from(amount),
				_ => 1u64,
			};

			let minimum_balance = buy.publish.price.saturating_mul(quantity as u128) + 1; // vault ID wil not have minimum balance after `buy()` transaction
			assert_ok!(Assets::force_create(
				Origin::root(),
				asset_id, // The identifier of the new asset. This must not be currently in use to identify an existing asset.
				dd.account_id, // The owner of this class of assets. The owner has full superuser permissions over this asset, but may later change and configure the permissions using transfer_ownership and set_team.
				true,          // Whether this asset needs users to have an existential deposit to hold this asset
				minimum_balance, // The minimum balance of this new asset that any single account must have. If an account’s balance is reduced below this, then it collapses to zero.
				true
			));

			publish_definition(dd.account_id, &buy);

			assert_ok!(Assets::mint(
				Origin::signed(dd.account_id),
				asset_id,
				dd.account_id_second,
				buy.publish.price.saturating_mul(quantity as u128) + minimum_balance,
			));

			assert_noop!(
				buy_(dd.account_id_second, &buy),
				Error::<Test>::ReceiverBelowMinimumBalance
			);
		});
	}

	#[test]
	fn buy_should_not_work_if_the_quantity_to_mint_is_greater_than_the_published_quantity_of_the_fragment_definition(
	) {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let mut buy = dd.buy_non_unique_with_limited_published_quantity;

			assert!(buy.publish.definition.unique.is_none()); // we use non-unique because we can create multiple instances in a single extrinsic call (basically we're lazy)
			assert!(buy.publish.quantity.is_some()); // published quantity exists exists

			publish_definition(dd.account_id, &buy);

			let minimum_balance = <Balances as fungible::Inspect<
				<Test as frame_system::Config>::AccountId,
			>>::minimum_balance();

			buy.buy_options = FragmentBuyOptions::Quantity(buy.publish.quantity.unwrap() + 1); // greater than the max supply of the fragment definition
			let quantity = match buy.buy_options {
				FragmentBuyOptions::Quantity(amount) => u64::from(amount),
				_ => 1u64,
			};
			_ = <Balances as fungible::Mutate<<Test as frame_system::Config>::AccountId>>::mint_into(
				&dd.account_id_second,
				buy.publish.price.saturating_mul(quantity as u128) + minimum_balance,
			);
			assert_noop!(buy_(dd.account_id_second, &buy), Error::<Test>::PublishedQuantityReached);

			buy.buy_options = FragmentBuyOptions::Quantity(buy.publish.quantity.unwrap()); // equal to the published quantity of the fragment definition
			assert_ok!(buy_(dd.account_id_second, &buy));
		});
	}

	#[test]
	fn buy_should_not_work_if_the_options_parameter_is_unique_but_the_the_fragment_definition_is_not_unique(
	) {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let buy_non_unique = dd.buy_non_unique; // fragment definition is not unique

			publish_definition(dd.account_id, &buy_non_unique);

			// We deposit (Price * 1) + `minimum_balance`
			// because that's all we need to deposit if our options parameter is unique
			let minimum_balance = <Balances as fungible::Inspect<
				<Test as frame_system::Config>::AccountId,
			>>::minimum_balance();
			_ = <Balances as fungible::Mutate<<Test as frame_system::Config>::AccountId>>::mint_into(
				&dd.account_id_second,
				buy_non_unique.publish.price.saturating_mul(1 as u128) + minimum_balance,
			);

			assert_noop!(
				buy_(
					dd.account_id_second,
					&Buy {
						buy_options: FragmentBuyOptions::UniqueData(b"I dati".to_vec()), // options parameter is unique
						..buy_non_unique
					}
				),
				// this error does not exist yet @sinkingsugar ???
				Error::<Test>::ParamsNotValid
			);
		});
	}

	#[test]
	fn buy_should_not_work_if_the_options_parameter_is_not_unique_but_the_the_fragment_definition_is_unique(
	) {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let buy_unique = dd.buy_unique; // fragment definition is unique

			publish_definition(dd.account_id, &buy_unique);

			// Since our options parameter is `FragmentBuyOptions::Quantity(123)`,
			// we deposit (Price * 123) + `minimum_balance`
			let minimum_balance = <Balances as fungible::Inspect<
				<Test as frame_system::Config>::AccountId,
			>>::minimum_balance();
			_ = <Balances as fungible::Mutate<<Test as frame_system::Config>::AccountId>>::mint_into(
				&dd.account_id_second,
				buy_unique.publish.price.saturating_mul(123 as u128) + minimum_balance,
			);

			assert_noop!(
				buy_(
					dd.account_id_second,
					&Buy {
						buy_options: FragmentBuyOptions::Quantity(123), // options parameter is not unique
						..buy_unique
					}
				),
				// this error does not exist yet @sinkingsugar ???
				Error::<Test>::ParamsNotValid
			);
		});
	}

	#[test]
	fn buy_should_not_work_if_the_sale_has_expired() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let mut buy = dd.buy_non_unique;
			buy.publish.expires = Some(999);

			assert!(buy.publish.expires.is_some()); // sale must have an expiration

			publish_definition(dd.account_id, &buy);

			// Deposit `quantity` to buyer's account
			let quantity = match buy.buy_options {
				FragmentBuyOptions::Quantity(amount) => u64::from(amount),
				_ => 1u64,
			};
			let minimum_balance = <Balances as fungible::Inspect<
				<Test as frame_system::Config>::AccountId,
			>>::minimum_balance();
			_ = <Balances as fungible::Mutate<<Test as frame_system::Config>::AccountId>>::mint_into(
				&dd.account_id_second,
				buy.publish.price.saturating_mul(quantity as u128) + minimum_balance,
			);

			run_to_block(buy.publish.expires.unwrap() - 1);
			assert_ok!(buy_(dd.account_id_second, &buy));
			run_to_block(buy.publish.expires.unwrap());
			assert_noop!(buy_(dd.account_id_second, &buy), Error::<Test>::Expired);
		});
	}

	#[test]
	fn buy_should_not_work_if_fragment_instance_was_already_created_with_the_same_unique_data() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let buy_unique = dd.buy_unique;

			publish_definition(dd.account_id, &buy_unique);

			// We deposit (Price * 1) + `minimum_balance`
			// because that's all we need to deposit if our options parameter is unique
			let minimum_balance = <Balances as fungible::Inspect<
				<Test as frame_system::Config>::AccountId,
			>>::minimum_balance();
			_ = <Balances as fungible::Mutate<<Test as frame_system::Config>::AccountId>>::mint_into(
				&dd.account_id_second,
				buy_unique.publish.price.saturating_mul(1 as u128) + minimum_balance,
			);

			assert_ok!(buy_(dd.account_id_second, &buy_unique));

			// We deposit (Price * 1) + `minimum_balance`
			// because that's all we need to deposit if our options parameter is unique
			let minimum_balance = <Balances as fungible::Inspect<
				<Test as frame_system::Config>::AccountId,
			>>::minimum_balance();
			_ = <Balances as fungible::Mutate<<Test as frame_system::Config>::AccountId>>::mint_into(
				&dd.account_id_second,
				buy_unique.publish.price.saturating_mul(1 as u128) + minimum_balance,
			);

			assert_noop!(
				buy_(dd.account_id_second, &buy_unique),
				// this error does not exist yet @sinkingsugar ???
				Error::<Test>::UniqueDataExists
			);
		});
	}

	#[test]
	#[ignore]
	fn buy_should_not_work_if_proto_is_detached() {
		todo!()
	}
}

use give_tests::{give_, mint_give_instance};
mod give_tests {
	use super::*;

	pub fn give_(
		// Review whether these many parameters are appropriate/needed @karan
		signer: <Test as frame_system::Config>::AccountId,
		give: &Give,
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

	pub fn mint_give_instance(
		signer: <Test as frame_system::Config>::AccountId,
		give: &Give
	) {
		assert_ok!(upload(signer, &give.mint.definition.proto_fragment));
		assert_ok!(create(signer, &give.mint.definition));
		assert_ok!(mint_(signer, &give.mint));
	}

	#[test]
	fn give_should_work_if_the_fragment_instance_does_not_have_copy_permissions() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let give = dd.give_no_copy_perms;

			mint_give_instance(dd.account_id, &give);
			assert_ok!(give_(dd.account_id, &give));

			System::assert_has_event(
				FragmentsEvent::InventoryRemoved {
					account_id: dd.account_id,
					definition_hash: give.mint.definition.get_definition_id(),
					fragment_id: (give.edition_id, give.copy_id),
				}.into()
			);
			System::assert_has_event(
				FragmentsEvent::InventoryAdded {
					account_id: give.to,
					definition_hash: give.mint.definition.get_definition_id(),
					fragment_id: (give.edition_id, give.copy_id),
				}.into()
			);

			assert_eq!(
				<Owners<Test>>::get(give.mint.definition.get_definition_id(), dd.account_id)
					.unwrap()
					.contains(&(Compact(give.edition_id), Compact(give.copy_id))),
				false
			);
			assert_eq!(
				<Inventory<Test>>::get(dd.account_id, give.mint.definition.get_definition_id())
					.unwrap()
					.contains(&(Compact(give.edition_id), Compact(give.copy_id))),
				false
			);

			assert_eq!(
				<Owners<Test>>::get(give.mint.definition.get_definition_id(), give.to)
					.unwrap()
					.contains(&(Compact(give.edition_id), Compact(give.copy_id))),
				true
			);
			assert_eq!(
				<Inventory<Test>>::get(give.to, give.mint.definition.get_definition_id())
					.unwrap()
					.contains(&(Compact(give.edition_id), Compact(give.copy_id))),
				true
			);

			assert_eq!(
				<Fragments<Test>>::get((
					give.mint.definition.get_definition_id(),
					give.edition_id,
					give.copy_id
				))
					.unwrap()
					.permissions,
				give.new_permissions.unwrap()
			);

		});
	}

	#[test]
	fn give_should_work_if_the_fragment_instance_has_copy_permissions() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let give = dd.give_copy_perms;

			mint_give_instance(dd.account_id, &give);
			assert_ok!(give_(dd.account_id, &give));

			System::assert_last_event(
				FragmentsEvent::InventoryAdded {
					account_id: give.to,
					definition_hash: give.mint.definition.get_definition_id(),
					fragment_id: (give.edition_id, give.copy_id + 1),
				}.into()
			);

			assert_eq!(
				<Owners<Test>>::get(give.mint.definition.get_definition_id(), dd.account_id)
					.unwrap()
					.contains(&(Compact(give.edition_id), Compact(give.copy_id))),
				true
			);
			assert_eq!(
				<Inventory<Test>>::get(dd.account_id, give.mint.definition.get_definition_id())
					.unwrap()
					.contains(&(Compact(give.edition_id), Compact(give.copy_id))),
				true
			);

			assert_eq!(
				<Owners<Test>>::get(give.mint.definition.get_definition_id(), give.to)
					.unwrap()
					.contains(&(Compact(give.edition_id), Compact(give.copy_id + 1))),
				true
			);
			assert_eq!(
				<Inventory<Test>>::get(give.to, give.mint.definition.get_definition_id())
					.unwrap()
					.contains(&(Compact(give.edition_id), Compact(give.copy_id + 1))),
				true
			);

			assert_eq!(
				<CopiesCount<Test>>::get((
					give.mint.definition.get_definition_id(),
					give.edition_id
				))
					.unwrap(),
				Compact(2)
			);

			assert_eq!(
				<Fragments<Test>>::get((
					give.mint.definition.get_definition_id(),
					give.edition_id,
					give.copy_id + 1
				))
					.unwrap()
					.permissions,
				give.new_permissions.unwrap()
			);
			assert_eq!(
				<Fragments<Test>>::get((
					give.mint.definition.get_definition_id(),
					give.edition_id,
					give.copy_id + 1
				))
					.unwrap()
					.expiring_at,
				give.expiration
			);

			assert!(<Expirations<Test>>::get(&give.expiration.unwrap()).unwrap().contains(&(
				give.mint.definition.get_definition_id(),
				Compact(give.edition_id),
				Compact(give.copy_id + 1)
			)));

		});
	}

	#[test]
	fn give_should_not_work_if_the_user_does_not_own_the_fragment_instance() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let give = dd.give_no_copy_perms;

			mint_give_instance(dd.account_id, &give);

			assert_noop!(
				give_(dd.account_id_second, &give),
				Error::<Test>::NoPermission
			);
		});
	}

	#[test]
	fn give_should_not_work_if_the_new_permissions_are_less_restrictive() {

		let vec_all_perms = vec![FragmentPerms::EDIT, FragmentPerms::TRANSFER, FragmentPerms::COPY];
		assert_eq!(
			vec_all_perms.iter().fold(FragmentPerms::NONE, |acc, &x| acc | x),
			FragmentPerms::ALL
		);
		let vec_all_perms_excl_transfer = vec_all_perms.clone().into_iter().filter(
			|&x| x != FragmentPerms::TRANSFER
		).collect::<Vec<FragmentPerms>>();

		for len_combo in 0..vec_all_perms_excl_transfer.len() {

			for vec_perms_excl_transfer in vec_all_perms_excl_transfer.clone().into_iter().combinations(len_combo) {
				let vec_perms = [vec_perms_excl_transfer, vec![FragmentPerms::TRANSFER]].concat(); // TRANSFER must be included, since we want to give it

				let vec_possible_additional_perms = vec_all_perms
					.clone()
					.into_iter()
					.filter(|x| !vec_perms.contains(x))
					.collect::<Vec<FragmentPerms>>();

				// Less Restrictive
				for len_combo in 1..=vec_possible_additional_perms.len() {
					for vec_new_perms in
					vec_possible_additional_perms.clone().into_iter().combinations(len_combo)
					{
						let vec_new_permissions_parameter = [vec_perms.clone(), vec_new_perms.clone()].concat();

						new_test_ext().execute_with(|| {
							let dd = DummyData::new();
							let mut give = dd.give_no_copy_perms;
							give.mint.definition.permissions = vec_perms
								.iter()
								.fold(FragmentPerms::NONE, |acc, &x| acc | x);
							give.new_permissions = Some(
								vec_new_permissions_parameter
									.iter()
									.fold(FragmentPerms::NONE, |acc, &x| acc | x),
							);
							// Should Not Work
							mint_give_instance(dd.account_id, &give);
							assert_noop!(give_(dd.account_id, &give), Error::<Test>::NoPermission);
						});
					}
				}

				// Equally or More Restrictive
				for len_combo in 0..=vec_perms.len() {
					for vec_new_permissions_parameter in
					vec_perms.clone().into_iter().combinations(len_combo)
					{
						new_test_ext().execute_with(|| {
							let dd = DummyData::new();
							let mut give = dd.give_no_copy_perms;
							give.mint.definition.permissions = vec_perms
								.iter()
								.fold(FragmentPerms::NONE, |acc, &x| acc | x);
							give.new_permissions = Some(
								vec_new_permissions_parameter
									.iter()
									.fold(FragmentPerms::NONE, |acc, &x| acc | x),
							);
							// Should Work
							mint_give_instance(dd.account_id, &give);
							assert_ok!(give_(dd.account_id, &give));
						});
					}
				}
			}
		}
	}

	#[test]
	fn give_should_not_work_if_the_fragment_instance_does_not_have_the_transfer_permission() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let mut give = dd.give_no_copy_perms;

			give.mint.definition.permissions = FragmentPerms::ALL - FragmentPerms::TRANSFER; // does not have transfer permission

			mint_give_instance(dd.account_id, &give);
			assert_noop!(give_(dd.account_id, &give), Error::<Test>::NoPermission);
		});
	}

	#[test]
	fn give_should_not_work_if_the_fragment_instance_expires_before_or_at_the_current_block_number() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let mut give = dd.give_copy_perms;
			assert!(give.expiration.is_some());
			give.new_permissions = Some(FragmentPerms::TRANSFER | FragmentPerms::COPY); // ensure that the copied instance can also be used to create copies
			mint_give_instance(dd.account_id, &give);
			assert_ok!(give_(dd.account_id, &give));


			let give_again = Give {
				copy_id: give.copy_id + 1,
				to: dd.account_id_second,
				..give.clone()
			};

			run_to_block(give.expiration.unwrap() - 1);
			assert_ok!(give_(give.to, &give_again));
			run_to_block(give.expiration.unwrap());
			assert_noop!(give_(give.to, &give_again), Error::<Test>::NotFound);
		});
	}

	#[test]
	fn give_should_not_work_if_the_expiration_parameter_is_lesser_than_or_equal_to_the_current_block_number() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let current_block_number = System::block_number();

			let give = dd.give_copy_perms;

			mint_give_instance(dd.account_id, &give);

			assert_noop!(
				give_(
					dd.account_id,
					&Give {
						expiration: Some(current_block_number),
						..give.clone()
					}
				),
				Error::<Test>::ParamsNotValid
			);
			assert_ok!(
				give_(
					dd.account_id,
					&Give {
						expiration: Some(current_block_number + 1),
						..give
					}
				)
			);
		});
	}

	#[test]
	fn give_should_not_change_the_expiration_of_the_copied_fragment_instance_if_the_expiration_parameter_is_greater_than_it(
	) {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let mut give = dd.give_copy_perms;
			assert!(give.expiration.is_some());
			give.new_permissions = Some(FragmentPerms::TRANSFER | FragmentPerms::COPY); // ensure that the copied instance can also be used to create copies

			mint_give_instance(dd.account_id, &give);
			assert_ok!(give_(dd.account_id, &give));

			assert_ok!(
				give_(
					give.to,
					&Give {
						copy_id: give.copy_id + 1,
						to: dd.account_id_second,
						expiration: Some(give.expiration.unwrap() - 1),
						..give.clone()
					}
				)
			);
			assert!(
				!<Expirations<Test>>::get(give.expiration.unwrap()).unwrap().contains(
					&(
						give.mint.definition.get_definition_id(),
						Compact(give.edition_id),
						Compact(give.copy_id + 2)
					)
				)
			);
			assert!(
				<Expirations<Test>>::get(give.expiration.unwrap() - 1).unwrap().contains(
					&(
						give.mint.definition.get_definition_id(),
						Compact(give.edition_id),
						Compact(give.copy_id + 2)
					)
				)
			);

			assert_ok!(
				give_(
					give.to,
					&Give {
						copy_id: give.copy_id + 1,
						to: dd.account_id_second,
						expiration: Some(give.expiration.unwrap() + 1),
						..give.clone()
					}
				)
			);
			assert!(!<Expirations<Test>>::contains_key(give.expiration.unwrap() + 1));
			assert!(
				<Expirations<Test>>::get(give.expiration.unwrap()).unwrap().contains(
					&(
						give.mint.definition.get_definition_id(),
						Compact(give.edition_id),
						Compact(give.copy_id + 3)
					)
				)
			);
		});
	}


	#[test]
	#[ignore]
	fn give_should_not_work_if_the_instance_is_detached() {
		todo!()
	}


}

mod create_account_tests {
	use super::*;

	pub fn create_account_(
		signer: <Test as frame_system::Config>::AccountId,
		create_account: &CreateAccount,
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

			assert_ok!(upload(dd.account_id, &create_account.mint.definition.proto_fragment));
			assert_ok!(create(dd.account_id, &create_account.mint.definition));
			assert_ok!(mint_(dd.account_id, &create_account.mint));

			assert_ok!(create_account_(dd.account_id, &create_account));
		});
	}

	#[test]
	fn create_account_should_not_work_if_the_user_is_not_the_owner_of_the_fragment_instance() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let create_account = dd.create_account;

			assert_ok!(upload(dd.account_id, &create_account.mint.definition.proto_fragment));
			assert_ok!(create(dd.account_id, &create_account.mint.definition));
			assert_ok!(mint_(dd.account_id, &create_account.mint));

			assert_noop!(
				create_account_(dd.account_id_second, &create_account),
				Error::<Test>::NotFound
			);
		});
	}
}

use resell_tests::resell_;
mod resell_tests {
	use super::*;

	pub fn resell_(
		signer: <Test as frame_system::Config>::AccountId,
		resell: &Resell
	) -> DispatchResult {
		FragmentsPallet::resell(
			Origin::signed(signer),
			resell.mint.definition.get_definition_id(),
			resell.edition_id,
			resell.copy_id,
			resell.new_permissions,
			resell.expiration,
			resell.secondary_sale_type.clone()
		)
	}

	pub fn mint_resell_instance(
		signer: <Test as frame_system::Config>::AccountId,
		resell: &Resell
	) {

		assert_ok!(upload(signer, &resell.mint.definition.proto_fragment));
		assert_ok!(create(signer, &resell.mint.definition));
		assert_ok!(mint_(signer, &resell.mint));
	}

	#[test]
	fn resell_should_work() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();
			let resell = dd.resell_normal;
			mint_resell_instance(dd.account_id, &resell);
			assert_ok!(resell_(dd.account_id, &resell));

			System::assert_last_event(
				FragmentsEvent::Resell {
					definition_hash: resell.mint.definition.get_definition_id(),
					fragment_id: (resell.edition_id, resell.copy_id),
				}.into()
			);

			assert!(
				Definition2SecondarySales::<Test>::get((resell.mint.definition.get_definition_id(), resell.edition_id, resell.copy_id)).unwrap()
					==
					SecondarySaleData {
						owner: dd.account_id,
						new_permissions: resell.new_permissions,
						expiration: resell.expiration,
						secondary_sale_type: resell.secondary_sale_type,
					}
			);

		});
	}

	#[test]
	fn resell_should_not_work_if_user_does_not_own_instance() {

		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let resell = dd.resell_normal;

			mint_resell_instance(dd.account_id, &resell);

			assert_noop!(resell_(dd.account_id_second, &resell), Error::<Test>::NoPermission);

		});
	}

	#[test]
	fn resell_should_not_work_if_the_instance_is_already_on_sale() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();
			let resell = dd.resell_normal;
			mint_resell_instance(dd.account_id, &resell);
			assert_ok!(resell_(dd.account_id, &resell));
			assert_noop!(resell_(dd.account_id, &resell), Error::<Test>::SaleAlreadyOpen);
		});
	}


	#[test]
	fn resell_should_not_work_if_the_new_permissions_are_less_restrictive() {

		let vec_all_perms = vec![FragmentPerms::EDIT, FragmentPerms::TRANSFER, FragmentPerms::COPY];
		assert_eq!(
			vec_all_perms.iter().fold(FragmentPerms::NONE, |acc, &x| acc | x),
			FragmentPerms::ALL
		);
		let vec_all_perms_excl_transfer = vec_all_perms.clone().into_iter().filter(
			|&x| x != FragmentPerms::TRANSFER
		).collect::<Vec<FragmentPerms>>();

		for len_combo in 0..vec_all_perms_excl_transfer.len() {

			for vec_perms_excl_transfer in vec_all_perms_excl_transfer.clone().into_iter().combinations(len_combo) {
				let vec_perms = [vec_perms_excl_transfer, vec![FragmentPerms::TRANSFER]].concat(); // TRANSFER must be included, since we want to give it

				let vec_possible_additional_perms = vec_all_perms
					.clone()
					.into_iter()
					.filter(|x| !vec_perms.contains(x))
					.collect::<Vec<FragmentPerms>>();

				// Less Restrictive
				for len_combo in 1..=vec_possible_additional_perms.len() {
					for vec_new_perms in
					vec_possible_additional_perms.clone().into_iter().combinations(len_combo)
					{
						let vec_new_permissions_parameter = [vec_perms.clone(), vec_new_perms.clone()].concat();

						new_test_ext().execute_with(|| {
							let dd = DummyData::new();
							let mut resell = dd.resell_normal;
							resell.mint.definition.permissions = vec_perms
								.iter()
								.fold(FragmentPerms::NONE, |acc, &x| acc | x);
							resell.new_permissions = Some(
								vec_new_permissions_parameter
									.iter()
									.fold(FragmentPerms::NONE, |acc, &x| acc | x),
							);
							// Should Not Work
							mint_resell_instance(dd.account_id, &resell);
							assert_noop!(resell_(dd.account_id, &resell), Error::<Test>::NoPermission);
						});
					}
				}

				// Equally or More Restrictive
				for len_combo in 0..=vec_perms.len() {
					for vec_new_permissions_parameter in
					vec_perms.clone().into_iter().combinations(len_combo)
					{
						new_test_ext().execute_with(|| {
							let dd = DummyData::new();
							let mut resell = dd.resell_normal;
							resell.mint.definition.permissions = vec_perms
								.iter()
								.fold(FragmentPerms::NONE, |acc, &x| acc | x);
							resell.new_permissions = Some(
								vec_new_permissions_parameter
									.iter()
									.fold(FragmentPerms::NONE, |acc, &x| acc | x),
							);
							// Should Work
							mint_resell_instance(dd.account_id, &resell);
							assert_ok!(resell_(dd.account_id, &resell));
						});
					}
				}
			}
		}
	}

	#[test]
	fn resell_should_not_work_if_the_fragment_instance_does_not_have_the_transfer_permission() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let mut resell = dd.resell_normal;
			resell.mint.definition.permissions = FragmentPerms::ALL - FragmentPerms::TRANSFER; // does not have transfer permission
			mint_resell_instance(dd.account_id, &resell);
			assert_noop!(resell_(dd.account_id, &resell), Error::<Test>::NoPermission);
		});
	}

	#[test]
	fn resell_should_not_work_if_the_fragment_instance_expires_before_or_at_the_current_block_number() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let mut give = dd.give_copy_perms;
			assert!(give.expiration.is_some());
			give.new_permissions = Some(FragmentPerms::TRANSFER | FragmentPerms::COPY); // ensure that the copied instance can also be used to create copies
			mint_give_instance(dd.account_id, &give);

			// Should Work if Instance Has Not Expired
			assert_ok!(give_(dd.account_id, &give));
			let resell = Resell {
				mint: give.mint.clone(),
				edition_id: give.edition_id,
				copy_id: give.copy_id + 1,
				new_permissions: None,
				expiration: None,
				secondary_sale_type: SecondarySaleType::Normal(7),
			};
			run_to_block(give.expiration.unwrap() - 1);
			assert_ok!(resell_(give.to, &resell));

			// Should Not Work if Instance Has Expired
			assert_ok!(give_(dd.account_id, &give));
			let resell = Resell {
				mint: give.mint,
				edition_id: give.edition_id,
				copy_id: give.copy_id + 2,
				new_permissions: None,
				expiration: None,
				secondary_sale_type: SecondarySaleType::Normal(7),
			};
			run_to_block(give.expiration.unwrap());
			assert_noop!(resell_(give.to, &resell), Error::<Test>::NotFound);
		});
	}

	#[test]
	fn resell_should_not_work_if_the_expiration_parameter_is_lesser_than_or_equal_to_the_current_block_number() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let current_block_number = System::block_number();

			let resell = dd.resell_normal;

			mint_resell_instance(dd.account_id, &resell);

			assert_noop!(
				resell_(
					dd.account_id,
					&Resell {
						expiration: Some(current_block_number),
						..resell.clone()
					}
				),
				Error::<Test>::ParamsNotValid
			);
			assert_ok!(
				resell_(
					dd.account_id,
					&Resell {
						expiration: Some(current_block_number + 1),
						..resell
					}
				)
			);
		});
	}

	#[test]
	fn resell_should_not_change_the_expiration_of_the_copied_fragment_instance_if_the_expiration_parameter_is_greater_than_it(
	) {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let mut give = dd.give_copy_perms;
			assert!(give.expiration.is_some());
			give.new_permissions = Some(FragmentPerms::TRANSFER | FragmentPerms::COPY); // ensure that the copied instance can also be used to create copies

			assert_ok!(upload(dd.account_id, &give.mint.definition.proto_fragment));
			assert_ok!(create(dd.account_id, &give.mint.definition));
			assert_ok!(mint_(dd.account_id, &give.mint));
			assert_ok!(give_(dd.account_id, &give));

			assert_ok!(
				give_(
					give.to,
					&Give {
						copy_id: give.copy_id + 1,
						to: dd.account_id_second,
						expiration: Some(give.expiration.unwrap() - 1),
						..give.clone()
					}
				)
			);
			assert!(
				!<Expirations<Test>>::get(give.expiration.unwrap()).unwrap().contains(
					&(
						give.mint.definition.get_definition_id(),
						Compact(give.edition_id),
						Compact(give.copy_id + 2)
					)
				)
			);
			assert!(
				<Expirations<Test>>::get(give.expiration.unwrap() - 1).unwrap().contains(
					&(
						give.mint.definition.get_definition_id(),
						Compact(give.edition_id),
						Compact(give.copy_id + 2)
					)
				)
			);

			assert_ok!(
				give_(
					give.to,
					&Give {
						copy_id: give.copy_id + 1,
						to: dd.account_id_second,
						expiration: Some(give.expiration.unwrap() + 1),
						..give.clone()
					}
				)
			);
			assert!(!<Expirations<Test>>::contains_key(give.expiration.unwrap() + 1));
			assert!(
				<Expirations<Test>>::get(give.expiration.unwrap()).unwrap().contains(
					&(
						give.mint.definition.get_definition_id(),
						Compact(give.edition_id),
						Compact(give.copy_id + 3)
					)
				)
			);
		});
	}

}

mod end_resale_tests {
	use super::*;

	pub fn end_resale_(
		signer: <Test as frame_system::Config>::AccountId,
		end_resale: &EndResale
	) -> DispatchResult {
		FragmentsPallet::end_resale(
			Origin::signed(signer),
			end_resale.resell.mint.definition.get_definition_id(),
			end_resale.resell.edition_id,
			end_resale.resell.copy_id,
		)
	}

	pub fn resell_instance(
		signer: <Test as frame_system::Config>::AccountId,
		end_resale: &EndResale
	) {
		assert_ok!(upload(signer, &end_resale.resell.mint.definition.proto_fragment));
		assert_ok!(create(signer, &end_resale.resell.mint.definition));
		assert_ok!(mint_(signer, &end_resale.resell.mint));
		assert_ok!(resell_(signer, &end_resale.resell));
	}



	#[test]
	fn end_resale_should_work() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let end_resale = dd.end_resale;

			resell_instance(dd.account_id, &end_resale);
			assert_ok!(end_resale_(dd.account_id, &end_resale));

			System::assert_last_event(
				FragmentsEvent::EndResale {
					definition_hash: end_resale.resell.mint.definition.get_definition_id(),
					fragment_id: (end_resale.resell.edition_id, end_resale.resell.copy_id),
				}.into()
			);

			assert!(
				!Definition2SecondarySales::<Test>::contains_key((end_resale.resell.mint.definition.get_definition_id(), end_resale.resell.edition_id, end_resale.resell.copy_id))
			);
		});
	}

	#[test]
	fn end_resale_should_not_work_if_user_does_not_own_the_instance() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let end_resale = dd.end_resale;

			resell_instance(dd.account_id, &end_resale);

			assert_noop!(end_resale_(dd.account_id_second, &end_resale), Error::<Test>::NoPermission);
		});
	}

	#[test]
	fn end_resale_should_not_work_if_instance_not_on_sale() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let end_resale = dd.end_resale;

			assert_ok!(upload(dd.account_id, &end_resale.resell.mint.definition.proto_fragment));
			assert_ok!(create(dd.account_id, &end_resale.resell.mint.definition));
			assert_ok!(mint_(dd.account_id, &end_resale.resell.mint));

			assert_noop!(end_resale_(dd.account_id, &end_resale), Error::<Test>::NotFound);
		});
	}
}

mod secondary_buy_tests {
	use super::*;

	pub fn secondary_buy_(
		signer: <Test as frame_system::Config>::AccountId,
		secondary_buy: &SecondaryBuy
	) -> DispatchResult {
		FragmentsPallet::secondary_buy(
			Origin::signed(signer),
			secondary_buy.resell.mint.definition.get_definition_id(),
			secondary_buy.resell.edition_id,
			secondary_buy.resell.copy_id,
			secondary_buy.options.clone()
		)
	}

	pub fn resell_instance(
		signer: <Test as frame_system::Config>::AccountId,
		secondary_buy: &SecondaryBuy
	) {
		assert_ok!(upload(signer, &secondary_buy.resell.mint.definition.proto_fragment));
		assert_ok!(create(signer, &secondary_buy.resell.mint.definition));
		assert_ok!(mint_(signer, &secondary_buy.resell.mint));
		assert_ok!(resell_(signer, &secondary_buy.resell));
	}

	#[test]
	fn secondary_buy_should_work_if_the_fragment_instance_does_not_have_copy_permissions() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let secondary_buy = dd.secondary_buy_no_copy_perms;

			resell_instance(dd.account_id, &secondary_buy);

			let price = match secondary_buy.resell.secondary_sale_type {
				SecondarySaleType::Normal(price) => price,
			};
			let minimum_balance = <Balances as fungible::Inspect<
				<Test as frame_system::Config>::AccountId,
			>>::minimum_balance();
			_ = <Balances as fungible::Mutate<<Test as frame_system::Config>::AccountId>>::mint_into(
				&dd.account_id_second,
				price + minimum_balance,
			);

			assert_ok!(secondary_buy_(dd.account_id_second, &secondary_buy));

			System::assert_has_event(
				FragmentsEvent::InventoryRemoved {
					account_id: dd.account_id,
					definition_hash: secondary_buy.resell.mint.definition.get_definition_id(),
					fragment_id: (secondary_buy.resell.edition_id, secondary_buy.resell.copy_id),
				}.into()
			);
			System::assert_has_event(
				FragmentsEvent::InventoryAdded {
					account_id: dd.account_id_second,
					definition_hash: secondary_buy.resell.mint.definition.get_definition_id(),
					fragment_id: (secondary_buy.resell.edition_id, secondary_buy.resell.copy_id),
				}.into()
			);
			System::assert_has_event(
				pallet_balances::Event::Transfer {
					from: dd.account_id_second,
					to: dd.account_id,
					amount: price,
				}.into()
			);

			assert_eq!(
				<Owners<Test>>::get(secondary_buy.resell.mint.definition.get_definition_id(), dd.account_id)
					.unwrap()
					.contains(&(Compact(secondary_buy.resell.edition_id), Compact(secondary_buy.resell.copy_id))),
				false
			);
			assert_eq!(
				<Inventory<Test>>::get(dd.account_id, secondary_buy.resell.mint.definition.get_definition_id())
					.unwrap()
					.contains(&(Compact(secondary_buy.resell.edition_id), Compact(secondary_buy.resell.copy_id))),
				false
			);

			assert_eq!(
				<Owners<Test>>::get(secondary_buy.resell.mint.definition.get_definition_id(), dd.account_id_second)
					.unwrap()
					.contains(&(Compact(secondary_buy.resell.edition_id), Compact(secondary_buy.resell.copy_id))),
				true
			);
			assert_eq!(
				<Inventory<Test>>::get(dd.account_id_second, secondary_buy.resell.mint.definition.get_definition_id())
					.unwrap()
					.contains(&(Compact(secondary_buy.resell.edition_id), Compact(secondary_buy.resell.copy_id))),
				true
			);

			assert_eq!(
				<Fragments<Test>>::get((
					secondary_buy.resell.mint.definition.get_definition_id(),
					secondary_buy.resell.edition_id,
					secondary_buy.resell.copy_id
				))
					.unwrap()
					.permissions,
				secondary_buy.resell.new_permissions.unwrap()
			);
		});
	}

	#[test]
	fn secondary_buy_should_work_if_the_fragment_instance_has_copy_permissions() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let secondary_buy = dd.secondary_buy_copy_perms;

			resell_instance(dd.account_id, &secondary_buy);

			let price = match secondary_buy.resell.secondary_sale_type {
				SecondarySaleType::Normal(price) => price,
			};
			let minimum_balance = <Balances as fungible::Inspect<
				<Test as frame_system::Config>::AccountId,
			>>::minimum_balance();
			_ = <Balances as fungible::Mutate<<Test as frame_system::Config>::AccountId>>::mint_into(
				&dd.account_id_second,
				price + minimum_balance,
			);

			assert_ok!(secondary_buy_(dd.account_id_second, &secondary_buy));

			System::assert_has_event(
				FragmentsEvent::InventoryAdded {
					account_id: dd.account_id_second,
					definition_hash: secondary_buy.resell.mint.definition.get_definition_id(),
					fragment_id: (secondary_buy.resell.edition_id, secondary_buy.resell.copy_id + 1),
				}.into()
			);
			System::assert_has_event(
				pallet_balances::Event::Transfer {
					from: dd.account_id_second,
					to: dd.account_id,
					amount: price,
				}.into()
			);

			assert_eq!(
				<Owners<Test>>::get(secondary_buy.resell.mint.definition.get_definition_id(), dd.account_id)
					.unwrap()
					.contains(&(Compact(secondary_buy.resell.edition_id), Compact(secondary_buy.resell.copy_id))),
				true
			);
			assert_eq!(
				<Inventory<Test>>::get(dd.account_id, secondary_buy.resell.mint.definition.get_definition_id())
					.unwrap()
					.contains(&(Compact(secondary_buy.resell.edition_id), Compact(secondary_buy.resell.copy_id))),
				true
			);

			assert_eq!(
				<Owners<Test>>::get(secondary_buy.resell.mint.definition.get_definition_id(), dd.account_id_second)
					.unwrap()
					.contains(&(Compact(secondary_buy.resell.edition_id), Compact(secondary_buy.resell.copy_id + 1))),
				true
			);
			assert_eq!(
				<Inventory<Test>>::get(dd.account_id_second, secondary_buy.resell.mint.definition.get_definition_id())
					.unwrap()
					.contains(&(Compact(secondary_buy.resell.edition_id), Compact(secondary_buy.resell.copy_id + 1))),
				true
			);

			assert_eq!(
				<CopiesCount<Test>>::get((
					secondary_buy.resell.mint.definition.get_definition_id(),
					secondary_buy.resell.edition_id
				))
					.unwrap(),
				Compact(2)
			);

			assert_eq!(
				<Fragments<Test>>::get((
					secondary_buy.resell.mint.definition.get_definition_id(),
					secondary_buy.resell.edition_id,
					secondary_buy.resell.copy_id + 1
				))
					.unwrap()
					.permissions,
				secondary_buy.resell.new_permissions.unwrap()
			);
			assert_eq!(
				<Fragments<Test>>::get((
					secondary_buy.resell.mint.definition.get_definition_id(),
					secondary_buy.resell.edition_id,
					secondary_buy.resell.copy_id + 1
				))
					.unwrap()
					.expiring_at,
				secondary_buy.resell.expiration
			);

			assert!(<Expirations<Test>>::get(&secondary_buy.resell.expiration.unwrap()).unwrap().contains(&(
				secondary_buy.resell.mint.definition.get_definition_id(),
				Compact(secondary_buy.resell.edition_id),
				Compact(secondary_buy.resell.copy_id + 1)
			)));
		});
	}


	#[ignore = "Currently there is only one sale type. So we can't test this"]
	#[test]
	fn secondary_buy_should_not_work_if_options_param_does_not_match_sale_type() {
		new_test_ext().execute_with(|| {
			todo!("Currently there is only one sale type. So we can't test this");
		});
	}


	#[test]
	fn secondary_buy_should_not_work_if_user_has_insufficient_balance_in_pallet_balances() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let secondary_buy = dd.secondary_buy;

			resell_instance(dd.account_id, &secondary_buy);

			let price = match secondary_buy.resell.secondary_sale_type {
				SecondarySaleType::Normal(price) => price,
			};
			let minimum_balance = <Balances as fungible::Inspect<
				<Test as frame_system::Config>::AccountId,
			>>::minimum_balance();

			_ = <Balances as fungible::Mutate<<Test as frame_system::Config>::AccountId>>::mint_into(
				&dd.account_id_second,
				price + minimum_balance - 1,
			);
			assert_noop!(secondary_buy_(dd.account_id_second, &secondary_buy), Error::<Test>::InsufficientBalance);

			_ = <Balances as fungible::Mutate<<Test as frame_system::Config>::AccountId>>::mint_into(
				&dd.account_id_third,
				price + minimum_balance,
			);
			assert_ok!(secondary_buy_(dd.account_id_third, &secondary_buy));

		});
	}

	#[test]
	fn secondary_buy_should_not_work_if_user_has_insufficient_balance_in_pallet_assets() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let asset_id = 0;

			let mut secondary_buy = dd.secondary_buy;
			secondary_buy.resell.mint.definition.metadata.currency = Currency::Custom(asset_id);

			let minimum_balance = 1;
			assert_ok!(Assets::force_create(
				Origin::root(),
				asset_id, // The identifier of the new asset. This must not be currently in use to identify an existing asset.
				dd.account_id, // The owner of this class of assets. The owner has full superuser permissions over this asset, but may later change and configure the permissions using transfer_ownership and set_team.
				true,          // Whether this asset needs users to have an existential deposit to hold this asset
				minimum_balance, // The minimum balance of this new asset that any single account must have. If an account’s balance is reduced below this, then it collapses to zero.
				true
			));

			resell_instance(dd.account_id, &secondary_buy);

			let price = match secondary_buy.resell.secondary_sale_type {
				SecondarySaleType::Normal(price) => price,
			};

			assert_ok!(Assets::mint(
				Origin::signed(dd.account_id),
				asset_id,
				dd.account_id_second,
				price + minimum_balance - 1,
			));
			assert_noop!(secondary_buy_(dd.account_id_second, &secondary_buy), Error::<Test>::InsufficientBalance);

			assert_ok!(Assets::mint(
				Origin::signed(dd.account_id),
				asset_id,
				dd.account_id_third,
				price + minimum_balance,
			));
			assert_ok!(secondary_buy_(dd.account_id_third, &secondary_buy));

		});
	}

	#[ignore = "will the definition vault's account ever fall below minimum balance?"]
	#[test]
	fn secondary_buy_should_work_if_the_definition_vault_id_will_have_a_minimum_balance_of_the_asset_after_transaction(
	) {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let asset_id = 0;

			let mut secondary_buy = dd.secondary_buy;
			secondary_buy.resell.mint.definition.metadata.currency = Currency::Custom(asset_id);

			let price = match secondary_buy.resell.secondary_sale_type {
				SecondarySaleType::Normal(price) => price,
			};

			let minimum_balance = price;
			assert_ok!(Assets::force_create(
				Origin::root(),
				asset_id, // The identifier of the new asset. This must not be currently in use to identify an existing asset.
				dd.account_id, // The owner of this class of assets. The owner has full superuser permissions over this asset, but may later change and configure the permissions using transfer_ownership and set_team.
				true,          // Whether this asset needs users to have an existential deposit to hold this asset
				minimum_balance, // The minimum balance of this new asset that any single account must have. If an account’s balance is reduced below this, then it collapses to zero.
				true
			));

			resell_instance(dd.account_id, &secondary_buy);

			assert_ok!(Assets::mint(
				Origin::signed(dd.account_id),
				asset_id,
				dd.account_id_second,
				price + minimum_balance,
			));

			assert_ok!(secondary_buy_(dd.account_id_second, &secondary_buy));
		});
	}

	#[ignore = "will the definition vault's account ever fall below minimum balance?"]
	#[test]
	fn secondary_buy_should_not_work_if_the_definition_vault_id_will_not_have_a_minimum_balance_of_the_asset_after_transaction(
	) {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let asset_id = 0;

			let mut secondary_buy = dd.secondary_buy;
			secondary_buy.resell.mint.definition.metadata.currency = Currency::Custom(asset_id);

			let price = match secondary_buy.resell.secondary_sale_type {
				SecondarySaleType::Normal(price) => price,
			};

			let minimum_balance = price + 1;
			assert_ok!(Assets::force_create(
				Origin::root(),
				asset_id, // The identifier of the new asset. This must not be currently in use to identify an existing asset.
				dd.account_id, // The owner of this class of assets. The owner has full superuser permissions over this asset, but may later change and configure the permissions using transfer_ownership and set_team.
				true,          // Whether this asset needs users to have an existential deposit to hold this asset
				minimum_balance, // The minimum balance of this new asset that any single account must have. If an account’s balance is reduced below this, then it collapses to zero.
				true
			));

			resell_instance(dd.account_id, &secondary_buy);

			assert_ok!(Assets::mint(
				Origin::signed(dd.account_id),
				asset_id,
				dd.account_id_second,
				price + minimum_balance,
			));

			assert_noop!(
				secondary_buy_(dd.account_id_second, &secondary_buy),
				Error::<Test>::ReceiverBelowMinimumBalance
			);
		});
	}

}


mod detach_tests {
	use super::*;

	pub fn detach_(
		signer: <Test as frame_system::Config>::AccountId,
		detach: &Detach,
	) -> DispatchResult {
		FragmentsPallet::detach(
			Origin::signed(signer),
			detach.mint.definition.get_definition_id(),
			detach.edition_id,
			detach.copy_id,
			detach.target_chain,
			detach.target_account.clone().try_into().unwrap()
		)
	}

	pub fn mint_detach_instance(
		signer: <Test as frame_system::Config>::AccountId,
		detach: &Detach
	) {
		assert_ok!(upload(signer, &detach.mint.definition.proto_fragment));
		assert_ok!(create(signer, &detach.mint.definition));
		assert_ok!(mint_(signer, &detach.mint));
	}

	#[test]
	fn detach_should_work() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let detach = dd.detach;

			mint_detach_instance(dd.account_id, &detach);
			assert_ok!(detach_(dd.account_id, &detach));


			assert_eq!(
				pallet_detach::DetachRequests::<Test>::get(),
				vec![
					pallet_detach::DetachRequest {
						hash: pallet_detach::DetachHash::Instance(
							detach.mint.definition.get_definition_id(),
							Compact(detach.edition_id),
							Compact(detach.copy_id),
						),
						target_chain: detach.target_chain,
						target_account: detach.target_account,
					},
				]
			);

		});
	}

	#[test]
	fn detach_should_not_work_if_user_does_not_own_the_instance() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let detach = dd.detach;

			mint_detach_instance(dd.account_id, &detach);
			assert_noop!(detach_(dd.account_id_second, &detach), Error::<Test>::NoPermission);
		});
	}

	#[test]
	fn detach_should_not_work_if_the_instance_does_not_exist() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let detach = dd.detach;

			// REVIEW - error name
			assert_noop!(detach_(dd.account_id, &detach), Error::<Test>::NoPermission);
		});
	}

	#[test]
	fn detach_should_not_work_if_the_detach_request_already_exists() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let detach = dd.detach;

			mint_detach_instance(dd.account_id, &detach);
			assert_ok!(detach_(dd.account_id, &detach));
			assert_noop!(detach_(dd.account_id, &detach), Error::<Test>::DetachRequestAlreadyExists);
		});
	}

}


mod get_definitions_tests {
	use super::*;

	#[test]
	fn get_definitions_should_work() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let definition = dd.definition;

			assert_ok!(upload(dd.account_id, &definition.proto_fragment));
			assert_ok!(create(dd.account_id, &definition));

			assert_eq!(
				serde_json::from_slice::<Value>(
					&FragmentsPallet::get_definitions(GetDefinitionsParams {
						limit: u64::MAX,
						return_owners: true,
						..Default::default()
					})
						.unwrap()
				)
					.unwrap(),
				json!({
					hex::encode(definition.get_definition_id()): {
						"name": String::from_utf8(definition.metadata.name).unwrap(),
						"num_instances": 0,
						"owner": {
							"type": "internal",
							"value": hex::encode(dd.account_id)
						}
					}
				})
			);
		});
	}

	#[test]
	fn get_definitions_should_work_if_owner_owns_definitions() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();
			let definition = dd.definition;
			assert_ok!(upload(dd.account_id, &definition.proto_fragment));
			assert_ok!(create(dd.account_id, &definition));

			assert_eq!(
				serde_json::from_slice::<Value>(
					&FragmentsPallet::get_definitions(GetDefinitionsParams {
						limit: u64::MAX,
						owner: Some(dd.account_id),
						return_owners: true,
						..Default::default()
					})
						.unwrap()
				)
					.unwrap(),
				json!({
					hex::encode(definition.get_definition_id()): {
						"name": String::from_utf8(definition.metadata.name).unwrap(),
						"num_instances": 0,
						"owner": {
							"type": "internal",
							"value": hex::encode(dd.account_id)
						}
					}
				})
			);
		});
	}

	#[test]
	fn get_definitions_should_not_work_if_owner_does_not_own_definitions() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();
			let definition = dd.definition;
			assert_ok!(upload(dd.account_id, &definition.proto_fragment));
			assert_ok!(create(dd.account_id, &definition));

			assert_eq!(
				FragmentsPallet::get_definitions(GetDefinitionsParams {
					limit: u64::MAX,
					owner: Some(dd.account_id_second),
					return_owners: true,
					..Default::default()
				}),
				Err("Owner not found".into())
			);
		});
	}
}

mod get_instances_tests {
	use super::*;
	use serde_json::Map;

	#[test]
	fn get_instances_should_work() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();
			let mint = dd.mint_non_unique;
			assert_ok!(upload(dd.account_id, &mint.definition.proto_fragment));
			assert_ok!(create(dd.account_id, &mint.definition));
			assert_ok!(mint_(dd.account_id, &mint));

			let mut correct_map_instances = Map::new();
			for edition_id in 1..=mint.get_quantity() {
				correct_map_instances.insert(format!("{}.1", edition_id), Map::new().into());
			}

			assert_eq!(
				serde_json::from_slice::<Value>(
					&FragmentsPallet::get_instances(GetInstancesParams {
						definition_hash: hex::encode(mint.definition.get_definition_id())
							.into_bytes(),
						limit: u64::MAX,
						..Default::default()
					})
						.unwrap()
				)
					.unwrap(),
				json!(correct_map_instances)
			)
		});
	}

	#[test]
	fn get_instances_should_work_if_only_return_first_copies_is_true() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();
			let give = dd.give_copy_perms;
			assert_ok!(upload(dd.account_id, &give.mint.definition.proto_fragment));
			assert_ok!(create(dd.account_id, &give.mint.definition));
			assert_ok!(mint_(dd.account_id, &give.mint));
			assert_ok!(give_(dd.account_id, &give));

			assert_eq!(
				serde_json::from_slice::<Value>(
					&FragmentsPallet::get_instances(GetInstancesParams {
						definition_hash: hex::encode(give.mint.definition.get_definition_id())
							.into_bytes(),
						limit: u64::MAX,
						only_return_first_copies: true,
						..Default::default()
					})
						.unwrap()
				)
					.unwrap(),
				json!({
					"1.1": {},
				})
			);
		});
	}

	#[test]
	fn get_instances_should_work_if_only_return_first_copies_is_false() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();
			let give = dd.give_copy_perms;
			assert_ok!(upload(dd.account_id, &give.mint.definition.proto_fragment));
			assert_ok!(create(dd.account_id, &give.mint.definition));
			assert_ok!(mint_(dd.account_id, &give.mint));
			assert_ok!(give_(dd.account_id, &give));

			assert_eq!(
				serde_json::from_slice::<Value>(
					&FragmentsPallet::get_instances(GetInstancesParams {
						definition_hash: hex::encode(give.mint.definition.get_definition_id())
							.into(),
						limit: u64::MAX,
						only_return_first_copies: false,
						..Default::default()
					})
						.unwrap()
				)
					.unwrap(),
				json!({
					"1.1": {},
					"1.2": {},
				})
			);
		});
	}

	#[test]
	fn get_instances_should_work_if_owner_owns_instances() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();
			let give = dd.give_no_copy_perms;
			assert_ok!(upload(dd.account_id, &give.mint.definition.proto_fragment));
			assert_ok!(create(dd.account_id, &give.mint.definition));
			assert_ok!(mint_(dd.account_id, &give.mint));
			assert_ok!(give_(dd.account_id, &give));

			assert_eq!(
				serde_json::from_slice::<Value>(
					&FragmentsPallet::get_instances(GetInstancesParams {
						definition_hash: hex::encode(give.mint.definition.get_definition_id())
							.into(),
						limit: u64::MAX,
						owner: Some(give.to),
						..Default::default()
					})
						.unwrap()
				)
					.unwrap(),
				json!({
					format!("{}.{}", give.edition_id, give.copy_id): {}
				})
			);
		});
	}
}

mod get_instance_owner_tests {
	use super::*;

	#[test]
	fn get_instance_owner_should_work() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();
			let mint = dd.mint_non_unique;
			assert_ok!(upload(dd.account_id, &mint.definition.proto_fragment));
			assert_ok!(create(dd.account_id, &mint.definition));
			assert_ok!(mint_(dd.account_id, &mint));

			assert_eq!(
				FragmentsPallet::get_instance_owner(GetInstanceOwnerParams {
					definition_hash: hex::encode(mint.definition.get_definition_id()).into(),
					edition_id: 1,
					copy_id: 1,
				})
					.unwrap(),
				hex::encode(dd.account_id).into_bytes()
			);
		});
	}

	#[test]
	fn get_instance_owner_should_not_work_if_instance_does_not_exist() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();
			let mint = dd.mint_non_unique;
			assert_ok!(upload(dd.account_id, &mint.definition.proto_fragment));
			assert_ok!(create(dd.account_id, &mint.definition));

			assert_eq!(
				FragmentsPallet::get_instance_owner(GetInstanceOwnerParams {
					definition_hash: hex::encode(mint.definition.get_definition_id()).into(),
					edition_id: 1,
					copy_id: 1,
				}),
				Err("Instance not found".into())
			);
		});
	}
}
