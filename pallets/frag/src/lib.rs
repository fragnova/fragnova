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

	pub struct FragAuthId;

	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for FragAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::ed25519::Signature;
		type GenericPublic = sp_core::ed25519::Public;
	}

	// implemented for mock runtime in test
	impl frame_system::offchain::AppCrypto<<Ed25519Signature as Verify>::Signer, Ed25519Signature>
		for FragAuthId
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
use sp_io::{
	crypto as Crypto, hashing::blake2_256, hashing::keccak_256, offchain, transaction_index,
};
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

use sp_clamor::{http_json_post, Hash256};

use scale_info::prelude::{format, string::String};

use serde_json::{json, Value};

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
	use sp_runtime::{
		offchain::HttpRequestStatus, traits::AccountIdConversion, MultiSignature,
		SaturatedConversion,
	};

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config + CreateSignedTransaction<Call<Self>> + pallet_assets::Config
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type WeightInfo: WeightInfo;

		#[pallet::constant]
		type FragToken: Get<<Self as pallet_assets::Config>::AssetId>;

		#[pallet::constant]
		type EthChainId: Get<u64>;

		/// The identifier type for an offchain worker.
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub type EthLockedFrag<T: Config> = StorageMap<_, Blake2_128Concat, H160, T::Balance>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
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

			// ! Notice we do this all in WASM... SLOWNESS here can be a issue
			let sig = libsecp256k1::Signature::parse_overflowing_slice(&signature.0[..64])
				.map_err(|_| Error::<T>::VerificationFailed)?;
			let ri = libsecp256k1::RecoveryId::parse(signature.0[64])
				.map_err(|_| Error::<T>::VerificationFailed)?;
			let msg = libsecp256k1::Message::parse_slice(&signature_hash[..])
				.map_err(|_| Error::<T>::VerificationFailed)?;
			let pub_key = libsecp256k1::recover(&msg, &sig, &ri)
				.map_err(|_| Error::<T>::VerificationFailed)?;
			let short_key = pub_key.serialize_compressed();

			// this is how substrate handles ecdsa publics
			let short_key = blake2_256(&short_key[..]);

			let who2 = T::AccountId::decode(&mut &short_key[..])
				.map_err(|_| Error::<T>::VerificationFailed)?;

			ensure!(who == who2, Error::<T>::VerificationFailed);

			// ! Check passed, derive eth key and send to offchain worker

			let eth_key = pub_key.serialize();
			let eth_key = &eth_key[1..];
			let eth_key = keccak_256(&eth_key[..]);
			let eth_key = &eth_key[12..];

			// TODO

			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(n: T::BlockNumber) {
			Self::sync_frag_stakes(n);
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn sync_frag_stakes(_block_number: T::BlockNumber) {
			let geth_uri = if let Some(geth) = sp_clamor::clamor::get_geth_url() {
				String::from_utf8(geth).unwrap()
			} else {
				log::debug!("No geth url found, skipping sync");
				return;
			};

			let last_id_ref = StorageValueRef::persistent(b"frag_sync_last_block");
			let last_id: Option<Vec<u8>> = last_id_ref.get().unwrap_or_default();
			let last_id = if let Some(last_id) = last_id {
				String::from_utf8(last_id).unwrap()
			} else {
				String::from("")
			};

			let req = json!({
				"jsonrpc": "2.0",
				"method": "eth_getLogs",
				"id": "0",
				"params": [{
					"fromBlock": "0x0",
					"toBlock": "0x1",
					"address": "0x0000000000000000000000000000000000000001",
					"topics": [
						"0x0000000000000000000000000000000000000000000000000000000000000001",
						"0x0000000000000000000000000000000000000000000000000000000000000002",
						"0x0000000000000000000000000000000000000000000000000000000000000003",
					],
				}]
			});

			let req = serde_json::to_string(&req).unwrap();
			let response_body = http_json_post(geth_uri.as_str(), req.as_bytes());
			let response_body = if let Ok(response) = response_body {
				response
			} else {
				log::error!("failed to get response from the graph");
				return;
			};

			let response = String::from_utf8(response_body).unwrap();
			log::trace!("response: {}", response);

			let v: Value = serde_json::from_str(&response).unwrap();

			// let records = v["data"]["lockEntities"].as_array().unwrap();
			// for (i, record) in records.iter().enumerate() {
			// 	let lock = record["lock"].as_bool().unwrap();
			// 	if lock {
			// 		let id = record["id"].as_str().unwrap();
			// 		log::trace!("Recording lock stake for {}", id);

			// 		let account = &record["owner"].as_str().unwrap()[2..];
			// 		let account = <[u8; 20]>::from_hex(account).unwrap();

			// 		let amount = record["amount"].as_str().unwrap();
			// 		let amount = amount.parse::<u128>().unwrap();

			// 		if let Err(e) = Signer::<T, T::AuthorityId>::any_account()
			// 			.send_unsigned_transaction(
			// 				|pub_key| EthStakeUpdate {
			// 					public: pub_key.public.clone(),
			// 					amount: amount.saturated_into(),
			// 					account: account.into(),
			// 				},
			// 				|payload, signature| Call::internal_update_stake {
			// 					data: payload,
			// 					signature,
			// 				},
			// 			)
			// 			.ok_or("No local accounts accounts available.")
			// 		{
			// 			log::error!(
			// 				"Failed to send unsigned eth sync transaction with error: {:?}",
			// 				e
			// 			);
			// 			return;
			// 		}

			// 		// update the last recorded event
			// 		if i == records.len() - 1 {
			// 			last_id_ref.set(&id.as_bytes().to_vec());
			// 		}
			// 	}
			// }
		}
	}
}
