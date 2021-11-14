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

use codec::{Compact, Encode};
use sp_io::{
	hashing::{blake2_256, keccak_256},
	storage as Storage, transaction_index,
};
use sp_std::vec::Vec;

use sp_chainblocks::{offchain_fragments, Fragment, FragmentHash};

use frame_support::{BoundedSlice, WeakBoundedVec};
use frame_system::offchain::{
	AppCrypto, CreateSignedTransaction, SendTransactionTypes, SubmitTransaction,
};

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: CreateSignedTransaction<Call<Self>> + frame_system::Config {
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
	#[pallet::getter(fn fragments)]
	pub type Fragments<T: Config> = StorageMap<_, Blake2_128Concat, FragmentHash, Fragment>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Upload(FragmentHash, T::AccountId),
		Update(FragmentHash, T::AccountId),
	}

	// Intermediates
	#[pallet::storage]
	pub(super) type BlockUploadFragments<T: Config> = StorageValue<_, Vec<FragmentHash>, ValueQuery>;
	#[pallet::storage]
	pub(super) type OffchainUploadFragments<T: Config> =
		StorageValue<_, Vec<FragmentHash>, ValueQuery>;
	#[pallet::storage]
	pub type PendingFragments<T: Config> = StorageMap<_, Blake2_128Concat, FragmentHash, Fragment>;

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
		#[pallet::weight(10_000)]
		pub fn confirm_upload(origin: OriginFor<T>, fragment_hash: FragmentHash) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// we need this to index transactions
			let extrinsic_index =
				<frame_system::Pallet<T>>::extrinsic_index().ok_or(Error::<T>::SystematicFailure)?;

			let fragment_info =
				<PendingFragments<T>>::get(&fragment_hash).ok_or(Error::<T>::SystematicFailure)?;

			// index immutable data for ipfs discovery
			let mut index_hash = [0u8; 32];
			index_hash[12..32].copy_from_slice(&fragment_hash);
			transaction_index::index(extrinsic_index, fragment_info.immutable_data_len, index_hash);

			// index mutable data for ipfs discovery as well
			transaction_index::index(
				extrinsic_index,
				fragment_info.mutable_data_len,
				fragment_info.mutable_hash,
			);

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
			// we use eth like/inspired keccak and 20 bytes hash
			// also useful for compatibility with EVM
			let fragment_hash = keccak_256(immutable_data.as_slice());
			let fragment_hash = &fragment_hash[12..32];
			let mut fragment_hash_arr: FragmentHash = [0u8; 20];
			fragment_hash_arr.copy_from_slice(fragment_hash);

			// make sure the fragment does not exist already!
			if <Fragments<T>>::contains_key(&fragment_hash_arr) ||
				<PendingFragments<T>>::contains_key(&fragment_hash_arr)
			{
				return Err(Error::<T>::FragmentExists.into())
			}

			// hash mutable data as well, this time blake2 is fine
			let mutable_hash = blake2_256(mutable_data.as_slice());

			// store in the state the fragment
			// block numbers will be added on finalize
			let fragment = Fragment {
				mutable_hash,
				include_cost,
				creator: who.encode(),
				immutable_block: 0,
				immutable_data_len: immutable_data.len() as u32,
				mutable_block: 0,
				mutable_data_len: mutable_data.len() as u32,
				references,
			};

			// store fragment metadata into the pending list
			// fragments need proper validation and further offchain metadata generation before
			// being usable and valid forever
			<PendingFragments<T>>::insert(fragment_hash_arr, fragment);

			// add to block fragments in order to fix block numbers on block finalize and send them to the
			// offchain worker
			<BlockUploadFragments<T>>::mutate(|fragments| fragments.push(fragment_hash_arr));

			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(n: T::BlockNumber) {
			let block_number: u32 = n.try_into().unwrap_or_default();
			// apparently we get block number from the runtime only on finalize
			// so we rewrite block numbers of fragments here
			let fragments = <BlockUploadFragments<T>>::take();
			for fragment_hash in fragments {
				log::debug!("on_finalize processing fragment upload {:?}", fragment_hash);

				<PendingFragments<T>>::mutate(fragment_hash, |fragment| {
					if let Some(ref mut fragment) = fragment {
						fragment.immutable_block = block_number;
						fragment.mutable_block = block_number;
						// <OffchainUploadFragments<T>>::mutate(|fragments| fragments.push(fragment_hash));
					}
				});
			}
		}

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
			// grab all fragments that are ready to be validated
			// let fragment_hashes = <OffchainUploadFragments<T>>::take();
			// for fragment_hash in fragment_hashes {
			// 	log::debug!("offchain_worker processing fragment upload {:?}", fragment_hash);

			// 	// get data from local storage and clear it as well from database
			// 	let key = [&fragment_hash[..], b"i"].concat();
			// 	if let Some(immutable_data) = Storage::get(&key) {
			// 		Storage::clear(&key);
			// 		let key = [&fragment_hash[..], b"m"].concat();
			// 		if let Some(mutable_data) = Storage::get(&key) {
			// 			Storage::clear(&key);
			// 			// do actual validation and other tasks with immutable_data and mutable_data
			// 			match offchain_fragments::on_new_fragment(&immutable_data, &mutable_data) {
			// 				Ok(()) => {
			// 					log::debug!("on_new_fragment processing {:?}", fragment_hash);
			// 				},
			// 				Err(()) => {
			// 					// in this case we just remove the fragment from the state because it is completely
			// 					// invalid it will always be in the transaction history for the block of archive
			// 					// nodes.. this is how blockchains work and so the ipfs bitswap will still work
			// 					// anyway on such nodes
			// 					<Fragments<T>>::remove(fragment_hash);
			// 				},
			// 			}
			// 		}
			// 	} else {
			// 		log::error!("on_new_fragment failed to get data from local storage {:?}", fragment_hash);
			// 	}
			// }
		}
	}
}
