#![cfg_attr(not(feature = "std"), no_std)]
use codec::Codec;
use sp_chainblocks::Hash256;
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
	pub trait ProtosApi<Tags, ProtoOwner>
	where
		Tags: Codec,
		ProtoOwner: Codec
	{
		fn get_by_tag(tags: Tags) -> Option<Vec<Hash256>>;

		fn get_by_tags(tags: Vec<Tags>, owner: Option<ProtoOwner>, limit: u32) -> Option<Vec<Hash256>>;

		// fn get_metadata_batch(batch: Vec<Hash256>, keys: Vec<String>) -> Vec<Hash256>;


	}




}
