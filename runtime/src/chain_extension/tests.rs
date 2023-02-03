#![cfg(test)]

use crate::chain_extension::mock::*;
use crate::chain_extension;
use crate::chain_extension::FuncId;

use frame_support::{assert_noop, assert_ok, dispatch::DispatchResult, BoundedVec};

use sp_fragnova::Hash256;
use protos::{
	categories::{Categories, TextCategories},
	permissions::FragmentPerms,
};

use pallet_protos::*;
use pallet_fragments::*;

#[test]
fn are_all_chain_extension_methods_tested() {

	let dummy_func_id = FuncId::GetProto;

	match dummy_func_id {
		FuncId::GetProto => protos_tests::get_proto_should_work(),
		FuncId::GetProtoIds => protos_tests::get_proto_ids_should_work(),
		FuncId::GetDefinition => fragments_tests::get_definition_should_work(),
		FuncId::GetInstance => fragments_tests::get_instance_should_work(),
		FuncId::GetInstanceIds => fragments_tests::get_instance_ids_should_work(),
		FuncId::GiveInstance => fragments_tests::give_instance_should_work(),
	};
}

use protos_tests::upload;
mod protos_tests {
	use super::*;

	pub fn upload(signer: <Test as frame_system::Config>::AccountId, proto_data: Vec<u8>) -> DispatchResult {
		ProtosPallet::upload(
			Origin::Signed(signer),
			Vec::<Hash256>::new(),
			Categories::Text(TextCategories::Plain),
			Vec::<BoundedVec<u8, _>>::new().try_into().unwrap(),
			None,
			UsageLicense::Closed,
			None,
			ProtoData::Local(proto_data.clone()),
		)
	}

	#[test]
	pub fn get_proto_should_work() {
		new_test_ext().execute_with(|| {

			assert_ok!(
				upload(
					sp_core::ed25519::Public::from_raw([1u8; 32]),
					b"Je suis Data".to_vec()
				)
			);



		});
	}

	#[test]
	pub fn get_proto_ids_should_work() {
		new_test_ext().execute_with(|| {

			assert_ok!(
				upload(
					sp_core::ed25519::Public::from_raw([1u8; 32]),
					b"Je suis Data".to_vec()
				)
			);
			assert_ok!(
				upload(
					sp_core::ed25519::Public::from_raw([1u8; 32]),
					b"Yo soy Data".to_vec()
				)
			);



		});
	}
}

mod fragments_tests {
	use sp_core::blake2_256;
	use super::*;

	fn create(signer: <Test as frame_system::Config>::AccountId, proto_hash: Hash256, name: Vec<u8>) -> DispatchResult {
		Fragments::<T>::create(
			Origin::Signed(signer),
			proto_hash,
			DefinitionMetadata::<BoundedVec<u8, _>, _> {
				name: name.try_into().unwrap(),
				currency: Currency::Native,
			},
			FragmentPerms::EDIT | FragmentPerms::TRANSFER,
			None,
			Some(7)
		)
	}

	#[test]
	pub fn get_definition_should_work() {
		new_test_ext().execute_with(|| {
			assert_ok!(
				upload(
					sp_core::ed25519::Public::from_raw([1u8; 32]),
					b"Je suis Data".to_vec()
				)
			);
			let proto_hash = blake2_256(&b"Je suis Data".to_vec());

			assert_ok!(
				create(
					sp_core::ed25519::Public::from_raw([1u8; 32]),
					proto_hash,
					b"Je suis un Nom".to_vec()
				)
			);

		});
	}

	#[test]
	pub fn get_instance_should_work() {
		new_test_ext().execute_with(|| {

		});
	}

	#[test]
	pub fn get_instance_ids_should_work() {
		new_test_ext().execute_with(|| {

		});
	}

	#[test]
	pub fn give_instance_should_work() {
		new_test_ext().execute_with(|| {

		});
	}
}
