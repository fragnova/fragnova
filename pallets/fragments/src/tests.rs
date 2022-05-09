use crate::{mock::*, Error, FragmentMetadata, Fragments, Proto2Fragments};
use codec::Encode;
use frame_support::{assert_noop, assert_ok};
use pallet_protos::{AuthData, Categories, ChainCategories, LinkedAsset};
use sp_clamor::Hash256;
use sp_core::Pair;
use sp_io::hashing::blake2_256;
use sp_std::collections::btree_set::BTreeSet;

fn initial_set_up_and_get_signature(
	data: Vec<u8>,
	references: Vec<Hash256>,
	nonce: u64,
) -> sp_core::ecdsa::Signature {
	let pair = sp_core::ecdsa::Pair::from_string("//Charlie", None).unwrap();
	let categories = (Categories::Chain(ChainCategories::Generic), <BTreeSet<Vec<u8>>>::new());

	let proto_hash = blake2_256(&data);
	let linked_asset: Option<LinkedAsset> = None;
	let signature: sp_core::ecdsa::Signature = pair.sign(
		&[
			&proto_hash[..],
			&references.encode(),
			&categories.encode(),
			&linked_asset.encode(),
			&nonce.encode(),
			&1.encode(),
		]
		.concat(),
	);
	assert_ok!(ProtosPallet::add_upload_auth(Origin::root(), pair.public()));
	signature
}

fn initial_upload_and_get_signature() -> AuthData {
	let data = DATA.as_bytes().to_vec();
	let references = vec![];
	let signature = initial_set_up_and_get_signature(data.clone(), references.clone(), 0);
	let auth_data = AuthData { signature, block: 1 };

	assert_ok!(ProtosPallet::upload(
		Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC)),
		auth_data.clone(),
		references,
		(Categories::Chain(ChainCategories::Generic), BTreeSet::new()),
		None,
		None,
		data,
	));
	auth_data
}

#[test]
fn create_should_works() {
	new_test_ext().execute_with(|| {
		initial_upload_and_get_signature();

		let fragment_data = FragmentMetadata {
			name: "name".as_bytes().to_vec(),
			external_url: "external_url".as_bytes().to_vec(),
		};

		let hash = blake2_256(&[&PROTO_HASH[..], &fragment_data.name.encode()].concat());

		assert_ok!(FragmentsPallet::create(
			Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC)),
			PROTO_HASH,
			fragment_data,
			true,
			true,
			None
		));
		assert!(Fragments::<Test>::contains_key(&hash));
		assert!(Proto2Fragments::<Test>::contains_key(&PROTO_HASH));
	});
}

#[test]
fn create_should_not_work_if_protos_not_found() {
	new_test_ext().execute_with(|| {
		let fragment_data = FragmentMetadata {
			name: "name".as_bytes().to_vec(),
			external_url: "external_url".as_bytes().to_vec(),
		};

		assert_noop!(
			FragmentsPallet::create(
				Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC)),
				PROTO_HASH,
				fragment_data,
				true,
				true,
				None
			),
			Error::<Test>::ProtoNotFound
		);
	});
}

#[test]
fn create_should_not_work_if_proto_owner_not_found() {
	new_test_ext().execute_with(|| {
		initial_upload_and_get_signature();
		pub const PUBLIC: [u8; 32] = [
			136, 65, 23, 149, 81, 74, 241, 98, 119, 101, 236, 239, 252, 189, 0, 39, 25, 240, 49,
			96, 79, 173, 215, 209, 136, 226, 220, 88, 91, 78, 26, 251,
		];

		let fragment_data = FragmentMetadata {
			name: "name".as_bytes().to_vec(),
			external_url: "external_url".as_bytes().to_vec(),
		};

		assert_noop!(
			FragmentsPallet::create(
				Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC)),
				PROTO_HASH,
				fragment_data,
				true,
				true,
				None
			),
			Error::<Test>::NoPermission
		);
	});
}

#[test]
fn create_should_not_work_if_fragment_already_exist() {
	new_test_ext().execute_with(|| {
		initial_upload_and_get_signature();

		let fragment_data = FragmentMetadata {
			name: "name".as_bytes().to_vec(),
			external_url: "external_url".as_bytes().to_vec(),
		};

		let hash = blake2_256(&[&PROTO_HASH[..], &fragment_data.name.encode()].concat());

		assert_ok!(FragmentsPallet::create(
			Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC)),
			PROTO_HASH,
			fragment_data.clone(),
			true,
			true,
			None
		));
		assert!(Fragments::<Test>::contains_key(&hash));

		assert_noop!(
			FragmentsPallet::create(
				Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC)),
				PROTO_HASH,
				fragment_data,
				true,
				true,
				None
			),
			Error::<Test>::AlreadyExist
		);
	});
}
