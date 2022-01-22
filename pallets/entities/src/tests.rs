use crate::{mock::*, Entities, EntityMetadata, Error, Fragment2Entities};
use codec::{Compact, Encode};
use fragments_pallet::{AuthData, IncludeInfo, LinkedAsset};
use frame_support::{assert_noop, assert_ok};
use sp_core::Pair;
use sp_io::hashing::blake2_256;

fn initial_set_up_and_get_signature(
	data: Vec<u8>,
	references: Vec<IncludeInfo>,
	nonce: u64,
) -> sp_core::ecdsa::Signature {
	let pair = sp_core::ecdsa::Pair::from_string("//Charlie", None).unwrap();

	let fragment_hash = blake2_256(&data);
	let linked_asset: Option<LinkedAsset> = None;
	let signature: sp_core::ecdsa::Signature = pair.sign(
		&[
			&fragment_hash[..],
			&references.encode(),
			&linked_asset.encode(),
			&nonce.encode(),
			&1.encode(),
		]
		.concat(),
	);
	assert_ok!(FragmentsPallet::add_upload_auth(Origin::root(), pair.public()));
	signature
}

fn initial_upload_and_get_signature() -> AuthData {
	let data = DATA.as_bytes().to_vec();
	let references = vec![IncludeInfo {
		fragment_hash: FRAGMENT_HASH,
		mutable_index: Some(Compact(1)),
		staked_amount: Compact(1),
	}];
	let signature = initial_set_up_and_get_signature(data.clone(), references.clone(), 0);
	let auth_data = AuthData { signature, block: 1 };

	assert_ok!(FragmentsPallet::upload(
		Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC)),
		references,
		None,
		None,
		auth_data.clone(),
		data,
	));
	auth_data
}

#[test]
fn create_should_works() {
	new_test_ext().execute_with(|| {
		initial_upload_and_get_signature();

		let entity_data = EntityMetadata {
			name: "name".as_bytes().to_vec(),
			external_url: "external_url".as_bytes().to_vec(),
		};

		let hash = blake2_256(&[&FRAGMENT_HASH[..], &entity_data.name.encode()].concat());

		assert_ok!(EntitiesPallet::create(
			Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC)),
			FRAGMENT_HASH,
			entity_data,
			true,
			true,
			None
		));
		assert!(Entities::<Test>::contains_key(&hash));
		assert!(Fragment2Entities::<Test>::contains_key(&FRAGMENT_HASH));
	});
}

#[test]
fn create_should_not_work_if_fragments_not_found() {
	new_test_ext().execute_with(|| {
		let entity_data = EntityMetadata {
			name: "name".as_bytes().to_vec(),
			external_url: "external_url".as_bytes().to_vec(),
		};

		assert_noop!(EntitiesPallet::create(
			Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC)),
			FRAGMENT_HASH,
			entity_data,
			true,
			true,
			None
		),
			Error::<Test>::FragmentNotFound
		);
	});
}

#[test]
fn create_should_not_work_if_fragment_owner_not_found() {
	new_test_ext().execute_with(|| {
		initial_upload_and_get_signature();
		pub const PUBLIC: [u8; 32] = [
			136, 65, 23, 149, 81, 74, 241, 98, 119, 101, 236, 239, 252, 189, 0, 39, 25, 240, 49,
			96, 79, 173, 215, 209, 136, 226, 220, 88, 91, 78, 26, 251,
		];

		let entity_data = EntityMetadata {
			name: "name".as_bytes().to_vec(),
			external_url: "external_url".as_bytes().to_vec(),
		};

		assert_noop!(EntitiesPallet::create(
			Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC)),
			FRAGMENT_HASH,
			entity_data,
			true,
			true,
			None
		),
			Error::<Test>::NoPermission
		);
	});
}

#[test]
fn create_should_not_work_if_entity_already_exist() {
	new_test_ext().execute_with(|| {
		initial_upload_and_get_signature();

		let entity_data = EntityMetadata {
			name: "name".as_bytes().to_vec(),
			external_url: "external_url".as_bytes().to_vec(),
		};

		let hash = blake2_256(&[&FRAGMENT_HASH[..], &entity_data.name.encode()].concat());

		assert_ok!(EntitiesPallet::create(
			Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC)),
			FRAGMENT_HASH,
			entity_data.clone(),
			true,
			true,
			None
		));
		assert!(Entities::<Test>::contains_key(&hash));

		assert_noop!(EntitiesPallet::create(
			Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC)),
			FRAGMENT_HASH,
			entity_data,
			true,
			true,
			None
		),
			Error::<Test>::EntityAlreadyExist
		);
	});
}
