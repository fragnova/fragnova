#![cfg_attr(not(feature = "std"), no_std)]
use codec::Codec;
use sp_clamor::Hash256;
use sp_std::vec::Vec;

use pallet_protos::GetProtosParams;

sp_api::decl_runtime_apis! {
	pub trait ProtosApi<Tags, AccountId>
	where
		Tags: Codec,
		AccountId: Codec
	{

		fn get_protos(params: GetProtosParams<AccountId, Vec<u8>>) -> Vec<u8>;


	}




}
