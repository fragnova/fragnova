use crate::{mock::*, Definitions, Error, FragmentMetadata, Proto2Fragments};
use codec::Encode;
use frame_support::{assert_noop, assert_ok};
use protos::categories::{Categories, TextCategories};
use protos::permissions::FragmentPerms;
use sp_io::hashing::blake2_128;
use pallet_protos::UsageLicense;

fn initial_upload() {
	let data = DATA.as_bytes().to_vec();
	let references = vec![];

	assert_ok!(ProtosPallet::upload(
		Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC)),
		references,
		Categories::Text(TextCategories::Plain),
		Vec::new(),
		None,
		UsageLicense::Closed,
		data,
	));
}

#[test]
fn create_should_works() {
	new_test_ext().execute_with(|| {
		initial_upload();

		let fragment_data = FragmentMetadata { name: "name".as_bytes().to_vec(), currency: None };

		let hash = blake2_128(
			&[&PROTO_HASH[..], &fragment_data.name.encode(), &fragment_data.currency.encode()]
				.concat(),
		);

		assert_ok!(FragmentsPallet::create(
			Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC)),
			PROTO_HASH,
			fragment_data,
			FragmentPerms::NONE,
			None,
			None
		));
		assert!(Definitions::<Test>::contains_key(&hash));
		assert!(Proto2Fragments::<Test>::contains_key(&PROTO_HASH));
	});
}

#[test]
fn create_should_not_work_if_protos_not_found() {
	new_test_ext().execute_with(|| {
		let fragment_data = FragmentMetadata { name: "name".as_bytes().to_vec(), currency: None };

		assert_noop!(
			FragmentsPallet::create(
				Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC)),
				PROTO_HASH,
				fragment_data,
				FragmentPerms::NONE,
				None,
				None
			),
			Error::<Test>::ProtoNotFound
		);
	});
}

#[test]
fn create_should_not_work_if_proto_owner_not_found() {
	new_test_ext().execute_with(|| {
		initial_upload();

		pub const PUBLIC: [u8; 32] = [
			136, 65, 23, 149, 81, 74, 241, 98, 119, 101, 236, 239, 252, 189, 0, 39, 25, 240, 49,
			96, 79, 173, 215, 209, 136, 226, 220, 88, 91, 78, 26, 251,
		];

		let fragment_data = FragmentMetadata { name: "name".as_bytes().to_vec(), currency: None };

		assert_noop!(
			FragmentsPallet::create(
				Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC)),
				PROTO_HASH,
				fragment_data,
				FragmentPerms::NONE,
				None,
				None
			),
			Error::<Test>::NoPermission
		);
	});
}

#[test]
fn create_should_not_work_if_fragment_already_exist() {
	new_test_ext().execute_with(|| {
		initial_upload();

		let fragment_data = FragmentMetadata { name: "name".as_bytes().to_vec(), currency: None };

		let hash = blake2_128(
			&[&PROTO_HASH[..], &fragment_data.name.encode(), &fragment_data.currency.encode()]
				.concat(),
		);

		assert_ok!(FragmentsPallet::create(
			Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC)),
			PROTO_HASH,
			fragment_data.clone(),
			FragmentPerms::NONE,
			None,
			None
		));
		assert!(Definitions::<Test>::contains_key(&hash));

		assert_noop!(
			FragmentsPallet::create(
				Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC)),
				PROTO_HASH,
				fragment_data,
				FragmentPerms::NONE,
				None,
				None
			),
			Error::<Test>::AlreadyExist
		);
	});
}
