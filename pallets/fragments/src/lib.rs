#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod weights;

use sp_core::crypto::KeyTypeId;

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
	hashing::{blake2_256, keccak_256},
	offchain, offchain_index, transaction_index,
};
use sp_std::vec::Vec;

use sp_chainblocks::{offchain_fragments, Fragment, FragmentHash};

use sp_core::offchain::StorageKind;

use frame_support::{traits::Randomness, BoundedSlice, WeakBoundedVec};
use frame_system::{
	self as system,
	offchain::{
		AppCrypto, CreateSignedTransaction, SendTransactionTypes, SendUnsignedTransaction,
		SignedPayload, Signer, SigningTypes, SubmitTransaction,
	},
};

/// Payload used by this example crate to hold price
/// data required to submit a transaction.
#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub struct ValidFragment<Public, BlockNumber> {
	block_number: BlockNumber,
	public: Public,
	fragment_hash: FragmentHash,
}

impl<T: SigningTypes> SignedPayload<T> for ValidFragment<T::Public, T::BlockNumber> {
	fn public(&self) -> T::Public {
		self.public.clone()
	}
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
	pub type Fragments<T: Config> = StorageMap<_, Blake2_128Concat, FragmentHash, Fragment>;

	#[pallet::storage]
	pub type UnverifiedFragments<T: Config> = StorageValue<_, Vec<FragmentHash>, ValueQuery>;

	#[pallet::storage]
	pub type VerifiedFragments<T: Config> = StorageValue<_, Vec<FragmentHash>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Upload(FragmentHash, T::AccountId),
		Update(FragmentHash, T::AccountId),
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
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// Fragment confirm function, used internally when a fragment is confirmed valid.
		#[pallet::weight(0)]
		pub fn confirm_upload(
			origin: OriginFor<T>,
			fragment_data: ValidFragment<T::Public, T::BlockNumber>,
			_signature: T::Signature,
		) -> DispatchResult {
			ensure_none(origin)?;

			// // we need this to index transactions
			// let extrinsic_index =
			// 	<frame_system::Pallet<T>>::extrinsic_index().ok_or(Error::<T>::SystematicFailure)?;

			// let fragment_hash = fragment_data.fragment_hash;

			// let fragment_info =
			// 	<UnverifiedFragments<T>>::get(&fragment_hash).ok_or(Error::<T>::SystematicFailure)?;

			// let creator = T::AccountId::decode(&mut fragment_info.creator.as_slice())
			// 	.map_err(|_| Error::<T>::SystematicFailure)?;

			// // index immutable data for ipfs discovery
			// let mut index_hash = [0u8; 32];
			// index_hash[12..32].copy_from_slice(&fragment_hash);
			// transaction_index::index(extrinsic_index, fragment_info.immutable_data_len, index_hash);

			// // index mutable data for ipfs discovery as well
			// transaction_index::index(
			// 	extrinsic_index,
			// 	fragment_info.mutable_data_len,
			// 	fragment_info.mutable_hash,
			// );

			// // remove from unverified fragments storage
			// <UnverifiedFragments<T>>::remove(&fragment_hash);
			// // insert to fragments storage
			// <Fragments<T>>::insert(fragment_hash, fragment_info);

			// Self::deposit_event(Event::Upload(fragment_hash, creator));

			Ok(())
		}

		/// Fragment upload function.
		#[pallet::weight(T::WeightInfo::store((immutable_data.len() as u32) + (mutable_data.len() as u32)))]
		pub fn upload(
			origin: OriginFor<T>,
			immutable_data: Vec<u8>,
			mutable_data: Vec<u8>,
			references: Option<Vec<FragmentHash>>,
			include_cost: Option<Compact<u128>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// hash the immutable data, this is also the unique fragment id
			// to compose the V1 Cid add this prefix to the hash: (str "z" (base58 "0x0155a0e40220"))
			let fragment_hash = blake2_256(immutable_data.as_slice());

			// make sure the fragment does not exist already!
			if <Fragments<T>>::contains_key(&fragment_hash) {
				return Err(Error::<T>::FragmentExists.into())
			}

			let block_number = <system::Pallet<T>>::block_number();
			let block_number: u32 = block_number.try_into().unwrap_or_default();

			// we need this to index transactions
			let extrinsic_index =
				<frame_system::Pallet<T>>::extrinsic_index().ok_or(Error::<T>::SystematicFailure)?;

			let immutable_data_len = immutable_data.len() as u32;
			let mutable_data_len = mutable_data.len() as u32;

			// hash mutable data as well, this time blake2 is fine
			let mutable_hash = blake2_256(mutable_data.as_slice());

			let owner = who.encode();

			// Write STATE from now, ensure no errors from now...

			// store in the state the fragment
			let fragment = Fragment {
				mutable_hash,
				include_cost,
				creator: owner.clone(),
				owner,
				immutable_block: block_number,
				mutable_block: block_number,
				references,
				verified: false,
			};

			// store fragment metadata
			<Fragments<T>>::insert(fragment_hash, fragment);

			// add to unverifed fragments list
			<UnverifiedFragments<T>>::mutate(|fragments| fragments.push(fragment_hash));

			// index immutable data for ipfs discovery
			transaction_index::index(extrinsic_index, immutable_data_len, fragment_hash);

			// index mutable data for ipfs discovery as well
			transaction_index::index(extrinsic_index, mutable_data_len, mutable_hash);

			// also emit event
			Self::deposit_event(Event::Upload(fragment_hash, who));

			Ok(())
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
			// // Firstly let's check that we call the right function.
			// if let Call::confirm_upload { ref fragment_data, ref signature } = call {
			// 	let signature_valid =
			// 		SignedPayload::<T>::verify::<T::AuthorityId>(fragment_data, signature.clone());
			// 	if !signature_valid {
			// 		return InvalidTransaction::BadProof.into()
			// 	}
			// 	ValidTransaction::with_tag_prefix("Fragments").propagate(true).build()
			// } else
			{
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
		/// succesfuly imported.
		/// Note that it's not guaranteed for offchain workers to run on EVERY block, there might
		/// be cases where some blocks are skipped, or for some the worker runs twice (re-orgs),
		/// so the code should be able to handle that.
		/// You can use `Local Storage` API to coordinate runs of the worker.
		fn offchain_worker(block_number: T::BlockNumber) {
			if offchain::is_validator() {
				// grab all fragments that are ready to be validated
				let fragment_hashes = <UnverifiedFragments<T>>::get();
				if fragment_hashes.is_empty() {
					return
				}

				log::info!(
					"Running fragments validation duties, fragments pending: {}",
					fragment_hashes.len()
				);

				let random_value =
					<pallet_randomness_collective_flip::Pallet<T>>::random(&b"fragments-offchain"[..]);
				let chain_seed = random_value.0.encode();
				let local_seed = offchain::random_seed();
				let seed = [&local_seed[..], &chain_seed[..]].concat();
				let seed = blake2_256(&seed);
				let (int_bytes, _) = seed.split_at(16);
				let seed = u128::from_le_bytes(int_bytes.try_into().unwrap());

				for fragment_hash in fragment_hashes {
					let chance = seed % 100;
					if chance < 10 {
						// 10% chance to validate
						log::debug!("offchain_worker processing fragment {:?}", fragment_hash);
						let fragment = <Fragments<T>>::get(&fragment_hash);

						// run chainblocks validation etc...
						let valid = true;
						if valid {
							// if valid, mark as verified
							// -- Sign using any account
							let result = Signer::<T, T::AuthorityId>::any_account()
								.send_unsigned_transaction(
									|account| ValidFragment {
										block_number,
										public: account.public.clone(),
										fragment_hash,
									},
									|payload, signature| Call::confirm_upload { fragment_data: payload, signature },
								)
								.ok_or("No local accounts accounts available.");
							if let Err(e) = result {
								log::error!("Error while processing fragment: {:?}", e);
							}
						}
					}
				}
			}
		}
	}
}
