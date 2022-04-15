#![cfg_attr(not(feature = "std"), no_std)]
use codec::Codec;
use sp_chainblocks::Hash256;
use sp_std::vec::Vec;

use scale_info::prelude::{string::String};

sp_api::decl_runtime_apis! {
	pub trait ProtosApi<Tags, AccountId>
	where
		Tags: Codec,
		AccountId: Codec
	{

		fn get_by_tags(tags: Vec<Tags>, owner: Option<AccountId>, limit: u32, from: u32, desc: bool) -> Vec<Hash256>;

		fn get_metadata_batch(batch: Vec<Hash256>, keys: Vec<Vec<u8>>) -> Vec<Option<Vec<Option<Hash256>>>>;

		fn get_protos(desc: bool, from: u32, limit: u32, metadata_keys: Option<Vec<Vec<u8>>>,
		  			  owner: Option<AccountId>, return_owners : bool, tags: Option<Vec<Tags>>) -> Vec<u8>;


	}




}
