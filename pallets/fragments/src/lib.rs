#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod weights;

use core::slice::Iter;
use scale_info::prelude::format;
use sp_core::{crypto::KeyTypeId, ecdsa, offchain::StorageKind};
use sp_runtime::offchain::HttpRequestStatus;

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
use sp_io::{crypto as Crypto, hashing::blake2_256, offchain, transaction_index};
use sp_std::{collections::btree_set::BTreeSet, vec, vec::Vec};

use sp_chainblocks::{offchain_fragments, FragmentHash, MutableDataHash};

use frame_support::traits::Randomness;
use frame_system::offchain::{
	AppCrypto, CreateSignedTransaction, SendUnsignedTransaction, SignedPayload, Signer,
	SigningTypes,
};

use sp_runtime::traits::IdentifyAccount;

/// Payload used by this example crate to hold price
/// data required to submit a transaction.
#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub struct FragmentValidation<Public, BlockNumber> {
	block_number: BlockNumber,
	public: Public,
	fragment_hash: FragmentHash,
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

#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug)]
pub struct Fragment<TAccountId> {
	/// Plain hash of indexed data.
	pub mutable_hash: Vec<MutableDataHash>,
	/// Include price of the fragment.
	pub include_cost: Option<Compact<u128>>,
	/// The original creator of the fragment.
	pub creator: TAccountId,
	/// The current owner of the fragment.
	pub owner: TAccountId,
	/// References to other fragments.
	pub references: Option<Vec<FragmentHash>>,
	/// If the fragment has been verified and is passed validation
	pub verified: bool,
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
	pub type Fragments<T: Config> =
		StorageMap<_, Blake2_128Concat, FragmentHash, Fragment<T::AccountId>>;

	#[pallet::storage]
	pub type VerifiedFragments<T: Config> = StorageMap<_, Blake2_128Concat, u128, FragmentHash>;

	#[pallet::storage]
	pub type DetachedFragments<T: Config> =
		StorageMap<_, Blake2_128Concat, FragmentHash, ExportData>;

	#[pallet::storage]
	pub type FragmentsNonces<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, FragmentHash, Blake2_128Concat, SupportedChains, u64>;

	#[pallet::storage]
	pub type VerifiedFragmentsSize<T: Config> = StorageValue<_, u128, ValueQuery>;

	#[pallet::storage]
	pub type UnverifiedFragments<T: Config> = StorageValue<_, BTreeSet<FragmentHash>, ValueQuery>;

	#[pallet::storage]
	pub type FragmentValidators<T: Config> = StorageValue<_, BTreeSet<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	pub type EthereumAuthorities<T: Config> = StorageValue<_, BTreeSet<ecdsa::Public>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Upload(FragmentHash),
		Update(FragmentHash),
		Verified(FragmentHash, bool),
		Exported(FragmentHash, Vec<u8>, Vec<u8>),
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
		/// The provided nonce override is too big
		NonceMismatch,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// Add validator public key to the list
		#[pallet::weight(T::WeightInfo::add_validator())]
		pub fn add_validator(origin: OriginFor<T>, public: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;

			log::debug!("New fragment validator: {:?}", public);

			<FragmentValidators<T>>::mutate(|validators| {
				validators.insert(public);
			});

			Ok(())
		}

		// Remove validator public key to the list
		#[pallet::weight(T::WeightInfo::remove_validator())]
		pub fn remove_validator(origin: OriginFor<T>, public: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;

			log::debug!("New fragment validator: {:?}", public);

			<FragmentValidators<T>>::mutate(|validators| {
				validators.remove(&public);
			});

			Ok(())
		}

		// Fragment confirm function, used internally when a fragment is confirmed valid.
		// TODO need to double check for Zero weight
		#[pallet::weight(25000)] // TODO #1 - weight
		pub fn internal_confirm_upload(
			origin: OriginFor<T>,
			fragment_data: FragmentValidation<T::Public, T::BlockNumber>,
			_signature: T::Signature,
		) -> DispatchResult {
			ensure_none(origin)?;

			let fragment_hash = fragment_data.fragment_hash;

			// remove from unverified
			<UnverifiedFragments<T>>::mutate(|unverified| {
				unverified.remove(&fragment_hash);
			});

			if fragment_data.result {
				let next = <VerifiedFragmentsSize<T>>::get();
				<VerifiedFragments<T>>::insert(next, fragment_hash);
				<VerifiedFragmentsSize<T>>::mutate(|index| {
					*index += 1;
				});

				ensure!(<Fragments<T>>::contains_key(&fragment_hash), Error::<T>::FragmentNotFound);

				<Fragments<T>>::mutate(&fragment_hash, |fragment| {
					let fragment = fragment.as_mut().unwrap();
					fragment.verified = true;
				});
			}

			// also emit event
			Self::deposit_event(Event::Verified(fragment_hash, fragment_data.result));

			log::debug!("Fragment {:?} confirmed, valid: {}", fragment_hash, fragment_data.result);

			Ok(())
		}

		/// Fragment upload function.
		// TODO #1 - weight
		#[pallet::weight(T::WeightInfo::upload())]
		pub fn upload(
			origin: OriginFor<T>,
			immutable_data: Vec<u8>,
			mutable_data: Vec<u8>,
			references: Option<Vec<FragmentHash>>,
			include_cost: Option<Compact<u128>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// hash the immutable data, this is also the unique fragment id
			// to compose the V1 Cid add this prefix to the hash: (str "z" (base58
			// "0x0155a0e40220"))
			let fragment_hash = blake2_256(immutable_data.as_slice());

			// make sure the fragment does not exist already!
			if <Fragments<T>>::contains_key(&fragment_hash) {
				return Err(Error::<T>::FragmentExists.into());
			}

			// we need this to index transactions
			let extrinsic_index = <frame_system::Pallet<T>>::extrinsic_index()
				.ok_or(Error::<T>::SystematicFailure)?;

			let immutable_data_len = immutable_data.len() as u32;
			let mutable_data_len = mutable_data.len() as u32;

			// hash mutable data as well, this time blake2 is fine
			let mutable_hash = blake2_256(mutable_data.as_slice());

			// Write STATE from now, ensure no errors from now...

			// store in the state the fragment
			let fragment = Fragment {
				mutable_hash: vec![mutable_hash],
				include_cost,
				creator: who.clone(),
				owner: who,
				references,
				verified: false,
			};

			// store fragment metadata
			<Fragments<T>>::insert(fragment_hash, fragment);

			// add to unverified fragments list
			<UnverifiedFragments<T>>::mutate(|fragments| fragments.insert(fragment_hash));

			// index immutable data for IPFS discovery
			transaction_index::index(extrinsic_index, immutable_data_len, fragment_hash);

			// index mutable data for IPFS discovery as well
			transaction_index::index(extrinsic_index, mutable_data_len, mutable_hash);

			// also emit event
			Self::deposit_event(Event::Upload(fragment_hash));

			Ok(())
		}

		/// Fragment upload function.
		// TODO #1 - weight
		#[pallet::weight(25_000)]
		pub fn update(
			origin: OriginFor<T>,
			fragment_hash: FragmentHash,
			mutable_data: Option<Vec<u8>>,
			include_cost: Option<Compact<u128>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let fragment =
				<Fragments<T>>::get(&fragment_hash).ok_or(Error::<T>::FragmentNotFound)?;

			ensure!(fragment.owner == who, Error::<T>::Unauthorized);
			ensure!(fragment.verified, Error::<T>::FragmentNotVerified);
			ensure!(
				!<DetachedFragments<T>>::contains_key(&fragment_hash),
				Error::<T>::FragmentDetached
			);

			// we need this to index transactions
			let extrinsic_index = <frame_system::Pallet<T>>::extrinsic_index()
				.ok_or(Error::<T>::SystematicFailure)?;

			// Write STATE from now, ensure no errors from now...

			<Fragments<T>>::mutate(&fragment_hash, |fragment| {
				let fragment = fragment.as_mut().unwrap();
				if let Some(mutable_data) = mutable_data {
					let mutable_data_len = mutable_data.len() as u32;
					// No failures from here on out
					let mutable_hash = blake2_256(mutable_data.as_slice());
					fragment.mutable_hash.push(mutable_hash);
					// index mutable data for IPFS discovery as well
					transaction_index::index(extrinsic_index, mutable_data_len, mutable_hash);
				}
				fragment.include_cost = include_cost;
			});

			// also emit event
			Self::deposit_event(Event::Update(fragment_hash));

			Ok(())
		}

		/// Detached a fragment from this chain by emitting an event that includes a signature.
		/// The remote target chain can attach this fragment by using this signature.
		#[pallet::weight(25_000)] // TODO #1 - weight
		pub fn detach(
			origin: OriginFor<T>,
			fragment_hash: FragmentHash,
			target_chain: SupportedChains,
			target_account: Vec<u8>, // an eth address or so
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// make sure the fragment exists
			let fragment = <Fragments<T>>::get(&fragment_hash);
			if let Some(fragment) = fragment {
				if fragment.owner != who {
					return Err(Error::<T>::Unauthorized.into());
				}
				let lock = <DetachedFragments<T>>::get(&fragment_hash);
				if lock.is_some() {
					return Err(Error::<T>::FragmentDetached.into());
				}
				if !fragment.verified {
					return Err(Error::<T>::FragmentNotVerified.into());
				}

				let chain_id = match target_chain {
					SupportedChains::EthereumMainnet => Some(1u32),
					SupportedChains::EthereumRinkeby => Some(4u32),
					SupportedChains::EthereumGoerli => Some(5u32),
				};

				let (signature, pub_key, nonce) = match target_chain {
					SupportedChains::EthereumMainnet
					| SupportedChains::EthereumRinkeby
					| SupportedChains::EthereumGoerli => {
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
							let nonce =
								<FragmentsNonces<T>>::get(&fragment_hash, target_chain.clone());
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
							// Sign the payload with a trusted validation key
							let signature = Crypto::ecdsa_sign(KEY_TYPE, key, &payload[..]);
							if let Some(signature) = signature {
								// No more failures from this path!!
								Ok((signature.encode(), key.encode(), nonce))
							} else {
								Err(Error::<T>::SigningFailed)
							}
						} else {
							Err(Error::<T>::NoValidator)
						}
					}
				}?;

				// Update nonce
				<FragmentsNonces<T>>::insert(&fragment_hash, target_chain.clone(), nonce);

				let data = ExportData { chain: target_chain, owner: target_account, nonce };

				// add to exported fragments map
				<DetachedFragments<T>>::insert(fragment_hash, data);

				// emit event
				Self::deposit_event(Event::Exported(fragment_hash, signature, pub_key));

				Ok(())
			} else {
				Err(Error::<T>::FragmentNotFound.into())
			}
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
			if let Call::internal_confirm_upload { ref fragment_data, ref signature } = call {
				let signature_valid =
					SignedPayload::<T>::verify::<T::AuthorityId>(fragment_data, signature.clone());
				if !signature_valid {
					return InvalidTransaction::BadProof.into();
				}
				let account = fragment_data.public.clone().into_account();
				if !<FragmentValidators<T>>::get().contains(&account) {
					return InvalidTransaction::BadProof.into();
				}
				log::debug!("Sending confirm_upload extrinsic");
				ValidTransaction::with_tag_prefix("Fragments")
					.and_provides(fragment_data.fragment_hash)
					.longevity(5)
					.propagate(true)
					.build()
			} else {
				InvalidTransaction::Call.into()
			}
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// Offchain Worker entry point.
		///
		/// By implementing `fn offchain_worker` you declare a new offchain worker.
		/// This function will be called when the node is fully synced and a new best block is
		/// successfully imported.
		/// Note that it's not guaranteed for offchain workers to run on EVERY block, there might
		/// be cases where some blocks are skipped, or for some the worker runs twice (re-orgs),
		/// so the code should be able to handle that.
		/// You can use `Local Storage` API to coordinate runs of the worker.
		fn offchain_worker(block_number: T::BlockNumber) {
			if offchain::is_validator() {
				Self::process_unverified_fragments(block_number);
				Self::maintain_mainchains_sync();
			}
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

impl<T: Config> Pallet<T> {
	pub fn gather_references(fragment: &Fragment<T::AccountId>) -> Vec<Vec<u8>> {
		let mut result = Vec::new();
		if let Some(references) = &fragment.references {
			for reference in references {
				let referenced = <Fragments<T>>::get(&reference);
				if let Some(referenced) = referenced {
					let current = (reference, referenced.include_cost);
					let references = Self::gather_references(&referenced);
					result.extend(references);
					result.push(current.encode());
				}
			}
		}
		result
	}

	pub fn random_u128() -> u128 {
		let random_value =
			<pallet_randomness_collective_flip::Pallet<T>>::random(&b"fragments-offchain"[..]);
		let chain_seed = random_value.0.encode();
		let local_seed = offchain::random_seed();
		let random = [&local_seed[..], &chain_seed[..]].concat();
		let random = blake2_256(&random);
		let (int_bytes, _) = random.split_at(16);
		u128::from_le_bytes(int_bytes.try_into().unwrap())
	}

	fn process_unverified_fragments(block_number: T::BlockNumber) {
		// grab all fragments that are ready to be validated
		let fragment_hashes = <UnverifiedFragments<T>>::get();
		if !fragment_hashes.is_empty() {
			log::debug!(
				"Running fragments validation duties, fragments pending: {}",
				fragment_hashes.len()
			);

			let random = Self::random_u128();
			for fragment_hash in fragment_hashes {
				let chance = random % 100;
				// TODO Need to modify this condition
				//if chance < 10 {
				// 10% chance to validate
				log::debug!("offchain_worker processing fragment {:?}", fragment_hash);
				let _fragment = <Fragments<T>>::get(&fragment_hash);
				// run chainblocks validation etc...
				let valid = offchain_fragments::on_new_fragment(&fragment_hash);
				// -- Sign using any account
				let result = Signer::<T, T::AuthorityId>::any_account()
					.send_unsigned_transaction(
						|account| FragmentValidation {
							block_number,
							public: account.public.clone(),
							fragment_hash,
							result: valid,
						},
						|payload, signature| Call::internal_confirm_upload {
							fragment_data: payload,
							signature,
						},
					)
					.ok_or("No local accounts accounts available.");
				if let Err(e) = result {
					log::error!("Error while processing fragment: {:?}", e);
				}
				//}
			}
		}
	}

	fn maintain_mainchains_sync() {
		for chain in SupportedChains::iterator() {
			let random = Self::random_u128();
			let chance = random % 100;
			if chance >= 10 {
				continue;
			}

			log::debug!("Running mainchain sync duties, chain: {:?}", chain);

			// key is our "salt" + chain enum index (scale encoded)
			let key = [&b"mainchain-sync"[..], &chain.encode()[..]].concat();
			let key = blake2_256(&key);
			let block_number = offchain::local_storage_get(StorageKind::PERSISTENT, &key[..]);
			let block_number = if let Some(block_number) = block_number {
				let block_number = u64::decode(&mut block_number.as_slice()).unwrap();
				block_number
			} else {
				0
			};
			let query = format!("query{{transferEntities(where: {{block_number_gt: \\\"{}\\\"}}){{from to fragment_hash block_number}}}}", block_number);
			let query = format!("{{\"query\": \"{}\"}}", query);
			let run = || -> Option<()> {
				let req = offchain::http_request_start(
					"POST",
					"https://api.thegraph.com/subgraphs/name/sinkingsugar/fragments",
					&[],
				)
				.ok()?;
				offchain::http_request_add_header(req, "Content-Type", "application/json").ok()?;
				offchain::http_request_add_header(req, "User-Agent", "clamor/0.1").ok()?;
				offchain::http_request_write_body(req, query.as_bytes(), None).ok()?;
				offchain::http_request_write_body(req, &[], None).ok()?;
				// Notice we should execute asyncronously but we risk to saturate our allowed
				// bandwidth
				if let Some(response) = offchain::http_response_wait(&[req], None).first() {
					match response {
						HttpRequestStatus::Finished(status) => {
							log::debug!("Finished http request, status: {:?}", status);
							let headers = offchain::http_response_headers(req);
							let mut len = 0;
							for header in headers {
								let name = sp_std::str::from_utf8(&header.0).ok()?;
								let value = sp_std::str::from_utf8(&header.1).ok()?;
								log::debug!("Header: {} = {}", name, value);
								if name == "content-length" {
									len = value.parse::<usize>().ok()?;
								}
							}
							if len > 0 {
								let mut res = Vec::new();
								res.resize(len, 0u8);
								let len =
									offchain::http_response_read_body(req, &mut res[..], None);
								match len {
									Ok(_len) => {
										let text = sp_std::str::from_utf8(&res).ok()?;
										log::debug!("Response body: {}", text);
									}
									Err(e) => {
										log::error!("Error while reading http response: {:?}", e)
									}
								}
							}
						}
						HttpRequestStatus::DeadlineReached => {
							log::error!("Deadline reached while waiting for http request");
						}
						HttpRequestStatus::Invalid => {
							log::error!("Invalid http request");
						}
						HttpRequestStatus::IoError => {
							log::error!("Io error while waiting for http request");
						}
					}
				}
				Some(())
			};
			if run().is_none() {
				log::error!("Error while running mainchain sync duties");
			}

			// if offchain::http_response_wait(&[req], None).first().is_none() {
			// 	log::error!("Error while waiting http request");
			// 	break
			// }
			// //...
			// offchain::local_storage_set(StorageKind::PERSISTENT, &key[..],
			// &block_number.encode()[..]);
		}
	}
}
