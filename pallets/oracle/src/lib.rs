//! pallet-oracle queries the ChainLink Price Feed smart contract on Ethereum to fetch the price of FRAG/USD.
//!
//! It works as an Offchain Worker that will be triggered after every block, to fetch the current price.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::traits::Get;
use frame_system::{
	self as system,
	offchain::{
		AppCrypto, CreateSignedTransaction, SendUnsignedTransaction,
		SignedPayload, Signer, SigningTypes,
	},
};
use sp_core::crypto::KeyTypeId;
use sp_clamor::http_json_post;
use serde_json::{json, Value};
use sp_runtime::{
	transaction_validity::{InvalidTransaction, TransactionValidity, ValidTransaction},
	RuntimeDebug,
};
use sp_std::vec::Vec;

#[cfg(test)]
pub mod tests;
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"frag");

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

/// **Traits** of the **ETH / USD Chainlink contract** on the **Ethereum (Goerli) network**
pub trait ChainLinkContract {
	/// **Return** the **contract address** of the **ETH / USD Chainlink contract**
	fn get_contract() -> &'static str {
		// https://docs.chain.link/docs/data-feeds/price-feeds/addresses/
		"0xD4a33860578De61DBAbDc8BFdb98FD742fA7028e"
	}
}

impl ChainLinkContract for () {}

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use std::collections::BTreeSet;
	use ethabi::ParamType;
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_core::ed25519;
	use sp_runtime::MultiSigner;

	/// This pallet's configuration trait
	#[pallet::config]
	pub trait Config:
		CreateSignedTransaction<Call<Self>>
		+ frame_system::Config
	{
		/// The identifier type for an offchain worker.
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;

		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// **Traits** of the **ETH / USD Chainlink contract** on the **Ethereum (Goerli) network*
		type ChainLinkContract: ChainLinkContract;

		/// Maximum number of prices.
		#[pallet::constant]
		type MaxPrices: Get<u32>;
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

	impl<T: SigningTypes> SignedPayload<T> for PricePayload<T::Public, T::BlockNumber> {
		fn public(&self) -> T::Public {
			self.public.clone()
		}
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Struct used to hold price data.
	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct PricePayload<Public, BlockNumber> {
		block_number: BlockNumber,
		price: u32,
		public: Public
	}

	/// A bounded sized vector of recently submitted prices.
	/// This is used to calculate average price.
	#[pallet::storage]
	#[pallet::getter(fn prices)]
	pub(super) type Prices<T: Config> = StorageValue<_, BoundedVec<u32, T::MaxPrices>, ValueQuery>;

	/// **StorageValue** that equals the **List of Clamor Account IDs** that both ***validate*** and ***send*** **unsigned transactions with signed payload**
	///
	/// NOTE: Only the Root User of the Clamor Blockchain (i.e the local node itself) can edit this list
	#[pallet::storage]
	pub type FragKeys<T: Config> = StorageValue<_, BTreeSet<ed25519::Public>, ValueQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(block_number: T::BlockNumber) {

			let res = Self::fetch_price_from_oracle(block_number);
			if let Err(e) = res {
				log::error!("Error: {}", e);
			}
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
			price_payload: PricePayload<T::Public, T::BlockNumber>,
			_signature: T::Signature,
		) -> DispatchResultWithPostInfo {
			// This ensures that the function can only be called via unsigned transaction.
			ensure_none(origin)?;

			// TODO: regenerate the same Ethereum message received

			Self::add_price(price_payload.block_number, price_payload.price);
			Ok(().into())
		}
	}

	/// Events for the pallet.
	#[allow(missing_docs)]
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event generated when new price is accepted to contribute to the average.
		NewPrice { price: u32, block_number: T::BlockNumber },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Error with connection with Geth client.
		GethConnectionError,
		/// Systematic failure - those errors should not happen.
		SystematicFailure
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
		pub fn fetch_price_from_oracle(block_number: T::BlockNumber) -> Result<(), &'static str> {

			let geth_uri = if let Some(geth) = sp_clamor::clamor::get_geth_url() {
				String::from_utf8(geth).unwrap()
			} else {
				return Err("Connection error with Geth.")
			};


			let contract = T::ChainLinkContract::get_contract();
			if let Err(e) = Self::fetch_price(block_number, &contract, &geth_uri) {
				log::error!("Failed to fetch price from oracle with error: {}", e);
				return Err(e)
			}
			Ok(())
		}

		fn fetch_price(
			block_number: T::BlockNumber,
			contract: &str,
			geth_uri: &str
		) -> Result<(), &'static str> {
			let req = json!({
				"jsonrpc": "2.0",
				"method": "eth_call", // https://ethereum.org/en/developers/docs/apis/json-rpc/#eth_call
				"params": [
					{
					"to": contract,
					// first 4 bytes of keccak_256(latestRoundData()) function, padded - Use https://emn178.github.io/online-tools/keccak_256.html
					"data": "feaf968c0000000000000000000000000000000000000000000000000000000000000000",
					},
					"latest"
				],
				 "id": "5", //goerli
			});

			let req = serde_json::to_string(&req).map_err(|_| "Invalid request")?;
			log::trace!("Request: {}", req);

			let response_body = http_json_post(&geth_uri, req.as_bytes()); // Get the latest block number of the Ethereum Blockchain
			let response_body = if let Ok(response) = response_body {
				response
			} else {
				return Err("Failed to get response from geth")
			};

			let response = String::from_utf8(response_body).map_err(|_| "Invalid response")?;
			log::trace!("Response: {}", response);

			let v: Value =
				serde_json::from_str(&response).map_err(|_| "Invalid response - json parse")?;
			let result = v["result"].as_str().ok_or("Invalid response - no result")?; // Get the latest block number of the Ethereum Blockchain
			let data = hex::decode(&result[2..]).map_err(|_| "Invalid response - invalid data")?;
			let data = ethabi::decode(
				//https://docs.chain.link/docs/data-feeds/price-feeds/api-reference/#latestrounddata
				&[ParamType::Tuple(vec![
					ParamType::Uint(80), // uint80 roundId
					ParamType::Int(256), // int256 answer
					ParamType::Uint(256), // uint256 startedAt
					ParamType::Uint(256), // uint256 updatedAt
					ParamType::Uint(80), // uint80 answeredInRound

				])],
				&data).map_err(|_| "Invalid response")?;
			let new_price = data[1].clone().into_int().ok_or_else(|| "Invalid price").unwrap().as_u32();
			log::trace!("New price: {}", new_price);

			// -- Sign using any account
			let (_, result) = Signer::<T, T::AuthorityId>::any_account()
				.send_unsigned_transaction(
					|account| PricePayload {
						block_number,
						price: new_price.into(),
						public: account.public.clone()
					},
					|payload, signature| Call::store_price {
						price_payload: payload,
						signature,
					},
				)
				.ok_or("No local accounts accounts available.")?;
			result.map_err(|()| "Unable to submit transaction")?;

			Ok(())
		}

		/// Add new price to the list.
		fn add_price(block_number: T::BlockNumber, price: u32) {
			log::info!("Adding to the average: {}", price);
			<Prices<T>>::mutate(|prices| {
				if prices.try_push(price).is_err() {
					prices[(price % T::MaxPrices::get()) as usize] = price; // insert into the bounded size vector
				}
			});

			let average = Self::average_price()
				.expect("The average is not empty, because it was just mutated; qed");
			log::info!("Current average price is: {}", average);

			// here we are raising the NewPrice event
			Self::deposit_event(Event::NewPrice { price, block_number });
		}

		/// Calculate current average price.
		fn average_price() -> Option<u32> {
			let prices = <Prices<T>>::get();
			if prices.is_empty() {
				None
			} else {
				Some(prices.iter().fold(0_u32, |a, b| a.saturating_add(*b)) / prices.len() as u32)
			}
		}

		fn validate_transaction_parameters(
			block_number: &T::BlockNumber,
			new_price: &u32,
		) -> TransactionValidity {
			// Let's make sure to reject transactions from the future.
			let current_block = <system::Pallet<T>>::block_number();
			if &current_block < block_number {
				return InvalidTransaction::Future.into()
			}

			ValidTransaction::with_tag_prefix("PriceFromOracleUpdate")
				.and_provides(new_price)
				// The transaction is only valid for next 5 blocks. After that it's
				// going to be revalidated by the pool.
				// TODO do we need this longevity? It could be useful since the price from Oracle is valid only for few hours
				//.longevity(5)
				// TODO or false? Price is of interest for all nodes in the network..
				.propagate(true)
				.build()
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
			if let Call::store_price { price_payload: ref payload, ref signature } = call {

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
				// I'm sure there is a way to do this without serialization but I can't spend so
				// much time fighting with rust
				let pub_key = payload.public.encode();
				let pub_key: ed25519::Public = {
					if let Ok(MultiSigner::Ed25519(pub_key)) =
					<MultiSigner>::decode(&mut &pub_key[..])
					{
						pub_key
					} else {
						// Return TransactionValidityError if the call is not allowed.
						return InvalidTransaction::BadSigner.into() // // 问Gio
					}
				};
				log::debug!("Public key: {:?}", pub_key);
				if !valid_keys.contains(&pub_key) {
					// return TransactionValidityError if the call is not allowed.
					return InvalidTransaction::BadSigner.into()
				}

				let signature_valid =
					SignedPayload::<T>::verify::<T::AuthorityId>(payload, signature.clone());
				if !signature_valid {
					return InvalidTransaction::BadProof.into()
				}
				Self::validate_transaction_parameters(&payload.block_number, &payload.price)
			} else {
				InvalidTransaction::Call.into()
			}
		}
	}
}
