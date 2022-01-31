#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod weights;

use sp_core::ecdsa;
use sp_chainblocks::{Hash256, SupportedChains, FragmentOwner, LinkedAsset};
use clamor_tools_pallet::DetachedFragments;


pub use pallet::*;
pub use weights::WeightInfo;

use codec::{Compact, Decode, Encode};
use sp_io::{
	crypto as Crypto,
	hashing::blake2_256,
	transaction_index,
};
use sp_std::{collections::btree_set::BTreeSet, vec, vec::Vec};

use frame_system::offchain::{
	CreateSignedTransaction, SignedPayload,
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

#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub struct DetachRequest {
	pub fragment_hash: Hash256,
	pub target_chain: SupportedChains,
	pub target_account: Vec<u8>, // an eth address or so
}


#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub struct IncludeInfo {
	pub fragment_hash: Hash256,
	pub mutable_index: Option<Compact<u32>>,
	pub staked_amount: Compact<u128>,
}

#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub struct AuthData {
	pub signature: ecdsa::Signature,
	pub block: u32,
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
	pub owner: FragmentOwner<TAccountId>,
	/// References to other fragments.
	/// Hash, mutable index, include cost.
	pub references: Vec<IncludeInfo>,
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
		+ clamor_tools_pallet::Config
		+ frame_system::Config
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig {
		pub upload_authorities: Vec<ecdsa::Public>,
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			Self { upload_authorities: Vec::new()}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			Pallet::<T>::initialize_upload_authorities(&self.upload_authorities);
		}
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
	pub type UploadAuthorities<T: Config> = StorageValue<_, BTreeSet<ecdsa::Public>, ValueQuery>;


	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Upload(Hash256),
		Update(Hash256),
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


		#[pallet::weight(<T as Config>::WeightInfo::add_upload_auth())]
		pub fn add_upload_auth(origin: OriginFor<T>, public: ecdsa::Public) -> DispatchResult {
			ensure_root(origin)?;

			log::debug!("New upload auth: {:?}", public);

			<UploadAuthorities<T>>::mutate(|validators| {
				validators.insert(public);
			});

			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::del_upload_auth())]
		pub fn del_upload_auth(origin: OriginFor<T>, public: ecdsa::Public) -> DispatchResult {
			ensure_root(origin)?;

			log::debug!("Removed upload auth: {:?}", public);

			<UploadAuthorities<T>>::mutate(|validators| {
				validators.remove(&public);
			});

			Ok(())
		}


		/// Fragment upload function.
		#[pallet::weight(<T as Config>::WeightInfo::upload(data.len() as u32))]
		pub fn upload(
			origin: OriginFor<T>,
			// we store this in the state as well
			references: Vec<IncludeInfo>,
			linked_asset: Option<LinkedAsset>,
			include_cost: Option<Compact<u128>>,
			auth: AuthData,
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

			<Pallet<T>>::ensure_upload_auth(&auth, &signature_hash)?;

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
				owner: if let Some(link) = linked_asset {
					FragmentOwner::ExternalAsset(link)
				} else {
					FragmentOwner::User(who.clone())
				},
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

			log::debug!("Uploaded fragment: {:?}", fragment_hash);

			Ok(())
		}

		/// Fragment upload function.
		#[pallet::weight(<T as Config>::WeightInfo::update(if let Some(data) = data { data.len() as u32} else { 50_000 }))]
		pub fn update(
			origin: OriginFor<T>,
			// fragment hash we want to update
			fragment_hash: Hash256,
			include_cost: Option<Compact<u128>>,
			auth: AuthData,
			// data we want to update last because of the way we store blocks (storage chain)
			data: Option<Vec<u8>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let nonce = <UserNonces<T>>::get(who.clone()).unwrap_or(0);

			let fragment: Fragment<T::AccountId> =
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

			<Pallet<T>>::ensure_upload_auth(&auth, &signature_hash)?;

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

			log::debug!("Updated fragment: {:?}", fragment_hash);

			Ok(())
		}

		/// Transfer fragment ownership
		#[pallet::weight(<T as Config>::WeightInfo::transfer())]
		pub fn transfer(
			origin: OriginFor<T>,
			fragment_hash: Hash256,
			new_owner: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// make sure the fragment exists
			let fragment: Fragment<T::AccountId> =
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
	}

	impl<T: Config> Pallet<T> {
		fn ensure_upload_auth(data: &AuthData, signature_hash: &[u8; 32]) -> DispatchResult {
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

			let current_block_number = <frame_system::Pallet<T>>::block_number();
			let max_delay = current_block_number + 100u32.into();
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

	}
}
