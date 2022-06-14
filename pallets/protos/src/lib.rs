#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod tests;

mod weights;

use protos::categories::Categories;

use sp_core::{ecdsa, H160, U256};

use codec::{Compact, Decode, Encode};
pub use pallet::*;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use sp_io::{hashing::blake2_256, transaction_index};
use sp_std::{collections::btree_map::BTreeMap, vec, vec::Vec};

pub use weights::WeightInfo;

use sp_clamor::{get_locked_frag_account, Hash256};

use scale_info::prelude::{
	format,
	string::{String, ToString},
};
use serde_json::{json, Map, Value};

use base58::ToBase58;

/// ¿
#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub enum LinkSource {
	// Generally we just store this data, we don't verify it as we assume auth service did it.
	// (Link signature, Linked block number, EIP155 Chain ID)
	Evm(ecdsa::Signature, u64, U256),
}

/// Types of Assets that are linked to a Proto-Fragment (e.g an ERC-721 Contract etc.)
#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub enum LinkedAsset {
	// Ethereum (ERC721 Contract address, Token ID, Link source)
	Erc721(H160, U256, LinkSource),
}

/// Types of Proto-Fragment Owner
#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub enum ProtoOwner<TAccountId> {
	// A regular account on this chain
	User(TAccountId),
	// An external asset not on this chain
	ExternalAsset(LinkedAsset),
}

#[derive(Encode, Decode, Clone, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct GetProtosParams<TAccountId, TString> {
	pub desc: bool,
	pub from: u64,
	pub limit: u64,
	pub metadata_keys: Vec<TString>,
	pub owner: Option<TAccountId>,
	pub return_owners: bool,
	pub categories: Vec<Categories>,
	pub tags: Vec<TString>,
	pub available: Option<bool>,
}

#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug)]
pub struct ProtoPatch<TBlockNumber> {
	/// The block when this patch was created
	pub block: TBlockNumber,
	/// The hash of this patch data
	pub data_hash: Hash256,
	/// A patch can add references to other protos, or remove references to other protos.
	pub references: Vec<Hash256>,
}

/// Struct of a Proto-Fragment
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug)]
pub struct Proto<TAccountId, TBlockNumber> {
	/// Block number this proto was included in
	pub block: TBlockNumber,
	/// Patches to this proto
	pub patches: Vec<ProtoPatch<TBlockNumber>>,
	/// Include price of the proto.
	/// If None, this proto can't be included into other protos
	pub include_cost: Option<Compact<u64>>,
	/// The original creator of the proto.
	pub creator: TAccountId,
	/// The current owner of the proto.
	pub owner: ProtoOwner<TAccountId>,
	/// References to other protos.
	pub references: Vec<Hash256>,
	/// Categories associated with this proto
	pub category: Categories,
	/// tags associated with this proto
	pub tags: Vec<Compact<u64>>,
	/// Metadata attached to the proto.
	pub metadata: BTreeMap<Compact<u64>, Hash256>,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, Twox64Concat};
	use frame_system::pallet_prelude::*;
	use pallet_detach::{DetachRequest, DetachRequests, DetachedHashes, SupportedChains};
	use sp_clamor::CID_PREFIX;
	use sp_runtime::{traits::Saturating, SaturatedConversion};

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ pallet_detach::Config
		+ pallet_tickets::Config
		+ pallet_randomness_collective_flip::Config
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type WeightInfo: WeightInfo;

		#[pallet::constant]
		type StorageBytesMultiplier: Get<u64>;

		#[pallet::constant]
		type StakeLockupPeriod: Get<u64>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub type Tags<T: Config> = StorageMap<_, Twox64Concat, Vec<u8>, u64>;

	#[pallet::storage]
	pub type TagsIndex<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	pub type MetaKeys<T: Config> = StorageMap<_, Twox64Concat, Vec<u8>, u64>;

	#[pallet::storage]
	pub type MetaKeysIndex<T: Config> = StorageValue<_, u64, ValueQuery>;

	/// Storage Map of Proto-Fragments where the key is the hash of the data of the Proto-Fragment, and the value is the Proto struct of the Proto-Fragment
	#[pallet::storage]
	pub type Protos<T: Config> =
		StorageMap<_, Identity, Hash256, Proto<T::AccountId, T::BlockNumber>>;

	/// Storage Map which keeps track of the Proto-Fragments by Category type.
	/// The key is the Category type and the value is a list of the hash of a Proto-Fragment
	// Not ideal but to have it iterable...
	#[pallet::storage]
	pub type ProtosByCategory<T: Config> = StorageMap<_, Twox64Concat, Categories, Vec<Hash256>>;

	/// UploadAuthorities is a StorageValue that keeps track of the set of ECDSA public keys of the upload authorities
	/// * Note: An upload authority (also known as the off-chain validator) provides the digital signature needed to upload a Proto-Fragment
	#[pallet::storage]
	pub type ProtosByOwner<T: Config> =
		StorageMap<_, Twox64Concat, ProtoOwner<T::AccountId>, Vec<Hash256>>;

	// Staking management
	// (Amount staked, Last stake time)
	#[pallet::storage]
	pub type ProtoStakes<T: Config> = StorageDoubleMap<
		_,
		Identity,
		Hash256,
		Twox64Concat,
		T::AccountId,
		(T::Balance, T::BlockNumber),
	>;

	#[pallet::storage]
	pub type AccountStakes<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Vec<Hash256>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A Proto-Fragment was uploaded
		Uploaded(Hash256, Vec<u8>),
		/// A Proto-Fragment was patched
		Patched(Hash256, Vec<u8>),
		/// A Proto-Fragment metadata has changed
		MetadataChanged(Hash256, Vec<u8>),
		/// A Proto-Fragment was detached
		Detached(Hash256, Vec<u8>),
		/// A Proto-Fragment was transferred
		Transferred(Hash256, T::AccountId),
		/// Stake was created
		Staked(Hash256, T::AccountId, T::Balance),
		/// Stake was unlocked
		Unstaked(Hash256, T::AccountId, T::Balance),
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
		/// Not enough FRAG staked
		NotEnoughStaked,
		/// Stake not found
		StakeNotFound,
		/// Reference not found
		ReferenceNotFound,
		/// Not enough tokens to stake
		InsufficientBalance,
		/// Cannot unstake yet
		StakeLocked,
		/// Cannot find FRAG link to use as stake funds
		NoFragLink,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		T::AccountId: AsRef<[u8]>,
	{
		/// Uploads a Proto-Fragment onto the Blockchain.
		/// Furthermore, this function also indexes `data` in the Blockchain's Database and makes it available via bitswap(IPFS) directly from every chain node permanently.
		///
		/// # Arguments
		///
		/// * `origin` - The origin of the extrinsic/dispatchable function
		/// * `references` - A list of references to other Proto-Fragments
		/// * `categories` - A list of categories to upload along with the Proto-Fragment
		/// * `linked_asset` - An asset that is linked with the uploaded Proto-Fragment (e.g an ERC-721 Contract)
		/// * `include_cost` (optional) -
		/// * `data` - The data of the Proto-Fragment (represented as a vector of bytes)
		#[pallet::weight(<T as pallet::Config>::WeightInfo::upload() + (data.len() as u64 * <T as pallet::Config>::StorageBytesMultiplier::get()))]
		pub fn upload(
			origin: OriginFor<T>,
			// we store this in the state as well
			references: Vec<Hash256>,
			category: Categories,
			tags: Vec<Vec<u8>>,
			linked_asset: Option<LinkedAsset>,
			include_cost: Option<Compact<u64>>,
			// let data come last as we record this size in blocks db (storage chain)
			// and the offset is calculated like
			// https://github.com/paritytech/substrate/blob/a57bc4445a4e0bfd5c79c111add9d0db1a265507/client/db/src/lib.rs#L1678
			data: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let current_block_number = <frame_system::Pallet<T>>::block_number();

			// hash the immutable data, this is also the unique proto id
			// to compose the V1 Cid add this prefix to the hash: (str "z" (base58
			// "0x0155a0e40220"))
			let proto_hash = blake2_256(&data);
			let data_len = data.len();

			// make sure the proto does not exist already!
			ensure!(!<Protos<T>>::contains_key(&proto_hash), Error::<T>::ProtoExists);

			// we need this to index transactions
			let extrinsic_index = <frame_system::Pallet<T>>::extrinsic_index()
				.ok_or(Error::<T>::SystematicFailure)?;

			// Check FRAG staking
			Self::check_staking_req(&references, &who)?;

			// ! Write STATE from now, ensure no errors from now...

			let owner = if let Some(link) = linked_asset {
				ProtoOwner::ExternalAsset(link)
			} else {
				ProtoOwner::User(who.clone())
			};

			let tags = tags
				.iter()
				.map(|s| {
					let tag_index = <Tags<T>>::get(s);
					if let Some(tag_index) = tag_index {
						<Compact<u64>>::from(tag_index)
					} else {
						let next_index = <TagsIndex<T>>::try_get().unwrap_or_default() + 1;
						<Tags<T>>::insert(s, next_index);
						// storing is dangerous inside a closure
						// but after this call we start storing..
						// so it's fine here
						<TagsIndex<T>>::put(next_index);
						<Compact<u64>>::from(next_index)
					}
				})
				.collect();

			// store in the state the proto
			let proto = Proto {
				block: current_block_number,
				patches: vec![],
				include_cost,
				creator: who.clone(),
				owner: owner.clone(),
				references,
				category: category.clone(),
				tags,
				metadata: BTreeMap::new(),
			};

			// store proto
			<Protos<T>>::insert(proto_hash, proto);

			// store by category
			<ProtosByCategory<T>>::append(category, proto_hash);

			<ProtosByOwner<T>>::append(owner, proto_hash);

			// index immutable data for IPFS discovery
			transaction_index::index(extrinsic_index, data_len as u32, proto_hash);

			let cid = [&CID_PREFIX[..], &proto_hash[..]].concat();
			let cid = cid.to_base58();
			let cid = [&b"z"[..], cid.as_bytes()].concat();

			// also emit event
			Self::deposit_event(Event::Uploaded(proto_hash, cid));

			log::debug!("Uploaded proto: {:?}", proto_hash);

			Ok(())
		}

		/// Patches (i.e modifies) the existing Proto-Fragment (whose hash is `proto_hash`) by appending the hash of `data` to the Vector field `patches` of the existing Proto-Fragment's Struct Instance.
		/// Furthermore, this function also indexes `data` in the Blockchain's Database and stores it in the IPFS
		/// To successfully patch a Proto-Fragment, the `auth` provided must be valid. Otherwise, an error is returned
		///
		/// # Arguments
		///
		/// * `origin` - The origin of the extrinsic / dispatchable function
		/// * `proto_hash` - The hash of the existing Proto-Fragment
		/// * `include_cost` (optional) -
		/// * `data` - The data of the Proto-Fragment (represented as a vector of bytes)
		#[pallet::weight(<T as pallet::Config>::WeightInfo::patch() + data.len() as u64 * <T as pallet::Config>::StorageBytesMultiplier::get())]
		pub fn patch(
			origin: OriginFor<T>,
			// proto hash we want to patch
			proto_hash: Hash256,
			include_cost: Option<Compact<u64>>,
			new_references: Vec<Hash256>,
			// data we want to patch last because of the way we store blocks (storage chain)
			data: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

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

			let data_hash = blake2_256(&data);

			let current_block_number = <frame_system::Pallet<T>>::block_number();

			// we need this to index transactions
			let extrinsic_index = <frame_system::Pallet<T>>::extrinsic_index()
				.ok_or(Error::<T>::SystematicFailure)?;

			// Check FRAG staking
			Self::check_staking_req(&new_references, &who)?;

			// Write STATE from now, ensure no errors from now...

			<Protos<T>>::mutate(&proto_hash, |proto| {
				let proto = proto.as_mut().unwrap();
				if !data.is_empty() {
					// No failures from here on out
					proto.patches.push(ProtoPatch {
						block: current_block_number,
						data_hash,
						references: new_references,
					});
					// index mutable data for IPFS discovery as well
					transaction_index::index(extrinsic_index, data.len() as u32, data_hash);
				}
				proto.include_cost = include_cost;
			});

			let cid = [&CID_PREFIX[..], &data_hash[..]].concat();
			let cid = cid.to_base58();
			let cid = [&b"z"[..], cid.as_bytes()].concat();

			// also emit event
			Self::deposit_event(Event::Patched(proto_hash, cid));

			log::debug!("Updated proto: {:?}", proto_hash);

			Ok(())
		}

		/// Transfer the ownership of a Proto-Fragment from `origin` to `new_owner`
		///
		/// # Arguments
		///
		/// * `origin` - The origin of the extrinsic / dispatchable function
		/// * `proto_hash` - The hash of the data of the Proto-Fragment that you want to transfer
		/// * `new_owner` - The AccountId of the account you want to transfer the Proto-Fragment to
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
			<ProtosByOwner<T>>::mutate(proto.owner, |proto_by_owner| {
				if let Some(list) = proto_by_owner {
					list.retain(|current_hash| proto_hash != *current_hash);
				}
			});

			// add proto to new owner
			<ProtosByOwner<T>>::append(new_owner_s.clone(), proto_hash);

			// update proto
			<Protos<T>>::mutate(&proto_hash, |proto| {
				let proto = proto.as_mut().unwrap();
				proto.owner = new_owner_s;
			});

			// emit event
			Self::deposit_event(Event::Transferred(proto_hash, new_owner));

			Ok(())
		}

		/// Alters the metadata of an existing Proto-Fragment (whose hash is `proto_hash`) by adding the key-value pair (`metadata_key.clone`,`blake2_256(&data.encode())`) to the BTreeMap field `metadata` of the existing Proto-Fragment's Struct Instance.
		/// Furthermore, this function also indexes `data` in the Blockchain's Database and stores it in the IPFS
		/// To successfully patch a Proto-Fragment, the `auth` provided must be valid. Otherwise, an error is returned
		///
		/// # Arguments
		///
		/// * `origin` - The origin of the extrinsic / dispatchable function
		/// * `proto_hash` - The hash of the existing Proto-Fragment
		/// * `metadata_key` - The key (of the key-value pair) that is added in the BTreeMap field `metadata` of the existing Proto-Fragment's Struct Instance
		/// * `data` - The hash of `data` is used as the value (of the key-value pair) that is added in the BTreeMap field `metadata` of the existing Proto-Fragment's Struct Instance
		#[pallet::weight(<T as pallet::Config>::WeightInfo::patch() + (data.len() as u64 * <T as pallet::Config>::StorageBytesMultiplier::get()))]
		pub fn set_metadata(
			origin: OriginFor<T>,
			// proto hash we want to update
			proto_hash: Hash256,
			// Think of "Vec<u8>" as String (something to do with WASM - that's why we use Vec<u8>)
			metadata_key: Vec<u8>,
			// data we want to update last because of the way we store blocks (storage chain)
			data: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

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

			let data_hash = blake2_256(&data);

			// we need this to index transactions
			let extrinsic_index = <frame_system::Pallet<T>>::extrinsic_index()
				.ok_or(Error::<T>::SystematicFailure)?;

			let metadata_key_index = {
				let index = <MetaKeys<T>>::get(metadata_key.clone());
				if let Some(index) = index {
					<Compact<u64>>::from(index)
				} else {
					let next_index = <MetaKeysIndex<T>>::try_get().unwrap_or_default() + 1;
					<MetaKeys<T>>::insert(metadata_key.clone(), next_index);
					// storing is dangerous inside a closure
					// but after this call we start storing..
					// so it's fine here
					<MetaKeysIndex<T>>::put(next_index);
					<Compact<u64>>::from(next_index)
				}
			};

			// Write STATE from now, ensure no errors from now...

			<Protos<T>>::mutate(&proto_hash, |proto| {
				let proto = proto.as_mut().unwrap();
				// update metadata
				proto.metadata.insert(metadata_key_index, data_hash);
			});

			// index data
			transaction_index::index(extrinsic_index, data.len() as u32, data_hash);

			// also emit event
			Self::deposit_event(Event::MetadataChanged(proto_hash, metadata_key.clone()));

			log::debug!("Added metadata to proto: {:x?} with key: {:x?}", proto_hash, metadata_key);

			Ok(())
		}

		/// Detach a proto from this chain by emitting an event that includes a signature.
		/// The remote target chain can attach this proto by using this signature.
		///
		///
		/// # Arguments
		///
		/// * `origin` - The origin of the extrinsic / dispatchable function
		/// * `proto_hash` - The hash of the existing Proto-Fragment to detach
		/// * `target_chain` - The key (of the key-value pair) that is added in the BTreeMap field `metadata` of the existing Proto-Fragment's Struct Instance
		/// * `target_account` - The public account address on the blockchain `target_chain` that we want to detach the existing Proto-Fragment into
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

		#[pallet::weight(50_000)]
		pub fn stake(
			origin: OriginFor<T>,
			proto_hash: Hash256,
			amount: T::Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;
			let who = get_locked_frag_account(&who).map_err(|_| Error::<T>::SystematicFailure)?;

			// make sure the proto exists
			ensure!(<Protos<T>>::contains_key(&proto_hash), Error::<T>::ProtoNotFound);

			// make sure user has enough FRAG
			let account = <pallet_tickets::EVMLinks<T>>::get(&who.clone())
				.ok_or_else(|| Error::<T>::NoFragLink)?;
			let eth_lock = <pallet_tickets::EthLockedFrag<T>>::get(&account)
				.ok_or_else(|| Error::<T>::NoFragLink)?;
			let balance = eth_lock.amount
				- <pallet_tickets::FragUsage<T>>::get(&who.clone())
					.ok_or_else(|| Error::<T>::NoFragLink)?;
			ensure!(balance >= amount, Error::<T>::InsufficientBalance);

			let current_block_number = <frame_system::Pallet<T>>::block_number();

			// ! from now we write...

			<pallet_tickets::FragUsage<T>>::mutate(&who, |usage| {
				usage.as_mut().unwrap().saturating_add(amount);
			});

			// take record of the stake
			<ProtoStakes<T>>::insert(proto_hash, &who, (amount, current_block_number));
			<AccountStakes<T>>::append(who.clone(), proto_hash.clone());

			// also emit event
			Self::deposit_event(Event::Staked(proto_hash, who, amount));

			Ok(())
		}

		#[pallet::weight(50_000)]
		pub fn unstake(origin: OriginFor<T>, proto_hash: Hash256) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			// make sure the proto exists
			ensure!(<Protos<T>>::contains_key(&proto_hash), Error::<T>::ProtoNotFound);

			// make sure user has enough FRAG
			let stake =
				<ProtoStakes<T>>::get(&proto_hash, &who).ok_or(Error::<T>::StakeNotFound)?;

			let current_block_number = <frame_system::Pallet<T>>::block_number();
			ensure!(
				current_block_number > (stake.1 + T::StakeLockupPeriod::get().saturated_into()),
				Error::<T>::StakeLocked
			);

			// ! from now we write...

			<pallet_tickets::FragUsage<T>>::mutate(&who, |usage| {
				usage.as_mut().unwrap().saturating_sub(stake.0);
			});

			// take record of the unstake
			<ProtoStakes<T>>::remove(proto_hash, &who);
			<AccountStakes<T>>::mutate(who.clone(), |stakes| {
				if let Some(stakes) = stakes {
					stakes.retain(|h| h != &proto_hash);
				}
			});

			// also emit event
			Self::deposit_event(Event::Unstaked(proto_hash, who, stake.0));

			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(_n: T::BlockNumber) {
			// drain unlinks
			let unlinks = <pallet_tickets::PendingUnlinks<T>>::take();
			for unlink in unlinks {
				// take emptying the storage
				let stakes = <AccountStakes<T>>::take(unlink.clone());
				if let Some(stakes) = stakes {
					for stake in stakes {
						<ProtoStakes<T>>::remove(stake, &unlink);
					}
				}
			}
		}
	}

	impl<T: Config> Pallet<T>
	where
		T::AccountId: AsRef<[u8]>,
	{
		fn check_staking_req(references: &[Hash256], who: &T::AccountId) -> DispatchResult {
			// Check FRAG staking
			// TODO this is not tested properly
			for reference in references.iter() {
				let proto = <Protos<T>>::get(reference);
				if let Some(proto) = proto {
					let owner = match proto.owner {
						ProtoOwner::User(owner) => Some(owner),
						_ => None,
					};

					if let Some(owner) = owner {
						if owner == *who {
							// owner can include freely
							continue;
						}
					}

					let cost = proto.include_cost;
					if let Some(cost) = cost {
						let cost: u64 = cost.into();
						if cost == 0 {
							// free
							continue;
						}
						let cost: T::Balance = cost.saturated_into();
						let stake = <ProtoStakes<T>>::get(reference, who.clone());
						if let Some(stake) = stake {
							ensure!(stake.0 >= cost, Error::<T>::NotEnoughStaked);
						} else {
							// Stake not found
							return Err(Error::<T>::StakeNotFound.into());
						}
					// OK to include, just continue
					} else {
						return Err(Error::<T>::Unauthorized.into());
					}
				} else {
					// Proto not found
					return Err(Error::<T>::ReferenceNotFound.into());
				}
			}
			Ok(())
		}

		fn filter_proto(proto_id: &Hash256, tags: &[Vec<u8>], categories: &[Categories], avail: Option<bool>) -> bool {
			if let Some(struct_proto) = <Protos<T>>::get(proto_id) {
				if let Some(avail) = avail {
					if avail && struct_proto.include_cost.is_none() {
						return false;
					} else if !avail && struct_proto.include_cost.is_some() {
						return false;
					}
				}

				if categories.len() == 0
				// Use any here to match any category towards proto
					|| categories.into_iter().any(|cat| *cat == struct_proto.category)
				{
					// Use all here to match all tags always
					if tags.len() == 0 {
						true
					} else {
						tags.into_iter().all(|tag| {
							let tag_idx = <Tags<T>>::get(tag);
							if let Some(tag_idx) = tag_idx {
								struct_proto.tags.contains(&Compact::from(tag_idx))
							} else {
								false
							}
						})
					}
				} else {
					false
				}
			} else {
				false
			}
		}

		pub fn get_protos(
			params: GetProtosParams<T::AccountId, Vec<u8>>,
		) -> Result<Vec<u8>, Vec<u8>> {
			let mut map = Map::new();

			let list_protos_final: Vec<Hash256> = if let Some(owner) = params.owner {
				// `owner` exists
				if let Some(list_protos_owner) =
					<ProtosByOwner<T>>::get(ProtoOwner::<T::AccountId>::User(owner))
				{
					// `owner` exists in `ProtosByOwner`
					if params.desc {
						// Sort in descending order
						list_protos_owner
							.into_iter()
							.rev()
							.filter(|proto_id| {
								Self::filter_proto(proto_id, &params.tags, &params.categories, params.available)
							})
							.skip(params.from as usize)
							.take(params.limit as usize)
							.collect::<Vec<Hash256>>()
					} else {
						// Sort in ascending order
						list_protos_owner
							.into_iter()
							.filter(|proto_id| {
								Self::filter_proto(proto_id, &params.tags, &params.categories, params.available)
							})
							.skip(params.from as usize)
							.take(params.limit as usize)
							.collect::<Vec<Hash256>>()
					}
				} else {
					// `owner` doesn't exist in `ProtosByOwner`
					return Err("Owner not found".into());
				}
			} else {
				// Notice this wastes time and memory and needs a better implementation
				let mut flat = Vec::<Hash256>::new();

				let cats = <ProtosByCategory<T>>::iter_keys().collect::<Vec<Categories>>();
				let cats: Vec<Categories> =
					if params.desc { cats.iter().rev().map(|x| x.clone()).collect() } else { cats };

				for category in cats {
					if params.categories.len() != 0 && !params.categories.contains(&category) {
						continue;
					}

					let protos = <ProtosByCategory<T>>::get(category);
					if let Some(protos) = protos {
						let collection: Vec<Hash256> = if params.desc {
							// Sort in descending order
							protos
								.into_iter()
								.rev()
								.filter(|proto_id| {
									Self::filter_proto(proto_id, &params.tags, &params.categories, params.available)
								})
								.collect()
						} else {
							// Sort in ascending order
							protos
								.into_iter()
								.filter(|proto_id| {
									Self::filter_proto(proto_id, &params.tags, &params.categories, params.available)
								})
								.collect()
						};
						flat.extend(collection);
					}
				}

				flat.iter()
					.skip(params.from as usize)
					.take(params.limit as usize)
					.map(|x| *x)
					.collect()
			};

			for proto_id in list_protos_final.into_iter() {
				map.insert(hex::encode(proto_id), Value::Object(Map::new()));
			}

			if params.return_owners || !params.metadata_keys.is_empty() {
				for (proto_id, map_proto) in map.iter_mut() {
					let array_proto_id: Hash256 = if let Ok(array_proto_id) = hex::decode(proto_id)
					{
						if let Ok(array_proto_id) = array_proto_id.try_into() {
							array_proto_id
						} else {
							return Err("Failed to convert proto_id to Hash256".into());
						}
					} else {
						return Err("Failed to decode proto_id".into());
					};

					let (owner, map_metadata, include_cost) =
						if let Some(proto) = <Protos<T>>::get(array_proto_id) {
							(proto.owner, proto.metadata, proto.include_cost)
						} else {
							return Err("Failed to get proto".into());
						};

					let map_proto = match map_proto {
						Value::Object(map_proto) => map_proto,
						_ => return Err("Failed to get map_proto".into()),
					};

					if let Some(include_cost) = include_cost {
						let n: u64 = include_cost.into();
						(*map_proto).insert(String::from("include_cost"), Value::Number(n.into()));
					} else {
						(*map_proto).insert(String::from("include_cost"), Value::Null);
					}

					if params.return_owners {
						let owner = match owner {
							ProtoOwner::User(account_id) => json!({
								"type": "internal",
								"value": hex::encode(account_id)
							}),
							ProtoOwner::ExternalAsset(linked_asset) => {
								let value = match linked_asset {
									LinkedAsset::Erc721(contract, token_id, source) => {
										let chain_id = match source {
											LinkSource::Evm(_sig, _block, chain_id) => chain_id,
										};
										json!({
											"type": "erc721",
											"value": {
												"contract": format!("0x{:x}", contract),
												"token_id": format!("0x{:x}", token_id),
												"chain_id": format!("0x{:x}", chain_id)
											}
										})
									},
								};
								json!({
									"type": "external",
									"value": value,
								})
							},
						};

						(*map_proto).insert(String::from("owner"), owner);
					}

					if !params.metadata_keys.is_empty() {
						for metadata_key in params.metadata_keys.iter() {
							let metadata_key_index = <MetaKeys<T>>::get(metadata_key.clone());
							let metadata_value = if let Some(metadata_key_index) =
								metadata_key_index
							{
								let metadata_key_index = <Compact<u64>>::from(metadata_key_index);

								if let Some(data_hash) = map_metadata.get(&metadata_key_index) {
									Value::String(hex::encode(data_hash))
								} else {
									Value::Null
								}
							} else {
								Value::Null
							};

							if let Ok(key) = String::from_utf8(metadata_key.clone()) {
								(*map_proto).insert(key, metadata_value);
							}
						}
					}
				}
			}

			let result = json!(map).to_string();

			Ok(result.into_bytes())
		}
	}
}
