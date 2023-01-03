#![cfg(test)]

use crate::*;

mod create_tests {
	use crate::{dummy_data::DummyData, mock::{new_test_ext, AliasesPallet, Origin, System, Test}, Config, Event as AliasesEvent, Namespaces};
	use frame_support::{assert_ok, dispatch::DispatchResult, traits::Currency};
	use sp_runtime::BoundedVec;

	pub fn create_namespace_(
		signer: <Test as frame_system::Config>::AccountId,
		name: Vec<u8>,
	) -> DispatchResult {
		let bounded_name: BoundedVec<u8, <Test as Config>::NameLimit> =
			name.clone().try_into().expect("alias name is too long");

		pallet_balances::Pallet::<Test>::make_free_balance_be(&signer, 1000000);
		AliasesPallet::create_namespace(Origin::signed(signer), bounded_name)
	}

	pub fn transfer_namespace_(
		signer: <Test as frame_system::Config>::AccountId,
		name: Vec<u8>,
		new_owner: <Test as frame_system::Config>::AccountId,
	) -> DispatchResult {
		let bounded_name: BoundedVec<u8, <Test as Config>::NameLimit> =
			name.clone().try_into().expect("alias name is too long");

		pallet_balances::Pallet::<Test>::make_free_balance_be(&signer, 1000000);
		AliasesPallet::transfer_namespace(Origin::signed(signer), bounded_name, new_owner)
	}

	#[test]
	fn create_namespace_should_work() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let account_id = dummy.account_id;
			let namespace = b"DC".to_vec();

			assert_ok!(create_namespace_(account_id, namespace.clone()));

			System::assert_has_event(AliasesEvent::NamespaceCreated {
				who: account_id, namespace: namespace.clone() }.into(),
			);
		});
	}

	#[test]
	fn transfer_namespace_should_work() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let account_id = dummy.account_id;
			let new_owner = dummy.account_id;
			let namespace = b"DC".to_vec();

			assert_ok!(create_namespace_(account_id.clone(), namespace.clone()));
			assert_ok!(transfer_namespace_(
				account_id.clone(),
				namespace.clone(),
				new_owner.clone()
			));

			System::assert_has_event(AliasesEvent::NamespaceTransferred {
				namespace: namespace.clone(), from: account_id.clone(), to: new_owner.clone()}.into(),
			);
			assert_eq!(<Namespaces<Test>>::get(&namespace).unwrap(), new_owner);
		});
	}
}
