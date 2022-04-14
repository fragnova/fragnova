#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod weights;

const LOCK_EVENT: &str = "0xeb49373c30c7ae230c318e69e8e8632f3831fc92d4a27cee08a8c91dd41ef03a";
const UNLOCK_EVENT: &str = "0x16a32b1d5be5f34a614fa537e89a714d2db2ea522ef95c42ea2ae79a7f3b5a85";

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

use ethabi::{ParamType, Token};

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

#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq, Eq)]
pub struct Unlinked<TAccount> {
	pub account: TAccount,
	pub external_account: H160,
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
	pub type EthLockedFrag<T: Config> = StorageMap<_, Blake2_128Concat, H160, (T::Balance, bool)>;

	#[pallet::storage]
	pub type EVMLinks<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BTreeSet<H160>>;

	// consumed by Protos pallet
	#[pallet::storage]
	pub type PendingUnlinks<T: Config> = StorageValue<_, Vec<Unlinked<T::AccountId>>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A link happened between native and ethereum account.
		Linked(T::AccountId, H160),
		/// A link was removed between native and ethereum account.
		Unlinked(T::AccountId, H160),
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
		/// Reference not found
		LinkNotFound,
		/// Not enough tokens to stake
		InsufficientBalance,
		/// Cannot unstake yet
		StakeLocked,
		/// Account already linked
		AccountAlreadyLinked,
		/// Account not linked
		AccountNotLinked,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// TODO
		#[pallet::weight(25_000)] // TODO #1 - weight
		pub fn link(origin: OriginFor<T>, signature: ecdsa::Signature) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let mut message = b"EVM2Fragnova".to_vec();
			message.extend_from_slice(&T::EthChainId::get().to_be_bytes());
			message.extend_from_slice(&sender.encode());
			let message_hash = keccak_256(&message);

			let recovered = Crypto::secp256k1_ecdsa_recover(&signature.0, &message_hash)
				.map_err(|_| Error::<T>::VerificationFailed)?;

			let eth_key = &recovered[1..];
			let eth_key = keccak_256(&eth_key[..]);
			let eth_key = &eth_key[12..];
			let eth_key = H160::from_slice(&eth_key[..]);

			// find the locked frag account
			let locked_frag =
				<EthLockedFrag<T>>::get(&eth_key).ok_or_else(|| Error::<T>::LinkNotFound)?;
			// ensure the account is not linked yet
			ensure!(!locked_frag.1, Error::<T>::AccountAlreadyLinked);

			<EVMLinks<T>>::mutate(sender.clone(), |links| match links {
				Some(links) => {
					links.insert(eth_key);
				},
				_ => {
					<EVMLinks<T>>::insert(sender.clone(), BTreeSet::from_iter(vec![eth_key]));
				},
			});

			<EthLockedFrag<T>>::insert(eth_key, (locked_frag.0, true));

			// also emit event
			Self::deposit_event(Event::Linked(sender, eth_key));

			Ok(())
		}

		/// TODO
		#[pallet::weight(25_000)] // TODO #1 - weight
		pub fn unlink(origin: OriginFor<T>, account: H160) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// find the locked frag account
			let locked_frag =
				<EthLockedFrag<T>>::get(&account).ok_or_else(|| Error::<T>::LinkNotFound)?;
			// ensure the account is not linked yet
			ensure!(locked_frag.1, Error::<T>::AccountNotLinked);

			<EVMLinks<T>>::mutate(sender.clone(), |links| {
				if let Some(links) = links {
					if links.remove(&account) {
						let unlinked =
							Unlinked { account: sender.clone(), external_account: account };
						<PendingUnlinks<T>>::append(&unlinked);
						Ok(())
					} else {
						Err(Error::<T>::LinkNotFound)
					}
				} else {
					Err(Error::<T>::LinkNotFound)
				}
			})?;

			<EthLockedFrag<T>>::insert(account, (locked_frag.0, false));

			// also emit event
			Self::deposit_event(Event::Unlinked(sender, account));

			Ok(())
		}

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
			let message_hash = keccak_256(message);

			let recovered = Crypto::secp256k1_ecdsa_recover_compressed(&signature.0, &message_hash)
				.map_err(|_| Error::<T>::VerificationFailed)?;

			// this is how substrate handles ecdsa publics
			let short_key = blake2_256(&recovered[..]);

			let who2 = T::AccountId::decode(&mut &short_key[..])
				.map_err(|_| Error::<T>::VerificationFailed)?;

			ensure!(who == who2, Error::<T>::VerificationFailed);

			// ! Check passed, derive eth key and send to offchain worker

			// let eth_key = pub_key.serialize();
			// let eth_key = &eth_key[1..];
			// let eth_key = keccak_256(&eth_key[..]);
			// let eth_key = &eth_key[12..];

			// TODO

			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(n: T::BlockNumber) {
			if let Err(error) = Self::sync_frag_stakes(n) {
				log::error!("Error syncing frag stakes: {:?}", error);
			}
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn sync_frag_stakes(_block_number: T::BlockNumber) -> Result<(), &'static str> {
			let geth_uri = if let Some(geth) = sp_clamor::clamor::get_geth_url() {
				String::from_utf8(geth).unwrap()
			} else {
				log::debug!("No geth url found, skipping sync");
				return Ok(()); // It is fine to have a node not syncing with eth
			};

			let contract = if let Some(contract) = sp_clamor::clamor::get_eth_contract() {
				String::from_utf8(contract).unwrap()
			} else {
				log::debug!("No contract address found, skipping sync");
				return Ok(()); // It is fine to have a node not syncing with eth
			};

			let last_block_ref = StorageValueRef::persistent(b"frag_sync_last_block");
			let last_block: Option<Vec<u8>> = last_block_ref.get().unwrap_or_default();
			let last_block = if let Some(last_block) = last_block {
				String::from_utf8(last_block).map_err(|_| "Invalid last block")?
			} else {
				String::from("0x0")
			};

			let req = json!({
				"jsonrpc": "2.0",
				"method": "eth_getLogs",
				"id": "0",
				"params": [{
					"fromBlock": last_block,
					"address": contract,
					"topics": [
						// [] to OR
						[LOCK_EVENT, UNLOCK_EVENT]
					],
				}]
			});

			let req = serde_json::to_string(&req).map_err(|_| "Invalid request")?;
			log::trace!("Request: {}", req);

			let response_body = http_json_post(geth_uri.as_str(), req.as_bytes());
			let response_body = if let Ok(response) = response_body {
				response
			} else {
				return Err("Failed to get response from geth");
			};

			let response = String::from_utf8(response_body).map_err(|_| "Invalid response")?;
			log::trace!("Response: {}", response);

			let v: Value =
				serde_json::from_str(&response).map_err(|_| "Invalid response - json parse")?;

			let logs = v["result"].as_array().ok_or_else(|| "Invalid response - no result")?;
			for log in logs {
				let topics =
					log["topics"].as_array().ok_or_else(|| "Invalid response - no topics")?;
				let topic = topics[0].as_str().ok_or_else(|| "Invalid response - no topic")?;
				let data = log["data"].as_str().ok_or_else(|| "Invalid response - no data")?;
				let data = hex::decode(&data[2..]).map_err(|_| "Invalid response - invalid data")?;
				let data = ethabi::decode(&[ParamType::Bytes, ParamType::Uint(256)], &data)
					.map_err(|_| "Invalid response - invalid eth data")?;
				let locked = match topic {
					LOCK_EVENT => true,
					UNLOCK_EVENT => false,
					_ => return Err("Invalid topic"),
				};

				log::trace!("Log: {:?}, lock: {:?}", data, locked);
			}

			Ok(())

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
			// 			last_block_ref.set(&id.as_bytes().to_vec());
			// 		}
			// 	}
			// }
		}
	}
}
