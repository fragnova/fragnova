#![cfg(test)]

use crate as pallet_protos;
use crate::{dummy_data::*, mock, mock::*, *};
use codec::Compact;
use frame_support::{assert_noop, assert_ok, dispatch::DispatchResult};
use protos::categories::TextCategories;
use std::collections::BTreeMap;
use upload_tests::upload;

mod upload_tests {
	use super::*;
	use sp_runtime::BoundedVec;

	pub fn upload(
		signer: <Test as frame_system::Config>::AccountId,
		proto: &ProtoFragment,
	) -> DispatchResult {
		ProtosPallet::upload(
			RuntimeOrigin::signed(signer),
			proto.references.clone(),
			proto.category.clone(),
			TryInto::<BoundedVec<BoundedVec<_, _>, <Test as pallet_protos::Config>::MaxTags>>::try_into(
				proto
					.tags
					.clone()
					.into_iter()
					.map(|tag: Vec<u8>| TryInto::<BoundedVec<u8, <Test as pallet_protos::Config>::StringLimit>>::try_into(tag).unwrap())
					.collect::<Vec<BoundedVec<_, _>>>()
			).unwrap(),
			proto.linked_asset.clone(),
			UsageLicense::Open,
			None,
			ProtoData::Local(proto.data.clone()),
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

			let correct_proto_struct = Proto {
				block: block_number,
				patches: Vec::new(),
				license: UsageLicense::Open,
				creator: dd.account_id,
				owner: ProtoOwner::User(dd.account_id),
				references: proto.references.clone(),
				category: proto.category.clone(),
				tags: Vec::new(), // proto.tags,
				metadata: BTreeMap::new(),
				data: ProtoData::Local(vec![]), // empty here if local
				cluster: None,
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

	#[test]
	fn upload_should_not_work_if_reference_does_not_exist() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();
			let mut proto = dd.proto_fragment;
			proto.references = vec![[222; 32]];
			assert_noop!(upload(dd.account_id, &proto), Error::<Test>::ReferenceNotFound);
		});
	}

	#[test]
	fn upload_should_not_work_if_trait_does_not_exist() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();
			let [trait_proto, shards_proto] = dd.proto_with_trait;
			assert_ok!(upload(dd.account_id, &trait_proto));
			assert_ok!(upload(dd.account_id, &shards_proto));
		});

		new_test_ext().execute_with(|| {
			let dd = DummyData::new();
			let [_trait_proto, mut shards_proto] = dd.proto_with_trait;
			assert_noop!(upload(dd.account_id, &shards_proto), Error::<Test>::ReferenceNotFound);

			shards_proto.references = vec![];
			assert_noop!(upload(dd.account_id, &shards_proto), Error::<Test>::TraitsNotImplemented);
		});

	}
}

mod patch_tests {
	use super::*;

	fn patch_(signer: <Test as frame_system::Config>::AccountId, patch: &Patch) -> DispatchResult {
		ProtosPallet::patch(
			RuntimeOrigin::signed(signer),
			patch.proto_fragment.clone().get_proto_hash(),
			Some(UsageLicense::Open),
			patch.new_references.clone(),
			None, // TODO
			Some(ProtoData::Local(patch.new_data.clone())),
		)
	}

	#[test]
	fn patch_should_work() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let block_number = System::block_number();

			let patch = dd.patch;

			assert_ok!(upload(dd.account_id, &patch.proto_fragment));

			assert_ok!(patch_(dd.account_id, &patch));
			let proto_struct = <Protos<Test>>::get(patch.proto_fragment.get_proto_hash()).unwrap();
			assert_eq!(proto_struct.license, UsageLicense::Open,);
			assert!(proto_struct.patches.contains(&ProtoPatch {
				block: block_number,
				data_hash: patch.get_data_hash(),
				references: patch.new_references.clone(),
				data: ProtoData::Local(vec![]) // this is empty if local
			}));

			let event = <frame_system::Pallet<Test>>::events()
				.pop()
				.expect("Expected at least one EventRecord to be found")
				.event;
			assert_eq!(
				event,
				mock::RuntimeEvent::from(pallet_protos::Event::Patched {
					proto_hash: patch.proto_fragment.get_proto_hash(),
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

	#[test]
	#[ignore]
	fn patch_should_not_work_if_proto_is_detached() {
		todo!()
	}
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

	#[test]
	#[ignore]
	fn transfer_should_not_work_if_proto_is_detached() {
		todo!()
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
			metadata.metadata_key.clone().try_into().unwrap(),
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
					metadata_key: metadata.metadata_key.clone()
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

	#[test]
	#[ignore]
	fn set_metadata_should_not_work_if_proto_is_detached() {
		todo!()
	}
}

mod detach_tests {
	use super::*;
	use pallet_detach::DetachCollection;

	pub fn detach_(
		signer: <Test as frame_system::Config>::AccountId,
		detach: &Detach,
	) -> DispatchResult {
		ProtosPallet::detach(
			RuntimeOrigin::signed(signer),
			detach
				.proto_fragments
				.iter()
				.map(|proto_fragment| proto_fragment.get_proto_hash())
				.collect::<Vec<Hash256>>(),
			detach.target_chain,
			detach.target_account.clone().try_into().unwrap(),
		)
	}

	#[test]
	fn detach_should_work() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let detach = dd.detach;

			detach.proto_fragments.iter().for_each(|proto_fragment| {
				assert_ok!(upload(dd.account_id, &proto_fragment));
			});
			assert_ok!(detach_(dd.account_id, &detach));

			assert_eq!(
				pallet_detach::DetachRequests::<Test>::get(),
				vec![pallet_detach::DetachRequest {
					collection: DetachCollection::Protos(
						detach
							.proto_fragments
							.iter()
							.map(|proto_fragment| proto_fragment.get_proto_hash())
							.collect()
					),
					target_chain: detach.target_chain,
					target_account: detach.target_account,
				},]
			);
		});
	}

	#[test]
	fn detach_should_not_work_if_user_does_not_own_the_proto() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let detach = dd.detach;

			detach.proto_fragments.iter().for_each(|proto_fragment| {
				assert_ok!(upload(dd.account_id, &proto_fragment));
			});
			assert_noop!(detach_(dd.account_id_second, &detach), Error::<Test>::Unauthorized);
		});
	}
	#[test]
	fn detach_should_not_work_if_the_proto_does_not_exist() {
		new_test_ext().execute_with(|| {
			let dd = DummyData::new();

			let detach = dd.detach;

			assert_noop!(detach_(dd.account_id, &detach), Error::<Test>::ProtoNotFound);
		});
	}

	// TODO
	#[test]
	#[ignore = "I have no idea how the enum `LinkedAssset` even works right now!"]
	fn detach_should_not_work_if_proto_is_owned_by_external_asset() {
		todo!("I have no idea how the enum `LinkedAssset` even works right now!");
	}
}

mod get_protos_tests {
	use super::*;
	// use protos::categories::{ShardsFormat, ShardsScriptInfo};
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
				"license": "open",
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
				"license": "open",
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
				"license": "open",
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
				"license": "open",
				"owner": {
					"type": "internal",
					"value": hex::encode(dd.account_id)
				},
			}})
			.to_string();

			assert_eq!(result_string, json_expected);
		});
	}

	// #[test]
	// fn get_protos_searching_by_multiple_categories_same_owner_should_work() {
	// 	new_test_ext().execute_with(|| {
	// 		// UPLOAD
	// 		let dd = DummyData::new();
	// 		// Two protos with different trait names
	// 		let proto1 = dd.proto_fragment_third;
	// 		let proto_text = dd.proto_fragment_second;
	// 		let proto_shard_script = dd.proto_shard_script;
	//
	// 		assert_ok!(upload(dd.account_id, &proto1));
	// 		assert_ok!(upload(dd.account_id, &proto_text));
	// 		assert_ok!(upload(dd.account_id, &proto_shard_script));
	//
	// 		// SEARCH
	// 		let shard_script_num_1: [u8; 8] = [4u8; 8];
	// 		let shard_script_num_2: [u8; 8] = [5u8; 8];
	// 		let shard_script = ShardsScriptInfo {
	// 			format: ShardsFormat::Edn,
	// 			requiring: vec![shard_script_num_1],
	// 			implementing: vec![shard_script_num_2],
	// 		};
	// 		let params = GetProtosParams {
	// 			desc: true,
	// 			from: 0,
	// 			limit: 10,
	// 			metadata_keys: Vec::new(),
	// 			owner: None,
	// 			return_owners: true,
	// 			categories: vec![
	// 				Categories::Trait(Some(twox_64(&proto1.data))),
	// 				Categories::Shards(shard_script),
	// 				Categories::Text(TextCategories::Plain),
	// 			],
	// 			tags: Vec::new(),
	// 			exclude_tags: Vec::new(),
	// 			available: Some(true),
	// 		};
	//
	// 		let result = ProtosPallet::get_protos(params).ok().unwrap();
	// 		let result_string = std::str::from_utf8(&result).unwrap();
	//
	// 		let proto_hash = proto1.get_proto_hash();
	// 		let encoded = hex::encode(&proto_hash);
	//
	// 		let proto_hash_2 = proto_shard_script.get_proto_hash();
	// 		let encoded2 = hex::encode(&proto_hash_2);
	//
	// 		let proto_hash_text = proto_text.get_proto_hash();
	// 		let encoded3 = hex::encode(&proto_hash_text);
	//
	// 		let json_expected = json!({
	// 			encoded: {
	// 			"license": "open",
	// 			"owner": {
	// 				"type": "internal",
	// 				"value": hex::encode(dd.account_id)
	// 			},
	// 		}, encoded2: {
	// 			"license": "open",
	// 			"owner": {
	// 				"type": "internal",
	// 				"value": hex::encode(dd.account_id)
	// 			},
	// 		}, encoded3: {
	// 			"license": "open",
	// 			"owner": {
	// 				"type": "internal",
	// 				"value": hex::encode(dd.account_id)
	// 			},
	// 		}})
	// 		.to_string();
	//
	// 		assert_eq!(result_string, json_expected);
	// 	});
	// }

	// #[test]
	// fn get_protos_filter_shards_by_implementing_requiring() {
	// 	new_test_ext().execute_with(|| {
	// 		// UPLOAD
	// 		let dd = DummyData::new();
	// 		// Two protos with different trait names
	// 		let proto_shard_script = dd.proto_shard_script;
	// 		let proto_shard_script_3 = dd.proto_shard_script_3;
	// 		let proto_shard_script_binary = dd.proto_shard_script_4;
	//
	// 		assert_ok!(upload(dd.account_id, &proto_shard_script));
	// 		assert_ok!(upload(dd.account_id, &proto_shard_script_3));
	// 		// This below has the same implementing and requiring of script_3, but different format (Binary)
	// 		assert_ok!(upload(dd.account_id, &proto_shard_script_binary));
	//
	// 		// SEARCH
	// 		let shard_script_num_4: [u8; 8] = [1u8; 8];
	// 		let shard_script_num_5: [u8; 8] = [7u8; 8];
	// 		let shard_script = ShardsScriptInfo {
	// 			format: ShardsFormat::Edn,
	// 			requiring: vec![shard_script_num_4],
	// 			implementing: vec![shard_script_num_5],
	// 		};
	// 		let params = GetProtosParams {
	// 			desc: true,
	// 			from: 0,
	// 			limit: 10,
	// 			metadata_keys: Vec::new(),
	// 			owner: None,
	// 			return_owners: true,
	// 			categories: vec![Categories::Shards(shard_script)],
	// 			tags: Vec::new(),
	// 			exclude_tags: Vec::new(),
	// 			available: Some(true),
	// 		};
	//
	// 		let result = ProtosPallet::get_protos(params).ok().unwrap();
	// 		let result_string = std::str::from_utf8(&result).unwrap();
	//
	// 		let proto_hash = proto_shard_script_3.get_proto_hash();
	// 		let encoded2 = hex::encode(&proto_hash);
	//
	// 		let json_expected = json!({
	// 			encoded2: {
	// 			"license": "open",
	// 			"owner": {
	// 				"type": "internal",
	// 				"value": hex::encode(dd.account_id)
	// 			},
	// 		}})
	// 		.to_string();
	//
	// 		assert_eq!(result_string, json_expected);
	// 	});
	// }

	// #[test]
	// fn get_protos_searching_by_multiple_categories_different_owner_should_work() {
	// 	new_test_ext().execute_with(|| {
	// 		// UPLOAD
	// 		let dd = DummyData::new();
	// 		// Two protos with different trait names
	// 		let proto1 = dd.proto_fragment_third;
	// 		let proto_text = dd.proto_fragment_second;
	// 		let proto_shard_script = dd.proto_shard_script;
	//
	// 		assert_ok!(upload(dd.account_id, &proto1));
	// 		assert_ok!(upload(dd.account_id_second, &proto_text));
	// 		assert_ok!(upload(dd.account_id, &proto_shard_script));
	//
	// 		// SEARCH
	// 		let shard_script_num_1: [u8; 8] = [4u8; 8];
	// 		let shard_script_num_2: [u8; 8] = [5u8; 8];
	// 		let shard_script = ShardsScriptInfo {
	// 			format: ShardsFormat::Edn,
	// 			requiring: vec![shard_script_num_1],
	// 			implementing: vec![shard_script_num_2],
	// 		};
	// 		let params = GetProtosParams {
	// 			desc: true,
	// 			from: 0,
	// 			limit: 10,
	// 			metadata_keys: Vec::new(),
	// 			owner: None,
	// 			return_owners: true,
	// 			categories: vec![
	// 				Categories::Trait(Some(twox_64(&proto1.data))),
	// 				Categories::Shards(shard_script),
	// 				Categories::Text(TextCategories::Plain),
	// 			],
	// 			tags: Vec::new(),
	// 			exclude_tags: Vec::new(),
	// 			available: Some(true),
	// 		};
	//
	// 		let result = ProtosPallet::get_protos(params).ok().unwrap();
	// 		let result_string = std::str::from_utf8(&result).unwrap();
	//
	// 		let proto_hash = proto1.get_proto_hash();
	// 		let encoded = hex::encode(&proto_hash);
	//
	// 		let proto_hash_2 = proto_shard_script.get_proto_hash();
	// 		let encoded2 = hex::encode(&proto_hash_2);
	//
	// 		let proto_hash_text = proto_text.get_proto_hash();
	// 		let encoded3 = hex::encode(&proto_hash_text);
	//
	// 		let json_expected = json!({
	// 			encoded: {
	// 			"license": "open",
	// 			"owner": {
	// 				"type": "internal",
	// 				"value": hex::encode(dd.account_id)
	// 			},
	// 		}, encoded2: {
	// 			"license": "open",
	// 			"owner": {
	// 				"type": "internal",
	// 				"value": hex::encode(dd.account_id)
	// 			},
	// 		}, encoded3: {
	// 			"license": "open",
	// 			"owner": {
	// 				"type": "internal",
	// 				"value": hex::encode(dd.account_id_second)
	// 			},
	// 		}})
	// 		.to_string();
	//
	// 		assert_eq!(result_string, json_expected);
	// 	});
	// }

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
				"license": "open",
				"owner": {
					"type": "internal",
					"value": hex::encode(dd.account_id)
				}}, encoded2: {
					"license": "open",
					"owner": {
						"type": "internal",
						"value": hex::encode(dd.account_id)
					},
			}})
			.to_string();

			assert_eq!(result_string, json_expected);
		});
	}

	// #[test]
	// fn get_protos_by_shards_script_should_work() {
	// 	new_test_ext().execute_with(|| {
	// 		// UPLOAD
	// 		let dd = DummyData::new();
	// 		// Two protos with different trait names
	// 		let proto1 = dd.proto_fragment_third;
	// 		let proto_shard_script = dd.proto_shard_script;
	//
	// 		assert_ok!(upload(dd.account_id, &proto1));
	// 		assert_ok!(upload(dd.account_id, &proto_shard_script));
	//
	// 		// SEARCH
	// 		let shard_script_num_1: [u8; 8] = [4u8; 8];
	// 		let shard_script_num_2: [u8; 8] = [5u8; 8];
	// 		let shard_script = ShardsScriptInfo {
	// 			format: ShardsFormat::Edn,
	// 			requiring: vec![shard_script_num_1],
	// 			implementing: vec![shard_script_num_2],
	// 		};
	// 		let params = GetProtosParams {
	// 			desc: true,
	// 			from: 0,
	// 			limit: 2,
	// 			metadata_keys: Vec::new(),
	// 			owner: None,
	// 			return_owners: true,
	// 			categories: vec![Categories::Shards(shard_script)],
	// 			tags: Vec::new(),
	// 			exclude_tags: Vec::new(),
	// 			available: Some(true),
	// 		};
	//
	// 		let result = ProtosPallet::get_protos(params).ok().unwrap();
	// 		let result_string = std::str::from_utf8(&result).unwrap();
	//
	// 		let proto_hash = proto_shard_script.get_proto_hash();
	// 		let encoded = hex::encode(&proto_hash);
	//
	// 		let json_expected = json!({
	// 			encoded: {
	// 			"license": "open",
	// 			"owner": {
	// 				"type": "internal",
	// 				"value": hex::encode(dd.account_id)
	// 			},
	// 		}})
	// 		.to_string();
	//
	// 		assert_eq!(result_string, json_expected);
	// 	});
	// }

	// #[test]
	// fn get_protos_by_shards_finds_nothing() {
	// 	new_test_ext().execute_with(|| {
	// 		// UPLOAD
	// 		let dd = DummyData::new();
	// 		// Two protos with different trait names
	// 		let proto1 = dd.proto_fragment_third;
	// 		let proto_shard_script = dd.proto_shard_script;
	//
	// 		assert_ok!(upload(dd.account_id, &proto1));
	// 		assert_ok!(upload(dd.account_id, &proto_shard_script));
	//
	// 		// SEARCH
	// 		let shard_script_num_1: [u8; 8] = [99u8; 8];
	// 		let shard_script_num_2: [u8; 8] = [99u8; 8];
	// 		let shard_script = ShardsScriptInfo {
	// 			format: ShardsFormat::Edn,
	// 			requiring: vec![shard_script_num_1],
	// 			implementing: vec![shard_script_num_2],
	// 		};
	// 		let params = GetProtosParams {
	// 			desc: true,
	// 			from: 0,
	// 			limit: 2,
	// 			metadata_keys: Vec::new(),
	// 			owner: None,
	// 			return_owners: true,
	// 			categories: vec![Categories::Shards(shard_script)],
	// 			tags: Vec::new(),
	// 			exclude_tags: Vec::new(),
	// 			available: Some(true),
	// 		};
	//
	// 		let result = ProtosPallet::get_protos(params).ok().unwrap();
	// 		let result_string = std::str::from_utf8(&result).unwrap();
	//
	// 		let json_expected = json!({}).to_string();
	//
	// 		assert_eq!(result_string, json_expected);
	// 	});
	// }

	// #[test]
	// fn get_protos_by_partial_implementing_shards_script_should_work() {
	// 	new_test_ext().execute_with(|| {
	// 		// UPLOAD
	// 		let dd = DummyData::new();
	// 		let proto1 = dd.proto_fragment_third;
	// 		let proto_shard_script = dd.proto_shard_script_2;
	//
	// 		assert_ok!(upload(dd.account_id, &proto1));
	// 		assert_ok!(upload(dd.account_id, &proto_shard_script));
	//
	// 		// SEARCH
	// 		let shard_script_num_1: [u8; 8] = [0u8; 8];
	// 		let shard_script_num_2: [u8; 8] = [5u8; 8];
	// 		let shard_script = ShardsScriptInfo {
	// 			format: ShardsFormat::Edn,
	// 			requiring: vec![shard_script_num_1],
	// 			implementing: vec![shard_script_num_2],
	// 		};
	// 		let params = GetProtosParams {
	// 			desc: true,
	// 			from: 0,
	// 			limit: 2,
	// 			metadata_keys: Vec::new(),
	// 			owner: None,
	// 			return_owners: true,
	// 			categories: vec![Categories::Shards(shard_script)],
	// 			tags: Vec::new(),
	// 			exclude_tags: Vec::new(),
	// 			available: Some(true),
	// 		};
	//
	// 		let result = ProtosPallet::get_protos(params).ok().unwrap();
	// 		let result_string = std::str::from_utf8(&result).unwrap();
	//
	// 		let proto_hash = proto_shard_script.get_proto_hash();
	// 		let encoded = hex::encode(&proto_hash);
	//
	// 		let json_expected = json!({
	// 			encoded: {
	// 			"license": "open",
	// 			"owner": {
	// 				"type": "internal",
	// 				"value": hex::encode(dd.account_id)
	// 			},
	// 		}})
	// 		.to_string();
	//
	// 		assert_eq!(result_string, json_expected);
	// 	});
	// }

	// #[test]
	// fn get_protos_by_generic_format() {
	// 	new_test_ext().execute_with(|| {
	// 		// UPLOAD
	// 		let dd = DummyData::new();
	// 		let proto_shard_script = dd.proto_shard_script_2;
	//
	// 		assert_ok!(upload(dd.account_id, &proto_shard_script));
	//
	// 		// SEARCH
	// 		let shard_script_num_1: [u8; 8] = [0u8; 8];
	// 		let shard_script_num_2: [u8; 8] = [0u8; 8];
	// 		let shard_script = ShardsScriptInfo {
	// 			format: ShardsFormat::Edn,
	// 			requiring: vec![shard_script_num_1],
	// 			implementing: vec![shard_script_num_2],
	// 		};
	// 		let params = GetProtosParams {
	// 			desc: true,
	// 			from: 0,
	// 			limit: 2,
	// 			metadata_keys: Vec::new(),
	// 			owner: None,
	// 			return_owners: true,
	// 			categories: vec![Categories::Shards(shard_script)],
	// 			tags: Vec::new(),
	// 			exclude_tags: Vec::new(),
	// 			available: Some(true),
	// 		};
	//
	// 		let result = ProtosPallet::get_protos(params).ok().unwrap();
	// 		let result_string = std::str::from_utf8(&result).unwrap();
	//
	// 		let proto_hash = proto_shard_script.get_proto_hash();
	// 		let encoded = hex::encode(&proto_hash);
	//
	// 		let json_expected = json!({
	// 			encoded: {
	// 			"license": "open",
	// 			"owner": {
	// 				"type": "internal",
	// 				"value": hex::encode(dd.account_id)
	// 			},
	// 		}})
	// 		.to_string();
	//
	// 		assert_eq!(result_string, json_expected);
	// 	});
	// }

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
