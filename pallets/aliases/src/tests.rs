#![cfg(test)]

mod tests {
	use crate::{
		dummy_data::{get_root_namespace, DummyData},
		mock::{new_test_ext, AliasesPallet, RuntimeOrigin, System, Test},
		Config, Event as AliasesEvent, LinkTarget, Namespaces, *,
	};
	use frame_support::{
		assert_noop, assert_ok,
		dispatch::DispatchResult,
		traits::{Currency, Len},
	};
	use sp_runtime::{traits::TypedGet, BoundedVec, DispatchError::BadOrigin};

	pub fn prepare_balance(account_id: <Test as frame_system::Config>::AccountId) {
		pallet_balances::Pallet::<Test>::make_free_balance_be(&account_id, 1000000);
	}

	pub fn create_namespace_(
		signer: <Test as frame_system::Config>::AccountId,
		name: Vec<u8>,
	) -> DispatchResult {
		let bounded_name: BoundedVec<u8, <Test as Config>::NameLimit> =
			name.clone().try_into().expect("alias name is too long");
		AliasesPallet::create_namespace(RuntimeOrigin::signed(signer), bounded_name)
	}

	pub fn delete_namespace_(
		signer: <Test as frame_system::Config>::AccountId,
		name: Vec<u8>,
	) -> DispatchResult {
		let bounded_name: BoundedVec<u8, <Test as Config>::NameLimit> =
			name.clone().try_into().expect("alias name is too long");
		AliasesPallet::delete_namespace(RuntimeOrigin::signed(signer), bounded_name)
	}

	pub fn transfer_namespace_(
		signer: <Test as frame_system::Config>::AccountId,
		name: Vec<u8>,
		new_owner: <Test as frame_system::Config>::AccountId,
	) -> DispatchResult {
		let bounded_name: BoundedVec<u8, <Test as Config>::NameLimit> =
			name.clone().try_into().expect("alias name is too long");
		AliasesPallet::transfer_namespace(RuntimeOrigin::signed(signer), bounded_name, new_owner)
	}

	pub fn create_alias_(
		signer: <Test as frame_system::Config>::AccountId,
		namespace: Vec<u8>,
		alias: Vec<u8>,
		target: LinkTarget<<Test as frame_system::Config>::AccountId>,
		as_root: bool,
	) -> DispatchResult {
		let bounded_namespace: BoundedVec<u8, <Test as Config>::NameLimit> =
			namespace.clone().try_into().expect("namespace is too long");
		let bounded_alias: BoundedVec<u8, <Test as Config>::NameLimit> =
			alias.clone().try_into().expect("alias is too long");

		if as_root {
			AliasesPallet::create_root_alias(RuntimeOrigin::root(), bounded_alias, target)
		} else {
			AliasesPallet::create_alias(
				RuntimeOrigin::signed(signer),
				bounded_namespace,
				bounded_alias,
				target,
			)
		}
	}

	pub fn update_alias_target_(
		signer: <Test as frame_system::Config>::AccountId,
		namespace: Vec<u8>,
		alias: Vec<u8>,
		new_target: LinkTarget<<Test as frame_system::Config>::AccountId>,
		as_root: bool,
	) -> DispatchResult {
		let bounded_namespace: BoundedVec<u8, <Test as Config>::NameLimit> =
			namespace.clone().try_into().expect("namespace is too long");
		let bounded_alias: BoundedVec<u8, <Test as Config>::NameLimit> =
			alias.clone().try_into().expect("alias is too long");

		let origin = if as_root { RuntimeOrigin::root() } else { RuntimeOrigin::signed(signer) };

		AliasesPallet::update_alias_target(origin, bounded_namespace, bounded_alias, new_target)
	}

	#[test]
	fn create_namespace_should_work() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let account_id = dummy.account_id;
			let namespace = b"dc".to_vec();

			prepare_balance(account_id.clone());

			assert_ok!(create_namespace_(account_id, namespace.clone()));

			System::assert_has_event(
				AliasesEvent::NamespaceCreated { who: account_id, namespace: namespace.clone() }
					.into(),
			);
			System::assert_has_event(
				pallet_balances::Event::Withdraw {
					who: account_id,
					amount: <Test as Config>::NamespacePrice::get(),
				}
				.into(),
			);
		});
	}

	#[test]
	fn create_namespace_without_balance_should_fail() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let account_id = dummy.account_id;
			let namespace = b"dc".to_vec();

			assert_noop!(
				create_namespace_(account_id, namespace.clone()),
				pallet_balances::Error::<Test>::InsufficientBalance
			);
		});
	}

	#[test]
	fn create_namespace_with_uppercase_fails() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let account_id = dummy.account_id;
			let namespace = b"DC".to_vec();

			prepare_balance(account_id.clone());

			assert_noop!(
				create_namespace_(account_id, namespace.clone()),
				Error::<Test>::InvalidInput
			);
		});
	}

	#[test]
	fn create_double_namespace_should_fail() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let account_id = dummy.account_id;
			let namespace = b"test".to_vec();

			prepare_balance(account_id.clone());

			assert_ok!(create_namespace_(account_id, namespace.clone()));
			assert_noop!(
				create_namespace_(account_id.clone(), namespace.clone()),
				Error::<Test>::NamespaceExists
			);
			assert_eq!(<Namespaces<Test>>::get(&namespace).len(), 1);
		});
	}

	#[test]
	fn delete_namespace_should_work() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let account_id = dummy.account_id;
			let namespace = b"dc".to_vec();

			prepare_balance(account_id.clone());

			assert_ok!(create_namespace_(account_id, namespace.clone()));
			assert_ok!(delete_namespace_(account_id, namespace.clone()));
			assert!(!<Namespaces<Test>>::contains_key(&namespace));

			System::assert_has_event(
				AliasesEvent::NamespaceDeleted { namespace: namespace.clone() }.into(),
			);
		});
	}

	#[test]
	fn delete_namespace_should_delete_all_associated_aliases() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let account_id = dummy.account_id;
			let namespace = b"dc".to_vec();
			let alias = b"batman".to_vec();
			let target = LinkTarget::Account(account_id);

			prepare_balance(account_id.clone());

			assert_ok!(create_namespace_(account_id.clone(), namespace.clone()));
			assert_ok!(create_alias_(
				account_id.clone(),
				namespace.clone(),
				alias.clone(),
				target,
				false
			));
			assert_ok!(delete_namespace_(account_id, namespace.clone()));
			assert!(!<Namespaces<Test>>::contains_key(&namespace));
			let alias_index = Pallet::<Test>::get_name_index(&alias).unwrap();
			assert!(!<Aliases<Test>>::contains_key(&namespace, alias_index));
		});
	}

	#[test]
	fn transfer_namespace_should_work() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let account_id = dummy.account_id;
			let new_owner = dummy.account_id_2;
			let namespace = b"dc".to_vec();

			prepare_balance(account_id.clone());

			assert_ok!(create_namespace_(account_id.clone(), namespace.clone()));
			assert_ok!(transfer_namespace_(
				account_id.clone(),
				namespace.clone(),
				new_owner.clone()
			));

			System::assert_has_event(
				AliasesEvent::NamespaceTransferred {
					namespace: namespace.clone(),
					from: account_id.clone(),
					to: new_owner.clone(),
				}
				.into(),
			);
			assert_eq!(<Namespaces<Test>>::get(&namespace).unwrap(), new_owner);
		});
	}

	// transfer namespace should fail if owner is not the signer
	#[test]
	fn transfer_namespace_should_fail() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let account_id = dummy.account_id;
			let new_owner = dummy.account_id_2;
			let namespace = b"dc".to_vec();

			prepare_balance(account_id.clone());
			prepare_balance(new_owner.clone());

			assert_ok!(create_namespace_(account_id.clone(), namespace.clone()));
			assert_noop!(
				transfer_namespace_(new_owner.clone(), namespace.clone(), account_id.clone()),
				Error::<Test>::NotAllowed
			);
		});
	}

	#[test]
	fn create_alias_should_work() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let account_id = dummy.account_id;
			let namespace = b"dc".to_vec();
			let alias = b"batman".to_vec();
			let target = LinkTarget::Account(account_id);

			prepare_balance(account_id.clone());

			assert_ok!(create_namespace_(account_id.clone(), namespace.clone()));
			assert_ok!(create_alias_(
				account_id.clone(),
				namespace.clone(),
				alias.clone(),
				target,
				false
			));

			System::assert_has_event(
				AliasesEvent::AliasCreated {
					who: account_id.clone(),
					namespace: namespace.clone(),
					alias: alias.clone(),
				}
				.into(),
			);
			let alias_index = Pallet::<Test>::get_name_index(&alias).unwrap();
			let stored_alias = <Aliases<Test>>::get(&namespace, &alias_index).unwrap();
			assert_eq!(stored_alias.cur_block_number, System::block_number());
			assert_eq!(stored_alias.prev_block_number, 0)
		});
	}

	// create alias should fail if invalid characters are used
	#[test]
	fn create_alias_should_fail() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let account_id = dummy.account_id;
			let namespace = b"dc".to_vec();
			let alias = b"batman@".to_vec();
			let target = LinkTarget::Account(account_id);

			prepare_balance(account_id.clone());

			assert_ok!(create_namespace_(account_id.clone(), namespace.clone()));
			assert_noop!(
				create_alias_(account_id.clone(), namespace.clone(), alias.clone(), target, false),
				Error::<Test>::InvalidInput
			);
		});
	}

	#[test]
	fn create_root_alias_should_work() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let account_id = dummy.account_id;
			let alias = b"batman".to_vec();
			let namespace = b"dc".to_vec();
			let target = LinkTarget::Account(account_id);
			let root_namespace = get_root_namespace();

			assert_ok!(create_alias_(
				account_id.clone(),
				namespace.clone(),
				alias.clone(),
				target,
				true // as root
			));

			let alias_index = Pallet::<Test>::get_name_index(&alias).unwrap();
			assert!(<Aliases<Test>>::contains_key(&root_namespace, &alias_index));

			System::assert_has_event(
				AliasesEvent::RootAliasCreated { root_namespace, alias: alias.clone() }.into(),
			);
		});
	}

	#[test]
	fn edit_alias_target_should_work() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let account_id = dummy.account_id;
			let account_id_2 = dummy.account_id_2;
			let namespace = b"dc".to_vec();
			let alias = b"batman".to_vec();
			let target = LinkTarget::Account(account_id);
			let new_target = LinkTarget::Account(account_id_2);

			prepare_balance(account_id.clone());

			assert_ok!(create_namespace_(account_id.clone(), namespace.clone()));
			assert_ok!(create_alias_(
				account_id.clone(),
				namespace.clone(),
				alias.clone(),
				target.clone(),
				false
			));
			assert_ok!(update_alias_target_(
				account_id.clone(),
				namespace.clone(),
				alias.clone(),
				new_target.clone(),
				false
			));
			assert_noop!(
				update_alias_target_(
					account_id.clone(),
					namespace.clone(),
					alias.clone(),
					new_target.clone(),
					true
				),
				BadOrigin
			);

			let alias_index = Pallet::<Test>::get_name_index(&alias).unwrap();
			let stored_alias = <Aliases<Test>>::get(&namespace, &alias_index).unwrap();

			let current_block_number = <frame_system::Pallet<Test>>::block_number();
			let new_target_versioned = LinkTargetVersioned {
				link_target: new_target.clone(),
				prev_block_number: stored_alias.cur_block_number,
				cur_block_number: current_block_number,
			};
			assert_eq!(
				<Aliases<Test>>::get(&namespace, &alias_index).unwrap(),
				new_target_versioned
			);
		});
	}
}
