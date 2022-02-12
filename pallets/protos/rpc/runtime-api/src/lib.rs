#![cfg_attr(not(feature = "std"), no_std)]
use codec::Codec;
use sp_chainblocks::Hash256;
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
		pub trait ProtosApi<Tags> where
				Tags: Codec, {
				fn get_proto_by_tags(tags: Tags) -> Option<Vec<Hash256>>;
		}
}
