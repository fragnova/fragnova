//! This pallet `detach` performs logic related to Detaching a Proto-Fragment from the Clamor Blockchain to an External Blockchain

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod weights;

use sp_core::{crypto::KeyTypeId, ecdsa, ed25519, U256};

/// Defines application identifier for crypto keys of this module.
///
/// Every module that deals with signatures needs to declare its unique identifier for
/// its crypto keys.
/// When offchain worker is signing transactions it's going to request keys of type
/// `KeyTypeId` from the keystore and use the ones it finds to sign the transaction.
/// The keys can be inserted manually via RPC (see `author_insertKey`).
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"frag");

/// Add a crypto module with an ed25519 signature key to ensure that your pallet owns an account 
/// that can be used for signing transactions.
pub mod crypto {
	use super::KEY_TYPE;
	use sp_core::ed25519::Signature as Ed25519Signature;
	use sp_runtime::{
		app_crypto::{app_crypto, ed25519},
		traits::Verify,
		MultiSignature, MultiSigner,
	};

	/// The app_crypto macro declares an account with an `ed25519` signature that is identified by KEY_TYPE. 
	/// Note that this doesn't create a new account. 
	/// The macro simply declares that a crypto account is available for this pallet. 
	/// You will need to initialize this account yourself.
	/// 
	/// More info: https://docs.substrate.io/how-to-guides/v3/ocw/transactions/	
	app_crypto!(ed25519, KEY_TYPE);

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

use codec::{Decode, Encode};
use sp_io::{crypto as Crypto, hashing::keccak_256, offchain_index};
use sp_runtime::{offchain::storage::StorageValueRef, MultiSigner};
use sp_std::{collections::btree_set::BTreeSet, vec, vec::Vec};

use sp_clamor::Hash256;

use frame_system::offchain::{
	AppCrypto, CreateSignedTransaction, SendUnsignedTransaction, SignedPayload, Signer,
	SigningTypes,
};

/// **Possible Blockchains into which a **Proto-Fragment** can be **detached**
#[derive(Encode, Decode, Copy, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub enum SupportedChains {
	EthereumMainnet,
	EthereumRinkeby,
	EthereumGoerli,
}

/// **Struct** that **represents** a **request to detach a Proto-Fragment** 
#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub struct DetachRequest {
	/// **Hash** of the **data** of the **Proto-Fragment**
	pub hash: Hash256,
	/// **External Blockchain** to **send** the **Proto-Fragment** to
	pub target_chain: SupportedChains,
	/// **Account Address** on the `target_chain` to send the `Proto-Fragment` to
	pub target_account: Vec<u8>, // an eth address or so
}

/// **Struct** that **represents** a ***signed* request** (signed by the Clamor Account Address `public`) to detach a **Proto-Fragment**
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq)]
pub struct DetachInternalData<TPublic> {
	/// Clamor Public Account Address (the account address should be in FragKey, otherwise it fails)  (问Gio)
	pub public: TPublic,
	/// Proto-Fragment's Data's Hash
	pub hash: Hash256,
	/// External Blockchain to transfer the Proto-Fragment to
	pub target_chain: SupportedChains,
	/// PublicAccount Address of the External Blockchain to transfer the Proto-Fragment to
	pub target_account: Vec<u8>, // an eth address or so
	/// Signature that is signed by an EthereumAuthority on the payload
	pub remote_signature: Vec<u8>,
	/// The number of signed detaches done by the `target_account` (whether successful or unsuccessful)
	pub nonce: u64,
}

/// To make your data structure (i.e the payload - i.e `DetachInternalData`) signable when sending unsigned transactions with signed payload, 
/// implement the SignedPayload trait.
impl<T: SigningTypes> SignedPayload<T> for DetachInternalData<T::Public> {
	fn public(&self) -> T::Public {
		self.public.clone()
	}
}

/// **Struct** that **contains information** about a **detached Proto-Fragment**
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq)]
pub struct ExportData {
	/// The **Blockchain** the **Proto-Fragment** was **detached onto**
	chain: SupportedChains,
	/// The account address (on the blockchain `SupportedChain`) that the Proto-Fragment was transfered into
	owner: Vec<u8>,
	/// For now we don't allow to re-attach but in the future we will,
	/// this nonce is in 1:1 relationship with the remote chain,
	/// so that e.g. if we detach on ethereum the message cannot be repeated and needs to go 1:1 with clamor (INCDT)
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
	#[derive(Default)]
	pub struct GenesisConfig {
		///
		pub eth_authorities: Vec<ecdsa::Public>,
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

	/// **StorageValue** that equals a **list of detach requests**
	#[pallet::storage]
	pub type DetachRequests<T: Config> = StorageValue<_, Vec<DetachRequest>, ValueQuery>;

	/// **StorageDoubleMap** that maps an **account address on an external blockchain and the external blockchain itself** to a **nonce** 
	#[pallet::storage]
	pub type DetachNonces<T: Config> =
		StorageDoubleMap<_, Twox64Concat, Vec<u8>, Twox64Concat, SupportedChains, u64>;

	/// **StorageMap** that maps a **detached Proto-Fragment's hash** to an ***ExportData* enum (this enum contains information about the Proto-Fragment' detachment)**
	#[pallet::storage]
	pub type DetachedHashes<T: Config> = StorageMap<_, Identity, Hash256, ExportData>;

	/// **StorageValue** that equals the **set of ECDSA public keys of the Ethereum accounts** that are **authorized to detach a Proto-Fragment** onto **Fragnova's Ethereum Smart Contract**
	#[pallet::storage]
	pub type EthereumAuthorities<T: Config> = StorageValue<_, BTreeSet<ecdsa::Public>, ValueQuery>;

	// These are the public keys representing the actual keys that can Sign messages
	// to present to external chains to detach onto
	/// **StorageValue** that equals the **List of Clamor Account IDs** that both ***validate*** and ***send*** **unsigned transactions with signed payload**
	/// NOTE: Only the Root User of the Clamor Blockchain (i.e the local node itself) can edit `FragKeys`
	#[pallet::storage]
	pub type FragKeys<T: Config> = StorageValue<_, BTreeSet<ed25519::Public>, ValueQuery>;

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
		/// Already detached
		Detached,
		/// No Validators are present
		NoValidator,
		/// Failed to sign message
		SigningFailed,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	/// Functions that are callable from outside the runtime.
	#[pallet::call]
	impl<T: Config> Pallet<T> {


		/// **Add** an **ECDSA public key** to the **set of Ethereum accounts that are authorized to detach a Proto-Fragment onto Fragnova's Ethereum Smart Contract**
		#[pallet::weight(T::WeightInfo::add_eth_auth())]
		pub fn add_eth_auth(origin: OriginFor<T>, public: ecdsa::Public) -> DispatchResult {
			ensure_root(origin)?;

			log::debug!("New eth auth: {:?}", public);

			<EthereumAuthorities<T>>::mutate(|validators| {
				validators.insert(public);
			});

			Ok(())
		}

		/// **Remove** an **ECDSA public key** from the **set of Ethereum accounts that are authorized to detach a Proto-Fragment onto Fragnova's Ethereum Smart Contract**
		#[pallet::weight(T::WeightInfo::del_eth_auth())]
		pub fn del_eth_auth(origin: OriginFor<T>, public: ecdsa::Public) -> DispatchResult {
			ensure_root(origin)?;

			log::debug!("Removed eth auth: {:?}", public);

			<EthereumAuthorities<T>>::mutate(|validators| {
				validators.remove(&public);
			});

			Ok(())
		}

		/// Add an **Ed25519 public key** to the **set of Clamor accounts that are authorized to sign a message**, where the **message** is a **request to detach a Proto-Fragment from Clamor**
		#[pallet::weight(T::WeightInfo::add_eth_auth())]
		pub fn add_key(origin: OriginFor<T>, public: ed25519::Public) -> DispatchResult {
			ensure_root(origin)?;

			log::debug!("New key: {:?}", public);

			<FragKeys<T>>::mutate(|validators| {
				validators.insert(public);
			});

			Ok(())
		}

		/// Rmove an **Ed25519 public key** from the **set of Clamor accounts that are authorized to sign a message**, where the **message** is a **request to detach a Proto-Fragment from Clamor**
		#[pallet::weight(T::WeightInfo::del_eth_auth())]
		pub fn del_key(origin: OriginFor<T>, public: ed25519::Public) -> DispatchResult {
			ensure_root(origin)?;

			log::debug!("Removed key: {:?}", public);

			<FragKeys<T>>::mutate(|validators| {
				validators.remove(&public);
			});

			Ok(())
		}

		// Detached a Proto-Fragment from this blockchain (Clamor) by emitting an event that includes a signature (note: the event is placed to the System pallet's runtime storage for the block this transaction runs it).
		// The owner of the Proto-Fragment can then attach the Proto-Fragment to the remote target blockchain by using the aforementioned signature .
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

			// add to Detached protos map
			<DetachedHashes<T>>::insert(data.hash, export_data);

			// emit event
			// The function `deposit_event` places the event in the System pallet's runtime storage for that block (https://docs.substrate.io/v3/runtime/events-and-errors/)
			Self::deposit_event(Event::Detached(data.hash, data.remote_signature.clone()));

			log::debug!("Detached hash: {:?} signature: {:?}", data.hash, data.remote_signature);

			Ok(())
		}
	}

	/// Define some logic that should be executed regularly in some context, for e.g. `on_initialize`.
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// During the block finalization phase, the **list** in **DetachRequests** is **stored in the Offchain DB** under the **index "fragments-detach-requests"**. Afterwhich, **DetachRequests** is **cleared**
		fn on_finalize(_n: T::BlockNumber) {
			// drain and process requests
			let requests = <DetachRequests<T>>::take();
			if !requests.is_empty() {
				log::debug!("Got {} detach requests", requests.len());
				offchain_index::set(b"fragments-detach-requests", &requests.encode());
			}
		}


		/// This function is being called after every block import (when fully synced).
		/// 
		/// Implementing this function on a module allows you to perform long-running tasks
		/// that make (by default) validators generate transactions that feed results
		/// of those long-running computations back on chain.
		fn offchain_worker(_n: T::BlockNumber) {
			<Pallet<T>>::process_detach_requests();
		}
	}

	/// By default, all unsigned transactions are rejected in Substrate. 
	/// To enable Substrate to accept certain unsigned transactions, you must implement the ValidateUnsigned trait for the pallet.
	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;

		/// For the call `Call:internal_finalize_detach` which is an unsigned transaction with a signed payload (see: https://docs.substrate.io/how-to-guides/v3/ocw/transactions/), 
		/// verify that when we put the signature parameter (written as `signature`) and the payload parameter (written as `data`) of the aforementioned call into an "Ethereum Verify function",
		/// it returns the public key that is in the payload parameter. 
		/// 
		/// Furthermore, also verify that `data.public` is in `FragKeys` - 问Gio
		/// 
		/// If both the aforementioned conditions meet, allow the call to execute. Otherwise, do not allow it to.
		/// 
		/// ## Footnote:
		/// 
		/// Validate unsigned call to this module.
		///
		/// By default unsigned transactions are disallowed, but implementing the validator
		/// here we make sure that some particular calls (the ones produced by offchain worker)
		/// are being whitelisted and marked as valid.
		fn validate_unsigned(source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			// Firstly let's check that we call the right function.
			if let Call::internal_finalize_detach { ref data, ref signature } = call {
				// ensure it's a local transaction sent by an offchain worker
				match source {
					TransactionSource::InBlock | TransactionSource::Local => {},
					_ => {
						log::debug!("Not a local transaction");
						return InvalidTransaction::Call.into();
					},
				}

				// check public is valid
				let valid_keys = <FragKeys<T>>::get(); // INC questo block (are these the public keys on Ethereum that Clamor owns?)
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
						return InvalidTransaction::BadSigner.into(); 
					}
				};
				log::debug!("Public key: {:?}", pub_key);
				if !valid_keys.contains(&pub_key) {
					return InvalidTransaction::BadSigner.into(); // 问Gio
				}
				// most expensive bit last
				let signature_valid =
					SignedPayload::<T>::verify::<T::AuthorityId>(data, signature.clone()); // Verifying a Data with a Signature Returns a Public Key 
				if !signature_valid {
					return InvalidTransaction::BadProof.into();
				}
				log::debug!("Sending detach finalization extrinsic");
				ValidTransaction::with_tag_prefix("Detach") // The tag prefix prevents other nodes to do the same transaction that have the same tag prefixes
					.and_provides((
						data.hash,
						data.target_chain,
						data.target_account.clone(),
						data.nonce,
					))
					.propagate(false)
					.build()
			} else {
				InvalidTransaction::Call.into()
			}
		}
	}

	impl<T: Config> Pallet<T> {
		/// Initializes the set of ECDSA public keys of the Ethereum accounts that are authorized to detach a Proto-Fragment
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

		/// Initializes the set of Ed25519 public keys of the Clamor accounts that are authorized to sign a message, where the message is a request to detach a Proto-Fragment from Clamor.
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

		// NOTE: This function runs off-chain, so it can access the block state, but cannot preform any alterations. More specifically alterations are not forbidden, but they are not persisted in any way
		/// Signs the list of detach requests using an authority in `EthereumAuthorities`.  
		/// The format of each detach request (which is then signed) is of a tuple as follows: 
		/// (<Proto-Fragment Hash>, <Target Chain ID>, <An Account Address on the Target Chain>, <Nonce>)
		/// 
		/// Then, for each of the signed detach requests - send an unsigned transaction with a signed payload onto the Clamor Blockchain
		/// (NOTE: the signed payload consists of a payload and a signature. 
		/// The payload is the detach request which is represented as a `DetachInternalData` struct (the struct contains the signature on the aforementioned tuple, amongst other things)
		/// and the signature is the signature obtained from signing the aforementioned payload using `Signer::<T, T::AuthorityId>::any_account()`) (问Gio)
		/// 
		/// NOTE: `Signer::<T, T::AuthorityId>::any_account()` uses any of the keys that was added using the RPC `author_insertKey` into Clamor (https://polkadot.js.org/docs/substrate/rpc/#insertkeykeytype-text-suri-text-publickey-bytes-bytes)
		fn process_detach_requests() {
			const FAILED: () = ();
			let requests = StorageValueRef::persistent(b"fragments-detach-requests"); // Reference to list of detach requests
			let _ =
				requests.mutate(|requests: Result<Option<Vec<DetachRequest>>, _>| match requests { // Loop through the list of detach requests
					Ok(Some(requests)) => {
						log::debug!("Got {} detach requests", requests.len());
						for request in requests {
							let chain_id = match request.target_chain {
								SupportedChains::EthereumMainnet => U256::from(1),
								SupportedChains::EthereumRinkeby => U256::from(4),
								SupportedChains::EthereumGoerli => U256::from(5),
							};

							let values = match request.target_chain {
								SupportedChains::EthereumMainnet
								| SupportedChains::EthereumRinkeby
								| SupportedChains::EthereumGoerli => {
									// check if we need to generate new ecdsa keys
									let ed_keys = Crypto::ed25519_public_keys(KEY_TYPE); // defined in service.rs
									// 
									let keys_ref =
										StorageValueRef::persistent(b"fragments-frag-ecdsa-keys"); // This key does not exist when the blockchain is first launched
									let keys = keys_ref
										.get::<BTreeSet<ed25519::Public>>()
										.unwrap_or_default(); // If `keys` doesn't exist, set it to `BTreeSet<ed25519::Public>`
									let mut keys =
										if let Some(keys) = keys { keys } else { BTreeSet::new() }; // If `keys` is None, set it to `BTreeSet<ed25519::Public>`
									// doing this cos mutate was insane...
									let mut edited = false;
									for ed_key in &ed_keys { // INC questo block
										if !keys.contains(ed_key) {
											let mut msg = b"fragments-frag-ecdsa-keys".to_vec();
											msg.append(&mut ed_key.to_vec());
											let signed =
												Crypto::ed25519_sign(KEY_TYPE, ed_key, &msg) 
													.unwrap(); // Determistically sign a constant message `msg` with Ed25519 key `ed_key`
											let key = keccak_256(&signed.0[..]); // Determistically Keccak Hash the determestic signature `signed`
											let mut key_hex = [0u8; 64]; 
											hex::encode_to_slice(key, &mut key_hex) 
												.map_err(|_| FAILED)?; // (Determinstically )UTF-8 encode the hexadecimal representation of `key` and save it in `key_hex` (see the example of `encode_to_slice` if you're confused)
											let key_hex = [b"0x", &key_hex[..]].concat();
											log::debug!("Adding new key from seed: {:?}", key_hex);
											// Generate an ECSDSA key for the given key type using the seed `seed` (WHICH WAS DETERMINSTICALLY COMPUTED 
											// FROM THE ED25519 KEY `ed_key`) and store it in the keystore. 
											let _public =
												Crypto::ecdsa_generate(KEY_TYPE, Some(key_hex)); 
											
											keys.insert(*ed_key);
											edited = true;
										}
									}
									if edited {
										/// Set the list of determinstically computed ECSDA keys `keys` (they were determintically computed using `ed_keys`) 
										/// to the `keys_ref` (i.e to **`StorageValueRef::persistent(b"fragments-frag-ecdsa-keys")`**)
										keys_ref.set(&keys);
									}
									// get local keys
									let keys = Crypto::ecdsa_public_keys(KEY_TYPE); // Get the list of ECDSA public keys for the key id `KEY_STORE` in the keystore 
									log::debug!("ecdsa local keys {:x?}", keys);
									// make sure the local key is in the global authorities set!
									let key = keys
										.iter()
										.find(|k| <EthereumAuthorities<T>>::get().contains(k)); // Get the first key and return it
									if let Some(key) = key {
										// This is critical, we send over to the ethereum smart
										// contract this signature The ethereum smart contract call
										// will be the following attach(proto_hash, local_owner,
										// signature, clamor_nonce); on this target chain the nonce
										// needs to be exactly the same as the one here
										let mut payload = request.hash.encode(); // The creation of the payload // Add `request.hash.encode()` to payload
										let mut chain_id_be: [u8; 32] = [0u8; 32];
										chain_id.to_big_endian(&mut chain_id_be);
										payload.extend(&chain_id_be[..]); // Add `chain_id_be` to payload
										let mut target_account: [u8; 20] = [0u8; 20];
										if request.target_account.len() != 20 {
											return Err(FAILED);
										}
										target_account.copy_from_slice(&request.target_account[..]);
										payload.extend(&target_account[..]); // Add `target_account` to payload
										let nonce = <DetachNonces<T>>::get(
											&request.target_account,
											request.target_chain,
										);
										let nonce = if let Some(nonce) = nonce {
											// add 1, remote will add 1
											let nonce = nonce.checked_add(1).unwrap();
											payload.extend(nonce.to_be_bytes()); // Add `nonce` to payload (if)
											nonce // for storage
										} else {
											// there never was a nonce
											payload.extend(1u64.to_be_bytes()); // Add `nonce` to payload (else)
											1u64
										};
										log::debug!(
											"payload: {:x?}, len: {}",
											payload,
											payload.len()
										);
										let payload = keccak_256(&payload); // Hash the payload!!
										log::debug!(
											"payload hash: {:x?}, len: {}",
											payload,
											payload.len()
										);
										let msg =
											[b"\x19Ethereum Signed Message:\n32", &payload[..]]
												.concat();
										let msg = keccak_256(&msg); // Hash the msg!!
										// Sign the payload with a trusted validation key
										let signature =
											Crypto::ecdsa_sign_prehashed(KEY_TYPE, key, &msg); // Sign the message `msg` with the ECDSA public key `key` (note: `key` is in `EthereumAuthorities`)
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
									log::debug!(
										"Executing unsigned transaction for detach; signature: {:x?}, nonce: {}",
										signature,
										nonce
									);


									// `send_unsigned_transaction` is returning a type of `Option<(Account<T>, Result<(), ()>)>`.
									//   The returned result means:
									//   - `None`: no account is available for sending transaction
									//   - `Some((account, Ok(())))`: transaction is successfully sent
									//   - `Some((account, Err(())))`: error occurred when sending the transaction
									if let Err(e) = Signer::<T, T::AuthorityId>::any_account() // `Signer::<T, T::AuthorityId>::any_account()` is the signer that signs the payload
										.send_unsigned_transaction( // INC questo block
											// this line is to prepare and return payload to be used
											|account| DetachInternalData { // `account` is the account `Signer::<T, T::AuthorityId>::any_account()`
												public: account.public.clone(), // 问Gio what is account.public and why is it supposed to be in FragKey
												hash: request.hash,
												target_chain: request.target_chain,
												target_account: request.target_account.clone(),
												remote_signature: signature.clone(),
												nonce,
											},
											// The second function closure returns the on-chain call with payload and signature passed in
											|payload, signature| Call::internal_finalize_detach {
												data: payload,
												signature,
											},
										)
										.ok_or("No local accounts accounts available.")
									{
										log::error!("Failed to send unsigned detach transaction with error: {:?}", e);
									}
								},
								Err(e) => {
									log::debug!("Failed to detach with error {:?}", e)
								},
							}
						}
						Ok(vec![])
					},
					_ => Err(FAILED),
				});
		}
	}
}
