#![cfg(test)]

use crate::chain_extension::mock::*;
use crate::chain_extension::FuncId;

use sp_core::{blake2_128, blake2_256};


use frame_support::{
	assert_ok,
	dispatch::DispatchResult,
	BoundedVec,
	weights::Weight,
};

use sp_runtime::{traits::Hash};

use codec::{Encode, Compact};
use frame_support::traits::Currency;

use sp_fragnova::Hash256;
use protos::{
	categories::{Categories, TextCategories},
	permissions::FragmentPerms,
};

pub const ALICE: sp_core::ed25519::Public = sp_core::ed25519::Public([1u8; 32]);
pub const BOB: sp_core::ed25519::Public = sp_core::ed25519::Public([2u8; 32]);
/// Copied from https://github.com/fragcolor-xyz/substrate/blob/fragnova-v0.0.6/frame/contracts/src/tests.rs#L387
pub const GAS_LIMIT: Weight = 100_000_000_000;

fn upload_dummy_contract(signer: <Test as frame_system::Config>::AccountId) -> <<Test as frame_system::Config>::Hashing as Hash>::Output {

	// We need to give `ALICE` some NOVA otherwise when we upload the contract, we'll get the error `StorageDepositNotEnoughFunds`.
	let _ = Balances::deposit_creating(&signer, 1_000_000); // I copied this line of code from https://github.com/fragcolor-xyz/substrate/blob/fragnova-v0.0.6/frame/contracts/src/tests.rs#L490

	let wasm_binary = std::fs::read("./src/chain_extension/dummy_contract/target/ink/dummy_contract.wasm").unwrap();
	let code_hash = <Test as frame_system::Config>::Hashing::hash(&wasm_binary);
	assert_ok!(
		Contracts::instantiate_with_code(
			Origin::signed(signer),
			0, // The balance to transfer from the `origin` to the newly created contract.
			GAS_LIMIT, // The gas limit enforced when executing the constructor.
			None, // The maximum amount of balance that can be charged/reserved from the caller to pay for the storage consumed.
			wasm_binary, // The contract code to deploy in raw bytes.
			blake2_256(b"new")[0..4].to_vec(), // The input data to pass to the contract constructor.
			vec![], // `salt` parameter. Used for the address derivation. See `pallet_contracts::Pallet::contract_address`
		)
	);
	// assert_ok!(
	// 	Contracts::bare_upload_code(
	// 		signer,
	// 		wasm_binary,
	// 		None
	// 	)
	// );
	code_hash
}

// If a new Enum Variant is added to `FuncId`, this function won't even compile!
#[test]
fn are_all_chain_extension_methods_tested() {

	let dummy_func_id = FuncId::GetProto;

	match dummy_func_id {
		FuncId::GetProto => protos_tests::get_proto_should_work,
		FuncId::GetProtoIds => protos_tests::get_proto_ids_should_work,
		FuncId::GetDefinition => fragments_tests::get_definition_should_work,
		FuncId::GetInstance => fragments_tests::get_instance_should_work,
		FuncId::GetInstanceIds => fragments_tests::get_instance_ids_should_work,
		FuncId::GiveInstance => fragments_tests::give_instance_should_work,
	};
}

use protos_tests::upload;
mod protos_tests {
	use super::*;

	pub fn upload(signer: <Test as frame_system::Config>::AccountId, proto_data: &Vec<u8>) -> DispatchResult {
		Protos::upload(
			Origin::signed(signer),
			Vec::<Hash256>::new(), // references
			Categories::Text(TextCategories::Plain), // category
			Vec::<BoundedVec<u8, _>>::new().try_into().unwrap(), // tags
			None, // linked_asset
			pallet_protos::UsageLicense::Closed, // license
			None, // cluster
			pallet_protos::ProtoData::Local(proto_data.clone()), //data
		)
	}

	#[test]
	pub fn get_proto_should_work() {
		new_test_ext().execute_with(|| {

			let code_hash = upload_dummy_contract(ALICE);
			let contract_address = Contracts::contract_address(&ALICE, &code_hash, &[]);

			let proto_data = b"Je suis Data".to_vec();
			assert_ok!(upload(ALICE, &proto_data));
			let proto_hash = blake2_256(&proto_data);

			let contract_result = Contracts::bare_call(
				ALICE,
				contract_address, // Address of the contract to call.
				0, // The balance to transfer from the origin to dest.
				GAS_LIMIT, // The gas limit enforced when executing the constructor.
				None, // The maximum amount of balance that can be charged from the caller to pay for the storage consumed.
				vec![&blake2_256(b"get_proto")[0..4], &proto_hash.encode()[..]].concat(), // The input data to pass to the contract.
				false // `debug` should only ever be set to true when executing as an RPC because it adds allocations and could be abused to drive the runtime into an OOM panic.
			);

			println!("the contract result is: {:?}", contract_result);

			assert_eq!(contract_result.result.as_ref().unwrap().flags.bits(), 0);
			assert_eq!(
				contract_result.result.unwrap().data.0,
				Some(pallet_protos::Protos::<Test>::get(&proto_hash).unwrap()).encode()
			);

		});
	}

	#[test]
	pub fn get_proto_ids_should_work() {
		new_test_ext().execute_with(|| {

			let code_hash = upload_dummy_contract(ALICE);
			let contract_address = Contracts::contract_address(&ALICE, &code_hash, &[]);

			let proto_data = b"Je suis Data".to_vec();
			assert_ok!(upload(ALICE, &proto_data));
			let proto_hash = blake2_256(&proto_data);

			let proto_data_second = b"Yo soy Data".to_vec();
			assert_ok!(upload(ALICE, &proto_data_second));
			let proto_hash_second = blake2_256(&proto_data_second);

			let contract_result = Contracts::bare_call(
				ALICE,
				contract_address, // Address of the contract to call.
				0, // The balance to transfer from the origin to dest.
				GAS_LIMIT, // The gas limit enforced when executing the constructor.
				None, // The maximum amount of balance that can be charged from the caller to pay for the storage consumed.
				vec![&blake2_256(b"get_proto_ids")[0..4], &ALICE.encode()[..]].concat(), // The input data to pass to the contract.
				false // `debug` should only ever be set to true when executing as an RPC because it adds allocations and could be abused to drive the runtime into an OOM panic.
			);

			assert_eq!(contract_result.result.as_ref().unwrap().flags.bits(), 0);
			assert_eq!(
				contract_result.result.unwrap().data.0,
				vec![proto_hash, proto_hash_second].encode()
			);

		});
	}
}

mod fragments_tests {
	use super::*;

	fn create(signer: <Test as frame_system::Config>::AccountId, proto_hash: &Hash256, definition_name: &Vec<u8>) -> DispatchResult {
		Fragments::create(
			Origin::signed(signer),
			proto_hash.clone(),
			pallet_fragments::DefinitionMetadata::<BoundedVec<u8, _>, _> {
				name: definition_name.clone().try_into().unwrap(),
				currency: pallet_fragments::Currency::Native,
			},
			FragmentPerms::EDIT | FragmentPerms::TRANSFER,
			None,
			None
		)
	}

	#[test]
	pub fn get_definition_should_work() {
		new_test_ext().execute_with(|| {

			let code_hash = upload_dummy_contract(ALICE);
			let contract_address = Contracts::contract_address(&ALICE, &code_hash, &[]);

			let proto_data = b"Je suis Data".to_vec();
			assert_ok!(upload(ALICE, &proto_data));
			let proto_hash = blake2_256(&proto_data);

			let definition_name = b"Je suis un Nom".to_vec();
			assert_ok!(create(ALICE, &proto_hash, &definition_name));
			let definition_hash = blake2_128(&[&proto_hash[..], &definition_name.encode(), &pallet_fragments::Currency::<<Test as pallet_assets::Config>::AssetId>::Native.encode()].concat());

			let contract_result = Contracts::bare_call(
				ALICE,
				contract_address, // Address of the contract to call.
				0, // The balance to transfer from the origin to dest.
				GAS_LIMIT, // The gas limit enforced when executing the constructor.
				None, // The maximum amount of balance that can be charged from the caller to pay for the storage consumed.
				vec![&blake2_256(b"get_definition")[0..4], &definition_hash.encode()[..]].concat(), // The input data to pass to the contract.
				false // `debug` should only ever be set to true when executing as an RPC because it adds allocations and could be abused to drive the runtime into an OOM panic.
			);

			assert_eq!(contract_result.result.as_ref().unwrap().flags.bits(), 0);
			assert_eq!(
				contract_result.result.unwrap().data.0,
				Some(pallet_fragments::Definitions::<Test>::get(&definition_hash).unwrap()).encode()
			);

		});
	}

	#[test]
	pub fn get_instance_should_work() {
		new_test_ext().execute_with(|| {

			let code_hash = upload_dummy_contract(ALICE);
			let contract_address = Contracts::contract_address(&ALICE, &code_hash, &[]);

			let proto_data = b"Je suis Data".to_vec();
			assert_ok!(upload(ALICE, &proto_data));
			let proto_hash = blake2_256(&proto_data);

			let definition_name = b"Je suis un Nom".to_vec();
			assert_ok!(create(ALICE, &proto_hash, &definition_name));
			let definition_hash = blake2_128(&[&proto_hash[..], &definition_name.encode(), &pallet_fragments::Currency::<<Test as pallet_assets::Config>::AssetId>::Native.encode()].concat());

			assert_ok!(
				Fragments::mint(
					Origin::signed(ALICE),
					definition_hash,
					pallet_fragments::FragmentBuyOptions::Quantity(1),
					None,
				)
			);

			let contract_result = Contracts::bare_call(
				ALICE,
				contract_address, // Address of the contract to call.
				0, // The balance to transfer from the origin to dest.
				GAS_LIMIT, // The gas limit enforced when executing the constructor.
				None, // The maximum amount of balance that can be charged from the caller to pay for the storage consumed.
				vec![&blake2_256(b"get_instance")[0..4], &(definition_hash, 1u64, 1u64).encode()[..]].concat(), // The input data to pass to the contract.
				false // `debug` should only ever be set to true when executing as an RPC because it adds allocations and could be abused to drive the runtime into an OOM panic.
			);

			assert_eq!(contract_result.result.as_ref().unwrap().flags.bits(), 0);
			assert_eq!(
				contract_result.result.unwrap().data.0,
				Some(pallet_fragments::Fragments::<Test>::get(&(definition_hash, 1, 1)).unwrap()).encode()
			);

		});
	}

	#[test]
	pub fn get_instance_ids_should_work() {
		new_test_ext().execute_with(|| {

			let code_hash = upload_dummy_contract(ALICE);
			let contract_address = Contracts::contract_address(&ALICE, &code_hash, &[]);

			let proto_data = b"Je suis Data".to_vec();
			assert_ok!(upload(ALICE, &proto_data));
			let proto_hash = blake2_256(&proto_data);

			let definition_name = b"Je suis un Nom".to_vec();
			assert_ok!(create(ALICE, &proto_hash, &definition_name));
			let definition_hash = blake2_128(&[&proto_hash[..], &definition_name.encode(), &pallet_fragments::Currency::<<Test as pallet_assets::Config>::AssetId>::Native.encode()].concat());

			assert_ok!(
				Fragments::mint(
					Origin::signed(ALICE),
					definition_hash,
					pallet_fragments::FragmentBuyOptions::Quantity(10),
					None,
				)
			);

			let contract_result = Contracts::bare_call(
				ALICE,
				contract_address, // Address of the contract to call.
				0, // The balance to transfer from the origin to dest.
				GAS_LIMIT, // The gas limit enforced when executing the constructor.
				None, // The maximum amount of balance that can be charged from the caller to pay for the storage consumed.
				vec![&blake2_256(b"get_instance_ids")[0..4], &(definition_hash, ALICE).encode()[..]].concat(), // The input data to pass to the contract.
				false // `debug` should only ever be set to true when executing as an RPC because it adds allocations and could be abused to drive the runtime into an OOM panic.
			);

			assert_eq!(contract_result.result.as_ref().unwrap().flags.bits(), 0);
			assert_eq!(
				contract_result.result.unwrap().data.0,
				pallet_fragments::Inventory::<Test>::get(ALICE, definition_hash).unwrap().encode()
			);

		});
	}

	#[test]
	pub fn give_instance_should_work() {
		new_test_ext().execute_with(|| {

			let code_hash = upload_dummy_contract(ALICE);
			let contract_address = Contracts::contract_address(&ALICE, &code_hash, &[]);

			let proto_data = b"Je suis Data".to_vec();
			assert_ok!(upload(ALICE, &proto_data));
			let proto_hash = blake2_256(&proto_data);

			let definition_name = b"Je suis un Nom".to_vec();
			assert_ok!(create(ALICE, &proto_hash, &definition_name));
			let definition_hash = blake2_128(&[&proto_hash[..], &definition_name.encode(), &pallet_fragments::Currency::<<Test as pallet_assets::Config>::AssetId>::Native.encode()].concat());

			assert_ok!(
				Fragments::mint(
					Origin::signed(ALICE),
					definition_hash,
					pallet_fragments::FragmentBuyOptions::Quantity(1),
					None,
				)
			);
			assert_ok!(
				Fragments::give(
					Origin::signed(ALICE),
					definition_hash, // definition_hash
					1, // edition_id
					1, // copy_id
					contract_address, // to
					None, // new_permissions
					None, // expiration
				)
			);

			let contract_result = Contracts::bare_call(
				ALICE,
				contract_address, // Address of the contract to call.
				0, // The balance to transfer from the origin to dest.
				GAS_LIMIT, // The gas limit enforced when executing the constructor.
				None, // The maximum amount of balance that can be charged from the caller to pay for the storage consumed.
				vec![&blake2_256(b"give_instance")[0..4], &(definition_hash, 1u64, 1u64, BOB, None::<Option<FragmentPerms>>, None::<Option<<Test as frame_system::Config>::BlockNumber>>).encode()[..]].concat(), // The input data to pass to the contract.
				false // `debug` should only ever be set to true when executing as an RPC because it adds allocations and could be abused to drive the runtime into an OOM panic.
			);

			assert_eq!(contract_result.result.as_ref().unwrap().flags.bits(), 0);
			// assert_eq!(
			// 	contract_result.result.unwrap().data.0,
			// 	().encode()
			// );

			assert_eq!(
				pallet_fragments::Inventory::<Test>::get(BOB, definition_hash).unwrap().contains(&(Compact(1), Compact(1))),
				true
			);

		});
	}
}
