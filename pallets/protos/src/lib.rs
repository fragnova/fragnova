#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod weights;
use codec::{Compact, Decode, Encode};
pub use pallet::*;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::{ecdsa, H160, U256};
use sp_io::{crypto as Crypto, hashing::blake2_256, transaction_index};
use sp_std::{
	collections::{btree_map::BTreeMap, btree_set::BTreeSet},
	vec,
	vec::Vec,
};
pub use weights::WeightInfo;

use sp_chainblocks::Hash256;

#[derive(Encode, Decode, Copy, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Tags {
	Code,
	Audio,
	Image,
}

#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum LinkSource {
	// Generally we just store this data, we don't verify it as we assume auth service did it.
	// (Link signature, Linked block number, EIP155 Chain ID)
	Evm(ecdsa::Signature, u64, U256),
}

#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum LinkedAsset {
	// Ethereum (ERC721 Contract address, Token ID, Link source)
	Erc721(H160, U256, LinkSource),
}

#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ProtoOwner<TAccountId> {
	// A regular account on this chain
	User(TAccountId),
	// An external asset not on this chain
	ExternalAsset(LinkedAsset),
}

#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub struct AuthData {
	pub signature: ecdsa::Signature,
	pub block: u32,
}

#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug)]
pub struct Proto<TAccountId, TBlockNumber> {
	/// Block number this proto was included in
	pub block: TBlockNumber,
	/// Plain hash of indexed data.
	pub patches: Vec<Hash256>,
	/// Base include cost, of referenced protos.
	pub base_cost: Compact<u128>,
	/// Include price of the proto.
	/// If None, this proto can't be included into other protos
	pub include_cost: Option<Compact<u128>>,
	/// The original creator of the proto.
	pub creator: TAccountId,
	/// The current owner of the proto.
	pub owner: ProtoOwner<TAccountId>,
	/// References to other protos.
	pub references: Vec<Hash256>,
	/// Tags associated with this proto
	pub tags: Vec<Tags>,
	/// Metadata attached to the proto.
	pub metadata: BTreeMap<Vec<u8>, Hash256>,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use pallet_detach::{DetachRequest, DetachRequests, DetachedHashes, SupportedChains};

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config + pallet_detach::Config + pallet_randomness_collective_flip::Config
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type WeightInfo: WeightInfo;
		#[pallet::constant]
		type StorageBytesMultiplier: Get<u64>;
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig {
		pub upload_authorities: Vec<ecdsa::Public>,
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			Self { upload_authorities: Vec::new() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			Pallet::<T>::initialize_upload_authorities(&self.upload_authorities);
		}
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub type UserNonces<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u64>;

	#[pallet::storage]
	pub type Protos<T: Config> =
		StorageMap<_, Identity, Hash256, Proto<T::AccountId, T::BlockNumber>>;

	#[pallet::storage]
	pub type ProtosByTag<T: Config> = StorageDoubleMap<_, Blake2_128Concat, Tags, Identity, Hash256, bool>;

	#[pallet::storage]
	pub type ProtosByOwner<T: Config> = StorageDoubleMap<_, Blake2_128Concat, ProtoOwner<T::AccountId>, Identity, Hash256, bool>;

	#[pallet::storage]
	pub type UploadAuthorities<T: Config> = StorageValue<_, BTreeSet<ecdsa::Public>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Uploaded(Hash256),
		Patched(Hash256),
		MetadataChanged(Hash256, Vec<u8>),
		Detached(Hash256, Vec<u8>),
		Transferred(Hash256, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Systematic failure - those errors should not happen.
		SystematicFailure,
		/// Proto not found
		ProtoNotFound,
		/// Proto already uploaded
		ProtoExists,
		/// Already detached
		Detached,
		/// Not the owner of the proto
		Unauthorized,
		/// Signature verification failed
		VerificationFailed,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(<T as pallet::Config>::WeightInfo::add_upload_auth())]
		pub fn add_upload_auth(origin: OriginFor<T>, public: ecdsa::Public) -> DispatchResult {
			ensure_root(origin)?;

			log::debug!("New upload auth: {:?}", public);

			<UploadAuthorities<T>>::mutate(|validators| {
				validators.insert(public);
			});

			Ok(())
		}

		#[pallet::weight(<T as pallet::Config>::WeightInfo::del_upload_auth())]
		pub fn del_upload_auth(origin: OriginFor<T>, public: ecdsa::Public) -> DispatchResult {
			ensure_root(origin)?;

			log::debug!("Removed upload auth: {:?}", public);

			<UploadAuthorities<T>>::mutate(|validators| {
				validators.remove(&public);
			});

			Ok(())
		}

		/// Proto upload function.
		#[pallet::weight(<T as pallet::Config>::WeightInfo::upload() + (data.len() as u64 * <T as pallet::Config>::StorageBytesMultiplier::get()))]
		pub fn upload(
			origin: OriginFor<T>,
			auth: AuthData,
			// we store this in the state as well
			references: Vec<Hash256>,
			tags: Vec<Tags>,
			linked_asset: Option<LinkedAsset>,
			include_cost: Option<Compact<u128>>,
			// let data come last as we record this size in blocks db (storage chain)
			// and the offset is calculated like
			// https://github.com/paritytech/substrate/blob/a57bc4445a4e0bfd5c79c111add9d0db1a265507/client/db/src/lib.rs#L1678
			data: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let nonce = <UserNonces<T>>::get(who.clone()).unwrap_or(0);

			// hash the immutable data, this is also the unique proto id
			// to compose the V1 Cid add this prefix to the hash: (str "z" (base58
			// "0x0155a0e40220"))
			let proto_hash = blake2_256(&data);
			let signature_hash = blake2_256(
				&[
					&proto_hash[..],
					&references.encode(),
					&tags.encode(),
					&linked_asset.encode(),
					&nonce.encode(),
					&auth.block.encode(),
				]
				.concat(),
			);

			let current_block_number = <frame_system::Pallet<T>>::block_number();

			<Pallet<T>>::ensure_auth(current_block_number, &auth, &signature_hash)?;

			// make sure the proto does not exist already!
			ensure!(!<Protos<T>>::contains_key(&proto_hash), Error::<T>::ProtoExists);

			// we need this to index transactions
			let extrinsic_index = <frame_system::Pallet<T>>::extrinsic_index()
				.ok_or(Error::<T>::SystematicFailure)?;

			// Calculate the base cost of inclusion
			let cost = references.iter().fold(0, |acc, ref_hash| {
				let ref_cost = if let Some(proto) = <Protos<T>>::get(ref_hash) {
					if let Some(cost) = proto.include_cost {
						cost.into()
					} else {
						0
					}
				} else {
					0
				};
				acc + ref_cost
			});

			let owner = if let Some(link) = linked_asset {
				ProtoOwner::ExternalAsset(link)
			} else {
				ProtoOwner::User(who.clone())
			};

			// Write STATE from now, ensure no errors from now...

			// store in the state the proto
			let proto = Proto {
				block: current_block_number,
				patches: vec![],
				base_cost: cost.into(),
				include_cost,
				creator: who.clone(),
				owner: owner.clone(),
				references,
				tags: tags.clone(),
				metadata: BTreeMap::new(),
			};

			// store proto
			<Protos<T>>::insert(proto_hash, proto);

			// store by tags
			for tag in tags {
				<ProtosByTag<T>>::insert(tag, proto_hash, true);
			}

			<ProtosByOwner<T>>::insert(owner, proto_hash, true);

			// index immutable data for IPFS discovery
			transaction_index::index(extrinsic_index, data.len() as u32, proto_hash);

			// advance nonces
			<UserNonces<T>>::insert(who, nonce + 1);

			// also emit event
			Self::deposit_event(Event::Uploaded(proto_hash));

			log::debug!("Uploaded proto: {:?}", proto_hash);

			Ok(())
		}

		/// Proto upload function.
		#[pallet::weight(<T as pallet::Config>::WeightInfo::patch() + if let Some(data) = data { data.len() as u64 * <T as pallet::Config>::StorageBytesMultiplier::get() } else { 0 })]
		pub fn patch(
			origin: OriginFor<T>,
			auth: AuthData,
			// proto hash we want to patch
			proto_hash: Hash256,
			include_cost: Option<Compact<u128>>,
			// data we want to patch last because of the way we store blocks (storage chain)
			data: Option<Vec<u8>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let nonce = <UserNonces<T>>::get(who.clone()).unwrap_or(0);

			let proto: Proto<T::AccountId, T::BlockNumber> =
				<Protos<T>>::get(&proto_hash).ok_or(Error::<T>::ProtoNotFound)?;

			match proto.owner {
				ProtoOwner::User(owner) => ensure!(owner == who, Error::<T>::Unauthorized),
				ProtoOwner::ExternalAsset(_ext_asset) =>
				// We don't allow updating external assets
				{
					ensure!(false, Error::<T>::Unauthorized)
				},
			};

			ensure!(!<DetachedHashes<T>>::contains_key(&proto_hash), Error::<T>::Detached);

			let data_hash = blake2_256(&data.encode());
			let signature_hash = blake2_256(
				&[&proto_hash[..], &data_hash[..], &nonce.encode(), &auth.block.encode()].concat(),
			);

			let current_block_number = <frame_system::Pallet<T>>::block_number();

			<Pallet<T>>::ensure_auth(current_block_number, &auth, &signature_hash)?;

			// we need this to index transactions
			let extrinsic_index = <frame_system::Pallet<T>>::extrinsic_index()
				.ok_or(Error::<T>::SystematicFailure)?;

			// Write STATE from now, ensure no errors from now...

			<Protos<T>>::mutate(&proto_hash, |proto| {
				let proto = proto.as_mut().unwrap();
				if let Some(data) = data {
					// No failures from here on out
					proto.patches.push(data_hash);
					// index mutable data for IPFS discovery as well
					transaction_index::index(extrinsic_index, data.len() as u32, data_hash);
				}
				proto.include_cost = include_cost;
			});

			// advance nonces
			<UserNonces<T>>::insert(who, nonce + 1);

			// also emit event
			Self::deposit_event(Event::Patched(proto_hash));

			log::debug!("Updated proto: {:?}", proto_hash);

			Ok(())
		}

		/// Transfer proto ownership
		#[pallet::weight(<T as pallet::Config>::WeightInfo::transfer())]
		pub fn transfer(
			origin: OriginFor<T>,
			proto_hash: Hash256,
			new_owner: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// make sure the proto exists
			let proto: Proto<T::AccountId, T::BlockNumber> =
				<Protos<T>>::get(&proto_hash).ok_or(Error::<T>::ProtoNotFound)?;

			// make sure the caller is the owner
			match proto.owner.clone() {
				ProtoOwner::User(owner) => ensure!(owner == who, Error::<T>::Unauthorized),
				ProtoOwner::ExternalAsset(_ext_asset) =>
				// We don't allow updating external assets
				{
					ensure!(false, Error::<T>::Unauthorized)
				},
			};

			// make sure the proto is not detached
			ensure!(!<DetachedHashes<T>>::contains_key(&proto_hash), Error::<T>::Detached);

			// collect new owner
			let new_owner_s = ProtoOwner::User(new_owner.clone());

			// WRITING STATE FROM NOW

			// remove proto from old owner
			<ProtosByOwner<T>>::remove(proto.owner, proto_hash);

			// add proto to new owner
			<ProtosByOwner<T>>::insert(new_owner_s.clone(), proto_hash, true);

			// update proto
			<Protos<T>>::mutate(&proto_hash, |proto| {
				let proto = proto.as_mut().unwrap();
				proto.owner = new_owner_s;
			});

			// emit event
			Self::deposit_event(Event::Transferred(proto_hash, new_owner));

			Ok(())
		}

		/// Proto upload function.
		#[pallet::weight(<T as pallet::Config>::WeightInfo::patch() + (data.len() as u64 * <T as pallet::Config>::StorageBytesMultiplier::get()))]
		pub fn set_metadata(
			origin: OriginFor<T>,
			auth: AuthData,
			// proto hash we want to update
			proto_hash: Hash256,
			metadata_key: Vec<u8>,
			// data we want to update last because of the way we store blocks (storage chain)
			data: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let nonce = <UserNonces<T>>::get(who.clone()).unwrap_or(0);

			let proto: Proto<T::AccountId, T::BlockNumber> =
				<Protos<T>>::get(&proto_hash).ok_or(Error::<T>::ProtoNotFound)?;

			match proto.owner {
				ProtoOwner::User(owner) => ensure!(owner == who, Error::<T>::Unauthorized),
				ProtoOwner::ExternalAsset(_ext_asset) =>
				// We don't allow updating external assets
				{
					ensure!(false, Error::<T>::Unauthorized)
				},
			};

			ensure!(!<DetachedHashes<T>>::contains_key(&proto_hash), Error::<T>::Detached);

			let data_hash = blake2_256(&data.encode());
			let signature_hash = blake2_256(
				&[&proto_hash[..], &data_hash[..], &nonce.encode(), &auth.block.encode()].concat(),
			);

			let current_block_number = <frame_system::Pallet<T>>::block_number();

			<Pallet<T>>::ensure_auth(current_block_number, &auth, &signature_hash)?;

			// we need this to index transactions
			let extrinsic_index = <frame_system::Pallet<T>>::extrinsic_index()
				.ok_or(Error::<T>::SystematicFailure)?;

			// Write STATE from now, ensure no errors from now...

			<Protos<T>>::mutate(&proto_hash, |proto| {
				let proto = proto.as_mut().unwrap();
				// update metadata
				proto.metadata.insert(metadata_key.clone(), data_hash);
			});

			// index data
			transaction_index::index(extrinsic_index, data.len() as u32, data_hash);

			// advance nonces
			<UserNonces<T>>::insert(who, nonce + 1);

			// also emit event
			Self::deposit_event(Event::MetadataChanged(proto_hash, metadata_key.clone()));

			log::debug!("Added metadata to proto: {:x?} with key: {:x?}", proto_hash, metadata_key);

			Ok(())
		}

		/// Detached a proto from this chain by emitting an event that includes a signature.
		/// The remote target chain can attach this proto by using this signature.
		#[pallet::weight(<T as pallet::Config>::WeightInfo::detach())]
		pub fn detach(
			origin: OriginFor<T>,
			proto_hash: Hash256,
			target_chain: SupportedChains,
			target_account: Vec<u8>, // an eth address or so
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// make sure the proto exists
			let proto: Proto<T::AccountId, T::BlockNumber> =
				<Protos<T>>::get(&proto_hash).ok_or(Error::<T>::ProtoNotFound)?;

			match proto.owner {
				ProtoOwner::User(owner) => ensure!(owner == who, Error::<T>::Unauthorized),
				ProtoOwner::ExternalAsset(_ext_asset) =>
				// We don't allow detaching external assets
				{
					ensure!(false, Error::<T>::Unauthorized)
				},
			};

			ensure!(!<DetachedHashes<T>>::contains_key(&proto_hash), Error::<T>::Detached);

			<DetachRequests<T>>::mutate(|requests| {
				requests.push(DetachRequest { hash: proto_hash, target_chain, target_account });
			});

			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(_n: T::BlockNumber) {}

		fn offchain_worker(_n: T::BlockNumber) {}
	}

	impl<T: Config> Pallet<T> {

		fn is_proto_having_any_tags(proto_hash: &Hash256, tags: &Vec<Tags>) -> bool {
			if let Some(struct_proto) = <Protos<T>>::get(proto_hash) {
				tags.into_iter().any(|tag| struct_proto.tags.contains(&tag))
			} else {
				false
			}
		}

		pub fn get_by_tags(tags: Vec<Tags>, owner: Option<T::AccountId>, limit: u32, from: u32, desc: bool) -> Vec<Hash256> {


			// log::info!("inside get_by_tags");
			// log::info!("tags: {:?}, owner: {:?}, limit: {}", tags, owner, limit);

			match owner {
				Some(owner) => {

					let iter_protos = <ProtosByOwner<T>>::iter_key_prefix(ProtoOwner::User(owner));

					let iter_protos_filtered = iter_protos.filter(|proto| Self::is_proto_having_any_tags(proto, &tags));
					let iter_protos_limited = iter_protos_filtered.skip(from as usize).take(limit as usize);
					iter_protos_limited.collect::<Vec<Hash256>>()

				},
				None => {
					let mut remaining = limit;

					let iter_protos = tags.into_iter().map(|tag| {
						let vector_protos = <ProtosByTag<T>>::iter_key_prefix(tag).take(remaining as usize).collect::<Vec<Hash256>>();
						remaining -= vector_protos.len() as u32;
						vector_protos
					}).flatten().skip(from as usize).take(limit as usize);

					iter_protos.collect::<Vec<Hash256>>()
				}

			}

		}


		pub fn get_metadata_batch(batch: Vec<Hash256>, keys: Vec<Vec<u8>>) -> Vec<Option<Vec<Hash256>>> {

			batch.into_iter().map(|proto_hash| -> Option<Vec<Hash256>> {

				let proto = <Protos<T>>::get(&proto_hash)?;

				let mut vec_data_hash : Vec<Hash256> = Vec::new();

				for key in &keys {
					let data_hash = proto.metadata.get(key)?;
					vec_data_hash.push(*data_hash);
				}

				Some(vec_data_hash)

			}).collect::<Vec<Option<Vec<Hash256>>>>()

		}

		fn ensure_auth(
			block_number: T::BlockNumber,
			data: &AuthData,
			signature_hash: &[u8; 32],
		) -> DispatchResult {
			// check if the signature is valid
			// we use and off chain services that ensure we are storing valid data
			let recover =
				Crypto::secp256k1_ecdsa_recover_compressed(&data.signature.0, signature_hash)
					.ok()
					.ok_or(Error::<T>::VerificationFailed)?;
			let recover = ecdsa::Public(recover);
			ensure!(
				<UploadAuthorities<T>>::get().contains(&recover),
				Error::<T>::VerificationFailed
			);

			let max_delay = block_number + 100u32.into();
			let signed_at: T::BlockNumber = data.block.into();
			ensure!(signed_at < max_delay, Error::<T>::VerificationFailed);

			Ok(())
		}

		fn initialize_upload_authorities(authorities: &[ecdsa::Public]) {
			if !authorities.is_empty() {
				assert!(
					<UploadAuthorities<T>>::get().is_empty(),
					"UploadAuthorities are already initialized!"
				);
				for authority in authorities {
					<UploadAuthorities<T>>::mutate(|authorities| {
						authorities.insert(authority.clone());
					});
				}
			}
		}
	}
}
