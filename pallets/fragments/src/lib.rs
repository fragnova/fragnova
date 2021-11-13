#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod weights;

pub use pallet::*;
pub use weights::WeightInfo;

use codec::{Compact, Decode, Encode};

use frame_system::{
	self as system,
	offchain::{
		AppCrypto, CreateSignedTransaction, SendSignedTransaction, SendUnsignedTransaction,
		SignedPayload, Signer, SigningTypes, SubmitTransaction,
	},
};

use sp_runtime::{
	offchain::{http, storage},
	traits::{BlakeTwo256, Hash, One, Saturating, Zero},
	AccountId32,
};
// https://substrate.dev/rustdocs/v3.0.0-monthly-2021-05/sp_runtime/offchain/http/index.html
use sp_io::{
	hashing::{blake2_256, keccak_256},
	offchain_index, transaction_index,
};
use sp_std::{ops, vec::Vec};

type FragmentHash = [u8; 20];
type MutableDataHash = [u8; 32];

#[derive(Encode, Decode, Clone, scale_info::TypeInfo)]
pub struct Fragment {
	/// Plain hash of indexed data.
	mutable_hash: MutableDataHash,
	/// Include price of the fragment.
	include_price: Option<Compact<u128>>,
	/// The original creator of the fragment.
	creator: Vec<u8>,
	// Immutable data of the fragment.
	immutable_block: u32,
	// Mutable data of the fragment.
	mutable_block: u32,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type WeightInfo: WeightInfo;
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
	pub(super) type BlockFragments<T: Config> = StorageValue<_, Vec<FragmentHash>, ValueQuery>;

	// The pallet's runtime storage items.
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// Systematic failure - those errors should not happen.
		SystematicFailure,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Fragment upload function.
		#[pallet::weight(T::WeightInfo::store((immutable_data.len() as u32) + (mutable_data.len() as u32)))]
		pub fn upload(
			origin: OriginFor<T>,
			immutable_data: Vec<u8>,
			mutable_data: Vec<u8>,
			references: Vec<FragmentHash>,
			include_cost: Option<Compact<u128>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// we need this to index transactions
			let extrinsic_index =
				<frame_system::Pallet<T>>::extrinsic_index().ok_or(Error::<T>::SystematicFailure)?;

			// hash the immutable data, this is also the unique fragment id
			// we use eth like/inspired keccak and 20 bytes hash
			// also useful for compatibility with EVM
			let fragment_hash = keccak_256(immutable_data.as_slice());
			let fragment_hash = &fragment_hash[12..32];
			let mut index_hash = [0u8; 32];
			index_hash[12..32].copy_from_slice(fragment_hash);
			let mut fragment_hash_arr: FragmentHash = [0u8; 20];
			fragment_hash_arr.copy_from_slice(fragment_hash);
			sp_io::transaction_index::index(extrinsic_index, immutable_data.len() as u32, index_hash);

			// hash mutable data as well, this time blake2 is fine
			let mutable_hash = blake2_256(mutable_data.as_slice());
			sp_io::transaction_index::index(extrinsic_index, mutable_data.len() as u32, mutable_hash);

			// store in the state the fragment
			// block numbers will be added on finalize
			let fragment = Fragment {
				mutable_hash: blake2_256(mutable_data.as_slice()),
				include_price: include_cost,
				creator: who.encode(),
				immutable_block: 0,
				mutable_block: 0,
			};

			// store fragment metadata
			<Fragments<T>>::insert(fragment_hash_arr, fragment);

			// add to block fragments in order to fix block numbers on block finalize
			<BlockFragments<T>>::mutate(|fragments| fragments.push(fragment_hash_arr));

			// emit event
			Self::deposit_event(Event::Upload(fragment_hash_arr, who));

			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(n: T::BlockNumber) {
			let block_number: u32 = n.try_into().unwrap_or_default();
			// apparently we get block number from the runtime only on finalize
			// so we rewrite block numbers of fragments here
			let fragments = <BlockFragments<T>>::take();
			for fragment_hash in fragments {
				let fragment = <Fragments<T>>::get(fragment_hash);
				if let Some(mut fragment) = fragment {
					fragment.immutable_block = block_number;
					fragment.mutable_block = block_number;
					<Fragments<T>>::insert(fragment_hash, fragment);
				}
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
			// Note that having logs compiled to WASM may cause the size of the blob to increase
			// significantly. You can use `RuntimeDebug` custom derive to hide details of the types
			// in WASM. The `sp-api` crate also provides a feature `disable-logging` to disable
			// all logging and thus, remove any logging from the WASM.
			log::info!("Hello World from offchain workers!");

			sp_chainblocks::my_interface::say_hello_world("Offchain worker\0");

			// // Since off-chain workers are just part of the runtime code, they have direct access
			// // to the storage and other included pallets.
			// //
			// // We can easily import `frame_system` and retrieve a block hash of the parent block.
			// let parent_hash = <system::Pallet<T>>::block_hash(block_number - 1u32.into());
			// log::debug!("Current block: {:?} (parent hash: {:?})", block_number, parent_hash);

			// // initiate a GET request to localhost:1234
			// let request: http::Request = http::Request::get("http://localhost:1234");
			// let pending = request
			// 	.add_header("X-Auth", "hunter2")
			// 	.send()
			// 	.unwrap();

			// // wait for the response indefinitely
			// let mut response = pending.wait().unwrap();

			// // then check the headers
			// let mut headers = response.headers().into_iter();
			// assert_eq!(headers.current(), None);

			// // and collect the body
			// let body = response.body();
			// assert_eq!(body.clone().collect::<Vec<_>>(), b"1234".to_vec());
			// assert_eq!(body.error(), &None);
		}
	}
}
