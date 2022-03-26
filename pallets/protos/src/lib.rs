#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod weights;

use sp_core::{crypto::KeyTypeId, ecdsa, ed25519, H160, U256};

/// Defines application identifier for crypto keys of this module.
///
/// Every module that deals with signatures needs to declare its unique identifier for
/// its crypto keys.
/// When offchain worker is signing transactions it's going to request keys of type
/// `KeyTypeId` from the keystore and use the ones it finds to sign the transaction.
/// The keys can be inserted manually via RPC (see `author_insertKey`).
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"frag");

/// Based on the above `KeyTypeId` we need to generate a pallet-specific crypto type wrappers.
/// We can use from supported crypto kinds (`sr25519`, `ed25519` and `ecdsa`) and augment
/// the types with this pallet-specific identifier.
pub mod crypto {
	use super::KEY_TYPE;
	use sp_core::ed25519::Signature as Ed25519Signature;
	use sp_runtime::{
		app_crypto::{app_crypto, ed25519},
		traits::Verify,
		MultiSignature, MultiSigner,
	};
	app_crypto!(ed25519, KEY_TYPE);

	pub struct DetachAuthId;

	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for DetachAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::ed25519::Signature;
		type GenericPublic = sp_core::ed25519::Public;
	}

	// implemented for mock runtime in test
	impl frame_system::offchain::AppCrypto<<Ed25519Signature as Verify>::Signer, Ed25519Signature>
		for DetachAuthId
	{
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::ed25519::Signature;
		type GenericPublic = sp_core::ed25519::Public;
	}
}

use codec::{Compact, Decode, Encode};
pub use pallet::*;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_io::{crypto as Crypto, hashing::blake2_256, offchain, transaction_index};
use sp_runtime::offchain::storage::StorageValueRef;
use sp_std::{
	collections::{btree_map::BTreeMap, btree_set::BTreeSet},
	vec,
	vec::Vec,
};

use frame_system::offchain::{
	AppCrypto, CreateSignedTransaction, SendUnsignedTransaction, SignedPayload, Signer,
	SigningTypes,
};

pub use weights::WeightInfo;

use sp_chainblocks::Hash256;

use scale_info::prelude::{format, string::String};

use serde_json::Value;

use frame_support::PalletId;
const PROTOS_PALLET_ID: PalletId = PalletId(*b"protos__");

/// Types of tags that can be attached to a Proto-Fragment to describe it (e.g Code, Audio, Video etc.)
#[derive(Encode, Decode, Copy, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Tags {
	Code,
	Audio,
	Image,
}

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

/// Struct that represents the digital signature (and other important information) given by the off-chain validator
#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub struct AuthData {
	pub signature: ecdsa::Signature,
	pub block: u32,
}

/// Struct of a Proto-Fragment
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug)]
pub struct Proto<TAccountId, TBlockNumber> {
	/// Block number this proto was included in
	pub block: TBlockNumber,
	/// Plain hash of indexed data.
	pub patches: Vec<Hash256>,
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

#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq, Eq)]
pub struct EthStakeUpdate<TPublic, TBalance> {
	pub public: TPublic,
	pub amount: TBalance,
	pub account: H160,
}

impl<T: SigningTypes, TBalance: Encode> SignedPayload<T> for EthStakeUpdate<T::Public, TBalance> {
	fn public(&self) -> T::Public {
		self.public.clone()
	}
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use hex::FromHex;
	use pallet_detach::{DetachRequest, DetachRequests, DetachedHashes, SupportedChains};
	use sp_runtime::{
		offchain::HttpRequestStatus, traits::AccountIdConversion, MultiSignature,
		SaturatedConversion,
	};

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ CreateSignedTransaction<Call<Self>>
		+ pallet_detach::Config
		+ pallet_randomness_collective_flip::Config
		+ pallet_assets::Config
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type WeightInfo: WeightInfo;

		#[pallet::constant]
		type StorageBytesMultiplier: Get<u64>;

		#[pallet::constant]
		type FragToken: Get<<Self as pallet_assets::Config>::AssetId>;

		#[pallet::constant]
		type StakeLockupPeriod: Get<u64>;

		#[pallet::constant]
		type EthChainId: Get<u64>;
	}

	#[pallet::genesis_config]
	#[derive(Default)]
	pub struct GenesisConfig {
		/// List of upload authorities (i.e off-chain validators). Any of them must validate a Proto-Fragment before it can be uploaded/patched/etc. on the Blockchain
		pub upload_authorities: Vec<ecdsa::Public>,
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

	/// Storage Map that represents the number of transactions the user has performed with the off-chain validator
	/// * The key is the AccountId of the user
	/// * The value is the number of transactions the user has performed with the off-chain validator
	#[pallet::storage]
	pub type UserNonces<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u64>;

	/// Storage Map of Proto-Fragments where the key is the hash of the data of the Proto-Fragment, and the value is the Proto struct of the Proto-Fragment
	#[pallet::storage]
	pub type Protos<T: Config> =
		StorageMap<_, Identity, Hash256, Proto<T::AccountId, T::BlockNumber>>;

	/// Storage Map which keeps track of the Proto-Fragments by Tag type.
	/// The key is the Tag type and the value is a list of the hash of a Proto-Fragment
	// Not ideal but to have it iterable...
	#[pallet::storage]
	pub type ProtosByTag<T: Config> = StorageMap<_, Blake2_128Concat, Tags, Vec<Hash256>>;

	/// UploadAuthorities is a StorageValue that keeps track of the set of ECDSA public keys of the upload authorities
	/// * Note: An upload authority (also known as the off-chain validator) provides the digital signature needed to upload a Proto-Fragment
	#[pallet::storage]
	pub type ProtosByOwner<T: Config> =
		StorageMap<_, Blake2_128Concat, ProtoOwner<T::AccountId>, Vec<Hash256>>;

	#[pallet::storage]
	pub type UploadAuthorities<T: Config> = StorageValue<_, BTreeSet<ecdsa::Public>, ValueQuery>;

	// Staking management
	// (Amount staked, Last stake time)
	#[pallet::storage]
	pub type ProtoStakes<T: Config> = StorageDoubleMap<
		_,
		Identity,
		Hash256,
		Blake2_128Concat,
		T::AccountId,
		(T::Balance, T::BlockNumber),
	>;

	#[pallet::storage]
	pub type EthLockedFrag<T: Config> = StorageMap<_, Blake2_128Concat, H160, T::Balance>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A Proto-Fragment was uploaded
		Uploaded(Hash256),
		/// A Proto-Fragment was patched
		Patched(Hash256),
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
		/// Signature verification failed
		VerificationFailed,
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
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Allows the Sudo Account to add an ECDSA public key to current set of designated upload authorities (i.e the designated off-chain validators)
		///
		/// # Arguements
		/// * `origin` - The origin of the extrisnic/dispatchable function
		/// * `public` - The ECDSA public key to add to the current set of designated upload authorities
		#[pallet::weight(<T as pallet::Config>::WeightInfo::add_upload_auth())]
		pub fn add_upload_auth(origin: OriginFor<T>, public: ecdsa::Public) -> DispatchResult {
			ensure_root(origin)?;

			log::debug!("New upload auth: {:?}", public);

			<UploadAuthorities<T>>::mutate(|validators| {
				validators.insert(public);
			});

			Ok(())
		}

		/// Allows the Sudo Account to remove an ECDSA public key from current set of designated upload authorities (i.e the designated off-chain validators)
		///
		/// # Arguements
		/// * `origin` - The origin of the extrisnic/dispatchable function
		/// * `public` - The ECDSA public key to remove from the current set of designated upload authorities
		#[pallet::weight(<T as pallet::Config>::WeightInfo::del_upload_auth())]
		pub fn del_upload_auth(origin: OriginFor<T>, public: ecdsa::Public) -> DispatchResult {
			ensure_root(origin)?;

			log::debug!("Removed upload auth: {:?}", public);

			<UploadAuthorities<T>>::mutate(|validators| {
				validators.remove(&public);
			});

			Ok(())
		}

		/// Uploads a Proto-Fragment onto the Blockchain. To successfully upload a Proto-Fragment, the `auth` provided must be valid. Otherwise, an error is returned
		/// Furthermore, this function also indexes `data` in the Blockchain's Database and stores it in the IPFS
		///
		/// # Arguments
		///
		/// * `origin` - The origin of the extrisnic/dispatchable function
		/// * `auth` - The digital signature given by the off-chain validator that validates the caller of the extrinsic to upload
		/// * `references` - A list of references to other Proto-Fragments
		/// * `tags` - A list of tags to upload along with the Proto-Fragment
		/// * `linked_asset` - An asset that is linked with the uploaded Proto-Frament (e.g an ERC-721 Contract)
		/// * `include_cost` (optional) -
		/// * `data` - The data of the Proto-Fragment (represented as a vector of bytes)
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

			// Check FRAG staking
			for reference in references.iter() {
				let cost = <Protos<T>>::get(reference).map(|p| p.include_cost);
				if let Some(cost) = cost {
					if let Some(cost) = cost {
						let cost: u128 = cost.into();
						let cost: T::Balance = cost.saturated_into();
						let stake = <ProtoStakes<T>>::get(reference, who.clone());
						if let Some(stake) = stake {
							ensure!(stake.0 >= cost, Error::<T>::NotEnoughStaked);
						} else {
							// Stake not found
							return Err(Error::<T>::StakeNotFound.into());
						}
					}
				// Free to include, just continue
				} else {
					// Proto not found
					return Err(Error::<T>::ReferenceNotFound.into());
				}
			}

			// ! Write STATE from now, ensure no errors from now...

			let owner = if let Some(link) = linked_asset {
				ProtoOwner::ExternalAsset(link)
			} else {
				ProtoOwner::User(who.clone())
			};

			// store in the state the proto
			let proto = Proto {
				block: current_block_number,
				patches: vec![],
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
				<ProtosByTag<T>>::append(tag, proto_hash);
			}

			<ProtosByOwner<T>>::append(owner, proto_hash);

			// index immutable data for IPFS discovery
			transaction_index::index(extrinsic_index, data.len() as u32, proto_hash);

			// advance nonces
			<UserNonces<T>>::insert(who, nonce + 1);

			// also emit event
			Self::deposit_event(Event::Uploaded(proto_hash));

			log::debug!("Uploaded proto: {:?}", proto_hash);

			Ok(())
		}

		/// Patches (i.e modifies) the existing Proto-Fragment (whose hash is `proto_hash`) by appending the hash of `data` to the Vector field `patches` of the existing Proto-Fragment's Struct Instance.
		/// Furthermore, this function also indexes `data` in the Blockchain's Database and stores it in the IPFS
		/// To successfully patch a Proto-Fragment, the `auth` provided must be valid. Otherwise, an error is returned
		///
		/// # Arguments
		///
		/// * `origin` - The origin of the extrisnic / dispatchable function
		/// * `auth` - The digital signature given by the off-chain validator that validates the caller of the extrinsic to upload
		/// * `proto_hash` - The hash of the existing Proto-Fragment
		/// * `tags` - A list of tags to upload along with the Proto-Fragment
		/// * `linked_asset` - An asset that is linked with the uploaded Proto-Frament (e.g an ERC-721 Contract)
		/// * `include_cost` (optional) -
		/// * `data` - The data of the Proto-Fragment (represented as a vector of bytes)
		#[pallet::weight(<T as pallet::Config>::WeightInfo::patch() + data.len() as u64 * <T as pallet::Config>::StorageBytesMultiplier::get())]
		pub fn patch(
			origin: OriginFor<T>,
			auth: AuthData,
			// proto hash we want to patch
			proto_hash: Hash256,
			include_cost: Option<Compact<u128>>,
			// data we want to patch last because of the way we store blocks (storage chain)
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

			let data_hash = blake2_256(&data);
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
				if data.len() > 0 {
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
		/// * `origin` - The origin of the extrisnic / dispatchable function
		/// * `auth` - The digital signature given by the off-chain validator that validates the caller of the extrinsic to upload
		/// * `proto_hash` - The hash of the existing Proto-Fragment
		/// * `metadata_key` - The key (of the key-value pair) that is added in the BTreeMap field `metadata` of the existing Proto-Fragment's Struct Instance
		/// * `data` - The hash of `data` is used as the value (of the key-value pair) that is added in the BTreeMap field `metadata` of the existing Proto-Fragment's Struct Instance
		#[pallet::weight(<T as pallet::Config>::WeightInfo::patch() + (data.len() as u64 * <T as pallet::Config>::StorageBytesMultiplier::get()))]
		pub fn set_metadata(
			origin: OriginFor<T>,
			auth: AuthData,
			// proto hash we want to update
			proto_hash: Hash256,
			// Think of "u8" as string (something to do with WASM - that's why we use u8)
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

			let data_hash = blake2_256(&data);
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
		///
		///
		/// # Arguments
		///
		/// * `origin` - The origin of the extrisnic / dispatchable function
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
			use frame_support::traits::fungibles::Transfer;

			let who = ensure_signed(origin.clone())?;

			// make sure the proto exists
			ensure!(<Protos<T>>::contains_key(&proto_hash), Error::<T>::ProtoNotFound);

			// make sure user has enough FRAG
			let balance = <pallet_assets::Pallet<T>>::balance(T::FragToken::get(), &who.clone());
			ensure!(balance >= amount, Error::<T>::InsufficientBalance);

			let current_block_number = <frame_system::Pallet<T>>::block_number();

			// ! from now we write...

			// transfer to pallet vault
			<pallet_assets::Pallet<T> as Transfer<T::AccountId>>::transfer(
				T::FragToken::get(),
				&who,
				&Self::account_id(),
				amount,
				true,
			)
			.map(|_| ())?;

			// take record of the stake
			<ProtoStakes<T>>::insert(proto_hash, &who, (amount, current_block_number));

			// also emit event
			Self::deposit_event(Event::Staked(proto_hash, who, amount));

			Ok(())
		}

		#[pallet::weight(50_000)]
		pub fn unstake(origin: OriginFor<T>, proto_hash: Hash256) -> DispatchResult {
			use frame_support::traits::fungibles::Transfer;

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

			// transfer to pallet vault
			<pallet_assets::Pallet<T> as Transfer<T::AccountId>>::transfer(
				T::FragToken::get(),
				&Self::account_id(),
				&who,
				stake.0,
				false,
			)
			.map(|_| ())?;

			// take record of the unstake
			<ProtoStakes<T>>::remove(proto_hash, &who);

			// also emit event
			Self::deposit_event(Event::Unstaked(proto_hash, who, stake.0));

			Ok(())
		}

		/// TODO
		#[pallet::weight(25_000)] // TODO #1 - weight
		pub fn internal_update_stake(
			origin: OriginFor<T>,
			_data: EthStakeUpdate<T::Public, T::Balance>,
			_signature: T::Signature,
		) -> DispatchResult {
			ensure_none(origin)?;

			// TODO

			Ok(())
		}

		/// TODO
		#[pallet::weight(25_000)] // TODO #1 - weight
		pub fn query_stake_update(
			origin: OriginFor<T>,
			signature: ecdsa::Signature,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let message = b"TODO";
			let signature_hash = blake2_256(message);

			// Recover ecdsa public here
			let recover = Crypto::secp256k1_ecdsa_recover_compressed(&signature.0, &signature_hash)
				.ok()
				.ok_or(Error::<T>::VerificationFailed)?;
			// this is how substrate handles ecdsa publics
			let recover = blake2_256(&recover);

			let who2 = T::AccountId::decode(&mut &recover[..])
				.map_err(|_| Error::<T>::VerificationFailed)?;

			ensure!(who == who2, Error::<T>::VerificationFailed);

			// TODO

			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(_n: T::BlockNumber) {
			let last_id_ref = StorageValueRef::persistent(b"protos_stake_sync_last_id");
			let last_id: Option<Vec<u8>> = last_id_ref.get().unwrap_or_default();
			let last_id = if let Some(last_id) = last_id {
				String::from_utf8(last_id).unwrap()
			} else {
				String::from("")
			};

			let request = offchain::http_request_start("POST",
				"https://api.thegraph.com/subgraphs/id/QmPggztWjfJtSVkckhuh8N58iY4Vk5XUo1Y6p47GpVubU6",
				&[]
			).unwrap(); // hard fail if fails... it should not...

			offchain::http_request_add_header(request, "Content-Type", "application/json").unwrap();

			let query = format!("{{ \"query\": \"{{lockEntities(where: {{id_gt: \\\"{}\\\"}}) {{id owner amount lock}}}}\"}}", last_id);
			log::trace!("query: {}", query);
			offchain::http_request_write_body(request, query.as_bytes(), None).unwrap();

			// send off the request
			offchain::http_request_write_body(request, &[], None).unwrap();
			let results = offchain::http_response_wait(&[request], None);
			let status = results[0];
			match status {
				HttpRequestStatus::Finished(status) => match status {
					200 => {
						let mut response_body: Vec<u8> = Vec::new();
						loop {
							let mut buffer = Vec::new();
							buffer.resize(1024, 0);
							let len = offchain::http_response_read_body(request, &mut buffer, None)
								.unwrap();
							if len == 0 {
								break;
							}
							response_body.extend_from_slice(&buffer[..len as usize]);
						}
						let response = String::from_utf8(response_body).unwrap();
						log::trace!("response: {}", response);

						let v: Value = serde_json::from_str(&response).unwrap();
						let records = v["data"]["lockEntities"].as_array().unwrap();
						for (i, record) in records.iter().enumerate() {
							let lock = record["lock"].as_bool().unwrap();
							if lock {
								let id = record["id"].as_str().unwrap();
								log::trace!("Recording lock stake for {}", id);

								let account = &record["owner"].as_str().unwrap()[2..];
								let account = <[u8; 20]>::from_hex(account).unwrap();

								let amount = record["amount"].as_str().unwrap();
								let amount = amount.parse::<u128>().unwrap();

								if let Err(e) = Signer::<T, T::AuthorityId>::any_account()
									.send_unsigned_transaction(
										|pub_key| EthStakeUpdate {
											public: pub_key.public.clone(),
											amount: amount.saturated_into(),
											account: account.into(),
										},
										|payload, signature| Call::internal_update_stake {
											data: payload,
											signature,
										},
									)
									.ok_or("No local accounts accounts available.")
								{
									log::error!("Failed to send unsigned eth sync transaction with error: {:?}", e);
								}

								// update the last recorded event
								if i == records.len() - 1 {
									last_id_ref.set(&id.as_bytes().to_vec());
								}
							}
						}
					},
					_ => {
						log::error!("Sync request had unexpected status: {}", status);
					},
				},
				_ => {
					log::error!("Sync request failed with status: {:?}", status);
				},
			}
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn get_by_tag(tags: Tags) -> Option<Vec<Hash256>> {
			<ProtosByTag<T>>::get(&tags)
		}

		/// Ensures that the  SECP256k1 ECDSA public key recovered from the digital signature and the blake2-256 hash of the message is of a designated upload authority
		/// Also ensures that the digital signature was not signed more than a certain number of blocks ago
		/// * `block number` - The latest block number
		/// * `data` - AuthData Struct which contains:
		///     * The digital signature
		///     * The value of the latest block number when the digital signature was signed
		/// * `signature_hash` - The blake2-256 hash of the message
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
						authorities.insert(*authority);
					});
				}
			}
		}

		pub fn account_id() -> T::AccountId {
			PROTOS_PALLET_ID.into_account()
		}
	}
}
