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
use sp_core::crypto::KeyTypeId;
use sp_fragnova::http_json_post;
use sp_runtime::transaction_validity::{InvalidTransaction, TransactionValidity, ValidTransaction};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

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

/// The encoded function in the Chainlink smart contract to call in eth_call (see README)
const CHAINLINK_CONTRACT_FUNCTION: &'static str =
	"0xfeaf968c0000000000000000000000000000000000000000000000000000000000000000";
/// The encoded function in the Uniswap smart contract to call in eth_call (see README)
const UNISWAP_CONTRACT_FUNCTION: &'static str = "0xf7729d43000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2000000000000000000000000dac17f958d2ee523a2206206994597c13d831ec700000000000000000000000000000000000000000000000000000000000001f40000000000000000000000000000000000000000000000000de0b6b3a76400000000000000000000000000000000000000000000000000000000000000000000";

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

/// **Traits** of the **Chainlink contract** on the **Ethereum (Goerli) network**
pub trait OracleContract {
	/// get the default oracle provider
	fn get_provider() -> OracleProvider {
		OracleProvider::Uniswap("0x547a514d5e3769680Ce22B2361c10Ea13619e8a9".encode()) // never used
	}
}

impl OracleContract for () {}

/// enum that represents the price feed provider.
/// `Vec<u8>` is the address of the smart contract to use.
#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub enum OracleProvider {
	/// Chainlink provider
	Chainlink(Vec<u8>),
	/// Uniswap provider (default)
	Uniswap(Vec<u8>),
}

pub use pallet::*;

use sp_io::hashing::blake2_256;
use sp_std::{collections::btree_set::BTreeSet, vec, vec::Vec};

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use core::str::FromStr;
	use ethabi::{ethereum_types::U256, ParamType};
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use scale_info::prelude::{format, string::String};
	use sp_core::{ed25519, offchain::Timestamp, H256};
	use sp_runtime::{
		traits::{ValidateUnsigned, Zero},
		transaction_validity::TransactionSource,
		MultiSigner,
	};

	const FETCH_TIMEOUT_PERIOD: u64 = 3000; // in milli-seconds

	/// This pallet's configuration trait
	#[pallet::config]
	pub trait Config: CreateSignedTransaction<Call<Self>> + frame_system::Config {
		/// The identifier type for an offchain worker.
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;

		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// **Traits** that allows to set the Oracle provider for the price feed of FRAG token
		type OracleProvider: OracleContract;

		/// Number of votes needed to do something
		#[pallet::constant]
		type Threshold: Get<u64>;
	}

	/// The Genesis Configuration for the Pallet.
	#[pallet::genesis_config]
	#[derive(Default)]
	pub struct GenesisConfig {
		/// **List of Fragnova Account IDs** that can ***validate*** and ***send*** **unsigned transactions with signed payload**
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

	impl OracleProvider {
		/// Get the smart contract address of the selected oracle provider for the price feed of FRAG token
		pub fn get_contract_address(&self) -> Vec<u8> {
			match self {
				OracleProvider::Chainlink(address) => address.clone(),
				// Uniswap v3 Quoter smart contract address: 0xb27308f9F90D607463bb33eA1BeBb41C27CE5AB6
				// https://docs.uniswap.org/contracts/v3/reference/periphery/lens/Quoter
				OracleProvider::Uniswap(address) => address.clone(),
			}
		}
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Struct used to hold price data received from the Chainlink Price Feed smart contract.
	/// Please refer to https://docs.chain.link/docs/data-feeds/price-feeds/api-reference/#latestrounddata.
	#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, scale_info::TypeInfo)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub struct OraclePrice<TPublic, TBlockNumber> {
		/// the latest price fetched from the oracle feed
		pub price: U256,
		/// The block number on Fragnova when this price was fetched from the oracle
		pub block_number: TBlockNumber,
		/// Fragnova Public Account Address (the account address should be in FragKey, otherwise it fails)
		pub public: TPublic,
		/// Reserved for future use
		pub _reserved1: Option<()>,
		/// Reserved for future use
		pub _reserved2: Option<()>,
		/// Reserved for future use
		pub _reserved3: Option<()>,
	}

	/// Storage use for the latest price received from the oracle.
	#[pallet::storage]
	#[pallet::getter(fn prices)]
	pub(super) type Price<T: Config> = StorageValue<_, u128, ValueQuery>;

	/// **StorageMap** that maps **a FRAG token locking or unlocking event** to a **number of votes ()**.
	/// The key for this map is:
	/// `blake2_256(encoded(<Amount of FRAG token that was locked/unlocked, Signature written by the owner of the FRAG token on a determinstic message,
	/// 					Whether it was locked or unlocked, Ethereum Block Number where it was locked/unlocked>))`
	#[pallet::storage]
	pub type EVMLinkVoting<T: Config> = StorageMap<_, Identity, H256, u64>;

	/// **StorageMap** that maps **a FRAG token locking or unlocking event** to a boolean indicating whether voting on the aforementioned event has ended**.
	#[pallet::storage]
	pub type EVMLinkVotingClosed<T: Config> = StorageMap<_, Identity, H256, T::BlockNumber>;

	/// **StorageValue** that equals the **List of Fragnova Account IDs** that both ***validate*** and ***send*** **unsigned transactions with signed payload**
	///
	/// NOTE: Only the Root User of the Fragnova Blockchain (i.e the local node itself) can edit this list
	#[pallet::storage]
	pub type FragKeys<T: Config> = StorageValue<_, BTreeSet<ed25519::Public>, ValueQuery>;

	/// **StorageValue** that contains the flag used to stop the Oracle.
	#[pallet::storage]
	pub type IsOracleStopped<T: Config> = StorageValue<_, bool, ValueQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(block_number: T::BlockNumber) {
			Self::fetch_price_from_oracle(block_number);
		}
	}

	/// A public part of the pallet.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Add `public` to the **list of Fragnova Account IDs** that can ***validate*** and ***send*** **unsigned transactions with signed payload**
		///
		/// NOTE: Only the Root User of the Fragnova Blockchain (i.e the local node itself) can edit this list
		#[pallet::weight(10000)]
		#[pallet::call_index(0)]
		pub fn add_key(origin: OriginFor<T>, public: ed25519::Public) -> DispatchResult {
			ensure_root(origin)?;

			log::debug!("New key: {:?}", public);

			<FragKeys<T>>::mutate(|validators| {
				validators.insert(public);
			});

			Ok(())
		}

		/// Remove a Fragnova Account ID from `FragKeys`
		/// NOTE: Only the Root User of the Fragnova Blockchain (i.e the local node itself) can call this function
		#[pallet::weight(10000)] // TODO
		#[pallet::call_index(1)]
		pub fn del_key(origin: OriginFor<T>, public: ed25519::Public) -> DispatchResult {
			ensure_root(origin)?;

			log::debug!("Removed key: {:?}", public);

			<FragKeys<T>>::mutate(|validators| {
				validators.remove(&public);
			});

			Ok(())
		}

		/// This function stores the original price (U256) received from the selected oracle provider.
		#[pallet::weight(10000)] // TODO
		#[pallet::call_index(2)]
		pub fn store_price(
			origin: OriginFor<T>,
			oracle_price: OraclePrice<T::Public, T::BlockNumber>,
			_signature: T::Signature,
		) -> DispatchResult {
			// This ensures that the function can only be called via unsigned transaction.
			ensure_none(origin)?;

			log::debug!("Store price: {:?}", oracle_price);

			let data_hash: H256 = oracle_price.using_encoded(blake2_256).into();

			let latest_price: u128 =
				oracle_price.price.try_into().map_err(|_| Error::<T>::SystematicFailure)?;
			ensure!(!latest_price.is_zero(), Error::<T>::PriceIsZero);

			let block_number = oracle_price.block_number;

			// voting
			let threshold = T::Threshold::get();
			if threshold > 1 {
				let current_votes = <EVMLinkVoting<T>>::get(&data_hash);
				if let Some(current_votes) = current_votes {
					// Number of votes for the key `data_hash` in EVMLinkVoting
					if current_votes + 1u64 < threshold {
						// Current Votes has not passed the threshold
						<EVMLinkVoting<T>>::insert(&data_hash, current_votes + 1);
						return Ok(())
					} else {
						// Current votes passes the threshold, let's remove the record
						<EVMLinkVoting<T>>::remove(&data_hash);
					}
				} else {
					// If key `data_hash` doesn't exist in EVMLinkVoting
					<EVMLinkVoting<T>>::insert(&data_hash, 1);
					return Ok(())
				}
			}

			log::trace!("Store new price: {}", latest_price);

			<Price<T>>::put(latest_price);

			Self::deposit_event(Event::NewPrice { price: latest_price, block_number });
			Ok(())
		}

		/// Circuit breaker!
		/// Stop the Oracle by changing the value of **OracleStop** flag.
		/// **true** = oracle stop from the next block,
		/// **false** = oracle keeps running (default).
		#[pallet::weight(10000)]
		#[pallet::call_index(3)]
		pub fn stop_oracle(origin: OriginFor<T>, stop_it: bool) -> DispatchResult {
			ensure_root(origin)?;

			log::debug!("Oracle stop flag: {:?}", stop_it);

			IsOracleStopped::<T>::put(stop_it);

			Self::deposit_event(Event::OracleStopFlag { is_stopped: stop_it });

			Ok(())
		}
	}

	/// Events for the pallet.
	#[allow(missing_docs)]
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event generated when new price is accepted.
		NewPrice { price: u128, block_number: T::BlockNumber },
		/// Oracle stop flag updated
		OracleStopFlag { is_stopped: bool },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Error with connection with Geth client.
		GethConnectionError,
		/// Error in case the price of FRAG is zero. It should never happen.
		PriceIsZero,
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

		/// A helper function to allow other pallets to fetch the latest FRAG price with correct decimals.
		///
		/// TODO: the decimals at the moment are USDT (6 decimals). It needs to be changed when FRAG pool will be available and the second token will be known.
		pub fn get_price() -> Result<u128, &'static str> {
			let price = <Price<T>>::get() as f64 / 1e6; // 1e6 is the number of decimal of USDT
			let price = format!("{:.0}", price);
			let price =
				u128::from_str(&price).map_err(|_| "Error while parsing price from oracle")?;

			Ok(price)
		}

		/// A helper function to fetch the price, sign payload and send an unsigned transaction.
		pub fn fetch_price_from_oracle(block_number: T::BlockNumber) {
			let is_oracle_stopped = <IsOracleStopped<T>>::get();
			if !is_oracle_stopped {
				// check that the oracle is NOT stopped
				let geth_uri = if let Some(geth) = sp_fragnova::fragnova::get_geth_url() {
					String::from_utf8(geth).unwrap()
				} else {
					log::debug!("Geth URL not set, skipping fetch price from oracle.");
					return
				};

				let oracle_provider: OracleProvider = T::OracleProvider::get_provider();

				if let Err(e) = Self::fetch_price(block_number, oracle_provider, &geth_uri) {
					log::error!("Failed to fetch price from oracle with error: {}", e);
				}
			} else {
				log::debug!("The IsOracleStopped flag is set on {:?}. Call stop_oracle(false) to restart it.", is_oracle_stopped);
				return
			}
		}

		fn fetch_price(
			block_number: T::BlockNumber,
			oracle_provider: OracleProvider,
			geth_uri: &str,
		) -> Result<(), &'static str> {
			let contract_address = oracle_provider.get_contract_address();
			let data = Self::get_eth_call_data(&oracle_provider);

			let req = json!({
				"jsonrpc": "2.0",
				"method": "eth_call", // https://ethereum.org/en/developers/docs/apis/json-rpc/#eth_call
				"params": [
					{
					"to": contract_address.as_slice(),
					"data": data,
					},
					"latest"
				],
				 "id": 5, //goerli
			});

			let req = serde_json::to_string(&req).map_err(|_| "Invalid request")?;
			log::trace!("Request: {}", req);

			let wait = Timestamp::from_unix_millis(FETCH_TIMEOUT_PERIOD);
			let response_body = http_json_post(&geth_uri, req.as_bytes(), Some(wait));
			let response_body = if let Ok(response) = response_body {
				response
			} else {
				return Err("Failed to get response from Ethereum.")
			};

			let response = String::from_utf8(response_body).map_err(|_| "Invalid response")?;
			log::trace!("Response: {}", response);

			let v: Value =
				serde_json::from_str(&response).map_err(|_| "Invalid response - json parse")?;
			let result = v["result"].as_str().ok_or("Invalid response - no result")?; // Get the latest block number of the Ethereum Blockchain
			let data = hex::decode(&result[2..]).map_err(|_| "Invalid response - invalid data")?;
			let price = Self::get_price_from_oracle_data(&oracle_provider, data)?;

			log::trace!("New price: {}", price);

			// -- Sign using any account
			Signer::<T, T::AuthorityId>::any_account()
				.send_unsigned_transaction(
					|account| OraclePrice {
						price,
						block_number,
						public: account.public.clone(),
						_reserved1: None,
						_reserved2: None,
						_reserved3: None,
					},
					|payload, signature| Call::store_price { oracle_price: payload, signature },
				)
				.ok_or_else(|| "Failed to sign transaction")?
				.1
				.map_err(|_| "Failed to send transaction")?;

			Ok(())
		}

		/// Get the data needed to execute **eth_call** to the selected oracle provider.
		///
		/// For **Chainlink** it encodes the call to `latestRoundData()` function from ChainLink Feed Price contract.
		/// It uses the first 4 bytes of `keccak_256(latestRoundData())`, padded - Use https://emn178.github.io/online-tools/keccak_256.html.
		///
		/// For **Uniswap** it encodes the call to `quoteExactInputSingle` function in the Quoter smart contracts that _returns the amount out received for a given exact input but for a swap of a single pool_:
		/// https://docs.uniswap.org/contracts/v3/reference/periphery/lens/Quoter#quoteexactinputsingle.
		///
		/// The pool used at this moment is ETH/USDT. (TODO: it needs to be changed when the FRAG pool will be available).
		pub fn get_eth_call_data(provider: &OracleProvider) -> &'static str {
			match provider {
				OracleProvider::Chainlink(_) => CHAINLINK_CONTRACT_FUNCTION,

				OracleProvider::Uniswap(_) =>
				// encoding of quoteExactInputSingle to ETH/USDT pool. TODO to change when FRAG pool will be known
					UNISWAP_CONTRACT_FUNCTION,
			}
		}

		/// Fetch the latest price from the selected oracle provider.
		pub fn get_price_from_oracle_data(
			provider: &OracleProvider,
			data: Vec<u8>,
		) -> Result<U256, &'static str> {
			match provider {
				OracleProvider::Chainlink(_) => {
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

					let tuple = data[0].clone().into_tuple().ok_or_else(|| "Invalid tuple")?;
					let _round_id =
						tuple[0].clone().into_uint().ok_or_else(|| "Invalid roundId")?;
					let price = tuple[1].clone().into_int().ok_or_else(|| "Invalid token")?;
					let _updated_at =
						tuple[3].clone().into_uint().ok_or_else(|| "Invalid updatedAt")?;
					let _answered_in_round =
						tuple[4].clone().into_uint().ok_or_else(|| "Invalid answeredInRound")?;

					/*
					The following data validations have been inspired by:
					- https://github.com/code-423n4/2021-08-notional-findings/issues/92
					- https://github.com/code-423n4/2022-02-hubble-findings/issues/123
					- https://ethereum.stackexchange.com/questions/133890/chainlink-latestrounddata-security-fresh-data-check-usage
					and other similar reports: https://github.com/search?q=latestrounddata+validation&type=issues
					*/
					ensure!(_round_id.gt(&U256::zero()), "Price from oracle is 0");
					ensure!(price.gt(&U256::zero()), "Price from oracle is <= 0");
					ensure!(!_updated_at.is_zero(), "UpdateAt = 0. Incomplete round.");
					ensure!(!_answered_in_round.is_zero(), "AnsweredInRound from oracle is 0");
					ensure!(_answered_in_round.ge(&_round_id), "Stale price");

					Ok(price)
				},

				OracleProvider::Uniswap(_) => {
					let data = ethabi::decode(&[ParamType::Uint(256)], &data)
						.map_err(|_| "Invalid response")?;

					let price: U256 = data[0].clone().into_uint().ok_or_else(|| "Invalid token")?;
					ensure!(price.gt(&U256::zero()), "Price from oracle is <= 0");
					Ok(price)
				},
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
			if let Call::store_price { ref oracle_price, ref signature } = call {
				// ensure it's a local transaction sent by an offchain worker
				match source {
					TransactionSource::InBlock | TransactionSource::Local => {},
					_ => {
						log::debug!("Not a local transaction");
						// Return TransactionValidityError˘ if the call is not allowed.
						return InvalidTransaction::Call.into()
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
						return InvalidTransaction::BadSigner.into()
					}
				};
				log::debug!("Public key: {:?}", pub_key);
				if !valid_keys.contains(&pub_key) {
					// return TransactionValidityError if the call is not allowed.
					return InvalidTransaction::BadSigner.into()
				}

				let signature_valid =
					SignedPayload::<T>::verify::<T::AuthorityId>(oracle_price, signature.clone());
				if !signature_valid {
					return InvalidTransaction::BadProof.into()
				}
				log::debug!("Sending store_price extrinsic");
				ValidTransaction::with_tag_prefix("PriceFromOracleUpdate")
					.and_provides((
						oracle_price.price,
						oracle_price.block_number,
						oracle_price.public.clone(),
					))
					.propagate(false)
					.build()
			} else {
				InvalidTransaction::Call.into()
			}
		}
	}
}
