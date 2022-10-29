#![cfg(test)]

use crate as pallet_protos;
use crate::{dummy_data::*, mock, mock::*, *};
use codec::Compact;
use frame_support::{assert_noop, assert_ok, dispatch::DispatchResult};
use std::collections::BTreeMap;
use upload_tests::upload;

use protos::categories::TextCategories;

mod upload_tests {
	use super::*;

	pub fn upload(
		signer: <Test as frame_system::Config>::AccountId,
		proto: &ProtoFragment,
	) -> DispatchResult {
		ProtosPallet::upload(
			RuntimeOrigin::signed(signer),
			proto.references.clone(),
			proto.category.clone(),
			proto.tags.clone(),
			proto.linked_asset.clone(),
			proto
				.include_cost
				.map(|cost| UsageLicense::Tickets(Compact::from(cost)))
				.unwrap_or(UsageLicense::Closed),
			proto.data.clone(),
		)
	}

	#[test]
	fn upload_should_work() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let block_number = System::block_number();

			let proto = dd.proto_fragment;

			assert_ok!(upload(dd.account_id, &proto));

			assert!(<Protos<Test>>::contains_key(proto.get_proto_hash()));

			let proto_struct = <Protos<Test>>::get(proto.get_proto_hash()).unwrap();

			// I am using `match` to ensure that this test case fails if a new field is ever added
			// to the `Proto` struct match proto_struct {
			// 	Proto {
			// 		block: 1,
			// 		patches: Vec::new(),
			// 		include_cost: proto.include_cost.map(|cost| Compact::from(cost)),
			// 		creator: dd.account_id,
			// 		owner: dd.account_id,
			// 		references: proto.references,
			// 		category: proto.category,
			// 		tags: proto.tags,
			// 		metadata: BTreeMap::new(),
			// 	} => (),
			// 	// _ => println!("Time to panic!!! Mayday"),
			// }

			let correct_proto_struct = Proto {
				block: block_number,
				patches: Vec::new(),
				license: proto
					.include_cost
					.map(|cost| UsageLicense::Tickets(Compact::from(cost)))
					.unwrap_or(UsageLicense::Closed),
				creator: dd.account_id,
				owner: ProtoOwner::User(dd.account_id),
				references: proto.references.clone(),
				category: proto.category.clone(),
				tags: Vec::new(), // proto.tags,
				metadata: BTreeMap::new(),
				accounts_info: AccountsInfo::default(),
			};

			// Ensure that this test case fails if a new field is ever added to the `Proto` struct
			match proto_struct {
				s if s == correct_proto_struct => (),
				_ => panic!("The correct `Proto` struct was not saved in the StorageMap `Protos`"),
			}

			assert!(<ProtosByCategory<Test>>::get(&proto.category)
				.unwrap()
				.contains(&proto.get_proto_hash()));
			assert!(<ProtosByOwner<Test>>::get(ProtoOwner::User(dd.account_id))
				.unwrap()
				.contains(&proto.get_proto_hash()));

			let event = <frame_system::Pallet<Test>>::events()
				.pop()
				.expect("Expected at least one EventRecord to be found")
				.event;
			assert_eq!(
				event,
				mock::RuntimeEvent::from(pallet_protos::Event::Uploaded {
					proto_hash: proto.get_proto_hash(),
					cid: proto.get_proto_cid()
				})
			);
		});
	}

	#[test]
	fn upload_should_not_work_if_proto_hash_exists() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();
			let proto = dd.proto_fragment;
			assert_ok!(upload(dd.account_id, &proto));
			assert_noop!(upload(dd.account_id, &proto), Error::<Test>::ProtoExists);
		});
	}
}

mod patch_tests {
	use super::*;

	fn patch_(signer: <Test as frame_system::Config>::AccountId, patch: &Patch) -> DispatchResult {
		ProtosPallet::patch(
			RuntimeOrigin::signed(signer),
			patch.proto_fragment.clone().get_proto_hash(),
			patch.include_cost.map(|cost| UsageLicense::Tickets(Compact::from(cost))),
			patch.new_references.clone(),
			None, // TODO
			patch.new_data.clone(),
		)
	}

	#[test]
	fn patch_should_work() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let block_number = System::block_number(); //@sinkingsugar

			let patch = dd.patch;

			assert_ok!(upload(dd.account_id, &patch.proto_fragment));

			assert_ok!(patch_(dd.account_id, &patch));
			let proto_struct = <Protos<Test>>::get(patch.proto_fragment.get_proto_hash()).unwrap();
			assert_eq!(
				proto_struct.license,
				patch
					.include_cost
					.map(|cost| UsageLicense::Tickets(Compact::from(cost)))
					.unwrap_or(UsageLicense::Closed)
			);
			assert!(proto_struct.patches.contains(&ProtoPatch {
				block: block_number,
				data_hash: patch.get_data_hash(),
				references: patch.new_references.clone()
			}));

			let event = <frame_system::Pallet<Test>>::events()
				.pop()
				.expect("Expected at least one EventRecord to be found")
				.event;
			assert_eq!(
				event,
				mock::RuntimeEvent::from(pallet_protos::Event::Patched {
					proto_hash: patch.proto_fragment.get_proto_hash(),
					cid: patch.get_data_cid()
				})
			);
		});
	}

	#[test]
	fn patch_should_not_work_if_user_does_not_own_proto() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let patch = dd.patch;

			assert_ok!(upload(dd.account_id, &patch.proto_fragment));

			assert_noop!(patch_(dd.account_id_second, &patch), Error::<Test>::Unauthorized);
		});
	}

	#[test]
	fn patch_should_not_work_if_proto_not_found() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();
			let patch = dd.patch;
			assert_noop!(patch_(dd.account_id, &patch), Error::<Test>::ProtoNotFound);
		});
	}

	// #[test]
	// fn patch_should_not_work_if_detached() {
	// 	let keystore = KeyStore::new();
	// 	let mut t = new_test_ext();

	// 	t.register_extension(KeystoreExt(Arc::new(keystore)));
	// 	t.execute_with(|| {
	// 		let pair = sp_core::ed25519::Pair::from_string("//Alice", None).unwrap();
	// 		let data = DATA.as_bytes().to_vec();
	// 		initial_upload();

	// 		sp_io::crypto::ecdsa_generate(KEY_TYPE, None);
	// 		let keys = sp_io::crypto::ecdsa_public_keys(KEY_TYPE);

	// 		<EthereumAuthorities<Test>>::mutate(|authorities| {
	// 			authorities.insert(keys.get(0).unwrap().clone());
	// 		});
	// 		assert_ok!(ProtosPallet::detach(
	// 			Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC1)),
	// 			PROTO_HASH,
	// 			SupportedChains::EthereumMainnet,
	// 			pair.to_raw_vec()
	// 		));

	// 		let detach_data = DetachInternalData {
	// 			public: sp_core::ed25519::Public::from_raw(PUBLIC1),
	// 			hash: PROTO_HASH,
	// 			remote_signature: vec![],
	// 			target_account: vec![],
	// 			target_chain: SupportedChains::EthereumGoerli,
	// 			nonce: 1,
	// 		};

	// 		assert_ok!(Detach::internal_finalize_detach(
	// 			Origin::none(),
	// 			detach_data,
	// 			pair.sign(DATA.as_bytes())
	// 		));

	// 		assert_noop!(
	// 			ProtosPallet::patch_(
	// 				Origin::signed(sp_core::ed25519::Public::from_raw(PUBLIC1)),
	// 				PROTO_HASH,
	// 				Some(Compact(123)),
	// 				vec![],
	// 				data,
	// 			),
	// 			Error::<Test>::Detached
	// 		);

	// 	});
	// }
}

mod transfer_tests {
	use super::*;

	fn transfer(
		signer: <Test as frame_system::Config>::AccountId,
		proto: &ProtoFragment,
		new_owner: <Test as frame_system::Config>::AccountId,
	) -> DispatchResult {
		ProtosPallet::transfer(RuntimeOrigin::signed(signer), proto.get_proto_hash(), new_owner)
	}

	#[test]
	fn transfer_should_work() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let proto = dd.proto_fragment;

			assert_ok!(upload(dd.account_id, &proto));

			assert_ok!(transfer(dd.account_id, &proto, dd.account_id_second));

			assert_eq!(
				<Protos<Test>>::get(proto.get_proto_hash()).unwrap().owner,
				ProtoOwner::User(dd.account_id_second)
			);

			assert_eq!(
				<ProtosByOwner<Test>>::get(ProtoOwner::User(dd.account_id))
					.unwrap()
					.contains(&proto.get_proto_hash()),
				false
			);
			assert!(<ProtosByOwner<Test>>::get(ProtoOwner::User(dd.account_id_second))
				.unwrap()
				.contains(&proto.get_proto_hash()));

			let event = <frame_system::Pallet<Test>>::events()
				.pop()
				.expect("Expected at least one EventRecord to be found")
				.event;
			assert_eq!(
				event,
				mock::RuntimeEvent::from(pallet_protos::Event::Transferred {
					proto_hash: proto.get_proto_hash(),
					owner_id: dd.account_id_second
				})
			);
		});
	}

	#[test]
	fn transfer_should_not_work_if_user_does_not_own_proto() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let proto = dd.proto_fragment;

			assert_ok!(upload(dd.account_id, &proto));

			assert_noop!(
				transfer(dd.account_id_second, &proto, dd.account_id),
				Error::<Test>::Unauthorized
			);
		});
	}

	#[test]
	fn transfer_should_not_work_if_proto_not_found() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let proto = dd.proto_fragment;

			assert_noop!(
				transfer(dd.account_id, &proto, dd.account_id_second),
				Error::<Test>::ProtoNotFound
			);
		});
	}
}

mod set_metadata_tests {
	use super::*;

	pub fn set_metadata(
		signer: <Test as frame_system::Config>::AccountId,
		metadata: &Metadata,
	) -> DispatchResult {
		ProtosPallet::set_metadata(
			RuntimeOrigin::signed(signer),
			metadata.proto_fragment.get_proto_hash(),
			metadata.metadata_key.clone(),
			metadata.data.clone(),
		)
	}

	#[test]
	fn set_metadata_should_work() {
		new_test_ext().execute_with(|| {
			let key_count = <MetaKeysIndex<Test>>::try_get().unwrap_or_default(); // @sinkingsugar

			let dd = DummyData::new();

			let metadata = dd.metadata;

			assert_ok!(upload(dd.account_id, &metadata.proto_fragment));

			assert_ok!(set_metadata(dd.account_id, &metadata));

			let metadata_map =
				<Protos<Test>>::get(metadata.proto_fragment.get_proto_hash()).unwrap().metadata;
			let metadata_key_index = <MetaKeys<Test>>::get(&metadata.metadata_key).unwrap();
			assert_eq!(
				metadata_map[&<Compact<u64>>::from(metadata_key_index)],
				metadata.get_data_hash()
			);

			assert_eq!(<MetaKeysIndex<Test>>::get(), key_count + 1); // @sinkingsugar

			let event = <frame_system::Pallet<Test>>::events()
				.pop()
				.expect("Expected at least one EventRecord to be found")
				.event;
			assert_eq!(
				event,
				mock::RuntimeEvent::from(pallet_protos::Event::MetadataChanged {
					proto_hash: metadata.proto_fragment.get_proto_hash(),
					cid: metadata.metadata_key.clone()
				})
			);
		});
	}

	#[test]
	fn set_metadata_should_not_work_if_user_does_not_own_proto() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();
			let metadata = dd.metadata;
			assert_ok!(upload(dd.account_id, &metadata.proto_fragment));
			assert_noop!(
				set_metadata(dd.account_id_second, &metadata),
				Error::<Test>::Unauthorized
			);
		});
	}

	#[test]
	fn set_metadata_should_not_work_if_proto_not_found() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();
			let metadata = dd.metadata;
			assert_noop!(set_metadata(dd.account_id, &metadata), Error::<Test>::ProtoNotFound);
		});
	}
}

mod get_protos_tests {
	use super::*;
	use protos::categories::{ShardsFormat, ShardsScriptInfo};
	use upload_tests::upload;

	#[test]
	fn get_protos_should_not_work_if_owner_does_not_exist() {
		new_test_ext().execute_with(|| {
			// UPLOAD
			let dd = DummyData::new();
			let proto = dd.proto_fragment_third;
			let proto_text = dd.proto_fragment_second;

			assert_ok!(upload(dd.account_id, &proto));
			assert_ok!(upload(dd.account_id_second, &proto_text));

			// SEARCH
			let params = GetProtosParams {
				desc: true,
				from: 10u64,
				limit: 20u64,
				metadata_keys: Vec::new(),
				owner: Some(sp_core::ed25519::Public::from_raw([13u8; 32])), /* different from
				                                                              * account_id */
				return_owners: false,
				categories: vec![Categories::Trait(Some(twox_64(&proto.data)))],
				tags: Vec::new(),
				exclude_tags: Vec::new(),
				available: Some(true),
			};

			let result: Result<Vec<u8>, Vec<u8>> = ProtosPallet::get_protos(params);
			assert_eq!(result.err(), Some("Owner not found".as_bytes().to_vec()));
		});
	}

	#[test]
	fn get_protos_by_category_other_than_trait_should_work() {
		new_test_ext().execute_with(|| {
			// UPLOAD
			let dd = DummyData::new();
			let proto = dd.proto_fragment;
			let proto_trait = dd.proto_fragment_fourth;

			assert_ok!(upload(dd.account_id, &proto));
			assert_ok!(upload(dd.account_id, &proto_trait));

			// SEARCH
			let params = GetProtosParams {
				desc: true,
				from: 0,
				limit: 2,
				metadata_keys: Vec::new(),
				owner: Some(dd.account_id),
				return_owners: true,
				categories: vec![Categories::Text(TextCategories::Plain)],
				tags: Vec::new(),
				exclude_tags: Vec::new(),
				available: Some(true),
			};

			let result = ProtosPallet::get_protos(params).ok().unwrap();
			let result_string = std::str::from_utf8(&result).unwrap();

			let proto_hash = proto.get_proto_hash();
			let encoded = hex::encode(&proto_hash);

			let json_expected = json!({
				encoded: {
				"tickets": Some(proto.include_cost),
				"owner": {
					"type": "internal",
					"value": hex::encode(dd.account_id)
				},
			}})
			.to_string();

			assert_eq!(result_string, json_expected);
		});
	}

	#[test]
	fn get_protos_by_exactly_same_trait_should_work() {
		new_test_ext().execute_with(|| {
			// UPLOAD
			let dd = DummyData::new();
			let proto = dd.proto_fragment_third;

			assert_ok!(upload(dd.account_id, &proto));

			// SEARCH
			let params = GetProtosParams {
				desc: true,
				from: 0,
				limit: 2,
				metadata_keys: Vec::new(),
				owner: Some(dd.account_id),
				return_owners: true,
				categories: vec![Categories::Trait(Some(twox_64(&proto.data)))],
				tags: Vec::new(),
				exclude_tags: Vec::new(),
				available: Some(true),
			};

			let result = ProtosPallet::get_protos(params).ok().unwrap();
			let result_string = std::str::from_utf8(&result).unwrap();

			let proto_hash = proto.get_proto_hash();
			let encoded = hex::encode(&proto_hash);

			let json_expected = json!({
				encoded: {
				"tickets": Some(proto.include_cost),
				"owner": {
					"type": "internal",
					"value": hex::encode(dd.account_id)
				},
			}})
			.to_string();

			assert_eq!(result_string, json_expected);
		});
	}

	#[test]
	fn get_protos_by_trait_should_work() {
		new_test_ext().execute_with(|| {
			// UPLOAD
			let dd = DummyData::new();
			let proto = dd.proto_fragment_third;

			assert_ok!(upload(dd.account_id, &proto));

			// SEARCH
			// This is searching a Proto using the same Trait name.
			// Note that Trait description is different from the trait uploaded.
			let params = GetProtosParams {
				desc: true,
				from: 0,
				limit: 2,
				metadata_keys: Vec::new(),
				owner: Some(dd.account_id),
				return_owners: true,
				categories: vec![Categories::Trait(Some(twox_64(&proto.data)))],
				tags: Vec::new(),
				exclude_tags: Vec::new(),
				available: Some(true),
			};

			let result = ProtosPallet::get_protos(params).ok().unwrap();
			let result_string = std::str::from_utf8(&result).unwrap();

			let proto_hash = proto.get_proto_hash();
			let encoded = hex::encode(&proto_hash);

			let json_expected = json!({
				encoded: {
				"tickets": Some(proto.include_cost),
				"owner": {
					"type": "internal",
					"value": hex::encode(dd.account_id)
				},
			}})
			.to_string();

			assert_eq!(result_string, json_expected);
		});
	}

	#[test]
	fn get_protos_by_trait_name_with_multiple_protos_stored_should_work() {
		new_test_ext().execute_with(|| {
			// UPLOAD
			let dd = DummyData::new();
			// Two protos with different trait names
			let proto1 = dd.proto_fragment_third;
			let proto2 = dd.proto_fragment_fourth;

			assert_ok!(upload(dd.account_id, &proto1));
			assert_ok!(upload(dd.account_id, &proto2));

			// SEARCH
			let params = GetProtosParams {
				desc: true,
				from: 0,
				limit: 2,
				metadata_keys: Vec::new(),
				owner: None,
				return_owners: true,
				categories: vec![Categories::Trait(Some(twox_64(&proto2.data)))],
				tags: Vec::new(),
				exclude_tags: Vec::new(),
				available: Some(true),
			};

			let result = ProtosPallet::get_protos(params).ok().unwrap();
			let result_string = std::str::from_utf8(&result).unwrap();

			let proto_hash = proto2.get_proto_hash();
			let encoded = hex::encode(&proto_hash);

			let json_expected = json!({
				encoded: {
				"tickets": Some(proto2.include_cost),
				"owner": {
					"type": "internal",
					"value": hex::encode(dd.account_id)
				},
			}})
			.to_string();

			assert_eq!(result_string, json_expected);
		});
	}

	#[test]
	fn get_protos_searching_by_multiple_categories_same_owner_should_work() {
		new_test_ext().execute_with(|| {
			// UPLOAD
			let dd = DummyData::new();
			// Two protos with different trait names
			let proto1 = dd.proto_fragment_third;
			let proto_text = dd.proto_fragment_second;
			let proto_shard_script = dd.proto_shard_script;

			assert_ok!(upload(dd.account_id, &proto1));
			assert_ok!(upload(dd.account_id, &proto_text));
			assert_ok!(upload(dd.account_id, &proto_shard_script));

			// SEARCH
			let shard_script_num_1: [u8; 8] = [4u8; 8];
			let shard_script_num_2: [u8; 8] = [5u8; 8];
			let shard_script = ShardsScriptInfo {
				format: ShardsFormat::Edn,
				requiring: vec![shard_script_num_1],
				implementing: vec![shard_script_num_2],
			};
			let params = GetProtosParams {
				desc: true,
				from: 0,
				limit: 10,
				metadata_keys: Vec::new(),
				owner: None,
				return_owners: true,
				categories: vec![
					Categories::Trait(Some(twox_64(&proto1.data))),
					Categories::Shards(shard_script),
					Categories::Text(TextCategories::Plain),
				],
				tags: Vec::new(),
				exclude_tags: Vec::new(),
				available: Some(true),
			};

			let result = ProtosPallet::get_protos(params).ok().unwrap();
			let result_string = std::str::from_utf8(&result).unwrap();

			let proto_hash = proto1.get_proto_hash();
			let encoded = hex::encode(&proto_hash);

			let proto_hash_2 = proto_shard_script.get_proto_hash();
			let encoded2 = hex::encode(&proto_hash_2);

			let proto_hash_text = proto_text.get_proto_hash();
			let encoded3 = hex::encode(&proto_hash_text);

			let json_expected = json!({
				encoded: {
				"tickets": Some(proto1.include_cost),
				"owner": {
					"type": "internal",
					"value": hex::encode(dd.account_id)
				},
			}, encoded2: {
				"tickets": Some(proto_shard_script.include_cost),
				"owner": {
					"type": "internal",
					"value": hex::encode(dd.account_id)
				},
			}, encoded3: {
				"tickets": Some(proto_text.include_cost),
				"owner": {
					"type": "internal",
					"value": hex::encode(dd.account_id)
				},
			}})
			.to_string();

			assert_eq!(result_string, json_expected);
		});
	}

	#[test]
	fn get_protos_filter_shards_by_implementing_requiring() {
		new_test_ext().execute_with(|| {
			// UPLOAD
			let dd = DummyData::new();
			// Two protos with different trait names
			let proto_shard_script = dd.proto_shard_script;
			let proto_shard_script_3 = dd.proto_shard_script_3;
			let proto_shard_script_binary = dd.proto_shard_script_4;

			assert_ok!(upload(dd.account_id, &proto_shard_script));
			assert_ok!(upload(dd.account_id, &proto_shard_script_3));
			// This below has the same implementing and requiring of script_3, but different format (Binary)
			assert_ok!(upload(dd.account_id, &proto_shard_script_binary));

			// SEARCH
			let shard_script_num_4: [u8; 8] = [1u8; 8];
			let shard_script_num_5: [u8; 8] = [7u8; 8];
			let shard_script = ShardsScriptInfo {
				format: ShardsFormat::Edn,
				requiring: vec![shard_script_num_4],
				implementing: vec![shard_script_num_5],
			};
			let params = GetProtosParams {
				desc: true,
				from: 0,
				limit: 10,
				metadata_keys: Vec::new(),
				owner: None,
				return_owners: true,
				categories: vec![Categories::Shards(shard_script)],
				tags: Vec::new(),
				exclude_tags: Vec::new(),
				available: Some(true),
			};

			let result = ProtosPallet::get_protos(params).ok().unwrap();
			let result_string = std::str::from_utf8(&result).unwrap();

			let proto_hash = proto_shard_script_3.get_proto_hash();
			let encoded2 = hex::encode(&proto_hash);

			let json_expected = json!({
				encoded2: {
				"tickets": Some(proto_shard_script_3.include_cost),
				"owner": {
					"type": "internal",
					"value": hex::encode(dd.account_id)
				},
			}})
			.to_string();

			assert_eq!(result_string, json_expected);
		});
	}

	#[test]
	fn get_protos_searching_by_multiple_categories_different_owner_should_work() {
		new_test_ext().execute_with(|| {
			// UPLOAD
			let dd = DummyData::new();
			// Two protos with different trait names
			let proto1 = dd.proto_fragment_third;
			let proto_text = dd.proto_fragment_second;
			let proto_shard_script = dd.proto_shard_script;

			assert_ok!(upload(dd.account_id, &proto1));
			assert_ok!(upload(dd.account_id_second, &proto_text));
			assert_ok!(upload(dd.account_id, &proto_shard_script));

			// SEARCH
			let shard_script_num_1: [u8; 8] = [4u8; 8];
			let shard_script_num_2: [u8; 8] = [5u8; 8];
			let shard_script = ShardsScriptInfo {
				format: ShardsFormat::Edn,
				requiring: vec![shard_script_num_1],
				implementing: vec![shard_script_num_2],
			};
			let params = GetProtosParams {
				desc: true,
				from: 0,
				limit: 10,
				metadata_keys: Vec::new(),
				owner: None,
				return_owners: true,
				categories: vec![
					Categories::Trait(Some(twox_64(&proto1.data))),
					Categories::Shards(shard_script),
					Categories::Text(TextCategories::Plain),
				],
				tags: Vec::new(),
				exclude_tags: Vec::new(),
				available: Some(true),
			};

			let result = ProtosPallet::get_protos(params).ok().unwrap();
			let result_string = std::str::from_utf8(&result).unwrap();

			let proto_hash = proto1.get_proto_hash();
			let encoded = hex::encode(&proto_hash);

			let proto_hash_2 = proto_shard_script.get_proto_hash();
			let encoded2 = hex::encode(&proto_hash_2);

			let proto_hash_text = proto_text.get_proto_hash();
			let encoded3 = hex::encode(&proto_hash_text);

			let json_expected = json!({
				encoded: {
				"tickets": Some(proto1.include_cost),
				"owner": {
					"type": "internal",
					"value": hex::encode(dd.account_id)
				},
			}, encoded2: {
				"tickets": Some(proto_shard_script.include_cost),
				"owner": {
					"type": "internal",
					"value": hex::encode(dd.account_id)
				},
			}, encoded3: {
				"tickets": Some(proto_text.include_cost),
				"owner": {
					"type": "internal",
					"value": hex::encode(dd.account_id_second)
				},
			}})
			.to_string();

			assert_eq!(result_string, json_expected);
		});
	}

	#[test]
	fn get_protos_by_trait_should_return_two_protos() {
		new_test_ext().execute_with(|| {
			// UPLOAD
			let dd = DummyData::new();
			let proto = dd.proto_fragment_fourth;
			let proto2 = dd.proto_fragment_fifth;
			// Upload twice the same Proto
			assert_ok!(upload(dd.account_id, &proto));
			assert_ok!(upload(dd.account_id, &proto2));

			// SEARCH
			let params = GetProtosParams {
				desc: true,
				from: 0,
				limit: 2,
				metadata_keys: Vec::new(),
				owner: None,
				return_owners: true,
				categories: vec![
					Categories::Trait(Some(twox_64(&proto.data))),
					Categories::Trait(Some(twox_64(&proto2.data))),
				],
				tags: Vec::new(),
				exclude_tags: Vec::new(),
				available: Some(true),
			};

			let result = ProtosPallet::get_protos(params).ok().unwrap();
			let result_string = std::str::from_utf8(&result).unwrap();

			let proto_hash = proto.get_proto_hash();
			let encoded = hex::encode(&proto_hash);
			let proto2_hash = proto2.get_proto_hash();
			let encoded2 = hex::encode(&proto2_hash);

			let json_expected = json!({
				encoded: {
				"tickets": Some(proto.include_cost),
				"owner": {
					"type": "internal",
					"value": hex::encode(dd.account_id)
				}}, encoded2: {
					"tickets": Some(proto2.include_cost),
					"owner": {
						"type": "internal",
						"value": hex::encode(dd.account_id)
					},
			}})
			.to_string();

			assert_eq!(result_string, json_expected);
		});
	}

	#[test]
	fn get_protos_by_shards_script_should_work() {
		new_test_ext().execute_with(|| {
			// UPLOAD
			let dd = DummyData::new();
			// Two protos with different trait names
			let proto1 = dd.proto_fragment_third;
			let proto_shard_script = dd.proto_shard_script;

			assert_ok!(upload(dd.account_id, &proto1));
			assert_ok!(upload(dd.account_id, &proto_shard_script));

			// SEARCH
			let shard_script_num_1: [u8; 8] = [4u8; 8];
			let shard_script_num_2: [u8; 8] = [5u8; 8];
			let shard_script = ShardsScriptInfo {
				format: ShardsFormat::Edn,
				requiring: vec![shard_script_num_1],
				implementing: vec![shard_script_num_2],
			};
			let params = GetProtosParams {
				desc: true,
				from: 0,
				limit: 2,
				metadata_keys: Vec::new(),
				owner: None,
				return_owners: true,
				categories: vec![Categories::Shards(shard_script)],
				tags: Vec::new(),
				exclude_tags: Vec::new(),
				available: Some(true),
			};

			let result = ProtosPallet::get_protos(params).ok().unwrap();
			let result_string = std::str::from_utf8(&result).unwrap();

			let proto_hash = proto_shard_script.get_proto_hash();
			let encoded = hex::encode(&proto_hash);

			let json_expected = json!({
				encoded: {
				"tickets": Some(proto_shard_script.include_cost),
				"owner": {
					"type": "internal",
					"value": hex::encode(dd.account_id)
				},
			}})
			.to_string();

			assert_eq!(result_string, json_expected);
		});
	}

	#[test]
	fn get_protos_by_shards_finds_nothing() {
		new_test_ext().execute_with(|| {
			// UPLOAD
			let dd = DummyData::new();
			// Two protos with different trait names
			let proto1 = dd.proto_fragment_third;
			let proto_shard_script = dd.proto_shard_script;

			assert_ok!(upload(dd.account_id, &proto1));
			assert_ok!(upload(dd.account_id, &proto_shard_script));

			// SEARCH
			let shard_script_num_1: [u8; 8] = [99u8; 8];
			let shard_script_num_2: [u8; 8] = [99u8; 8];
			let shard_script = ShardsScriptInfo {
				format: ShardsFormat::Edn,
				requiring: vec![shard_script_num_1],
				implementing: vec![shard_script_num_2],
			};
			let params = GetProtosParams {
				desc: true,
				from: 0,
				limit: 2,
				metadata_keys: Vec::new(),
				owner: None,
				return_owners: true,
				categories: vec![Categories::Shards(shard_script)],
				tags: Vec::new(),
				exclude_tags: Vec::new(),
				available: Some(true),
			};

			let result = ProtosPallet::get_protos(params).ok().unwrap();
			let result_string = std::str::from_utf8(&result).unwrap();

			let json_expected = json!({}).to_string();

			assert_eq!(result_string, json_expected);
		});
	}

	#[test]
	fn get_protos_by_partial_implementing_shards_script_should_work() {
		new_test_ext().execute_with(|| {
			// UPLOAD
			let dd = DummyData::new();
			let proto1 = dd.proto_fragment_third;
			let proto_shard_script = dd.proto_shard_script_2;

			assert_ok!(upload(dd.account_id, &proto1));
			assert_ok!(upload(dd.account_id, &proto_shard_script));

			// SEARCH
			let shard_script_num_1: [u8; 8] = [0u8; 8];
			let shard_script_num_2: [u8; 8] = [5u8; 8];
			let shard_script = ShardsScriptInfo {
				format: ShardsFormat::Edn,
				requiring: vec![shard_script_num_1],
				implementing: vec![shard_script_num_2],
			};
			let params = GetProtosParams {
				desc: true,
				from: 0,
				limit: 2,
				metadata_keys: Vec::new(),
				owner: None,
				return_owners: true,
				categories: vec![Categories::Shards(shard_script)],
				tags: Vec::new(),
				exclude_tags: Vec::new(),
				available: Some(true),
			};

			let result = ProtosPallet::get_protos(params).ok().unwrap();
			let result_string = std::str::from_utf8(&result).unwrap();

			let proto_hash = proto_shard_script.get_proto_hash();
			let encoded = hex::encode(&proto_hash);

			let json_expected = json!({
				encoded: {
				"tickets": Some(proto_shard_script.include_cost),
				"owner": {
					"type": "internal",
					"value": hex::encode(dd.account_id)
				},
			}})
			.to_string();

			assert_eq!(result_string, json_expected);
		});
	}

	#[test]
	fn get_protos_by_generic_format() {
		new_test_ext().execute_with(|| {
			// UPLOAD
			let dd = DummyData::new();
			let proto_shard_script = dd.proto_shard_script_2;

			assert_ok!(upload(dd.account_id, &proto_shard_script));

			// SEARCH
			let shard_script_num_1: [u8; 8] = [0u8; 8];
			let shard_script_num_2: [u8; 8] = [0u8; 8];
			let shard_script = ShardsScriptInfo {
				format: ShardsFormat::Edn,
				requiring: vec![shard_script_num_1],
				implementing: vec![shard_script_num_2],
			};
			let params = GetProtosParams {
				desc: true,
				from: 0,
				limit: 2,
				metadata_keys: Vec::new(),
				owner: None,
				return_owners: true,
				categories: vec![Categories::Shards(shard_script)],
				tags: Vec::new(),
				exclude_tags: Vec::new(),
				available: Some(true),
			};

			let result = ProtosPallet::get_protos(params).ok().unwrap();
			let result_string = std::str::from_utf8(&result).unwrap();

			let proto_hash = proto_shard_script.get_proto_hash();
			let encoded = hex::encode(&proto_hash);

			let json_expected = json!({
				encoded: {
				"tickets": Some(proto_shard_script.include_cost),
				"owner": {
					"type": "internal",
					"value": hex::encode(dd.account_id)
				},
			}})
			.to_string();

			assert_eq!(result_string, json_expected);
		});
	}

	#[test]
	fn get_protos_should_exclude_tags() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();
			let mut proto = dd.proto_fragment;
			let mut proto_second = dd.proto_fragment_second;

			proto.tags = vec![b"2D".to_vec()];
			proto_second.tags = vec![b"NSFW".to_vec()];

			assert_ok!(upload(dd.account_id, &proto));
			assert_ok!(upload(dd.account_id, &proto_second));

			assert_eq!(
				serde_json::from_slice::<Value>(
					&ProtosPallet::get_protos(GetProtosParams {
						limit: u64::MAX,
						..Default::default()
					})
					.unwrap()
				)
				.unwrap(),
				json!({
					hex::encode(proto.get_proto_hash()): {},
					hex::encode(proto_second.get_proto_hash()): {},
				})
			);

			assert_eq!(
				serde_json::from_slice::<Value>(
					&ProtosPallet::get_protos(GetProtosParams {
						limit: u64::MAX,
						exclude_tags: proto_second.tags, // exclude tags!
						..Default::default()
					})
					.unwrap()
				)
				.unwrap(),
				json!({
					hex::encode(proto.get_proto_hash()): {},
				})
			);
		});
	}
}

mod get_genealogy_tests {
	use super::*;
	use set_metadata_tests::set_metadata;

	#[test]
	fn get_genealogy_should_work_when_get_ancestors_is_true() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let proto = dd.proto_fragment;
			assert_ok!(upload(dd.account_id, &proto));

			let mut proto_second = dd.proto_fragment_second;
			proto_second.references = vec![proto.get_proto_hash()];
			assert_ok!(upload(dd.account_id, &proto_second));

			let mut proto_third = dd.proto_fragment_third;
			proto_third.references = vec![proto_second.get_proto_hash()];
			assert_ok!(upload(dd.account_id, &proto_third));

			assert_eq!(
				serde_json::from_slice::<Value>(
					&ProtosPallet::get_genealogy(GetGenealogyParams {
						proto_hash: hex::encode(proto_third.get_proto_hash()).into_bytes(),
						get_ancestors: true,
					})
						.unwrap()
				)
					.unwrap(),
				json!({
					hex::encode(proto_third.get_proto_hash()): [
						hex::encode(proto_second.get_proto_hash())
					],
					hex::encode(proto_second.get_proto_hash()): [
						hex::encode(proto.get_proto_hash())
					],
					hex::encode(proto.get_proto_hash()): [],
				})
			);
		});
	}

	#[test]
	fn get_genealogy_should_work_when_get_ancestors_is_false() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let proto = dd.proto_fragment;
			assert_ok!(upload(dd.account_id, &proto));

			let mut proto_second = dd.proto_fragment_second;
			proto_second.references = vec![proto.get_proto_hash()];
			assert_ok!(upload(dd.account_id, &proto_second));

			let mut proto_third = dd.proto_fragment_third;
			proto_third.references = vec![proto_second.get_proto_hash()];
			assert_ok!(upload(dd.account_id, &proto_third));

			assert_eq!(
				serde_json::from_slice::<Value>(
					&ProtosPallet::get_genealogy(GetGenealogyParams {
						proto_hash: hex::encode(proto.get_proto_hash()).into_bytes(),
						get_ancestors: false,
					})
					.unwrap()
				)
				.unwrap(),
				json!({
					hex::encode(proto.get_proto_hash()): [
						hex::encode(proto_second.get_proto_hash())
					],
					hex::encode(proto_second.get_proto_hash()): [
						hex::encode(proto_third.get_proto_hash())
					],
					hex::encode(proto_third.get_proto_hash()): [],
				})
			);
		});
	}

	#[test]
	fn set_metadata_should_not_work_if_user_does_not_own_proto() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();
			let metadata = dd.metadata;
			assert_ok!(upload(dd.account_id, &metadata.proto_fragment));
			assert_noop!(
				set_metadata(dd.account_id_second, &metadata),
				Error::<Test>::Unauthorized
			);
		});
	}

	#[test]
	fn set_metadata_should_not_work_if_proto_not_found() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();
			let metadata = dd.metadata;
			assert_noop!(set_metadata(dd.account_id, &metadata), Error::<Test>::ProtoNotFound);
		});
	}
}

mod ban_tests {

	use super::*;

	pub fn ban(proto: &ProtoFragment) -> DispatchResult {
		ProtosPallet::ban(RuntimeOrigin::root(), proto.get_proto_hash())
	}

	#[test]
	fn ban_should_work() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();
			let proto = dd.proto_fragment;
			assert_ok!(upload(dd.account_id, &proto));
			assert_ok!(ban(&proto));
			assert!(!<ProtosByCategory<Test>>::get(&proto.category)
				.unwrap_or_default()
				.contains(&proto.get_proto_hash()));
			assert!(!<ProtosByOwner<Test>>::get(ProtoOwner::User(dd.account_id))
				.unwrap_or_default()
				.contains(&proto.get_proto_hash()));
		});
	}

	#[test]
	fn ban_should_not_work_if_proto_does_not_exist() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();
			let proto = dd.proto_fragment;
			assert_noop!(ban(&proto), Error::<Test>::ProtoNotFound);
		});
	}

	#[test]
	fn ban_should_not_work_if_caller_is_not_root() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();
			let proto = dd.proto_fragment;
			assert_ok!(upload(dd.account_id, &proto));
			assert_noop!(
				ProtosPallet::ban(RuntimeOrigin::signed(dd.account_id), proto.get_proto_hash()),
				sp_runtime::DispatchError::BadOrigin
			);
		});
	}
}
