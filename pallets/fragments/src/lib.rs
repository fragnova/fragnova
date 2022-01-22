#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod weights;

use core::slice::Iter;
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

	pub struct FragmentsAuthId;

	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for FragmentsAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::ed25519::Signature;
		type GenericPublic = sp_core::ed25519::Public;
	}

	// implemented for mock runtime in test
	impl frame_system::offchain::AppCrypto<<Ed25519Signature as Verify>::Signer, Ed25519Signature>
		for FragmentsAuthId
	{
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::ed25519::Signature;
		type GenericPublic = sp_core::ed25519::Public;
	}
}

pub use pallet::*;
pub use weights::WeightInfo;

use codec::{Compact, Decode, Encode};
use sp_io::{
	crypto as Crypto,
	hashing::{blake2_256, keccak_256},
	offchain_index, transaction_index,
};
use sp_runtime::{offchain::storage::StorageValueRef, MultiSigner};
use sp_std::{
	collections::{btree_map::BTreeMap, btree_set::BTreeSet},
	vec,
	vec::Vec,
};

use sp_chainblocks::Hash256;

use frame_system::offchain::{
	AppCrypto, CreateSignedTransaction, SendUnsignedTransaction, SignedPayload, Signer,
	SigningTypes,
};

/// data required to submit a transaction.
#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub struct FragmentValidation<Public, BlockNumber> {
	block_number: BlockNumber,
	public: Public,
	fragment_hash: Hash256,
	result: bool,
}

impl<T: SigningTypes> SignedPayload<T> for FragmentValidation<T::Public, T::BlockNumber> {
	fn public(&self) -> T::Public {
		self.public.clone()
	}
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub enum SupportedChains {
	EthereumMainnet,
	EthereumRinkeby,
	EthereumGoerli,
}

#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub struct DetachRequest {
	pub fragment_hash: Hash256,
	pub target_chain: SupportedChains,
	pub target_account: Vec<u8>, // an eth address or so
}

#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq)]
pub struct DetachInternalData<TPublic> {
	public: TPublic,
	fragment_hash: Hash256,
	target_chain: SupportedChains,
	target_account: Vec<u8>, // an eth address or so
	remote_signature: Vec<u8>,
	nonce: u64,
}

impl<T: SigningTypes> SignedPayload<T> for DetachInternalData<T::Public> {
	fn public(&self) -> T::Public {
		self.public.clone()
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub enum LinkSource {
	// Generally we just store this data, we don't verify it as we assume auth service did it.
	// (Link signature, Linked block number, EIP155 Chain ID)
	Evm(ecdsa::Signature, u64, U256),
}

#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub enum LinkedAsset {
	// Ethereum (ERC721 Contract address, Token ID, Link source)
	Erc721(H160, U256, LinkSource),
}

#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub enum FragmentOwner<TAccountId> {
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
pub struct Fragment<TAccountId, TBlockNumber> {
	/// Block number this fragment was included in
	pub block: TBlockNumber,
	/// Plain hash of indexed data.
	pub updates: Vec<Hash256>,
	/// Base include cost, of referenced fragments.
	pub base_cost: Compact<u128>,
	/// Include price of the fragment.
	/// If None, this fragment can't be included into other fragments
	pub include_cost: Option<Compact<u128>>,
	/// The original creator of the fragment.
	pub creator: TAccountId,
	/// The current owner of the fragment.
	pub owner: FragmentOwner<TAccountId>,
	/// References to other fragments.
	pub references: BTreeSet<Hash256>,
	/// Metadata attached to the fragment.
	pub metadata: BTreeMap<Vec<u8>, Hash256>,
}

#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq)]
pub struct ExportData {
	chain: SupportedChains,
	owner: Vec<u8>,
	nonce: u64,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		CreateSignedTransaction<Call<Self>>
		+ pallet_randomness_collective_flip::Config
		+ frame_system::Config
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type WeightInfo: WeightInfo;
		/// The identifier type for an offchain worker.
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig {
		pub upload_authorities: Vec<ecdsa::Public>,
		pub eth_authorities: Vec<ecdsa::Public>,
		pub keys: Vec<ed25519::Public>,
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			Self { upload_authorities: Vec::new(), eth_authorities: Vec::new(), keys: Vec::new() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			Pallet::<T>::initialize_upload_authorities(&self.upload_authorities);
			Pallet::<T>::initialize_eth_authorities(&self.eth_authorities);
			Pallet::<T>::initialize_keys(&self.keys);
		}
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub type UserNonces<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u64>;

	#[pallet::storage]
	pub type Fragments<T: Config> =
		StorageMap<_, Blake2_128Concat, Hash256, Fragment<T::AccountId, T::BlockNumber>>;

	#[pallet::storage]
	pub type DetachRequests<T: Config> = StorageValue<_, Vec<DetachRequest>, ValueQuery>;

	#[pallet::storage]
	pub type DetachedFragments<T: Config> = StorageMap<_, Blake2_128Concat, Hash256, ExportData>;

	#[pallet::storage]
	pub type DetachNonces<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, Vec<u8>, Blake2_128Concat, SupportedChains, u64>;

	#[pallet::storage]
	pub type EthereumAuthorities<T: Config> = StorageValue<_, BTreeSet<ecdsa::Public>, ValueQuery>;

	#[pallet::storage]
	pub type UploadAuthorities<T: Config> = StorageValue<_, BTreeSet<ecdsa::Public>, ValueQuery>;

	#[pallet::storage]
	pub type FragKeys<T: Config> = StorageValue<_, BTreeSet<ed25519::Public>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Upload(Hash256),
		Update(Hash256),
		MetadataAdded(Hash256, Vec<u8>),
		Detached(Hash256, Vec<u8>),
		Transfer(Hash256, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Systematic failure - those errors should not happen.
		SystematicFailure,
		/// Fragment not found
		FragmentNotFound,
		/// Fragment already uploaded
		FragmentExists,
		/// Require sudo user
		SudoUserRequired,
		/// Unsupported chain to lock asset into
		UnsupportedChain,
		/// Fragment is already detached
		FragmentDetached,
		/// Not the owner of the fragment
		Unauthorized,
		/// No Validators are present
		NoValidator,
		/// Failed to sign message
		SigningFailed,
		/// Signature verification failed
		VerificationFailed,
		/// The provided nonce override is too big
		NonceMismatch,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// Add validator public key to the list
		#[pallet::weight(T::WeightInfo::add_eth_auth())]
		pub fn add_eth_auth(origin: OriginFor<T>, public: ecdsa::Public) -> DispatchResult {
			ensure_root(origin)?;

			log::debug!("New eth auth: {:?}", public);

			<EthereumAuthorities<T>>::mutate(|validators| {
				validators.insert(public);
			});

			Ok(())
		}

		// Remove validator public key to the list
		#[pallet::weight(T::WeightInfo::del_eth_auth())]
		pub fn del_eth_auth(origin: OriginFor<T>, public: ecdsa::Public) -> DispatchResult {
			ensure_root(origin)?;

			log::debug!("Removed eth auth: {:?}", public);

			<EthereumAuthorities<T>>::mutate(|validators| {
				validators.remove(&public);
			});

			Ok(())
		}

		#[pallet::weight(T::WeightInfo::add_upload_auth())]
		pub fn add_upload_auth(origin: OriginFor<T>, public: ecdsa::Public) -> DispatchResult {
			ensure_root(origin)?;

			log::debug!("New upload auth: {:?}", public);

			<UploadAuthorities<T>>::mutate(|validators| {
				validators.insert(public);
			});

			Ok(())
		}

		#[pallet::weight(T::WeightInfo::del_upload_auth())]
		pub fn del_upload_auth(origin: OriginFor<T>, public: ecdsa::Public) -> DispatchResult {
			ensure_root(origin)?;

			log::debug!("Removed upload auth: {:?}", public);

			<UploadAuthorities<T>>::mutate(|validators| {
				validators.remove(&public);
			});

			Ok(())
		}

		#[pallet::weight(T::WeightInfo::add_upload_auth())]
		pub fn add_key(origin: OriginFor<T>, public: ed25519::Public) -> DispatchResult {
			ensure_root(origin)?;

			log::debug!("New key: {:?}", public);

			<FragKeys<T>>::mutate(|validators| {
				validators.insert(public);
			});

			Ok(())
		}

		#[pallet::weight(T::WeightInfo::del_upload_auth())]
		pub fn del_key(origin: OriginFor<T>, public: ed25519::Public) -> DispatchResult {
			ensure_root(origin)?;

			log::debug!("Removed key: {:?}", public);

			<FragKeys<T>>::mutate(|validators| {
				validators.remove(&public);
			});

			Ok(())
		}

		/// Fragment upload function.
		#[pallet::weight(T::WeightInfo::upload(data.len() as u32))]
		pub fn upload(
			origin: OriginFor<T>,
			auth: AuthData,
			// we store this in the state as well
			references: BTreeSet<Hash256>,
			linked_asset: Option<LinkedAsset>,
			include_cost: Option<Compact<u128>>,
			// let data come last as we record this size in blocks db (storage chain)
			// and the offset is calculated like
			// https://github.com/paritytech/substrate/blob/a57bc4445a4e0bfd5c79c111add9d0db1a265507/client/db/src/lib.rs#L1678
			data: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let nonce = <UserNonces<T>>::get(who.clone()).unwrap_or(0);

			// hash the immutable data, this is also the unique fragment id
			// to compose the V1 Cid add this prefix to the hash: (str "z" (base58
			// "0x0155a0e40220"))
			let fragment_hash = blake2_256(&data);
			let signature_hash = blake2_256(
				&[
					&fragment_hash[..],
					&references.encode(),
					&linked_asset.encode(),
					&nonce.encode(),
					&auth.block.encode(),
				]
				.concat(),
			);

			let current_block_number = <frame_system::Pallet<T>>::block_number();

			<Pallet<T>>::ensure_auth(current_block_number, &auth, &signature_hash)?;

			// make sure the fragment does not exist already!
			ensure!(!<Fragments<T>>::contains_key(&fragment_hash), Error::<T>::FragmentExists);

			// we need this to index transactions
			let extrinsic_index = <frame_system::Pallet<T>>::extrinsic_index()
				.ok_or(Error::<T>::SystematicFailure)?;

			// Calculate the base cost of inclusion
			let cost = references.iter().fold(0, |acc, ref_hash| {
				let ref_cost = if let Some(fragment) = <Fragments<T>>::get(ref_hash) {
					if let Some(cost) = fragment.include_cost {
						cost.into()
					} else {
						0
					}
				} else {
					0
				};
				acc + ref_cost
			});

			// Write STATE from now, ensure no errors from now...

			// store in the state the fragment
			let fragment = Fragment {
				block: current_block_number,
				updates: vec![],
				base_cost: cost.into(),
				include_cost,
				creator: who.clone(),
				owner: if let Some(link) = linked_asset {
					FragmentOwner::ExternalAsset(link)
				} else {
					FragmentOwner::User(who.clone())
				},
				references,
				metadata: BTreeMap::new(),
			};

			// store fragment metadata
			<Fragments<T>>::insert(fragment_hash, fragment);

			// index immutable data for IPFS discovery
			transaction_index::index(extrinsic_index, data.len() as u32, fragment_hash);

			// advance nonces
			<UserNonces<T>>::insert(who, nonce + 1);

			// also emit event
			Self::deposit_event(Event::Upload(fragment_hash));

			log::debug!("Uploaded fragment: {:?}", fragment_hash);

			Ok(())
		}

		/// Fragment upload function.
		#[pallet::weight(T::WeightInfo::update(if let Some(data) = data { data.len() as u32} else { 50_000 }))]
		pub fn update(
			origin: OriginFor<T>,
			auth: AuthData,
			// fragment hash we want to update
			fragment_hash: Hash256,
			include_cost: Option<Compact<u128>>,
			// data we want to update last because of the way we store blocks (storage chain)
			data: Option<Vec<u8>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let nonce = <UserNonces<T>>::get(who.clone()).unwrap_or(0);

			let fragment: Fragment<T::AccountId, T::BlockNumber> =
				<Fragments<T>>::get(&fragment_hash).ok_or(Error::<T>::FragmentNotFound)?;

			match fragment.owner {
				FragmentOwner::User(owner) => ensure!(owner == who, Error::<T>::Unauthorized),
				FragmentOwner::ExternalAsset(_ext_asset) =>
				// We don't allow updating external assets
					ensure!(false, Error::<T>::Unauthorized),
			};

			ensure!(
				!<DetachedFragments<T>>::contains_key(&fragment_hash),
				Error::<T>::FragmentDetached
			);

			let data_hash = blake2_256(&data.encode());
			let signature_hash = blake2_256(
				&[&fragment_hash[..], &data_hash[..], &nonce.encode(), &auth.block.encode()]
					.concat(),
			);

			let current_block_number = <frame_system::Pallet<T>>::block_number();

			<Pallet<T>>::ensure_auth(current_block_number, &auth, &signature_hash)?;

			// we need this to index transactions
			let extrinsic_index = <frame_system::Pallet<T>>::extrinsic_index()
				.ok_or(Error::<T>::SystematicFailure)?;

			// Write STATE from now, ensure no errors from now...

			<Fragments<T>>::mutate(&fragment_hash, |fragment| {
				let fragment = fragment.as_mut().unwrap();
				if let Some(data) = data {
					// No failures from here on out
					fragment.updates.push(data_hash);
					// index mutable data for IPFS discovery as well
					transaction_index::index(extrinsic_index, data.len() as u32, data_hash);
				}
				fragment.include_cost = include_cost;
			});

			// advance nonces
			<UserNonces<T>>::insert(who, nonce + 1);

			// also emit event
			Self::deposit_event(Event::Update(fragment_hash));

			log::debug!("Updated fragment: {:?}", fragment_hash);

			Ok(())
		}

		/// Detached a fragment from this chain by emitting an event that includes a signature.
		/// The remote target chain can attach this fragment by using this signature.
		#[pallet::weight(25_000)] // TODO #1 - weight
		pub fn detach(
			origin: OriginFor<T>,
			fragment_hash: Hash256,
			target_chain: SupportedChains,
			target_account: Vec<u8>, // an eth address or so
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// make sure the fragment exists
			let fragment: Fragment<T::AccountId, T::BlockNumber> =
				<Fragments<T>>::get(&fragment_hash).ok_or(Error::<T>::FragmentNotFound)?;

			match fragment.owner {
				FragmentOwner::User(owner) => ensure!(owner == who, Error::<T>::Unauthorized),
				FragmentOwner::ExternalAsset(_ext_asset) =>
				// We don't allow detaching external assets
					ensure!(false, Error::<T>::Unauthorized),
			};

			ensure!(
				!<DetachedFragments<T>>::contains_key(&fragment_hash),
				Error::<T>::FragmentDetached
			);

			<DetachRequests<T>>::mutate(|requests| {
				requests.push(DetachRequest { fragment_hash, target_chain, target_account });
			});

			Ok(())
		}

		/// Detached a fragment from this chain by emitting an event that includes a signature.
		/// The remote target chain can attach this fragment by using this signature.
		#[pallet::weight(25_000)] // TODO #1 - weight
		pub fn internal_finalize_detach(
			origin: OriginFor<T>,
			data: DetachInternalData<T::Public>,
			_signature: T::Signature,
		) -> DispatchResult {
			ensure_none(origin)?;

			// Update nonce
			<DetachNonces<T>>::insert(&data.target_account, data.target_chain, data.nonce);

			let export_data = ExportData {
				chain: data.target_chain,
				owner: data.target_account,
				nonce: data.nonce,
			};

			// add to Detached fragments map
			<DetachedFragments<T>>::insert(data.fragment_hash, export_data);

			// emit event
			Self::deposit_event(Event::Detached(data.fragment_hash, data.remote_signature.clone()));

			log::debug!(
				"Detached fragment with hash: {:?} signature: {:?}",
				data.fragment_hash,
				data.remote_signature
			);

			Ok(())
		}

		/// Transfer fragment ownership
		#[pallet::weight(T::WeightInfo::transfer())]
		pub fn transfer(
			origin: OriginFor<T>,
			fragment_hash: Hash256,
			new_owner: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// make sure the fragment exists
			let fragment: Fragment<T::AccountId, T::BlockNumber> =
				<Fragments<T>>::get(&fragment_hash).ok_or(Error::<T>::FragmentNotFound)?;

			match fragment.owner {
				FragmentOwner::User(owner) => ensure!(owner == who, Error::<T>::Unauthorized),
				FragmentOwner::ExternalAsset(_ext_asset) =>
				// We don't allow updating external assets
					ensure!(false, Error::<T>::Unauthorized),
			};

			ensure!(
				!<DetachedFragments<T>>::contains_key(&fragment_hash),
				Error::<T>::FragmentDetached
			);

			// update fragment
			<Fragments<T>>::mutate(&fragment_hash, |fragment| {
				let fragment = fragment.as_mut().unwrap();
				fragment.owner = FragmentOwner::User(new_owner.clone());
			});

			// emit event
			Self::deposit_event(Event::Transfer(fragment_hash, new_owner));

			Ok(())
		}

		/// Fragment upload function.
		#[pallet::weight(T::WeightInfo::upload(data.len() as u32))]
		pub fn set_metadata(
			origin: OriginFor<T>,
			auth: AuthData,
			// fragment hash we want to update
			fragment_hash: Hash256,
			metadata_key: Vec<u8>,
			// data we want to update last because of the way we store blocks (storage chain)
			data: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let nonce = <UserNonces<T>>::get(who.clone()).unwrap_or(0);

			let fragment: Fragment<T::AccountId, T::BlockNumber> =
				<Fragments<T>>::get(&fragment_hash).ok_or(Error::<T>::FragmentNotFound)?;

			match fragment.owner {
				FragmentOwner::User(owner) => ensure!(owner == who, Error::<T>::Unauthorized),
				FragmentOwner::ExternalAsset(_ext_asset) =>
				// We don't allow updating external assets
					ensure!(false, Error::<T>::Unauthorized),
			};

			ensure!(
				!<DetachedFragments<T>>::contains_key(&fragment_hash),
				Error::<T>::FragmentDetached
			);

			let data_hash = blake2_256(&data.encode());
			let signature_hash = blake2_256(
				&[&fragment_hash[..], &data_hash[..], &nonce.encode(), &auth.block.encode()]
					.concat(),
			);

			let current_block_number = <frame_system::Pallet<T>>::block_number();

			<Pallet<T>>::ensure_auth(current_block_number, &auth, &signature_hash)?;

			// we need this to index transactions
			let extrinsic_index = <frame_system::Pallet<T>>::extrinsic_index()
				.ok_or(Error::<T>::SystematicFailure)?;

			// Write STATE from now, ensure no errors from now...

			<Fragments<T>>::mutate(&fragment_hash, |fragment| {
				let fragment = fragment.as_mut().unwrap();
				// update metadata
				fragment.metadata.insert(metadata_key.clone(), data_hash);
			});

			// index data
			transaction_index::index(extrinsic_index, data.len() as u32, data_hash);

			// advance nonces
			<UserNonces<T>>::insert(who, nonce + 1);

			// also emit event
			Self::deposit_event(Event::MetadataAdded(fragment_hash, metadata_key.clone()));

			log::debug!(
				"Added metadata to fragment: {:x?} with key: {:x?}",
				fragment_hash,
				metadata_key
			);

			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(_n: T::BlockNumber) {
			// drain and process requests
			let requests = <DetachRequests<T>>::take();
			if !requests.is_empty() {
				log::debug!("Got {} detach requests", requests.len());
				offchain_index::set(b"fragments-detach-requests", &requests.encode());
			}
		}

		fn offchain_worker(_n: T::BlockNumber) {
			<Pallet<T>>::process_detach_requests();
		}
	}

	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;

		/// Validate unsigned call to this module.
		///
		/// By default unsigned transactions are disallowed, but implementing the validator
		/// here we make sure that some particular calls (the ones produced by offchain worker)
		/// are being whitelisted and marked as valid.
		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			// Firstly let's check that we call the right function.
			if let Call::internal_finalize_detach { ref data, ref signature } = call {
				// check public is valid
				let valid_keys = <FragKeys<T>>::get();
				log::debug!("Valid keys: {:?}", valid_keys);
				// I'm sure there is a way to do this without serialization but I can't spend so
				// much time fighting with rust
				let pub_key = data.public.encode();
				let pub_key: ed25519::Public = {
					if let Ok(MultiSigner::Ed25519(pub_key)) =
						<MultiSigner>::decode(&mut &pub_key[..])
					{
						pub_key
					} else {
						return InvalidTransaction::BadSigner.into()
					}
				};
				log::debug!("Public key: {:?}", pub_key);
				if !valid_keys.contains(&pub_key) {
					return InvalidTransaction::BadSigner.into()
				}
				// most expensive bit last
				let signature_valid =
					SignedPayload::<T>::verify::<T::AuthorityId>(data, signature.clone());
				if !signature_valid {
					return InvalidTransaction::BadProof.into()
				}
				log::debug!("Sending detach finalization extrinsic");
				ValidTransaction::with_tag_prefix("Fragments-Detach")
					.and_provides(data.fragment_hash)
					.and_provides(data.target_chain)
					.and_provides(data.target_account.clone())
					.and_provides(data.nonce)
					.longevity(5)
					.propagate(true)
					.build()
			} else {
				InvalidTransaction::Call.into()
			}
		}
	}

	impl<T: Config> Pallet<T> {
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

		fn initialize_eth_authorities(authorities: &[ecdsa::Public]) {
			if !authorities.is_empty() {
				assert!(
					<EthereumAuthorities<T>>::get().is_empty(),
					"EthereumAuthorities are already initialized!"
				);
				for authority in authorities {
					<EthereumAuthorities<T>>::mutate(|authorities| {
						authorities.insert(authority.clone());
					});
				}
			}
		}

		fn initialize_keys(keys: &[ed25519::Public]) {
			if !keys.is_empty() {
				assert!(<FragKeys<T>>::get().is_empty(), "FragKeys are already initialized!");
				for key in keys {
					<FragKeys<T>>::mutate(|keys| {
						keys.insert(*key);
					});
				}
			}
		}

		fn process_detach_requests() {
			const FAILED: () = ();
			let requests = StorageValueRef::persistent(b"fragments-detach-requests");
			let _ = requests.mutate(|requests: Result<Option<Vec<DetachRequest>>, _>| match requests {
					Ok(Some(requests)) => {
						log::debug!("Got {} detach requests", requests.len());
						for request in requests {
							let chain_id = match request.target_chain {
								SupportedChains::EthereumMainnet => U256::from(1),
								SupportedChains::EthereumRinkeby => U256::from(4),
								SupportedChains::EthereumGoerli => U256::from(5),
							};

							let values = match request.target_chain {
								SupportedChains::EthereumMainnet |
								SupportedChains::EthereumRinkeby |
								SupportedChains::EthereumGoerli => {
									// check if we need to generate new ecdsa keys
									let ed_keys = Crypto::ed25519_public_keys(KEY_TYPE);
									let keys_ref = StorageValueRef::persistent(b"fragments-frag-ecdsa-keys");
									let keys = keys_ref.get::<BTreeSet<ed25519::Public>>().unwrap_or_default();
									let mut keys = if let Some(keys) = keys {
										keys
									} else {
										BTreeSet::new()
									};
									// doing this cos mutate was insane...
									let mut edited = false;
									for ed_key in &ed_keys {
										if !keys.contains(ed_key) {
											let signed = Crypto::ed25519_sign(KEY_TYPE, ed_key, b"fragments-frag-ecdsa-keys").unwrap();
											let key = keccak_256(&signed.0[..]);
											let mut key_hex = [0u8; 64];
											hex::encode_to_slice(key, &mut key_hex).map_err(|_| FAILED)?;
											let key_hex = [b"0x", &key_hex[..]].concat();
											log::debug!("Adding new key from seed: {:?}", key_hex);
											let _public = Crypto::ecdsa_generate(KEY_TYPE, Some(key_hex));
											keys.insert(*ed_key);
											edited = true;
										}
									}
									if edited {
										// commit it back
										keys_ref.set(&keys);
									}
									// get local keys
									let keys = Crypto::ecdsa_public_keys(KEY_TYPE);
									log::debug!("ecdsa local keys {:x?}", keys);
									// make sure the local key is in the global authorities set!
									let key = keys
										.iter()
										.find(|k| <EthereumAuthorities<T>>::get().contains(k));
									if let Some(key) = key {
										// This is critical, we send over to the ethereum smart
										// contract this signature The ethereum smart contract call
										// will be the following attach(fragment_hash, local_owner,
										// signature, clamor_nonce); on this target chain the nonce
										// needs to be exactly the same as the one here
										let mut payload = request.fragment_hash.encode();
										let mut chain_id_be: [u8; 32] = [0u8; 32];
										chain_id.to_big_endian(&mut chain_id_be);
										payload.extend(&chain_id_be[..]);
										let mut target_account: [u8; 20] = [0u8; 20];
										if request.target_account.len() != 20 {
											return Err(FAILED);
										}
										target_account.copy_from_slice(&request.target_account[..]);
										payload.extend(&target_account[..]);
										let nonce = <DetachNonces<T>>::get(
											&request.target_account,
											request.target_chain,
										);
										let nonce = if let Some(nonce) = nonce {
											// add 1, remote will add 1
											let nonce = nonce.checked_add(1).unwrap();
											payload.extend(nonce.to_be_bytes());
											nonce // for storage
										} else {
											// there never was a nonce
											payload.extend(1u64.to_be_bytes());
											1u64
										};
										log::debug!("payload: {:x?}, len: {}", payload, payload.len());
										let payload = keccak_256(&payload);
										log::debug!("payload hash: {:x?}, len: {}", payload, payload.len());
										let msg = [
											b"\x19Ethereum Signed Message:\n32",
											&payload[..],
										]
										.concat();
										let msg = keccak_256(&msg);
										// Sign the payload with a trusted validation key
										let signature = Crypto::ecdsa_sign_prehashed(KEY_TYPE, key, &msg);
										if let Some(signature) = signature {
											// No more failures from this path!!
											let mut signature = signature.0.to_vec();
											// fix signature ending for ethereum
											signature[64] += 27u8;
											Ok((signature, nonce))
										} else {
											Err(Error::<T>::SigningFailed)
										}
									} else {
										Err(Error::<T>::NoValidator)
									}
								},
							};


							match values {
								Ok((signature, nonce)) => {
									// exec unsigned transaction from here
									log::debug!("Executing unsigned transaction for detach; signature: {:x?}, nonce: {}", signature, nonce);
									if let Err(e) = Signer::<T, T::AuthorityId>::any_account()
									.send_unsigned_transaction(
										|account| DetachInternalData {
											public: account.public.clone(),
											fragment_hash: request.fragment_hash,
											target_chain: request.target_chain,
											target_account: request.target_account.clone(),
											remote_signature: signature.clone(),
											nonce,
										},
										|payload, signature| Call::internal_finalize_detach {
											data: payload,
											signature,
										},
									)
									.ok_or("No local accounts accounts available.") {
										log::error!("Failed to send unsigned detach transaction with error: {:?}", e);
									}
								},
								Err(e) => {log::debug!("Failed to detach with error {:?}", e)},
							}
						}
						Ok(vec![])
					},
					_ => Err(FAILED),
				});
		}
	}
}

impl SupportedChains {
	pub fn iterator() -> Iter<'static, SupportedChains> {
		static CHAINS: [SupportedChains; 3] = [
			SupportedChains::EthereumMainnet,
			SupportedChains::EthereumRinkeby,
			SupportedChains::EthereumGoerli,
		];
		CHAINS.iter()
	}
}
