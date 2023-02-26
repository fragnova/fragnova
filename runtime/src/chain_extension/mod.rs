use codec::{
	Encode,
	Compact,
};
use frame_support::{
	dispatch::RawOrigin,
	traits::Get
};
use sp_runtime::{
	traits::{
		StaticLookup,
	},
	DispatchError,
};
use sp_std::vec::Vec;
use pallet_contracts::chain_extension::{
	ChainExtension,
	Environment,
	Ext,
	InitState,
	RetVal,
	SysConfig, // `frame_system::Config` is re-exported as "SysConfig" in `pallet_contracts::chain_extension` (https://paritytech.github.io/substrate/master/pallet_contracts/chain_extension/trait.SysConfig.html#)
};
use sp_fragnova::{
	Hash128,
	Hash256,
	protos::{
		Proto,
		ProtoOwner,
	},
	fragments::{
		FragmentDefinition,
		FragmentInstance,
		InstanceUnit
	}
};
use pallet_fragments::WeightInfo; // this is a trait

use protos::permissions::FragmentPerms;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[derive(Default)]
pub struct MyExtension;

/// We're using enums for function IDs because contrary to raw u16 it enables
/// exhaustive matching, which results in cleaner code.
enum FuncId {
	/// Get the `Proto` struct of the Proto-Fragment which has an ID of `proto_hash`
	GetProto,
	/// Get the list of Proto-Fragments that are owned by `owner`
	GetProtoIds,

	/// Get the `FragmentDefinition` struct of the Fragment Definition which has the ID of `definition_hash`
	GetDefinition,
	/// Get the `FragmentInstance` struct of the Fragment Instance whose Fragment Definition ID is `definition_hash`,
	/// whose Edition ID is `edition_id` and whose Copy ID is `copy_id`
	GetInstance,
	/// Get the list of Fragment Instances of the Fragment Definition `definition_hash` that are owned by `owner`
	GetInstanceIds,
	/// Give a Fragment Instance (that is owned by the smart contract) to `to`.
	GiveInstance,
}

impl TryFrom<u16> for FuncId {
	type Error = DispatchError;

	fn try_from(func_id: u16) -> Result<Self, Self::Error> {
		let id = match func_id {
			0x0b00 => Self::GetProto,
			0x0b01 => Self::GetProtoIds,

			0x0c00 => Self::GetDefinition,
			0x0c01 => Self::GetInstance,
			0x0c02 => Self::GetInstanceIds,
			0x0c03 => Self::GiveInstance,
			_ => {
				log::error!("Called an unregistered `func_id`: {:}", func_id);
				return Err(DispatchError::Other("Unimplemented func_id"))
			}
		};

		Ok(id)
	}
}

/// A trait used to extend the set of contract callable functions.
///
/// In order to create a custom chain extension this trait must be implemented and supplied
/// to the pallet contracts configuration trait as the associated type of the same name.
///
/// Source: https://paritytech.github.io/substrate/master/pallet_contracts/chain_extension/trait.ChainExtension.html#
impl<T> ChainExtension<T> for MyExtension
	where
		T: pallet_contracts::Config + pallet_protos::Config + pallet_fragments::Config,
		<T as SysConfig>::AccountId: AsRef<[u8]>,
{

	/// Call the chain extension logic.
	///
	/// This is the only function that needs to be implemented in order to write a
	/// chain extension. It is called whenever a contract calls the `seal_call_chain_extension` (https://paritytech.github.io/substrate/master/pallet_contracts/api_doc/seal0/trait.Api.html#tymethod.seal_call_chain_extension)
	/// imported wasm function.
	///
	/// # Parameters
	/// - `env`: Access to the remaining arguments (of the contract function that was called) and the execution environment (of the contract).
	///
	/// # Return
	///
	/// In case of `Err` the contract execution is immediately suspended and the passed error
	/// is returned to the caller. Otherwise the value of [`RetVal`] determines the exit
	/// behaviour.
	///
	/// Source: https://paritytech.github.io/substrate/master/pallet_contracts/chain_extension/trait.ChainExtension.html#tymethod.call
	fn call<E: Ext>(
		&mut self,
		env: Environment<E, InitState>,
	) -> Result<RetVal, DispatchError>
		where
			E: Ext<T = T>,
	{
		let func_id = FuncId::try_from(env.func_id())?;

		// When working with chain extensions we "communicate" between our contract and the runtime
		// using a memory buffer.
		//
		// We can read encoded method arguments from this buffer. We can also write the result of
		// our computations into this buffer, which can then get used by ink!.
		//
		// Source: https://github.com/HCastano/decoded-2022-demo/blob/master/runtime/src/chain_extension.rs and https://www.youtube.com/watch?v=yykPQF0tkqk
		let mut env = env.buf_in_buf_out();

		match func_id {
			FuncId::GetProto => {
				let proto_hash: Hash256 = env.read_as()?;
				// We are supposed to charge weight even if we read a storage value according to @bkchr: https://substrate.stackexchange.com/questions/7071/in-the-runtime-chain-extension-should-we-be-charging-weight-if-we-are-reading-a
				// Furthermore, an actual Substrate Blockchain does `<T as frame_system::Config>::DbWeight::get().reads(1)` to read a storage value here: https://github.com/AstarNetwork/astar-frame/blob/b2f888fafecb7257e68c5e9f0e9e661d1f8007c9/chain-extensions/dapps-staking/src/lib.rs#L150-L156
				env.charge_weight(<T as SysConfig>::DbWeight::get().reads(1))?;
				let output: Option<Proto<T::AccountId, T::BlockNumber>> = pallet_protos::Protos::<T>::get(&proto_hash);
				// TODO Review - Should `weights_per_byte` be `None`? In the examples (https://github.com/paritytech/ink/blob/master/examples/rand-extension/runtime/chain-extension-example.rs and https://github.com/paritytech/ink/blob/master/examples/psp22-extension/runtime/psp22-extension-example.rs) and in https://github.com/AstarNetwork/astar-frame/search?q=env.write,
				// I only see `None` - but in our case we are outputting a a struct that has a `Vec` field!
				env.write(&output.encode(), false, None).map_err(|_| {
					DispatchError::Other("ChainExtension failed to get the proto")
				})?;
			},
			FuncId::GetProtoIds => {
				let owner: T::AccountId = env.read_as()?; // TODO Review - Shouldn't `owner` parameter be of type `ProtoOwner<T::AccountId>` instead of `T::AccountId`
				env.charge_weight(<T as SysConfig>::DbWeight::get().reads(1))?;
				let output: Vec<Hash256> = pallet_protos::ProtosByOwner::<T>::get(ProtoOwner::<T::AccountId>::User(owner)).unwrap_or_default();
				// TODO Review - Should `weights_per_byte` be `None`? In the examples (https://github.com/paritytech/ink/blob/master/examples/rand-extension/runtime/chain-extension-example.rs and https://github.com/paritytech/ink/blob/master/examples/psp22-extension/runtime/psp22-extension-example.rs) and in https://github.com/AstarNetwork/astar-frame/search?q=env.write,
				// I only see `None` - but in our case we are outputting a `Vec`!
				env.write(&output.encode(), false, None).map_err(|_| {
					DispatchError::Other("ChainExtension failed to get list of proto IDs")
				})?;
			},

			FuncId::GetDefinition => {
				let definition_hash: Hash128 = env.read_as()?;
				env.charge_weight(<T as SysConfig>::DbWeight::get().reads(1))?;
				let output: Option<FragmentDefinition<Vec<u8>, T::AssetId, T::AccountId, T::BlockNumber>> = pallet_fragments::Definitions::<T>::get(&definition_hash);
				// TODO Review - Should `weights_per_byte` be `None`? In the examples (https://github.com/paritytech/ink/blob/master/examples/rand-extension/runtime/chain-extension-example.rs and https://github.com/paritytech/ink/blob/master/examples/psp22-extension/runtime/psp22-extension-example.rs) and in https://github.com/AstarNetwork/astar-frame/search?q=env.write,
				// I only see `None` - but in our case we are outputting a a struct that has a `Vec` field!
				env.write(&output.encode(), false, None).map_err(|_| {
					DispatchError::Other("ChainExtension failed to get the definition")
				})?;
			},
			FuncId::GetInstance => {
				let (definition_hash, edition_id, copy_id): (Hash128, InstanceUnit, InstanceUnit) = env.read_as()?;
				env.charge_weight(<T as SysConfig>::DbWeight::get().reads(1))?;
				let output: Option<FragmentInstance<T::BlockNumber>> = pallet_fragments::Fragments::<T>::get((definition_hash, edition_id, copy_id));
				// TODO Review - Should `weights_per_byte` be `None`? In the examples (https://github.com/paritytech/ink/blob/master/examples/rand-extension/runtime/chain-extension-example.rs and https://github.com/paritytech/ink/blob/master/examples/psp22-extension/runtime/psp22-extension-example.rs) and in https://github.com/AstarNetwork/astar-frame/search?q=env.write,
				// I only see `None` - but in our case we are outputting a a struct that has a `Vec` field!
				env.write(&output.encode(), false, None).map_err(|_| {
					DispatchError::Other("ChainExtension failed to get the fragment instance")
				})?;
			},
			FuncId::GetInstanceIds => {
				let (definition_hash, owner): (Hash128, T::AccountId) = env.read_as()?;
				env.charge_weight(<T as SysConfig>::DbWeight::get().reads(1))?;
				let output: Vec<(Compact<InstanceUnit>, Compact<InstanceUnit>)> = pallet_fragments::Inventory::<T>::get(owner, definition_hash).unwrap_or_default();
				// TODO Review - Should `weights_per_byte` be `None`? In the examples (https://github.com/paritytech/ink/blob/master/examples/rand-extension/runtime/chain-extension-example.rs and https://github.com/paritytech/ink/blob/master/examples/psp22-extension/runtime/psp22-extension-example.rs) and in https://github.com/AstarNetwork/astar-frame/search?q=env.write,
				// I only see `None` - but in our case we are outputting a `Vec`!
				env.write(&output.encode(), false, None).map_err(|_| {
					DispatchError::Other("ChainExtension failed to get the fragment instance IDs")
				})?;
			},
			FuncId::GiveInstance => {
				// We are using `env.read_as_unbounded::<T: Decode>()` to read the function input instead of `env.read_as::<T: Decode + MaxEncodedLen>()` because the latter throws an error!
				//
				// # Footnote:
				//
				// I think the reason `env.read_as::<T: Decode + MaxEncodedLen>()` (https://paritytech.github.io/substrate/master/pallet_contracts/chain_extension/struct.Environment.html#method.read_as)
				// doesn't work (even though `T` (i.e the tuple we want to decode) implements `MaxEncodedLen`)
				// is because some of the tuple elements are optional (so if they're `None` - SCALE encodes them as `0x00` which is just 1 byte).
				// Therefore, `T` does not have a fixed size (if it's `None` it's 1 byte, otherwise it's more than 1 byte).
				// And `env.read_as()` is defined as: "Reads and decodes **a type with a size fixed at compile time** from contract memory."
				//
				// The exact same error was reported here: https://github.com/paritytech/substrate/issues/11305
				//
				// # Footnote 2:
				//
				// We are using `env.read_as_unbounded()` just like it was used in this tutorial: https://www.youtube.com/watch?v=yykPQF0tkqk / https://github.com/HCastano/decoded-2022-demo/blob/master/runtime/src/chain_extension.rs#L104
				//
				let (definition_hash, edition_id, copy_id, to, new_permissions, expiration):
					(Hash128, InstanceUnit, InstanceUnit, T::AccountId, Option<FragmentPerms>, Option<T::BlockNumber>) = env.read_as_unbounded(env.in_len())?;

				// This is the exact same expression that is in the weight macro of `fragments.give()`
				let weight = <T as pallet_fragments::Config>::WeightInfo::benchmark_give_instance_that_has_copy_perms().max(
					<T as pallet_fragments::Config>::WeightInfo::benchmark_give_instance_that_does_not_have_copy_perms()
				);
				// We need to ensure that we're charging weight to account for the amount of compute
				// used by the call to our pallet. This is something we typically don't have to
				// worry about in the context of smart contracts since they're gas metered.
				//
				// Source: https://github.com/HCastano/decoded-2022-demo/blob/master/runtime/src/chain_extension.rs and https://www.youtube.com/watch?v=yykPQF0tkqk
				env.charge_weight(weight)?;
				// TODO Review - At the moment, we don't care about contracts calling contracts - so we will just return `DispatchError` if `fragments.give()` fails.
				//
				// # Footnote:
				//
				// When the chain extension fails, the error can be handled gracefully (see the quotation below for further information about this),
				// if the chain extension returns `Ok(RetVal::Converging(x))` (where `x` is the error code number) instead of `DispatchError`.
				// Here is an example of this: https://github.com/HCastano/decoded-2022-demo/blob/master/runtime/src/chain_extension.rs#L115.
				//
				// “When you’re writing ink! contracts, if your message (i.e your smart contract function) returns a `Result<T, E>`
				// and it’s called from another contract - you can handle this (i.e the error `E` of the Result) gracefully.
				// Things don’t necessarily need to revert. If a user calls the message and it fails, then things do revert.”
				//
				// Source: https://www.youtube.com/watch?v=yykPQF0tkqk&t=1285s
				//
				// # Footnote 2:
				//
				// In the future, we can map pallet errors to `Ok(RetVal::Converging(x))` (where `x` is the error code number) like this: https://github.com/AstarNetwork/astar-frame/blob/polkadot-v0.9.33/chain-extensions/dapps-staking/src/lib.rs#L199-L210
				pallet_fragments::Pallet::<T>::give(
					RawOrigin::Signed(env.ext().address().clone()).into(),
					definition_hash,
					edition_id,
					copy_id,
					T::Lookup::unlookup(to),
					new_permissions,
					expiration
				)?;
			},
		};

		Ok(RetVal::Converging(0))
	}
}
