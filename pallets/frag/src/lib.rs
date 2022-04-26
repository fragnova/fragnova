#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod weights;

const LOCK_EVENT: &str = "0x83a932dce34e6748d366fededbe6d22c5c1272c439426f8620148e8215160b3f";
const UNLOCK_EVENT: &str = "0xf9480f9ead9b82690f56cdb4730f12763ca2f50ce1792a255141b71789dca7fe";
const CONFIRMATIONS_NUMBER: u64 = 15;

use sp_core::{crypto::KeyTypeId, ecdsa, ed25519, H160, H256, U256};

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
use sp_runtime::MultiSigner;
use sp_std::{collections::btree_set::BTreeSet, vec, vec::Vec};

use frame_system::offchain::{
	AppCrypto, CreateSignedTransaction, SendUnsignedTransaction, SignedPayload, Signer,
	SigningTypes,
};

pub use weights::WeightInfo;

use sp_clamor::http_json_post;

use scale_info::prelude::{format, string::String};

use serde_json::{json, Value};

use ethabi::ParamType;

#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq, Eq)]
pub struct EthLockUpdate<TPublic> {
	pub public: TPublic,
	pub amount: U256,
	pub sender: H160,
	pub signature: ecdsa::Signature,
	pub lock: bool,
	pub block_number: u64,
}

#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq, Eq)]
pub struct EthLock<TBalance, TBlockNum> {
	pub amount: TBalance,
	pub block_number: TBlockNum,
}

impl<T: SigningTypes> SignedPayload<T> for EthLockUpdate<T::Public> {
	fn public(&self) -> T::Public {
		self.public.clone()
	}
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, Blake2_128Concat};
	use frame_system::pallet_prelude::*;
	use sp_runtime::SaturatedConversion;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config + CreateSignedTransaction<Call<Self>> + pallet_balances::Config
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type WeightInfo: WeightInfo;

		#[pallet::constant]
		type EthChainId: Get<u64>;

		#[pallet::constant]
		type Threshold: Get<u64>;

		/// The identifier type for an offchain worker.
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
	}

	#[pallet::genesis_config]
	#[derive(Default)]
	pub struct GenesisConfig {
		pub keys: Vec<ed25519::Public>,
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			Pallet::<T>::initialize_keys(&self.keys);
		}
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub type EthLockedFrag<T: Config> =
		StorageMap<_, Identity, H160, EthLock<T::Balance, T::BlockNumber>>;

	#[pallet::storage]
	pub type EVMLinks<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, H160>;

	#[pallet::storage]
	pub type EVMLinksReverse<T: Config> = StorageMap<_, Identity, H160, T::AccountId>;

	#[pallet::storage]
	pub type FragUsage<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, T::Balance>;

	#[pallet::storage]
	pub type EVMLinkVoting<T: Config> = StorageMap<_, Identity, H256, u64>;

	#[pallet::storage]
	pub type EVMLinkVotingClosed<T: Config> = StorageMap<_, Identity, H256, T::BlockNumber>;
	// consumed by Protos pallet
	#[pallet::storage]
	pub type PendingUnlinks<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	/// These are the public keys representing the actual keys that can Sign messages
	/// to present to external chains to detach onto
	#[pallet::storage]
	pub type FragKeys<T: Config> = StorageValue<_, BTreeSet<ed25519::Public>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A link happened between native and ethereum account.
		Linked(T::AccountId, H160),
		/// A link was removed between native and ethereum account.
		Unlinked(T::AccountId, H160),
		/// ETH side lock was updated
		Locked(H160, T::Balance),
		/// ETH side lock was unlocked
		Unlocked(H160, T::Balance),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Systematic failure - those errors should not happen.
		SystematicFailure,
		/// Signature verification failed
		VerificationFailed,
		/// Link already processed
		LinkAlreadyProcessed,
		/// Reference not found
		LinkNotFound,
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
		#[pallet::weight(25_000)] // TODO #1 - weight
		pub fn add_key(origin: OriginFor<T>, public: ed25519::Public) -> DispatchResult {
			ensure_root(origin)?;

			log::debug!("New key: {:?}", public);

			<FragKeys<T>>::mutate(|validators| {
				validators.insert(public);
			});

			Ok(())
		}

		#[pallet::weight(25_000)] // TODO #1 - weight
		pub fn del_key(origin: OriginFor<T>, public: ed25519::Public) -> DispatchResult {
			ensure_root(origin)?;

			log::debug!("Removed key: {:?}", public);

			<FragKeys<T>>::mutate(|validators| {
				validators.remove(&public);
			});

			Ok(())
		}

		/// TODO
		#[pallet::weight(25_000)] // TODO #1 - weight
		pub fn link(origin: OriginFor<T>, signature: ecdsa::Signature) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// the idea is to prove to this chain that the sender knows the private key of the external address
			let mut message = b"EVM2Fragnova".to_vec();
			message.extend_from_slice(&T::EthChainId::get().to_be_bytes());
			message.extend_from_slice(&sender.encode());
			let message_hash = keccak_256(&message);

			let recovered = Crypto::secp256k1_ecdsa_recover(&signature.0, &message_hash)
				.map_err(|_| Error::<T>::VerificationFailed)?;

			let eth_key = keccak_256(&recovered[..]);
			let eth_key = &eth_key[12..];
			let eth_key = H160::from_slice(&eth_key[..]);

			ensure!(!<EVMLinks<T>>::contains_key(&sender), Error::<T>::AccountAlreadyLinked);
			ensure!(!<EVMLinksReverse<T>>::contains_key(eth_key), Error::<T>::AccountAlreadyLinked);

			<EVMLinks<T>>::insert(sender.clone(), eth_key);
			<EVMLinksReverse<T>>::insert(eth_key, sender.clone());
			let zero: T::Balance = 0u32.saturated_into();
			<FragUsage<T>>::insert(sender.clone(), zero);

			// also emit event
			Self::deposit_event(Event::Linked(sender, eth_key));

			Ok(())
		}

		/// TODO
		#[pallet::weight(25_000)] // TODO #1 - weight
		pub fn unlink(origin: OriginFor<T>, account: H160) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::unlink_account(sender, account)
		}

		/// TODO
		#[pallet::weight(25_000)] // TODO #1 - weight
		pub fn internal_lock_update(
			origin: OriginFor<T>,
			data: EthLockUpdate<T::Public>,
			_signature: T::Signature,
		) -> DispatchResult {
			ensure_none(origin)?;

			let data_tuple =
				(data.amount, data.sender, data.signature.clone(), data.lock, data.block_number);
			let data_hash: H256 = data_tuple.using_encoded(blake2_256).into();

			ensure!(
				!<EVMLinkVotingClosed<T>>::contains_key(data_hash),
				Error::<T>::LinkAlreadyProcessed
			);

			let mut message = if data.lock { b"FragLock".to_vec() } else { b"FragUnlock".to_vec() };
			message.extend_from_slice(&data.sender.0[..]);
			message.extend_from_slice(&T::EthChainId::get().to_be_bytes());
			let amount: [u8; 32] = data.amount.into();
			message.extend_from_slice(&amount[..]);
			let message_hash = keccak_256(&message);

			let message = [b"\x19Ethereum Signed Message:\n32", &message_hash[..]].concat();
			let message_hash = keccak_256(&message);

			let signature = data.signature;
			let sender = data.sender;

			let pub_key = Crypto::secp256k1_ecdsa_recover(&signature.0, &message_hash)
				.map_err(|_| Error::<T>::VerificationFailed)?;
			let pub_key = keccak_256(&pub_key[..]);
			let pub_key = &pub_key[12..];
			ensure!(pub_key == sender.0, Error::<T>::VerificationFailed);

			let amount: u128 = data.amount.try_into().map_err(|_| Error::<T>::SystematicFailure)?;

			if !data.lock {
				ensure!(amount == 0, Error::<T>::SystematicFailure);
			} else {
				ensure!(amount > 0, Error::<T>::SystematicFailure);
			}

			// verifications ended, let's proceed with voting count and writing

			let threshold = T::Threshold::get();
			if threshold > 1 {
				let current_votes = <EVMLinkVoting<T>>::get(&data_hash);
				if let Some(current_votes) = current_votes {
					if current_votes + 1u64 < threshold {
						<EVMLinkVoting<T>>::insert(&data_hash, current_votes + 1);
						return Ok(());
					} else {
						// we are good to go, but let's remove the record
						<EVMLinkVoting<T>>::remove(&data_hash);
					}
				} else {
					<EVMLinkVoting<T>>::insert(&data_hash, 1);
					return Ok(());
				}
			}

			// writing here at end

			let amount: T::Balance = amount.saturated_into();
			let current_block_number = <frame_system::Pallet<T>>::block_number();

			if data.lock {
				// ! TODO TEST
				let linked = <EVMLinksReverse<T>>::get(sender.clone());
				if let Some(linked) = linked {
					let used = <FragUsage<T>>::get(linked.clone());
					if let Some(used) = used {
						if used > amount {
							// this is bad, the user just reduced his balance so we just "slash" all the stakes
							// reset usage counter
							<FragUsage<T>>::remove(linked.clone());
							// force dereferencing of protos and more
							<PendingUnlinks<T>>::append(linked.clone());
						}
					}
				}

				// also emit event
				Self::deposit_event(Event::Locked(sender, amount));
			} else {
				// ! TODO TEST

				// if we have any link to this account, then force unlinking
				let linked = <EVMLinksReverse<T>>::get(sender.clone());
				if let Some(linked) = linked {
					Self::unlink_account(linked, sender.clone())?;
				}

				// also emit event
				Self::deposit_event(Event::Unlocked(sender, amount));
			}

			// write this later as unlink_account can fail
			<EthLockedFrag<T>>::insert(
				sender.clone(),
				EthLock { amount, block_number: current_block_number },
			);

			// also record link hash
			<EVMLinkVotingClosed<T>>::insert(data_hash, current_block_number);

			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(n: T::BlockNumber) {
			if let Err(error) = Self::sync_frag_locks(n) {
				log::error!("Error syncing frag locks: {:?}", error);
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
		fn validate_unsigned(source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			// Firstly let's check that we call the right function.
			if let Call::internal_lock_update { ref data, ref signature } = call {
				// ensure it's a local transaction sent by an offchain worker
				if source != TransactionSource::Local {
					return InvalidTransaction::Call.into();
				}

				// check public is valid
				let valid_keys = <FragKeys<T>>::get();
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
					return InvalidTransaction::BadSigner.into();
				}

				// most expensive bit last
				let signature_valid =
					SignedPayload::<T>::verify::<T::AuthorityId>(data, signature.clone());
				if !signature_valid {
					return InvalidTransaction::BadProof.into();
				}

				log::debug!("Sending frag lock update extrinsic");
				ValidTransaction::with_tag_prefix("FragLockUpdate")
					.and_provides(data.public.clone())
					.and_provides(data.amount)
					.and_provides(data.sender)
					.and_provides(data.signature.clone())
					.and_provides(data.lock)
					.and_provides(data.block_number)
					.and_provides(pub_key)
					.longevity(5)
					.propagate(true)
					.build()
			} else {
				InvalidTransaction::Call.into()
			}
		}
	}

	impl<T: Config> Pallet<T> {
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

		fn sync_frag_locks(_block_number: T::BlockNumber) -> Result<(), &'static str> {
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

			let req = json!({
				"jsonrpc": "2.0",
				"method": "eth_blockNumber",
				"id": 1
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

			let current_block = v["result"].as_str().ok_or("Invalid response - no result")?;
			let current_block = u64::from_str_radix(&current_block[2..], 16)
				.map_err(|_| "Invalid response - invalid block number")?;
			log::trace!("Current block: {}", current_block);

			let last_block_ref = StorageValueRef::persistent(b"frag_sync_last_block");
			let last_block: Option<Vec<u8>> = last_block_ref.get().unwrap_or_default();
			let last_block = if let Some(last_block) = last_block {
				String::from_utf8(last_block).map_err(|_| "Invalid last block")?
			} else {
				String::from("0x0")
			};

			let to_block = current_block.saturating_sub(CONFIRMATIONS_NUMBER);
			let to_block = format!("0x{:x}", to_block);

			let req = json!({
				"jsonrpc": "2.0",
				"method": "eth_getLogs",
				"id": "0",
				"params": [{
					"fromBlock": last_block,
					"toBlock": to_block,
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
				let block_number =
					log["blockNumber"].as_str().ok_or("Invalid response - no block number")?;
				let block_number = u64::from_str_radix(&block_number[2..], 16)
					.map_err(|_| "Invalid response - invalid block number")?;
				let topics =
					log["topics"].as_array().ok_or_else(|| "Invalid response - no topics")?;
				let topic = topics[0].as_str().ok_or_else(|| "Invalid response - no topic")?;
				let data = log["data"].as_str().ok_or_else(|| "Invalid response - no data")?;
				let data =
					hex::decode(&data[2..]).map_err(|_| "Invalid response - invalid data")?;
				let data = ethabi::decode(&[ParamType::Bytes, ParamType::Uint(256)], &data)
					.map_err(|_| "Invalid response - invalid eth data")?;
				let locked = match topic {
					LOCK_EVENT => true,
					UNLOCK_EVENT => false,
					_ => return Err("Invalid topic"),
				};

				let sender = topics[1].as_str().ok_or_else(|| "Invalid response - no sender")?;
				let sender =
					hex::decode(&sender[2..]).map_err(|_| "Invalid response - invalid sender")?;
				let sender = H160::from_slice(&sender[12..]);

				let eth_signature = data[0].clone().into_bytes().ok_or_else(|| "Invalid data")?;
				let eth_signature: ecdsa::Signature =
					(&eth_signature[..]).try_into().map_err(|_| "Invalid data")?;

				let amount = data[1].clone().into_uint().ok_or_else(|| "Invalid data")?;

				log::trace!(
					"Block: {}, sender: {}, locked: {}, amount: {}, signature: {:?}",
					block_number,
					sender,
					locked,
					amount,
					eth_signature.clone(),
				);

				Signer::<T, T::AuthorityId>::any_account()
					.send_unsigned_transaction(
						|account| EthLockUpdate {
							public: account.public.clone(),
							amount,
							sender,
							signature: eth_signature.clone(),
							lock: locked,
							block_number,
						},
						|payload, signature| Call::internal_lock_update {
							data: payload,
							signature,
						},
					)
					.ok_or_else(|| "Failed to sign transaction")?
					.1
					.map_err(|_| "Failed to send transaction")?;
			}

			last_block_ref.set(&to_block.as_bytes().to_vec());

			Ok(())
		}

		fn unlink_account(sender: T::AccountId, account: H160) -> DispatchResult {
			ensure!(<EVMLinks<T>>::contains_key(sender.clone()), Error::<T>::AccountNotLinked);
			ensure!(<EVMLinksReverse<T>>::contains_key(account), Error::<T>::AccountNotLinked);

			<EVMLinks<T>>::remove(sender.clone());
			<EVMLinksReverse<T>>::remove(account);
			// reset usage counter
			<FragUsage<T>>::remove(sender.clone());
			// force dereferencing of protos and more
			<PendingUnlinks<T>>::append(sender.clone());

			// also emit event
			Self::deposit_event(Event::Unlinked(sender, account));

			Ok(())
		}
	}
}
