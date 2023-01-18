//! This pallet `detach` performs logic related to Detaching a Proto-Fragment from the Clamor
//! Blockchain to an External Blockchain

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod dummy_data;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod uomo_contento;

#[allow(missing_docs)]
mod weights;

use sp_core::{crypto::KeyTypeId, ecdsa, ed25519, U256};

/// Defines application identifier for crypto keys of this module.
///
/// Every module that deals with signatures needs to declare its unique identifier for
/// its crypto keys.
/// When offchain worker is signing transactions it's going to request keys of type
/// `KeyTypeId` from the keystore and use the ones it finds to sign the transaction.
/// The keys can be inserted manually via RPC (see `author_insertKey`).
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"deta");

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

	// The app_crypto macro declares an account with an `ed25519` signature that is identified by
	// KEY_TYPE. Note that this doesn't create a new account.
	// The macro simply declares that a crypto account is available for this pallet.
	// You will need to initialize this account yourself.
	//
	// More info: https://docs.substrate.io/how-to-guides/v3/ocw/transactions/
	app_crypto!(ed25519, KEY_TYPE);

	/// The identifier type for an offchain worker.
	pub struct DetachAuthId;

	// implemented for runtime
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

pub use pallet::*;
pub use weights::WeightInfo;

use codec::{Decode, Encode, Compact};
use sp_io::{crypto as Crypto, hashing::keccak_256, offchain_index};
use sp_runtime::{offchain::storage::StorageValueRef, MultiSigner};
use sp_std::{collections::btree_set::BTreeSet, vec, vec::Vec};

use sp_clamor::{Hash128, Hash256, InstanceUnit};

use frame_system::offchain::{
	AppCrypto, CreateSignedTransaction, SendUnsignedTransaction, SignedPayload, Signer,
	SigningTypes,
};

use uomo_contento::{merkle_root, Keccak256};

/// Enum representing a "detachable thing" (i.e a Proto-Fragment or a Fragment Instance) that the User wants to detach from the Clamor Blockchain
#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub enum DetachHash {
	/// A Proto-Fragment (identified by its Hash)
	Proto(Hash256),
	/// A Fragment Instance (identified as a tuple of its Fragment Definition Hash, its Edition ID and its Copy ID)
	Instance(Hash128, Compact<InstanceUnit>, Compact<InstanceUnit>),
}

/// Enum representing a collection of "detachable things" that the User wants to detach from the Clamor Blockchain
#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub enum DetachCollection {
	/// List of Proto-Fragments (identified by its Hash)
	Protos(Vec<Hash256>),
	/// List of Fragment Instances (identified as a tuple of its Fragment Definition Hash, its Edition ID and its Copy ID)
	Instances(Vec<(Hash128, Compact<InstanceUnit>, Compact<InstanceUnit>)>),
}
impl DetachCollection {

	/// Get the Collection type
	fn get_type(&self) -> Vec<u8> {
		match self {
			Self::Protos(_) => b"Proto-Fragment".to_vec(),
			Self::Instances(_) => b"Fragment Instance".to_vec(),
		}
	}

	/// Get the ABI-encoded list of hashes
	///
	/// For Proto-Fragments, the encoding is: `abi.encodePacked(protoHash)`, where `protoHash` is of type `bytes32`
	/// For Fragment Instances, the encoding is: `abi.encodePacked(definitionHash, editionId, copyId)`, where:
	/// 	- `definitionHash` is of type `bytes16`
	/// 	- `editionId` is of type `uint64`
	/// 	- `copyId` is of type `uint64`
	fn get_abi_encoded_hashes(&self) -> Vec<Vec<u8>> {
		match self {
			Self::Protos(proto_hashes) => proto_hashes.iter().map(|proto_hash| proto_hash.to_vec()).collect(),
			Self::Instances(instances) => instances.iter().map(
				|(definition_hash, Compact(edition_id), Compact(copy_id))|
					[&definition_hash[..], &edition_id.to_be_bytes()[..], &copy_id.to_be_bytes()[..]].concat()
			).collect()
		}
	}
}

/// **Possible Blockchains** into which a **Proto-Fragment** can be **detached**
#[derive(Encode, Decode, Copy, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub enum SupportedChains {
	/// Ethereum Mainnet Chain
	EthereumMainnet,
	/// Ethereum Rinkeby Chain
	EthereumRinkeby,
	/// Ethereum Goerli Chain
	EthereumGoerli,
}

/// **Struct** that represents a **request to detach a collection of "detachable thing"s (see enum `DetachHashes` to see what type of collections can be detached)** from the Clamor Blockchain
#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub struct DetachRequest {
	/// Collection of "detachable thing"s
	pub collection: DetachCollection,
	/// **External Blockchain** in which the "detachable thing"s can be attached into, after the "detachable thing"s are detached
	pub target_chain: SupportedChains,
	/// Public Account Address in the External Blockchain to transfer the ownership of the "detachable thing"s to
	pub target_account: Vec<u8>, // an eth address or so
}

/// Payload represents information about a collection of "detachable thing"s (see enum `DetachHash` to see what type of collections can be detached) that will be detached
///
/// Note: This Payload that will be attached to the unsigned transaction `Call::internal_finalize_detach` which will be sent on-chain
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq)]
pub struct DetachInternalData<TPublic> {
	/// Public key that is expected to have a matching key in the keystore, which should be used to sign the payload
	///
	/// See this struct's implementation of `SignedPayload` for more information.
	pub public: TPublic,
	/// Collection of "detachable thing"s
	pub collection: DetachCollection,
	/// **Merkle Root** of a **Binary Merkle Tree created using `hashes`**
	pub merkle_root: Hash256,
	/// **External Blockchain** in which the "detachable thing" can be attached into, after the "detachable thing" is detached
	pub target_chain: SupportedChains,
	/// Public Account Address in the External Blockchain to transfer the ownership of the "detachable thing" to
	pub target_account: Vec<u8>, // an eth address or so
	/// Signature obtained by signing the detach request using a Fragnova-authorized account.
	/// After the "detachable thing" is detached, this signature can be presented to the External Blockchain to attach the "detached thing" to the External Blockchain.
	pub remote_signature: Vec<u8>,
	/// Number of times the the `target_account` on the `target_chain` was specified as the new owner when a "detachable thing" (e.g a Proto-Fragment or a Fragment Instance) was detached.
	pub nonce: u64,
}

/// Implementing the `SignedPayload` Trait allows `DetachInternalData` to be used as a signed payload that can be attached to unsigned transactions that are sent on-chain
/// See: https://paritytech.github.io/substrate/master/frame_system/offchain/trait.SignedPayload.html#
impl<T: SigningTypes> SignedPayload<T> for DetachInternalData<T::Public> {
	/// Return a public key that is expected to have a matching key in the keystore, which should be used to sign the payload.
	///
	/// See: https://paritytech.github.io/substrate/master/frame_system/offchain/trait.SignedPayload.html#tymethod.public
	fn public(&self) -> T::Public {
		self.public.clone()
	}
}

/// **Struct** that **contains information** about **a "detached thing"** (e.g **a detached Proto-Fragment** or a **detached Fragment Instance**) that was detached from the Clamor Blockchain
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq)]
pub struct ExportData {
	/// **External Blockchain** the **"detached thing"** can be **attached to**
	chain: SupportedChains,
	/// Public Account Address (in the blockchain `chain`) to assign ownership of the "detached thing" to
	owner: Vec<u8>,
	// For now we don't allow to re-attach but in the future we will,
	// this nonce is in 1:1 relationship with the remote chain,
	// so that e.g. if we detach on ethereum the message cannot be repeated and needs to go 1:1 with clamor
	/// Detach-Nonce of the Public Account Address  `owner` (in the external blockchain `chain`) when the "detached thing" was detached
	nonce: u64,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use sp_core::ed25519::Public;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		CreateSignedTransaction<Call<Self>>
		+ pallet_randomness_collective_flip::Config
		+ frame_system::Config
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// Weight functions needed for pallet_detach.
		type WeightInfo: WeightInfo;
		/// The identifier type for an offchain worker.
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
	}

	/// The Genesis Configuration for the Pallet.
	#[pallet::genesis_config]
	#[derive(Default)]
	pub struct GenesisConfig {
		/// **List of ECDSA public keys of the Ethereum accounts** that are **authorized to detach a Proto-Fragment** onto **Fragnova's Ethereum Smart Contract**
		pub eth_authorities: Vec<ecdsa::Public>,
		/// **List of Ed25519 Public Keys** that can both ***validate*** and ***send*** **unsigned transactions with signed payload**
		pub keys: Vec<ed25519::Public>,
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			Pallet::<T>::initialize_eth_authorities(&self.eth_authorities);
			Pallet::<T>::initialize_keys(&self.keys);
		}
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// **StorageValue** that equals the **list of detach requests**
	#[pallet::storage]
	pub type DetachRequests<T: Config> = StorageValue<_, Vec<DetachRequest>, ValueQuery>;

	/// **StorageDoubleMap** that maps an **account address on an external blockchain and the external blockchain**
	/// to a **nonce**.
	/// This nonce indicates the number of times the account address was specified as the new owner when a "detachable thing" (see enum `DetachHash` to see what type of things can be detached) was detached.
	///
	/// Note: In Fragnova's Smart Contract `CollectionFactory.sol`, the mapping state variable `nonces`'s (`mapping(address => uint64) nonces`) nonce type is also `uint64`
	#[pallet::storage]
	pub type DetachNonces<T: Config> =
		StorageDoubleMap<_, Twox64Concat, Vec<u8>, Twox64Concat, SupportedChains, u64>;

	/// **StorageMap** that maps a **detached Proto-Fragment or a detached Fragment Instance** to an ***ExportData* enum (this enum contains information about the detachment)**
	#[pallet::storage]
	pub type DetachedHashes<T: Config> = StorageMap<_, Identity, DetachHash, ExportData>;

	/// **StorageValue** that equals the **exclusive set of Ethereum accounts (represented here as ECDSA public keys)** that are
	/// **authorized by Fragnova's Ethereum Smart Contract** to attach Proto-Fragment(s) into the aforementioned Smart Contract
	///
	/// Note: Only the Sudo User can edit `EthereumAuthorities`
	///
	/// Note 2: All the ECDSA public keys in `EthereumAuthorities` have been deterministically computed using the Ed25519 keys in the StorageValue `DetachKeys`
	#[pallet::storage]
	pub type EthereumAuthorities<T: Config> = StorageValue<_, BTreeSet<ecdsa::Public>, ValueQuery>;

	// These are the public keys representing the actual keys that can Sign messages
	// to present to external chains to detach onto
	/// **StorageValue** that equals the **set of Ed25519 Public keys** that both ***validate*** and ***send*** **unsigned transactions with signed payload**
	///
	/// Note: Only the Sudo User can edit `DetachKeys`
	#[pallet::storage]
	pub type DetachKeys<T: Config> = StorageValue<_, BTreeSet<ed25519::Public>, ValueQuery>;

	#[allow(missing_docs)]
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A collection of "detachable thing"s was detached
		CollectionDetached { merkle_root: Hash256, remote_signature: Vec<u8>, collection_type: Vec<u8>, collection: DetachCollection },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Already detached
		Detached,
		/// No Validators are present
		NoValidator,
		/// Failed to sign message
		SigningFailed,
		/// Length of the Target Account in the Target Blockchain Does Not Adhere to the Target Blockchain's Specification
		TargetAccountLengthIsIncorrect,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	/// Functions that are callable from outside the runtime.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// **Add** an **ECDSA public key** into **`EthereumAuthorities`**
		///
		/// Note: Only the Sudo User can edit `EthereumAuthorities`
		#[pallet::weight(T::WeightInfo::add_eth_auth())]
		pub fn add_eth_auth(origin: OriginFor<T>, public: ecdsa::Public) -> DispatchResult {
			ensure_root(origin)?;

			log::debug!("New eth auth: {:?}", public);

			<EthereumAuthorities<T>>::mutate(|validators| {
				validators.insert(public);
			});

			Ok(())
		}

		/// **Remove** an **ECDSA public key** from **`EthereumAuthorities`**
		///
		/// Note: Only the Sudo User can edit `EthereumAuthorities`
		#[pallet::weight(T::WeightInfo::del_eth_auth())]
		pub fn del_eth_auth(origin: OriginFor<T>, public: ecdsa::Public) -> DispatchResult {
			ensure_root(origin)?;

			log::debug!("Removed eth auth: {:?}", public);

			<EthereumAuthorities<T>>::mutate(|validators| {
				validators.remove(&public);
			});

			Ok(())
		}

		/// **Add** an **Ed25519 public key** to `DetachKeys`
		///
		/// Note: Only the Sudo User can edit `EthereumAuthorities`
		#[pallet::weight(T::WeightInfo::add_eth_auth())]
		pub fn add_key(origin: OriginFor<T>, public: ed25519::Public) -> DispatchResult {
			ensure_root(origin)?;

			log::debug!("New key: {:?}", public);

			<DetachKeys<T>>::mutate(|validators| {
				validators.insert(public);
			});

			Ok(())
		}

		/// **Remove** an **Ed25519 public key** from `DetachKeys`
		///
		/// Note: Only the Sudo User can edit `EthereumAuthorities`
		#[pallet::weight(T::WeightInfo::del_eth_auth())]
		pub fn del_key(origin: OriginFor<T>, public: ed25519::Public) -> DispatchResult {
			ensure_root(origin)?;

			log::debug!("Removed key: {:?}", public);

			<DetachKeys<T>>::mutate(|validators| {
				validators.remove(&public);
			});

			Ok(())
		}

		/// Detach a Proto-Fragment from Clamor by emitting an event that includes a signature.
		#[pallet::weight(25_000)] // TODO - weight
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

			// add to `DetachedHashes` map
			match &data.collection {
				DetachCollection::Protos(proto_hashes) => {
					proto_hashes.iter().for_each(|proto_hash| {

						let detach_hash = DetachHash::Proto(*proto_hash);

						<DetachedHashes<T>>::insert(detach_hash.clone(), export_data.clone()); // TODO Review - Should `DetachHash` implement the trait `Copy`?

						log::debug!("Detached hash: {:?} signature: {:?}", detach_hash, data.remote_signature);
					});
				},
				DetachCollection::Instances(instances) => {
					instances.iter().for_each(|(definition_hash, edition_id, copy_id)| {

						let detach_hash = DetachHash::Instance(*definition_hash, *edition_id, *copy_id);

						<DetachedHashes<T>>::insert(detach_hash.clone(), export_data.clone()); // TODO Review - Should `DetachHash` implement the trait `Copy`?

						log::debug!("Detached hash: {:?} signature: {:?}", detach_hash, data.remote_signature);
					});

				}
			}

			Self::deposit_event(Event::CollectionDetached {
				merkle_root: data.merkle_root,
				remote_signature: data.remote_signature,
				collection_type: data.collection.get_type(),
				collection: data.collection,
			});

			Ok(())
		}
	}

	/// Define some logic that should be executed regularly in some context, for e.g. `on_initialize`.
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// During the block finalization phase, the **list** in **DetachRequests** is **stored in the Offchain DB** under the **key "detach-requests"**.
		/// After which, **DetachRequests** is **cleared**.
		///
		/// Note: `on_finalize` is executed at the end of block after all extrinsic are dispatched.
		fn on_finalize(_n: T::BlockNumber) {
			// drain and process requests
			let requests = <DetachRequests<T>>::take();
			if !requests.is_empty() {
				log::debug!("Got {} detach requests", requests.len());
				// Write a key value pair to the Offchain DB database in a buffered fashion.
				// Source: https://paritytech.github.io/substrate/master/sp_io/offchain_index/fn.set.html#
				offchain_index::set(b"detach-requests", &requests.encode());
			}
		}

		/// Implementing this function on a module allows you to perform long-running tasks
		/// that make (by default) validators generate transactions that feed results
		/// of those long-running computations back on chain.
		///
		/// NOTE: This function runs off-chain, so it can access the block state,
		/// but cannot preform any alterations. More specifically alterations are
		/// not forbidden, but they are not persisted in any way after the worker
		/// has finished.
		///
		/// This function is being called after every block import (when fully synced).
		///
		/// Implement this and use any of the `Offchain` `sp_io` set of APIs
		/// to perform off-chain computations, calls and submit transactions
		/// with results to trigger any on-chain changes.
		/// Any state alterations are lost and are not persisted.
		fn offchain_worker(_n: T::BlockNumber) {
			<Pallet<T>>::process_detach_requests();
		}
	}

	/// By default, all unsigned transactions are rejected in Substrate.
	/// To enable Substrate to accept certain unsigned transactions, you must implement the ValidateUnsigned trait for the pallet.
	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;

		/// Whitelist and mark as valid the call `Call::internal_finalize_detach` only if:
		/// 1. The signer that signed the signed payload of the call is in `FragKeys`
		/// 2. The signature of the call can be verified against the signed payload of the call
		///
		/// Important Developer Note: Currently in this function, we are "force type casting" the signer that signed the payload of `Call:internal_finalize_detach`
		/// from `T::Public` to a `MultiSigner`.
		/// This is not ideal since we should not be making any assumptions about `T::Public`.
		///
		/// Footnote:
		///
		/// Validate unsigned call to this module.
		///
		/// By default unsigned transactions are disallowed, but implementing the validator
		/// here we make sure that some particular calls (the ones produced by offchain worker)
		/// are being whitelisted and marked as valid.
		fn validate_unsigned(source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			// Firstly let's check that we call the right function.
			if let Call::internal_finalize_detach {
				ref data, ref signature
			} = call {
				// ensure it's a local transaction sent by an offchain worker
				match source {
					TransactionSource::InBlock | TransactionSource::Local => {},
					_ => {
						log::debug!("Not a local transaction");
						return InvalidTransaction::Call.into();
					},
				}

				// I'm sure there is a way to do this without serialization but I can't spend so
				// much time fighting with rust
				let signer = data.public.encode(); // Note: `signer` is public key of the signer that signed the signed payload `data` and thus produced the signature `signature`
				// convert from `T::Public` to `ed25519::Public`
				let signer: ed25519::Public = {
					if let Ok(MultiSigner::Ed25519(signer)) = <MultiSigner>::decode(&mut &signer[..])
					{
						signer
					} else {
						return InvalidTransaction::BadSigner.into();
					}
				};
				log::debug!("Public key: {:?}", signer);

				log::debug!("Valid keys: {:?}", <DetachKeys<T>>::get());
				// signer must be in `DetachKeys`
				if !<DetachKeys<T>>::get().contains(&signer) {
					return InvalidTransaction::BadSigner.into();
				}

				// most expensive bit last

				// Verify signature `signature` against SignedPayload object `data`. Returns a bool indicating whether the signature is valid or not.
				//
				// Source: https://paritytech.github.io/substrate/master/frame_system/offchain/trait.SignedPayload.html#method.verify
				if !SignedPayload::<T>::verify::<T::AuthorityId>(data, signature.clone()) {
					return InvalidTransaction::BadProof.into();
				}
				log::debug!("Sending detach finalization extrinsic");
				// The tag prefix prevents other nodes to do the same transaction that have the same tag prefixes
				ValidTransaction::with_tag_prefix("Detach")
					// This transaction does not require anything else to go before into the pool.
					// In theory we could require `previous_unsigned_at` transaction to go first,
					// but it's not necessary in our case.
					//.and_requires()
					// We set the `provides` tag to be the same as `next_unsigned_at`. This makes
					// sure only one transaction produced after `next_unsigned_at` will ever
					// get to the transaction pool and will end up in the block.
					// We can still have multiple transactions compete for the same "spot",
					// and the one with higher priority will replace other one in the pool.
					.and_provides((
						data.merkle_root, // TODO Review - what if by co-incidence two collections that contain different things have the same merkle-root?
						data.target_chain,
						data.target_account.clone(),
						data.nonce,
					))
					// It's fine to propagate that transaction to other peers, which means it can be
					// created even by nodes that don't produce blocks.
					// Note that sometimes it's better to keep it for yourself (if you are the block
					// producer), since for instance in some schemes others may copy your solution and
					// claim a reward.
					.propagate(false)
					.build()
			} else {
				InvalidTransaction::Call.into()
			}
		}
	}

	impl<T: Config> Pallet<T> {
		/// Set the initial set of ECDSA public keys in `EthereumAuthorities`
		fn initialize_eth_authorities(authorities: &[ecdsa::Public]) {
			if !authorities.is_empty() {
				assert!(
					<EthereumAuthorities<T>>::get().is_empty(),
					"EthereumAuthorities are already initialized!"
				);
				for authority in authorities {
					<EthereumAuthorities<T>>::mutate(|authorities| {
						authorities.insert(*authority);
					});
				}
			}
		}

		/// Set the initial set of Ed25519 public keys in `DetachKeys`
		fn initialize_keys(keys: &[ed25519::Public]) {
			if !keys.is_empty() {
				assert!(<DetachKeys<T>>::get().is_empty(), "DetachKeys are already initialized!");
				for key in keys {
					<DetachKeys<T>>::mutate(|keys| {
						keys.insert(*key);
					});
				}
			}
		}


		/// Adds newly-found Ed25519 keys to the persistent local storage under the key `b"detach-ecdsa-keys"`.
		/// Furthermore, for each newly-found Ed25519 key - an ECDSA key is deterministically computed (using the Ed25519 key) which is then added to the keystore under the key id `KEY_TYPE`.
		fn add_newly_found_ed25519_and_ecdsa_keys() {
			// Get a storage value reference (i.e `StorageValueRef`) of the key `b"detach-ecdsa-keys"` in the persistent local storage
			let storage_ref =
				StorageValueRef::persistent(b"detach-ecdsa-keys");
			let mut persisted_ed25119_keys =
				Self::get_ed25519_keys_from_storage_value_reference(&storage_ref);

			// doing it like this cos mutate was insane...
			let mut edited = false;

			// Add Ed25519 keys that are not currently in the persistent local storage (under the key `b"detach-ecdsa-keys"`) to the persistent local storage.
			// Furthermore - for each Ed25519 key that's currently not in the persistent local storage, deterministically compute an ECDSA key and add it to the keystore under key id `KEY_TYPE`.
			//
			// Footnote:
			// `sp_io::crypto::ed25519_public_keys` returns all ed25519 public keys for the given key id (in our case it's `KEY_TYPE`) from the keystore
			// Source: https://docs.rs/sp-io/latest/sp_io/crypto/fn.ed25519_public_keys.html
			for ed25519_key in &Crypto::ed25519_public_keys(KEY_TYPE) {
				// Ed25519 key `ed25519_key` is not in `persisted_ed25119_keys`
				if !persisted_ed25119_keys.contains(ed25519_key) {
					let ecdsa_seed_hex =
						Self::generate_ecdsa_seed_from_ed25519_key(&ed25519_key).unwrap();
					log::debug!("Adding new key from seed: {:?}", ecdsa_seed_hex);
					// `sp_io::crypto::ecdsa_generate` generates an ecdsa key for the given key type using an optional seed (in our case it's `Some(key_hex)`)
					// and store it in the keystore. The seed needs to be a valid utf8.
					// Source: https://paritytech.github.io/substrate/master/sp_io/crypto/fn.ecdsa_generate.html#
					//
					// Very Important Note: Since this function "store[s] it [i.e the eccdsa key] in the keystore",
					// the ecdsa key will be retrievable when doing `sp_io::crypto::ecdsa_public_keys(KEY_TYPE)`
					let _public =
						Crypto::ecdsa_generate(KEY_TYPE, Some(ecdsa_seed_hex));
					persisted_ed25119_keys.insert(*ed25519_key);
					edited = true;
				}
			}
			if edited {
				// Update the storage value that is referred by `storage_ref` to `persisted_ed25119_keys`
				storage_ref.set(&persisted_ed25119_keys);
			}
		}

		fn get_merkle_root(detach_collection: &DetachCollection) -> Hash256 {
			let detach_hashes = detach_collection.get_abi_encoded_hashes();
			// TODO Review - Should we sort the leaves?
			// detach_hashes.sort_by(|a, b| Keccak256::hash(a).cmp(&Keccak256::hash(b)));
			merkle_root::<Keccak256, _>(detach_hashes).into()
		}

		/// Returns a Tuple of the following things:
		/// 1. A **Signature** obtained by **signing the detach request `request` using a Fragnova-authorized account**.
		///
		/// Note: Since it was signed by a Fragnova-authorized account, this signature can be presented to the External Blockchain `requests.target_chain`
		/// to attach the "detached thing" (e.g a detached Proto-Fragment or a detached Fragment Instance) to the External Blockchain.
		///
		/// 2. The Detach-Nonce of the detach request's target account. (See `DetachNonces` for more information).
		///
		/// 3. **Merkle Root** of a **Binary Merkle Tree created using `request.hashes`**
		fn get_detach_signature_and_detach_nonce_and_merkle_root(request: &DetachRequest) -> Result<(Vec<u8>, u64, Hash256), Error<T>> {
			match request.target_chain {
				SupportedChains::EthereumMainnet
				| SupportedChains::EthereumRinkeby
				| SupportedChains::EthereumGoerli => {

					Self::add_newly_found_ed25519_and_ecdsa_keys();

					// `sp_io::crypto::ecdsa_public_keys` returns all ecdsa public keys for the given key id from the keystore. (in our case the key id is `KEY_TYPE`).
					// Source: https://docs.rs/sp-io/latest/sp_io/crypto/fn.ecdsa_public_keys.html
					let ecdsa_keys = Crypto::ecdsa_public_keys(KEY_TYPE);
					log::debug!("ecdsa local keys {:x?}", ecdsa_keys);

					// make sure the local key is in the global authorities set!
					let ethereum_authority = ecdsa_keys
						.iter()
						.find(|k| <EthereumAuthorities<T>>::get().contains(k)).ok_or(Error::<T>::NoValidator)?;

					let merkle_root = Self::get_merkle_root(&request.collection);
					// Note: In Fragnova's Smart Contract `CollectionFactory.sol`, the mapping state variable `nonces`'s (`mapping(address => uint64) nonces`) nonce type is also `uint64`
					let nonce =
						<DetachNonces<T>>::get(&request.target_account, request.target_chain).unwrap_or_default().checked_add(1).unwrap();
					// Note: In Solidity, `block.chainid` is of type `uint256`
					let mut chain_id_be: [u8; 32] = [0u8; 32]; // "be" stands for big-endian
					match request.target_chain {
						SupportedChains::EthereumMainnet => U256::from(1),
						SupportedChains::EthereumRinkeby => U256::from(4),
						SupportedChains::EthereumGoerli => U256::from(5),
					}.to_big_endian(&mut chain_id_be);

					let payload = [
						&request.collection.get_type()[..],
						&merkle_root[..],
						&chain_id_be,
						&TryInto::<[u8; 20]>::try_into(request.target_account.clone()).map_err(|_| Error::<T>::TargetAccountLengthIsIncorrect)?,
						&nonce.to_be_bytes(), // "be" stands for big-endian
					].concat();
					log::debug!(
						"payload: {:x?}, len: {}",
						payload,
						payload.len()
					);

					// Get the Ethereum specific signature of the payload
					let signature = Self::eth_sign_payload(&ethereum_authority, &payload).ok_or(Error::<T>::SigningFailed)?;

					Ok((signature.0.to_vec(), nonce, merkle_root))
				},
			}

		}

		/// Signs the list of detach requests using a Fragnova-authorized account.
		/// Then, for each of the signed detach requests - send an unsigned transaction on-chain
		/// that will cause an event to be emitted which will contain the detach request's signature.
		/// This signature can be then used in the target chain to attach the "detachable thing" (see enum `DetachHash` to see what type of things can be detached) to the target chain.
		///
		/// The format of each detach request (which is then signed by a Fragnova-authorized account) is as follows:
		/// keccak_256(<Proto-Fragment Hash> ‖ <Target Chain ID> ‖ <Public Account Address in Target Chain to assign ownership of Proto-Fragment to> ‖ <Detach Nonce of Public Account Address in Target Chain (see `DetachNonces`)>)
		///
		/// Note: On the Target Chain, the "attach nonce" needs to be exactly the same as the detach nonce here.
		pub fn process_detach_requests() {

			const FAILED: () = ();

			/*
			Use `StorageValueRef::persistent(b"detach-requests")`
			to get a reference to the storage value that's under the key `b"detach-requests"` in the persistent local storage.
			and use `StorageValueRef::mutate()`
			to set that storage value to a new value.

			In our case, the storage value is the vector of detach requests.
			And in our case, we set the storage value to an empty vector after processing the vector of detach requests

			Reference: https://paritytech.github.io/substrate/master/sp_runtime/offchain/storage/struct.StorageValueRef.html#method.mutate)
			 */
			let _ =
				StorageValueRef::persistent(b"detach-requests")
					.mutate(|requests: Result<Option<Vec<DetachRequest>>, _>| {

						let requests = requests.map_err(|_| FAILED)?.ok_or(FAILED)?;

						log::debug!("Got {} detach requests", requests.len());
						for request in requests { // Iterate through the list of detach requests

							let tuple_signature_nonce_merkle_root = Self::get_detach_signature_and_detach_nonce_and_merkle_root(&request);

							match tuple_signature_nonce_merkle_root {
								Err(e) => {
									log::debug!("Failed to detach with error {:?}", e)
								},
								// begin process to send unsigned transaction from here
								Ok((signature, nonce, merkle_root)) => {
									log::debug!(
										"Executing unsigned transaction for detach; signature: {:x?}, nonce: {}",
										signature,
										nonce
									);

									/*
									Sign using any account

									Footnote:
									Since this pallet only has one key type in the keystore (i.e `KeyTypeId(*b"deta")`),
									we can just use `any_account() to retrieve a key (that is of the aforementioned key type).
									Reference: https://paritytech.github.io/substrate/master/frame_system/offchain/struct.Signer.html
									*/
									if let Err(e) = Signer::<T, T::AuthorityId>::any_account()
										/*
										Send an unsigned transaction with a signed payload on-chain.
										This method takes `f` and `f2` where:
										- `f` is called for every account and is expected to return a `SignedPayload` object.
										- `f2` is then called with the `SignedPayload` returned by `f` and the signature and is
										expected to return a `Call` object to be embedded into transaction.
										Source: https://paritytech.github.io/substrate/master/frame_system/offchain/trait.SendUnsignedTransaction.html#tymethod.send_unsigned_transaction
										*/
										.send_unsigned_transaction(
											|account| DetachInternalData {
												// Public key that is expected to have a matching key in the keystore, which should be used to sign the payload
												// Note: See the implementation of `SignedPayload` for `DetachInternalData` to understand more
												public: account.public.clone(),
												collection: request.collection.clone(),
												merkle_root,
												target_chain: request.target_chain,
												target_account: request.target_account.clone(),
												remote_signature: signature.clone(),
												nonce,
											},
											// Note: `Call` is an enum that gets generated by the macro `#[pallet::Call]`
											// You can view it in Clamor's Rustdoc: https://fragcolor-xyz.github.io/clamor/doc/pallet_detach/pallet/enum.Call.html#
											|payload, signature| Call::internal_finalize_detach {
												data: payload,
												signature,
											},
										)
										.ok_or("No local accounts accounts available.") {
										log::error!("Failed to send unsigned detach transaction with error: {:?}", e);
									}
								},
							}
						}

						// We set the storage value to an empty vector after processing the vector of detach requests
						Ok::<Vec<DetachRequest>, ()>(vec![])
					}
				);
		}

		/// Deterministically compute an ECDSA Seed using the Ed25519 public key `ed25519_key`.
		/// The ECDSA Seed is then returned as a UTF-8 encoded hexadecimal.
		fn generate_ecdsa_seed_from_ed25519_key(ed25519_key: &Public) -> Result<Vec<u8>, ()> {
			const FAILED: () = ();
			let mut msg = b"detach-ecdsa-keys".to_vec();
			msg.append(&mut ed25519_key.to_vec());
			// Sign the fixed message `b"detach-ecdsa-keys" ‖ ed25519_key` using Ed25519 key `ed25519_key`
			let signature = Crypto::ed25519_sign(KEY_TYPE, ed25519_key, &msg).unwrap();
			let ecdsa_seed = keccak_256(&signature.0[..]);

			let ecdsa_seed_hex = [
				&b"0x"[..],
				&TryInto::<[u8; 64]>::try_into(hex::encode(ecdsa_seed).into_bytes()).map_err(|_| FAILED)? // actually there's no need to throw any error I think...
			].concat();

			Ok(ecdsa_seed_hex)
		}

		// Sign the payload `payload` using the ecdsa `key` and return the signature.
		// Note: The signature is an Ethereum specific signature
		fn eth_sign_payload(key: &ecdsa::Public, payload: &Vec<u8>) -> Option<ecdsa::Signature> {
			let payload_hash = keccak_256(&payload);
			log::debug!("payload hash: {:x?}, len: {}", payload_hash, payload_hash.len());
			let msg = [b"\x19Ethereum Signed Message:\n32", &payload_hash[..]].concat();
			let msg = keccak_256(&msg);

			// Sign the given a pre-hashed msg `msg` with the ecdsa key that corresponds to the given public key `key` and key type `KEY_TYPE` in the keystore. Returns the signature.
			// Source: https://paritytech.github.io/substrate/master/sp_io/crypto/fn.ecdsa_sign_prehashed.html#
			let signature = Crypto::ecdsa_sign_prehashed(KEY_TYPE, key, &msg).map(|mut signature| {
				signature.0[64] += 27u8; // fix signature ending for ethereum
				signature
			});

			signature
		}

		/// Return the Set of Ed25519 public keys that are stored in the StorageValueRef `storage_ref`, if any. Otherwise, return an empty set.
		fn get_ed25519_keys_from_storage_value_reference(storage_ref: &StorageValueRef) -> BTreeSet<ed25519::Public> {
			// If `stored_keys` doesn't exist, set it to `BTreeSet<ed25519::Public>`
			let stored_keys = storage_ref.get::<BTreeSet<ed25519::Public>>().unwrap_or_default();
			// If `keys` is None, set it to `BTreeSet<ed25519::Public>`
			let keys = if let Some(keys) = stored_keys { keys } else { BTreeSet::new() };
			keys
		}

	}
}
