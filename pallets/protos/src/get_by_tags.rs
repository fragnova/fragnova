use crate::common::*;

#[test]
fn get_by_tags_should_work() {
	new_test_ext().execute_with(|| {
		let data = DATA.as_bytes().to_vec();
		let references = vec![PROTO_HASH];

		let signature = initial_set_up_and_get_signature(data.clone(), references.clone(), 0, vec![Tags::Code]);

		let auth_data = AuthData { signature, block: 1 };

		assert_ok!(ProtosPallet::upload(
			Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC1)),
			auth_data,
			references,
			vec![Tags::Code],
			None,
			None,
			data,
		));

		let protos = ProtosPallet::get_by_tags(vec![Tags::Code], None, 69, 0, false);

		assert!(protos == vec![PROTO_HASH]);

		let protos = ProtosPallet::get_by_tags(vec![Tags::Code], Some(sp_core::ed25519::Public::from_raw(PUBLIC1)), 69, 0, false);

		assert!(protos == vec![PROTO_HASH]);

	});
}



#[test]
fn get_by_tags_should_not_work_if_tag_is_incorrect() {
	new_test_ext().execute_with(|| {
				let data = DATA.as_bytes().to_vec();
		let references = vec![PROTO_HASH];

		let signature = initial_set_up_and_get_signature(data.clone(), references.clone(), 0, vec![Tags::Code]);

		let auth_data = AuthData { signature, block: 1 };

		assert_ok!(ProtosPallet::upload(
			Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC1)),
			auth_data,
			references,
			vec![Tags::Code],
			None,
			None,
			data,
		));

		let protos = ProtosPallet::get_by_tags(vec![Tags::Audio], Some(sp_core::ed25519::Public::from_raw(PUBLIC2)), 69, 0, false);

		assert!(protos.is_empty());

		let protos = ProtosPallet::get_by_tags(vec![Tags::Image], Some(sp_core::ed25519::Public::from_raw(PUBLIC2)), 69, 0, false);

		assert!(protos.is_empty());

	});
}



#[test]
fn get_by_tags_should_not_work_if_owner_is_incorrect() {
	new_test_ext().execute_with(|| {
				let data = DATA.as_bytes().to_vec();
		let references = vec![PROTO_HASH];

		let signature = initial_set_up_and_get_signature(data.clone(), references.clone(), 0, vec![Tags::Code]);

		let auth_data = AuthData { signature, block: 1 };

		assert_ok!(ProtosPallet::upload(
			Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC1)),
			auth_data,
			references,
			vec![Tags::Code],
			None,
			None,
			data,
		));

		let protos = ProtosPallet::get_by_tags(vec![Tags::Code], Some(sp_core::ed25519::Public::from_raw(PUBLIC2)), 69, 0, false);

		assert!(protos.is_empty());

		let protos = ProtosPallet::get_by_tags(vec![Tags::Image], Some(sp_core::ed25519::Public::from_raw(PUBLIC2)), 69, 0, false);

		assert!(protos.is_empty());

	});
}



