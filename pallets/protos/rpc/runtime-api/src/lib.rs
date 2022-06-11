#![cfg_attr(not(feature = "std"), no_std)]
use codec::Codec;

use pallet_protos::GetProtosParams;
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
	pub trait ProtosApi<AccountId>
	where
		AccountId: Codec
	{
		fn get_protos(params: GetProtosParams<AccountId, Vec<u8>>) -> Result<Vec<u8>, Vec<u8>>;
	}
}
