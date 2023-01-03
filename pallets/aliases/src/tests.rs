#![cfg(test)]

use crate::{dummy_data::*, mock::*, *};
use crate::Event as AliasesEvent;

mod create_tests {
	use frame_support::assert_ok;
	use frame_support::dispatch::DispatchResult;
	use frame_support::traits::Currency;
	use sp_runtime::BoundedVec;
	use crate::Config;
	use crate::dummy_data::DummyData;
	use crate::mock::{AliasesPallet, new_test_ext, Origin, Test};

	pub fn create_namespace_(
		signer: <Test as frame_system::Config>::AccountId,
		name: Vec<u8>,
	) -> DispatchResult {
		let bounded_name: BoundedVec<u8, <Test as Config>::NameLimit> =
			name.clone().try_into().expect("alias name is too long");

		pallet_balances::Pallet::<Test>::make_free_balance_be(&signer, 1000000);
		AliasesPallet::create_namespace(Origin::signed(signer), bounded_name)
	}

	#[test]
	fn create_alias_should_work() {
		new_test_ext().execute_with(|| {
			let dummy = DummyData::new();
			let account_id = dummy.account_id;
			let namespace = b"DC".to_vec();

			assert_ok!(create_namespace_(account_id, namespace));
		});
	}
}
