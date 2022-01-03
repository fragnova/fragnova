#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod weights;

use core::slice::Iter;
use sp_core::{crypto::KeyTypeId, ecdsa};

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
	use sp_core::sr25519::Signature as Sr25519Signature;
	use sp_runtime::{
		app_crypto::{app_crypto, sr25519},
		traits::Verify,
		MultiSignature, MultiSigner,
	};
	app_crypto!(sr25519, KEY_TYPE);

	pub struct FragmentsAuthId;

	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for FragmentsAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}

	// implemented for mock runtime in test
	impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
		for FragmentsAuthId
	{
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}

pub use pallet::*;
pub use weights::WeightInfo;

use codec::{Compact, Decode, Encode};
use sp_io::{
	crypto as Crypto,
	hashing::{blake2_256, keccak_256},
	transaction_index,
};
use sp_std::{collections::btree_set::BTreeSet, vec, vec::Vec};

use sp_chainblocks::Hash256;

use frame_system::offchain::{AppCrypto, CreateSignedTransaction, SignedPayload, SigningTypes};

/// Payload used by this example crate to hold price
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

#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub enum SupportedChains {
	EthereumMainnet,
	EthereumRinkeby,
	EthereumGoerli,
}

#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub struct IncludeInfo {
	pub fragment_hash: Hash256,
	pub mutable_index: Option<Compact<u32>>,
	pub staked_amount: Compact<u128>,
}

#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug)]
pub struct Fragment<TAccountId> {
	/// Plain hash of indexed data.
	pub mutable_hash: Vec<Hash256>,
	/// Include price of the fragment.
	/// If None, this fragment can't be included into other fragments
	pub include_cost: Option<Compact<u128>>,
	/// The original creator of the fragment.
	pub creator: TAccountId,
	/// The current owner of the fragment.
	pub owner: TAccountId,
	/// References to other fragments.
	/// Hash, mutable index, include cost.
	pub references: Vec<IncludeInfo>,
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

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub type UserNonces<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u64>;

	#[pallet::storage]
	pub type Fragments<T: Config> =
		StorageMap<_, Blake2_128Concat, Hash256, Fragment<T::AccountId>>;

	#[pallet::storage]
	pub type FragmentsList<T: Config> = StorageMap<_, Blake2_128Concat, u128, Hash256>;

	#[pallet::storage]
	pub type FragmentsListSize<T: Config> = StorageValue<_, u128, ValueQuery>;

	#[pallet::storage]
	pub type DetachedFragments<T: Config> = StorageMap<_, Blake2_128Concat, Hash256, ExportData>;

	#[pallet::storage]
	pub type DetachNonces<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, Hash256, Blake2_128Concat, SupportedChains, u64>;

	#[pallet::storage]
	pub type EthereumAuthorities<T: Config> = StorageValue<_, BTreeSet<ecdsa::Public>, ValueQuery>;

	#[pallet::storage]
	pub type UploadAuthorities<T: Config> = StorageValue<_, BTreeSet<ecdsa::Public>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Upload(Hash256),
		Update(Hash256),
		Verified(Hash256, bool),
		Exported(Hash256, Vec<u8>, Vec<u8>),
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
		/// Fragment is not verified yet or failed verification!
		FragmentNotVerified,
		/// Not the owner of the fragment
		Unauthorized,
		/// No Validators are present
		NoValidator,
		/// Failed to sign message
		SigningFailed,
		/// Signature verification failed
		SignatureVerificationFailed,
		/// The provided nonce override is too big
		NonceMismatch,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// Add validator public key to the list
		#[pallet::weight(25_000)] // TODO #1 - weight
		pub fn add_eth_auth(origin: OriginFor<T>, public: ecdsa::Public) -> DispatchResult {
			ensure_root(origin)?;

			log::debug!("New eth auth: {:?}", public);

			<EthereumAuthorities<T>>::mutate(|validators| {
				validators.insert(public);
			});

			Ok(())
		}

		// Remove validator public key to the list
		#[pallet::weight(25_000)] // TODO #1 - weight
		pub fn del_eth_auth(origin: OriginFor<T>, public: ecdsa::Public) -> DispatchResult {
			ensure_root(origin)?;

			log::debug!("New eth auth: {:?}", public);

			<EthereumAuthorities<T>>::mutate(|validators| {
				validators.remove(&public);
			});

			Ok(())
		}

		#[pallet::weight(25_000)] // TODO #1 - weight
		pub fn add_upload_auth(origin: OriginFor<T>, public: ecdsa::Public) -> DispatchResult {
			ensure_root(origin)?;

			log::debug!("New upload auth: {:?}", public);

			<UploadAuthorities<T>>::mutate(|validators| {
				validators.insert(public);
			});

			Ok(())
		}

		#[pallet::weight(25_000)] // TODO #1 - weight
		pub fn del_upload_auth(origin: OriginFor<T>, public: ecdsa::Public) -> DispatchResult {
			ensure_root(origin)?;

			log::debug!("New upload auth: {:?}", public);

			<UploadAuthorities<T>>::mutate(|validators| {
				validators.remove(&public);
			});

			Ok(())
		}

		/// Fragment upload function.
		// TODO #1 - weight
		#[pallet::weight(T::WeightInfo::store(data.len() as u32))]
		pub fn upload(
			origin: OriginFor<T>,
			// we store this in the state as well
			references: Vec<IncludeInfo>,
			include_cost: Option<Compact<u128>>,
			signature: ecdsa::Signature,
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
			let signature_hash =
				blake2_256(&[&fragment_hash[..], &references.encode(), &nonce.encode()].concat());

			<Pallet<T>>::ensure_upload_auth(&signature, &signature_hash)?;

			// make sure the fragment does not exist already!
			ensure!(!<Fragments<T>>::contains_key(&fragment_hash), Error::<T>::FragmentExists);

			// we need this to index transactions
			let extrinsic_index = <frame_system::Pallet<T>>::extrinsic_index()
				.ok_or(Error::<T>::SystematicFailure)?;

			// Write STATE from now, ensure no errors from now...

			// store in the state the fragment
			let fragment = Fragment {
				mutable_hash: vec![],
				include_cost,
				creator: who.clone(),
				owner: who.clone(),
				references,
			};

			// store fragment metadata
			<Fragments<T>>::insert(fragment_hash, fragment);

			// add to enumerable fragments list
			let next = <FragmentsListSize<T>>::get();
			<FragmentsList<T>>::insert(next, fragment_hash);
			<FragmentsListSize<T>>::mutate(|index| {
				*index += 1;
			});

			// index immutable data for IPFS discovery
			transaction_index::index(extrinsic_index, data.len() as u32, fragment_hash);

			// advance nonces
			<UserNonces<T>>::insert(who, nonce + 1);

			// also emit event
			Self::deposit_event(Event::Upload(fragment_hash));

			Ok(())
		}

		/// Fragment upload function.
		// TODO #1 - weight
		#[pallet::weight(T::WeightInfo::store(if let Some(data) = data { data.len() as u32} else { 50_000 }))]
		pub fn update(
			origin: OriginFor<T>,
			// fragment hash we want to update
			fragment_hash: Hash256,
			include_cost: Option<Compact<u128>>,
			signature: ecdsa::Signature,
			// data we want to update last because of the way we store blocks (storage chain)
			data: Option<Vec<u8>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let nonce = <UserNonces<T>>::get(who.clone()).unwrap_or(0);

			let fragment: Fragment<T::AccountId> =
				<Fragments<T>>::get(&fragment_hash).ok_or(Error::<T>::FragmentNotFound)?;

			ensure!(fragment.owner == who, Error::<T>::Unauthorized);
			ensure!(
				!<DetachedFragments<T>>::contains_key(&fragment_hash),
				Error::<T>::FragmentDetached
			);

			let data_hash = blake2_256(&data.encode());
			let signature_hash =
				blake2_256(&[&fragment_hash[..], &data_hash[..], &nonce.encode()].concat());

			<Pallet<T>>::ensure_upload_auth(&signature, &signature_hash)?;

			// we need this to index transactions
			let extrinsic_index = <frame_system::Pallet<T>>::extrinsic_index()
				.ok_or(Error::<T>::SystematicFailure)?;

			// Write STATE from now, ensure no errors from now...

			<Fragments<T>>::mutate(&fragment_hash, |fragment| {
				let fragment = fragment.as_mut().unwrap();
				if let Some(data) = data {
					// No failures from here on out
					fragment.mutable_hash.push(data_hash);
					// index mutable data for IPFS discovery as well
					transaction_index::index(extrinsic_index, data.len() as u32, data_hash);
				}
				fragment.include_cost = include_cost;
			});

			// advance nonces
			<UserNonces<T>>::insert(who, nonce + 1);

			// also emit event
			Self::deposit_event(Event::Update(fragment_hash));

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
			let fragment: Fragment<T::AccountId> =
				<Fragments<T>>::get(&fragment_hash).ok_or(Error::<T>::FragmentNotFound)?;

			ensure!(fragment.owner == who, Error::<T>::Unauthorized);

			ensure!(
				!<DetachedFragments<T>>::contains_key(&fragment_hash),
				Error::<T>::FragmentDetached
			);

			let chain_id = match target_chain {
				SupportedChains::EthereumMainnet => Some(1u32),
				SupportedChains::EthereumRinkeby => Some(4u32),
				SupportedChains::EthereumGoerli => Some(5u32),
			};

			let (signature, pub_key, nonce) = match target_chain {
				SupportedChains::EthereumMainnet |
				SupportedChains::EthereumRinkeby |
				SupportedChains::EthereumGoerli => {
					// get local keys
					let keys = Crypto::ecdsa_public_keys(KEY_TYPE);
					// make sure the local key is in the global authorities set!
					let key = keys.iter().find(|k| <EthereumAuthorities<T>>::get().contains(k));
					if let Some(key) = key {
						// This is critical, we send over to the ethereum smart contract this
						// signature The ethereum smart contract call will be the following
						// attach(fragment_hash, local_owner, signature, clamor_nonce);
						// on this target chain the nonce needs to be exactly the same as the
						// one here
						let mut payload = fragment_hash.encode();
						payload.extend(chain_id.encode());
						payload.extend(target_account.clone());
						let nonce = <DetachNonces<T>>::get(&fragment_hash, target_chain.clone());
						let nonce = if let Some(nonce) = nonce {
							// add 1, remote will add 1
							let nonce = nonce.checked_add(1).unwrap();
							payload.extend(nonce.encode());
							nonce // for storage
						} else {
							// there never was a nonce
							payload.extend(1u64.encode());
							1u64
						};
						let msg =
							[&b"\x19Ethereum Signed Message:\n32"[..], &keccak_256(&payload)[..]]
								.concat();
						let msg = keccak_256(&msg);
						// Sign the payload with a trusted validation key
						let signature = Crypto::ecdsa_sign(KEY_TYPE, key, &msg[..]);
						if let Some(signature) = signature {
							// No more failures from this path!!
							Ok((signature.encode(), key.encode(), nonce))
						} else {
							Err(Error::<T>::SigningFailed)
						}
					} else {
						Err(Error::<T>::NoValidator)
					}
				},
			}?;

			// Update nonce
			<DetachNonces<T>>::insert(&fragment_hash, target_chain.clone(), nonce);

			let data = ExportData { chain: target_chain, owner: target_account, nonce };

			// add to exported fragments map
			<DetachedFragments<T>>::insert(fragment_hash, data);

			// emit event
			Self::deposit_event(Event::Exported(fragment_hash, signature, pub_key));

			Ok(())
		}

		/// Transfer fragment ownership
		#[pallet::weight(25_000)] // TODO #1 - weight
		pub fn transfer(
			origin: OriginFor<T>,
			fragment_hash: Hash256,
			new_owner: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// make sure the fragment exists
			let fragment: Fragment<T::AccountId> =
				<Fragments<T>>::get(&fragment_hash).ok_or(Error::<T>::FragmentNotFound)?;

			ensure!(fragment.owner == who, Error::<T>::Unauthorized);
			ensure!(
				!<DetachedFragments<T>>::contains_key(&fragment_hash),
				Error::<T>::FragmentDetached
			);

			// update fragment
			<Fragments<T>>::mutate(&fragment_hash, |fragment| {
				let fragment = fragment.as_mut().unwrap();
				fragment.owner = new_owner.clone();
			});

			// emit event
			Self::deposit_event(Event::Transfer(fragment_hash, new_owner));

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn ensure_upload_auth(
			signature: &ecdsa::Signature,
			signature_hash: &[u8; 32],
		) -> DispatchResult {
			// check if the signature is valid
			// we use and off chain services that ensure we are storing valid data
			let recover = Crypto::secp256k1_ecdsa_recover_compressed(&signature.0, &signature_hash)
				.ok()
				.ok_or(Error::<T>::SignatureVerificationFailed)?;
			let recover = ecdsa::Public(recover);
			ensure!(
				<UploadAuthorities<T>>::get().contains(&recover),
				Error::<T>::SignatureVerificationFailed
			);

			Ok(())
		}
	}
}

impl SupportedChains {
	pub fn iterator() -> Iter<'static, SupportedChains> {
		static DIRECTIONS: [SupportedChains; 3] = [
			SupportedChains::EthereumMainnet,
			SupportedChains::EthereumRinkeby,
			SupportedChains::EthereumGoerli,
		];
		DIRECTIONS.iter()
	}
}
