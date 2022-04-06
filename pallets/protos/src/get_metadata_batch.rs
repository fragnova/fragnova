use crate::common::*;

#[test]
fn get_metadata_batch_should_work() {
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

		let block_number = 1u32;

		set_authority("//Charlie");
		let signature = get_ecdsa_signature_for_metadata("//Charlie", PROTO_HASH, METADATA_DATA.into(), 1, block_number);


		let auth_data = AuthData { signature: signature, block: block_number };

		assert_ok!(ProtosPallet::set_metadata(
			Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC1)),
			auth_data,
			PROTO_HASH,
			"description".into(),
			METADATA_DATA.into()
		));

		let protos = ProtosPallet::get_metadata_batch(vec![PROTO_HASH], vec!["description".into()]);

		let metadata_hash : Hash256 = blake2_256(&METADATA_DATA.to_vec());

		assert!(protos == vec![Some(vec![Some(metadata_hash)])]);

	});
}

