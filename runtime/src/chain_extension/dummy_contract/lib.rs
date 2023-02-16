#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

use fragnova_extensions::FragnovaEnvironment;

#[ink::contract(env = crate::FragnovaEnvironment)]
mod dummy_contract {
    use scale::Compact;
    use ink_prelude::vec::Vec;
    use fragnova_extensions::{AssetId, MyChainExtensionError};
    use sp_fragnova::{
        Hash128,
        Hash256,
        protos::{
            Proto
        },
        fragments::{
            FragmentDefinition,
            FragmentInstance,
            InstanceUnit
        }
    };
    use protos::permissions::FragmentPerms;

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    #[derive(Default)]
    pub struct DummyContract {}

    impl DummyContract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Default::default()
        }

        #[ink(message)]
        pub fn get_proto(&self, proto_hash: Hash256) -> Option<Proto<AccountId, BlockNumber>> {
            self.env().extension().get_proto(proto_hash)
        }
        #[ink(message)]
        pub fn get_proto_ids(&self, owner: AccountId) -> Vec<Hash256> {
            self.env().extension().get_proto_ids(owner)
        }

        #[ink(message)]
        pub fn get_definition(&self, definition_hash: Hash128) -> Option<FragmentDefinition<Vec<u8>, AssetId, AccountId, BlockNumber>> {
            self.env().extension().get_definition(definition_hash)
        }
        #[ink(message)]
        pub fn get_instance(&self, definition_hash: Hash128, edition_id: InstanceUnit, copy_id: InstanceUnit) -> Option<FragmentInstance<BlockNumber>> {
            self.env().extension().get_instance(definition_hash, edition_id, copy_id)
        }
        #[ink(message)]
        pub fn get_instance_ids(&self, definition_hash: Hash128, owner: AccountId) -> Vec<(Compact<InstanceUnit>, Compact<InstanceUnit>)> {
            self.env().extension().get_instance_ids(definition_hash, owner)
        }
        // It seems that if your ink! contract message modifies anything in a pallet (as opposed to just read), it should have a `&mut self` receiver instead of `&self` receiver.
        // That's what Substrate did here: https://github.com/paritytech/ink/blob/3eb6eb06db97de1d418b62816fd6c97a973aa82b/examples/psp22-extension/lib.rs#L188-L263
        // (notice that the ink! contract messages that purely read things from a pallet have a `&self` receiver while those that modify things have a `&mut self` receiver)
        //
        // That's why even we have made the receiver `&mut self` here
        #[ink(message)]
        pub fn give_instance(&mut self, definition_hash: Hash128, edition_id: InstanceUnit, copy_id: InstanceUnit, to: AccountId, new_permissions: Option<FragmentPerms>, expirations: Option<BlockNumber>) -> Result<(), MyChainExtensionError> {
            self.env().extension().give_instance(definition_hash, edition_id, copy_id, to, new_permissions, expirations)
        }

    }
}
