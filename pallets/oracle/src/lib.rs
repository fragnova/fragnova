//! pallet-oracle queries the ChainLink Price Feed smart contract on Ethereum to fetch the price of FRAG/USD.
//!
//! It works as an Offchain Worker that will be triggered after every block, to fetch the current price.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::traits::Get;
use frame_system::offchain::{
	AppCrypto, CreateSignedTransaction, SendUnsignedTransaction, SignedPayload, Signer,
	SigningTypes,
};
use serde_json::{json, Value};
use sp_clamor::http_json_post;
use sp_core::crypto::KeyTypeId;
use sp_runtime::transaction_validity::{InvalidTransaction, TransactionValidity, ValidTransaction};

#[cfg(test)]
pub mod tests;
/// Defines application identifier for crypto keys of this module.
///
/// Every module that deals with signatures needs to declare its unique identifier for
/// its crypto keys.
/// When offchain worker is signing transactions it's going to request keys of type
/// `KeyTypeId` from the keystore and use the ones it finds to sign the transaction.
/// The keys can be inserted manually via RPC (see `author_insertKey`).
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"orac");

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

	// The app_crypto macro declares an account with an ed25519 signature that is identified by KEY_TYPE.
	// Note that this doesn't create a new account. The macro simply declares that a crypto account
	// is available for this pallet. You will need to initialize this account in the next step.
	//
	// https://docs.substrate.io/how-to-guides/v3/ocw/transactions/
	app_crypto!(ed25519, KEY_TYPE);

	/// The identifier type for an offchain worker.
	pub struct FragAuthId;

	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for FragAuthId {
		type RuntimeAppPublic = Public;
		type GenericPublic = sp_core::ed25519::Public;
		type GenericSignature = sp_core::ed25519::Signature;
	}

	// implemented for mock runtime in test
	impl frame_system::offchain::AppCrypto<<Ed25519Signature as Verify>::Signer, Ed25519Signature>
		for FragAuthId
	{
		type RuntimeAppPublic = Public;
		type GenericPublic = sp_core::ed25519::Public;
		type GenericSignature = sp_core::ed25519::Signature;
	}
}

/// **Traits** of the **ETH / USD Chainlink contract** on the **Ethereum (Goerli) network**
pub trait OracleContract {
	/// **Return** the **contract address** of the **ETH / USD Chainlink contract**
	fn get_contract() -> &'static str {
		"0xABCD"
	}
}

impl OracleContract for () {}

pub use pallet::*;

use scale_info::prelude::string::String;
use sp_io::hashing::blake2_256;
use sp_std::{collections::btree_set::BTreeSet, vec, vec::Vec};

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use ethabi::ethereum_types::H256;
	use ethabi::ParamType;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_core::{ed25519, U256};
	use sp_core::offchain::Timestamp;
	use sp_runtime::MultiSigner;

	const FETCH_TIMEOUT_PERIOD: u64 = 3000; // in milli-seconds

	/// This pallet's configuration trait
	#[pallet::config]
	pub trait Config: CreateSignedTransaction<Call<Self>> + frame_system::Config {
		/// The identifier type for an offchain worker.
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;

		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// **Traits** of the **ETH / USD Chainlink contract** on the **Ethereum (Goerli) network*
		type OracleContract: OracleContract;

		/// Number of votes needed to do something (问Gio)
		#[pallet::constant]
		type Threshold: Get<u64>;
	}

	/// The Genesis Configuration for the Pallet.
	#[pallet::genesis_config]
	#[derive(Default)]
	pub struct GenesisConfig {
		/// **List of Clamor Account IDs** that can ***validate*** and ***send*** **unsigned transactions with signed payload**
		pub keys: Vec<ed25519::Public>,
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			Pallet::<T>::initialize_keys(&self.keys);
		}
	}

	impl<T: SigningTypes> SignedPayload<T> for OraclePrice<T::Public, T::BlockNumber> {
		fn public(&self) -> T::Public {
			self.public.clone()
		}
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Struct used to hold price data received from the Chainlink Price Feed smart contract.
	/// Please refer to https://docs.chain.link/docs/data-feeds/price-feeds/api-reference/#latestrounddata.
	#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, scale_info::TypeInfo)]
	pub struct OraclePrice<TPublic, TBlockNumber> {
		/// The round ID
		pub round_id: U256,
		/// The price
		pub price: U256,
		/// Timestamp of when the round started
		pub started_at: U256,
		/// Timestamp of when the round was updated
		pub updated_at: U256,
		/// The round ID of the round in which the answer was computed
		pub answered_in_round: U256,
		/// The block number on Clamor when this price was fetched from the oracle
		pub block_number: TBlockNumber,
		/// Clamor Public Account Address (the account address should be in FragKey, otherwise it fails)
		pub public: TPublic,
	}

	/// Storage use for the latest price received from the oracle.
	#[pallet::storage]
	#[pallet::getter(fn prices)]
	pub(super) type Price<T: Config> = StorageValue<_, u32, ValueQuery>;

	/// **StorageMap** that maps **a FRAG token locking or unlocking event** to a **number of votes ()**.
	/// The key for this map is:
	/// `blake2_256(encoded(<Amount of FRAG token that was locked/unlocked, Signature written by the owner of the FRAG token on a determinstic message,
	/// 					Whether it was locked or unlocked, Ethereum Block Number where it was locked/unlocked>))`
	#[pallet::storage]
	pub type EVMLinkVoting<T: Config> = StorageMap<_, Identity, H256, u64>;

	/// **StorageMap** that maps **a FRAG token locking or unlocking event** to a boolean indicating whether voting on the aforementioned event has ended**.
	#[pallet::storage]
	pub type EVMLinkVotingClosed<T: Config> = StorageMap<_, Identity, H256, T::BlockNumber>;

	/// **StorageValue** that equals the **List of Clamor Account IDs** that both ***validate*** and ***send*** **unsigned transactions with signed payload**
	///
	/// NOTE: Only the Root User of the Clamor Blockchain (i.e the local node itself) can edit this list
	#[pallet::storage]
	pub type FragKeys<T: Config> = StorageValue<_, BTreeSet<ed25519::Public>, ValueQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(block_number: T::BlockNumber) {
			Self::fetch_price_from_oracle(block_number);
		}
	}

	/// A public part of the pallet.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Add `public` to the **list of Clamor Account IDs** that can ***validate*** and ***send*** **unsigned transactions with signed payload**
		///
		/// NOTE: Only the Root User of the Clamor Blockchain (i.e the local node itself) can edit this list
		#[pallet::weight(10000)]
		pub fn add_key(origin: OriginFor<T>, public: ed25519::Public) -> DispatchResult {
			ensure_root(origin)?;

			log::debug!("New key: {:?}", public);

			<FragKeys<T>>::mutate(|validators| {
				validators.insert(public);
			});

			Ok(())
		}

		/// Remove a Clamor Account ID from `FragKeys`

		/// NOTE: Only the Root User of the Clamor Blockchain (i.e the local node itself) can call this function
		#[pallet::weight(10000)] // TODO
		pub fn del_key(origin: OriginFor<T>, public: ed25519::Public) -> DispatchResult {
			ensure_root(origin)?;

			log::debug!("Removed key: {:?}", public);

			<FragKeys<T>>::mutate(|validators| {
				validators.remove(&public);
			});

			Ok(())
		}

		/// This function stores the new price received from the Oracle into a structure that contains a list of the last N-prices.
		#[pallet::weight(10000)] // TODO
		pub fn store_price(
			origin: OriginFor<T>,
			oracle_price: OraclePrice<T::Public, T::BlockNumber>,
			_signature: T::Signature,
		) -> DispatchResult {
			// This ensures that the function can only be called via unsigned transaction.
			ensure_none(origin)?;

			log::debug!("Store price: {:?}", oracle_price);

			let data_hash: H256 = oracle_price.using_encoded(blake2_256).into();

			let latest_price: u32 =
				oracle_price.price.try_into().map_err(|_| Error::<T>::SystematicFailure)?;
			let block_number = oracle_price.block_number;

			// voting
			let threshold = T::Threshold::get();
			if threshold > 1 {
				// 问Gio
				let current_votes = <EVMLinkVoting<T>>::get(&data_hash);
				if let Some(current_votes) = current_votes {
					// Number of votes for the key `data_hash` in EVMLinkVoting
					if current_votes + 1u64 < threshold {
						// Current Votes has not passed the threshold
						<EVMLinkVoting<T>>::insert(&data_hash, current_votes + 1);
						return Ok(());
					} else {
						// Current votes passes the threshold, let's remove the record
						<EVMLinkVoting<T>>::remove(&data_hash);
					}
				} else {
					// If key `data_hash` doesn't exist in EVMLinkVoting
					<EVMLinkVoting<T>>::insert(&data_hash, 1);
					return Ok(());
				}
			}

			log::trace!("Store new price: {}", latest_price);

			<Price<T>>::put(latest_price);

			Self::deposit_event(Event::NewPrice { price: latest_price, block_number });
			Ok(().into())
		}
	}

	/// Events for the pallet.
	#[allow(missing_docs)]
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event generated when new price is accepted.
		NewPrice { price: u32, block_number: T::BlockNumber },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Error with connection with Geth client.
		GethConnectionError,
		/// Systematic failure - those errors should not happen.
		SystematicFailure,
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

		/// A helper function to fetch the price, sign payload and send an unsigned transaction
		pub fn fetch_price_from_oracle(block_number: T::BlockNumber) {
			let geth_uri = if let Some(geth) = sp_clamor::clamor::get_geth_url() {
				String::from_utf8(geth).unwrap()
			} else {
				log::debug!("Geth URL not set, skipping fetch price from oracle.");
				return;
			};

			let oracle_address = if let Some(address) = sp_clamor::clamor::get_oracle_address() {
				String::from_utf8(address).unwrap()
			} else {
				log::debug!("Oracle address not set, skipping fetch price from oracle.");
				return; // It is fine to have a node not syncing with eth
			};

			if let Err(e) = Self::fetch_price(block_number, &oracle_address, &geth_uri) {
				log::error!("Failed to fetch price from oracle with error: {}", e);
			}
		}

		fn fetch_price(
			block_number: T::BlockNumber,
			contract: &str,
			geth_uri: &str,
		) -> Result<(), &'static str> {
			let req = json!({
				"jsonrpc": "2.0",
				"method": "eth_call", // https://ethereum.org/en/developers/docs/apis/json-rpc/#eth_call
				"params": [
					{
					"to": contract,
						// call `latestRoundData()` function from ChainLink Feed Price contract
					// `data` is first 4 bytes of `keccak_256(latestRoundData())`, padded - Use https://emn178.github.io/online-tools/keccak_256.html
					"data": "0xfeaf968c0000000000000000000000000000000000000000000000000000000000000000",
					},
					"latest"
				],
				 "id": 5, //goerli
			});

			let req = serde_json::to_string(&req).map_err(|_| "Invalid request")?;
			log::trace!("Request: {}", req);

			let wait= Timestamp::from_unix_millis(FETCH_TIMEOUT_PERIOD);
			let response_body = http_json_post(&geth_uri, req.as_bytes(), Some(wait));
			let response_body = if let Ok(response) = response_body {
				response
			} else {
				return Err("Failed to get response from Ethereum.");
			};

			let response = String::from_utf8(response_body).map_err(|_| "Invalid response")?;
			log::trace!("Response: {}", response);

			let v: Value =
				serde_json::from_str(&response).map_err(|_| "Invalid response - json parse")?;
			let result = v["result"].as_str().ok_or("Invalid response - no result")?; // Get the latest block number of the Ethereum Blockchain
			let data = hex::decode(&result[2..]).map_err(|_| "Invalid response - invalid data")?;
			log::trace!("data: {:?}", data);
			let data = ethabi::decode(
				//https://docs.chain.link/docs/data-feeds/price-feeds/api-reference/#latestrounddata
				&[ParamType::Tuple(vec![
					ParamType::Uint(80),  // uint80 roundId
					ParamType::Int(256),  // int256 answer
					ParamType::Uint(256), // uint256 startedAt
					ParamType::Uint(256), // uint256 updatedAt
					ParamType::Uint(80),  // uint80 answeredInRound
				])],
				&data,
			)
			.map_err(|_| "Invalid response")?;
			let tuple = data[0].clone().into_tuple().ok_or_else(|| "Invalid data")?;
			let round_id = tuple[0].clone().into_uint().ok_or_else(|| "Invalid roundId")?;
			let price = tuple[1].clone().into_int().ok_or_else(|| "Invalid token")?;
			let started_at = tuple[2].clone().into_uint().ok_or_else(|| "Invalid startedAt")?;
			let updated_at = tuple[3].clone().into_uint().ok_or_else(|| "Invalid updatedAt")?;
			let answered_in_round =
				tuple[4].clone().into_uint().ok_or_else(|| "Invalid answeredInRound")?;

			ensure!(!price.is_zero(), "Price from oracle is zero");

			log::trace!("New price: {}", price);

			// -- Sign using any account
			Signer::<T, T::AuthorityId>::any_account()
				.send_unsigned_transaction(
					|account| OraclePrice {
						round_id,
						price,
						started_at,
						updated_at,
						answered_in_round,
						block_number,
						public: account.public.clone(),
					},
					|payload, signature| Call::store_price { oracle_price: payload, signature },
				)
				.ok_or_else(|| "Failed to sign transaction")?
				.1
				.map_err(|_| "Failed to send transaction")?;

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
		fn validate_unsigned(source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			// Firstly let's check that we call the right function.
			if let Call::store_price { ref oracle_price, ref signature } = call {
				// ensure it's a local transaction sent by an offchain worker
				match source {
					TransactionSource::InBlock | TransactionSource::Local => {},
					_ => {
						log::debug!("Not a local transaction");
						// Return TransactionValidityError˘ if the call is not allowed.
						return InvalidTransaction::Call.into();
					},
				}

				// check public is valid
				let valid_keys = <FragKeys<T>>::get();
				log::debug!("Valid keys: {:?}", valid_keys);
				let pub_key = oracle_price.public.encode();
				let pub_key: ed25519::Public = {
					if let Ok(MultiSigner::Ed25519(pub_key)) =
						<MultiSigner>::decode(&mut &pub_key[..])
					{
						pub_key
					} else {
						// Return TransactionValidityError if the call is not allowed.
						return InvalidTransaction::BadSigner.into(); // // 问Gio
					}
				};
				log::debug!("Public key: {:?}", pub_key);
				if !valid_keys.contains(&pub_key) {
					// return TransactionValidityError if the call is not allowed.
					return InvalidTransaction::BadSigner.into();
				}

				let signature_valid =
					SignedPayload::<T>::verify::<T::AuthorityId>(oracle_price, signature.clone());
				if !signature_valid {
					return InvalidTransaction::BadProof.into();
				}
				log::debug!("Sending store_price extrinsic");
				ValidTransaction::with_tag_prefix("PriceFromOracleUpdate")
					.and_provides(oracle_price.price)
					// The transaction is only valid for next 5 blocks. After that it's
					// going to be revalidated by the pool.
					// TODO param to consider. It could be useful since the price from Oracle is valid only for few hours
					//.longevity(5)
					.propagate(false)
					.build()
			} else {
				InvalidTransaction::Call.into()
			}
		}
	}
}
