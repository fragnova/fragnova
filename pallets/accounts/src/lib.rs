//! This pallet `frag` performs logic related to FRAG Token
//!
//! IMPORTANT NOTE: The term "lock" refers to the *"effective transfer"* of some ERC-20 FRAG tokens from Fragnova-owned FRAG Ethereum Smart Contract to the Clamor Blockchain.
//!
//! The term "unlock" refers to the *"effective transfer"* of all the previously locked ERC-20 FRAG tokens from the Clamor Blockchain to the aforementioned Fragnova-owned FRAG Ethereum Smart Contract.
//!
//! The term "staking" refers to the staking of the FRAG Token that is done in the Clamor Blockchain itself. It is different to the term "locking" which is defined above.
//!
//! IMPORTANT: locking != staking

#![cfg_attr(not(feature = "std"), no_std)]

#[allow(missing_docs)]
#[cfg(any(test, feature = "compile-dummy-data"))]
pub mod dummy_data;

#[cfg(test)]
pub mod mock;

#[cfg(test)]
pub mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[allow(missing_docs)]
mod weights;

/// keccak256(Lock(address,bytes,uint256)). Try it here: https://emn178.github.io/online-tools/keccak_256.html
///
/// https://github.com/fragcolor-xyz/hasten-contracts/blob/clamor/contracts/FragToken.sol
const LOCK_EVENT: &str = "0x83a932dce34e6748d366fededbe6d22c5c1272c439426f8620148e8215160b3f";
/// keccak256(Lock(address,bytes,uint256))
///
/// https://github.com/fragcolor-xyz/hasten-contracts/blob/clamor/contracts/FragToken.sol
const UNLOCK_EVENT: &str = "0xf9480f9ead9b82690f56cdb4730f12763ca2f50ce1792a255141b71789dca7fe";

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

use codec::{Decode, Encode};
pub use pallet::*;

use sp_io::{
	crypto as Crypto,
	hashing::{blake2_256, keccak_256},
};
use sp_runtime::{offchain::storage::StorageValueRef, MultiSigner};
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

use frame_support::traits::{
	ReservableCurrency,
	tokens::fungibles::Mutate,
};

/// TODO: Documentation
pub type DiscordID = u64;

/// TODO: Documentation
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq, Eq)]
pub enum ExternalID {
	/// TODO: Documentation
	Discord(DiscordID),
}

/// **Traits** of the **FRAG Token Smart Contract** on the **Ethereum Blockchain**
pub trait EthFragContract {
	/// **Return** the **contract address** of the **FRAG Token Smart Contract**
	fn get_partner_contracts() -> Vec<String> {
		vec![String::from("0xBADF00D")]
	}
}

impl EthFragContract for () {}

/// **Struct** representing a **recent confirmed (i.e with sufficient blockchain confirmations) log** for the **event `Lock` or `Unlock`** of the **FRAG token Smart Contract**
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq, Eq)]
pub struct EthLockUpdate<TPublic> {
	/// Public Account Address of What? (问Gio)
	pub public: TPublic,
	/// If the event was `Lock`, it represents the **total amount of FRAG token** that is **currently locked** (not just the newly locked FRAG token) on the **FRAG Token Smart Contract**
	/// Otherwise, if the event was `Unlock`, it must equal the ***total amount* of FRAG token that was previously locked** on the **FRAG Token Smart Contract**
	pub amount: U256,
	/// If the event was `Lock`, it represents the lock period of the FRAG token.
	/// If the event was `Unlock`, it is 999.
	pub lock_period: U256,
	/// **Ethereum Account Address** that emitted the `Lock` or `Unlock` event when they had called the smart contract function `lock()` or `unlock()` respectively
	pub sender: H160,
	/// The lock/unlock signature signed by the Ethereum Account ID
	pub signature: ecdsa::Signature,
	/// Whether the event was `Lock` or `Unlock`
	pub lock: bool,
	/// Block number in which the event was emitted
	pub block_number: u64,
}

/// **Struct** representing the details about the **total amount of locked FRAG Token of a particular Ethereum Account** in the **Fragnova-owned Ethereum Smart Contract** .
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq, Eq)]
pub struct EthLock<TBalance, TBlockNum> {
	/// Total amount of FRAG token locked (not just the newly locked FRAG token) by a particular Ethereum Account
	pub amount: TBalance,
	/// Clamor Block Number in which the locked FRAG tokens was "effectively transfered" to the Clamor Blockchain
	pub block_number: TBlockNum,
	/// The FRAG lock period chosen by the user on Ethereum and received from the Lock event
	pub lock_period: U256,
}

impl<T: SigningTypes> SignedPayload<T> for EthLockUpdate<T::Public> {
	fn public(&self) -> T::Public {
		self.public.clone()
	}
}

/// **Struct** representing the details about accounts created off-chain by various parties and integrations.
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq, Eq)]
pub struct AccountInfo<TAccountID, TMoment> {
	/// The actual account ID
	pub account_id: TAccountID,
	/// The timestamp when this account was created
	pub created_at: TMoment,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, Twox64Concat};
	use frame_support::traits::fungible::Unbalanced;
	use frame_system::pallet_prelude::*;
	use sp_runtime::SaturatedConversion;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ CreateSignedTransaction<Call<Self>>
		+ pallet_balances::Config
		+ pallet_proxy::Config
		+ pallet_timestamp::Config
		+ pallet_assets::Config
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Weight functions needed for pallet_protos.
		type WeightInfo: WeightInfo;

		/// The Ethereum Chain ID that the Fragnova-owned Ethereum Smart Contract is deployed on.
		/// This should be the Ethereum Mainnet's Chain ID.
		#[pallet::constant]
		type EthChainId: Get<u64>;

		/// The **number of confirmations required** to consider a **transaction** on the **Ethereum Blockchain** ***final*** (https://www.youtube.com/watch?v=gP5zcHD8tJU)
		#[pallet::constant]
		type EthConfirmations: Get<u64>;

		/// **Traits** of the **FRAG Token Smart Contract** on the **Ethereum Blockchain**
		type EthFragContract: EthFragContract;

		/// Number of votes needed to do something (问Gio)
		#[pallet::constant]
		type Threshold: Get<u64>;

		/// The identifier type for an offchain worker.
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;

		/// Asset ID of the fungible asset "TICKET"
		#[pallet::constant]
		type TicketsAssetId: Get<<Self as pallet_assets::Config>::AssetId>;
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

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// **StorageMap** that maps an **Ethereum Account ID** to a to an ***Ethlock* struct of the aforementioned Ethereum Account Id (the struct contains the amount of FRAG token locked, amongst other things)**
	#[pallet::storage]
	pub type EthLockedFrag<T: Config> =
		StorageMap<_, Identity, H160, EthLock<<T as pallet_assets::Config>::Balance, T::BlockNumber>>;

	/// This **StorageMap** maps an Ethereum AccountID to an amount of Tickets received until a Clamor Account ID is not linked.
	#[pallet::storage]
	pub type EthReservedTickets<T: Config> = StorageMap<_, Identity, H160, <T as pallet_assets::Config>::Balance>;

	/// This **StorageMap** maps an Ethereum AccountID to an amount of NOVA received until a Clamor Account ID is not linked.
	#[pallet::storage]
	pub type EthReservedNova<T: Config> = StorageMap<_, Identity, H160, <T as pallet_balances::Config>::Balance>;

	/// **StorageMap** that maps a **Clamor Account ID** to an **Ethereum Account ID**,
	/// where **both accounts** are **owned by the same owner**.
	///
	/// NOTE: The **Ethereum Account ID** had to be **manually mapped to the Clamor Account ID** by the owner itself to show up in this `StorageMap`.
	#[pallet::storage]
	pub type EVMLinks<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, H160>;

	/// This **StorageMap** is the reverse mapping of `EVMLinks`.
	#[pallet::storage]
	pub type EVMLinksReverse<T: Config> = StorageMap<_, Identity, H160, T::AccountId>;

	/// **StorageMap** that maps **a FRAG token locking or unlocking event** to a **number of votes ()**.
	/// The key for this map is:
	/// `blake2_256(encoded(<Amount of FRAG token that was locked/unlocked, Signature written by the owner of the FRAG token on a determinstic message,
	/// 					Whether it was locked or unlocked, Ethereum Block Number where it was locked/unlocked>))`
	#[pallet::storage]
	pub type EVMLinkVoting<T: Config> = StorageMap<_, Identity, H256, u64>;

	/// **StorageMap** that maps **a FRAG token locking or unlocking event** to a boolean indicating whether voting on the aforementioned event has ended**.
	#[pallet::storage]
	pub type EVMLinkVotingClosed<T: Config> = StorageMap<_, Identity, H256, T::BlockNumber>;
	// consumed by Protos pallet
	/// **List of Clamor Accounts** whose **(FRAG staking)-related Storage Items** are **yet to be cleared**
	#[pallet::storage]
	pub type PendingUnlinks<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	// These are the public keys representing the actual keys that can Sign messages
	// to present to external chains to detach onto
	/// **StorageValue** that equals the **List of Clamor Account IDs** that both ***validate*** and ***send*** **unsigned transactions with signed payload**
	///
	/// NOTE: Only the Root User of the Clamor Blockchain (i.e the local node itself) can edit `this list
	#[pallet::storage]
	pub type FragKeys<T: Config> = StorageValue<_, BTreeSet<ed25519::Public>, ValueQuery>;

	/// The map between external accounts and the local accounts that are linked to them. (Discord, Telegram, etc)
	#[pallet::storage]
	pub type ExternalID2Account<T: Config> =
		StorageMap<_, Twox64Concat, ExternalID, AccountInfo<T::AccountId, T::Moment>>;

	/// The authorities able to sponsor accounts and link them to external accounts.
	#[pallet::storage]
	pub type ExternalAuthorities<T: Config> = StorageValue<_, BTreeSet<T::AccountId>, ValueQuery>;

	#[allow(missing_docs)]
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A link happened between native and ethereum account.
		Linked { sender: T::AccountId, eth_key: H160 },
		/// A link was removed between native and ethereum account.
		Unlinked { sender: T::AccountId, eth_key: H160 },
		/// ETH side lock was updated
		Locked { eth_key: H160, balance: <T as pallet_assets::Config>::Balance, lock_period: U256 },
		/// ETH side lock was unlocked
		Unlocked { eth_key: H160, balance: <T as pallet_assets::Config>::Balance },
		/// A new sponsored account was added
		SponsoredAccount { sponsor: T::AccountId, sponsored: T::AccountId, external_id: ExternalID },
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
		/// Account linked to different account
		DifferentAccountLinked,
		/// Account already exists
		AccountAlreadyExists,
		/// Too many proxies
		TooManyProxies,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.

	/// Add a Clamor Account ID to `FragKeys`.
	///
	/// NOTE: Only the Root User of the Clamor Blockchain (i.e the local node itself) can call this function
	#[pallet::call]
	impl<T: Config> Pallet<T> {

		/// Add `public` to the **list of Clamor Account IDs** that can ***validate*** and ***send*** **unsigned transactions with signed payload**
		///
		/// NOTE: Only the Root User of the Clamor Blockchain (i.e the local node itself) can edit this list
		#[pallet::weight(25_000)] // TODO - weight
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
		#[pallet::weight(25_000)] // TODO - weight
		pub fn del_key(origin: OriginFor<T>, public: ed25519::Public) -> DispatchResult {
			ensure_root(origin)?;

			log::debug!("Removed key: {:?}", public);

			<FragKeys<T>>::mutate(|validators| {
				validators.remove(&public);
			});

			Ok(())
		}

		// Firstly: Verify the `signature` for the message keccak_256(b"EVM2Fragnova", T::EthChainId::get(), sender)
		// Secondly: After verification, recover the public key used to sign the aforementioned `signature` for the aforementioned message
		// Third: Add
		// TODO
		/// **Link** the **Clamor public account address that calls this extrinsic** with the
		/// **public account address that is returned from verifying the signature `signature` for
		/// the message `keccak_256(b"EVM2Fragnova", T::EthChainId::get(), sender)`** (NOTE: The
		/// returned public account address is of the account that signed the signature
		/// `signature`).
		///
		/// After linking, also emit an event indicating that the two accounts were linked.
		#[pallet::weight(25_000)] // TODO - weight
		pub fn link(origin: OriginFor<T>, signature: ecdsa::Signature) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// the idea is to prove to this chain that the sender knows the private key of the external address
			let mut message = b"EVM2Fragnova".to_vec();
			message.extend_from_slice(&T::EthChainId::get().to_be_bytes());
			message.extend_from_slice(&sender.encode());
			let message_hash = keccak_256(&message);

			let recovered = Crypto::secp256k1_ecdsa_recover(&signature.0, &message_hash)
				.map_err(|_| Error::<T>::VerificationFailed)?; // Verify the `signature` for the message keccak_256(b"EVM2Fragnova", T::EthChainId::get(), sender)

			let eth_key = keccak_256(&recovered[..]);
			let eth_key = &eth_key[12..];
			let eth_key = H160::from_slice(&eth_key[..]);

			ensure!(!<EVMLinks<T>>::contains_key(&sender), Error::<T>::AccountAlreadyLinked);
			ensure!(!<EVMLinksReverse<T>>::contains_key(eth_key), Error::<T>::AccountAlreadyLinked);

			<EVMLinks<T>>::insert(sender.clone(), eth_key);
			<EVMLinksReverse<T>>::insert(eth_key, sender.clone());

			// also emit event
			Self::deposit_event(Event::Linked { sender, eth_key });

			Ok(())
		}

		// TODO
		/// Unlink the **Clamor public account address that calls this extrinsic** from **its linked EVM public account address**
		#[pallet::weight(25_000)] // TODO - weight
		pub fn unlink(origin: OriginFor<T>, account: H160) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::unlink_account(sender, account)
		}

		/// Update 'data'
		///
		/// TODO
		#[pallet::weight(25_000)] // TODO - weight
		pub fn internal_lock_update(
			origin: OriginFor<T>,
			data: EthLockUpdate<T::Public>,
			_signature: T::Signature,
		) -> DispatchResult {
			ensure_none(origin)?;

			log::debug!("Lock update: {:?}", data);

			let data_tuple = (
				data.amount,
				data.lock_period,
				data.sender,
				data.signature.clone(),
				data.lock,
				data.block_number,
			);
			let data_hash: H256 = data_tuple.using_encoded(blake2_256).into();

			ensure!(
				!<EVMLinkVotingClosed<T>>::contains_key(data_hash), // Make sure `data_hash` isn't in `EVMLinkVotingClosed`
				Error::<T>::LinkAlreadyProcessed
			);

			// We compose the exact same message `message` as **was composed** when the external function `lock(amount, signature, period)` or `unlock(amount, signature)` of the FRAG Token Ethereum Smart Contract was called (https://github.com/fragcolor-xyz/hasten-contracts/blob/clamor/contracts/FragToken.sol)
			let mut message = if data.lock { b"FragLock".to_vec() } else { b"FragUnlock".to_vec() }; // Add b"FragLock" or b"FragUnlock" to message
			message.extend_from_slice(&data.sender.0[..]); // Add data.sender.0 ("msg.sender" in Solidity) to message
			message.extend_from_slice(&T::EthChainId::get().to_be_bytes()); // Add EthChainId ("block.chainid" in Solidity) to message
			let amount: [u8; 32] = data.amount.into();
			message.extend_from_slice(&amount[..]); // Add amount to message
			if data.lock {
				let lock_period: [u8; 32] = data.lock_period.into();
				message.extend_from_slice(&lock_period[..]); // Add amount to message
			}

			let message_hash = keccak_256(&message); // This should be

			let message = [b"\x19Ethereum Signed Message:\n32", &message_hash[..]].concat();
			let message_hash = keccak_256(&message);

			let signature = data.signature;
			let sender = data.sender;

			// We check if the `message_hash` and **the signature in the emitted event `Lock` or `Unlock`**
			// **allow us** to **recover the Ethereum sender account address** that was in the **same aforementioned emitted event `Lock` or `Unlock`**
			let pub_key = Crypto::secp256k1_ecdsa_recover(&signature.0, &message_hash)
				.map_err(|_| Error::<T>::VerificationFailed)?;
			let pub_key = keccak_256(&pub_key[..]);
			let pub_key = &pub_key[12..];
			ensure!(pub_key == sender.0, Error::<T>::VerificationFailed);

			let data_amount: u128 = data.amount.try_into().map_err(|_| Error::<T>::SystematicFailure)?;

			if !data.lock {
				ensure!(data_amount == 0, Error::<T>::SystematicFailure);
			} else {
				ensure!(data_amount > 0, Error::<T>::SystematicFailure);
			}

			// verifications ended, let's proceed with voting count and writing

			let threshold = T::Threshold::get();
			let _ = Self::vote_count_and_write(&data_hash, threshold);

			// writing here at end (THE lines below only execute if the number of votes received by `data_hash` passes the `threshold`)

			let current_block_number = <frame_system::Pallet<T>>::block_number();

			let lock_period: U256 =
				data.lock_period.try_into().map_err(|_| Error::<T>::SystematicFailure)?;
			let amount: <T as pallet_assets::Config>::Balance = data_amount.saturated_into();
			let nova_amount: <T as pallet_balances::Config>::Balance = data_amount.saturated_into();

			//TODO - get FRAG price from oracle
			let tickets_amount = Self::calculate_tickets_percentage_to_mint(data_amount);

			if data.lock {
				// If FRAG tokens were locked on Ethereum
				let linked = <EVMLinksReverse<T>>::get(sender.clone()); // Get the Clamor Account linked with the Ethereum Account `sender`
				if let Some(linked) = linked {
					// mint Tickets for the linked user
					<pallet_assets::Pallet<T> as Mutate<T::AccountId>>::mint_into(
						T::TicketsAssetId::get(),
						&linked,
						tickets_amount)?;
					// assign NOVA

					<pallet_balances::Pallet<T> as Unbalanced<T::AccountId>>::set_balance(
						&linked,
						nova_amount)?;
				} else {
					// Ethereum Account ID (H160) not linked to Clamor Account ID
					// So, register the amount of tickets and NOVA owned by the H160 account for later linking
					<EthReservedTickets<T>>::insert(
						sender.clone(),
						tickets_amount,
					);
					<EthReservedNova<T>>::insert(
						sender.clone(),
						nova_amount,
					);
				}
				// also emit event
				Self::deposit_event(Event::Locked { eth_key: sender, balance: amount, lock_period });
				<EthLockedFrag<T>>::insert(
					// VERY IMPORTANT TO NOTE
					sender.clone(),
					EthLock { amount, block_number: current_block_number, lock_period },
				);
			} else {
				// If we want to unlock all the FRAG tokens that were
				// if we have any link to this account, then force unlinking
				let linked = <EVMLinksReverse<T>>::get(sender.clone());
				if let Some(linked) = linked {
					Self::unlink_account(linked, sender.clone())?; // Unlink Ethereum Account `sender` and Clamor Account `linked`
				}
				// also emit event
				Self::deposit_event(Event::Unlocked { eth_key: sender, balance: amount }); // 问Gio for clarification

				// The amount here will be zero for both, since it is an UNLOCK event
				<EthLockedFrag<T>>::insert(
					sender.clone(),
					EthLock { amount, block_number: current_block_number, lock_period },
				);
				<EthReservedTickets<T>>::insert(
					sender.clone(),
					amount,
				);

				let nova_amount_unlock: <T as pallet_balances::Config>::Balance = amount.saturated_into();
				<EthReservedNova<T>>::insert(
					sender.clone(),
					nova_amount_unlock,
				);
			}

			// also record link hash
			<EVMLinkVotingClosed<T>>::insert(data_hash, current_block_number); // Declare that the `data_hash`'s voting has ended

			Ok(())
		}

		/// TODO
		#[pallet::weight(25_000)] // TODO - weight
		pub fn sponsor_account(origin: OriginFor<T>, external_id: ExternalID) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(<ExternalAuthorities<T>>::get().contains(&who), Error::<T>::SystematicFailure);

			// generate a unique, deterministic hash that we decode into our account
			let hash = blake2_256(
				&[&b"fragnova-sponsor-account"[..], &who.encode(), &external_id.encode()].concat(),
			);
			let account =
				T::AccountId::decode(&mut &hash[..]).map_err(|_| Error::<T>::SystematicFailure)?;

			ensure!(
				!pallet_proxy::Proxies::<T>::contains_key(&account),
				Error::<T>::AccountAlreadyExists
			);

			// use the same logic of proxy anonymous
			let proxy_def = pallet_proxy::ProxyDefinition {
				delegate: who.clone(),
				proxy_type: T::ProxyType::default(),
				delay: T::BlockNumber::default(),
			};
			let bounded_proxies: BoundedVec<_, T::MaxProxies> =
				vec![proxy_def].try_into().map_err(|_| Error::<T>::TooManyProxies)?;

			// ! Writing state

			let deposit = T::ProxyDepositBase::get() + T::ProxyDepositFactor::get();
			<T as pallet_proxy::Config>::Currency::reserve(&who, deposit)?;

			pallet_proxy::Proxies::<T>::insert(&account, (bounded_proxies, deposit));

			<ExternalID2Account<T>>::insert(
				external_id.clone(),
				AccountInfo {
					account_id: account.clone(),
					created_at: <pallet_timestamp::Pallet<T>>::get(),
				},
			);

			Self::deposit_event(Event::SponsoredAccount {
				sponsor: who,
				sponsored: account,
				external_id,
			});

			Ok(())
		}

		/// Add a sponsor account to the list of sponsors able to sponsor external accounts.
		#[pallet::weight(25_000)] // TODO - weight
		pub fn add_sponsor(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;

			log::debug!("New key: {:?}", account);

			<ExternalAuthorities<T>>::mutate(|authorities| {
				authorities.insert(account);
			});

			Ok(())
		}

		/// Remove a sponsor account to the list of sponsors able to sponsor external accounts.
		#[pallet::weight(25_000)] // TODO - weight
		pub fn remove_sponsor(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;

			log::debug!("Removed key: {:?}", account);

			<ExternalAuthorities<T>>::mutate(|authorities| {
				authorities.remove(&account);
			});

			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// This function is being called after every block import (when fully synced).
		///
		/// Implementing this function on a module allows you to perform long-running tasks
		/// that make (by default) validators generate transactions that feed results
		/// of those long-running computations back on chain.
		fn offchain_worker(n: T::BlockNumber) {
			Self::sync_partner_contracts(n);
		}
	}

	/// By default, all unsigned transactions are rejected in Substrate.
	/// To enable Substrate to accept certain unsigned transactions, you must implement the ValidateUnsigned trait for the pallet.
	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;

		/// For the call `Call:internal_lock_update` which is an unsigned transaction with a signed payload (see: https://docs.substrate.io/how-to-guides/v3/ocw/transactions/),
		/// verify that when we put the signature parameter (written as `signature`) and the payload parameter (written as `data`) of the aforementioned call into an "Ethereum Verify function",
		/// it returns the public key that is in the payload parameter.
		///
		/// Furthermore, also verify that `data.public` is in `FragKeys` - 问Gio
		///
		/// If both the aforementioned, allow the call to execute. Otherwise, do not allow it to.
		///
		/// ## Footnote:
		///
		/// Validate unsigned call to this module.
		///
		/// By default unsigned transactions are disallowed, but implementing the validator
		/// here we make sure that some particular calls (the ones produced by offchain worker)
		/// are being whitelisted and marked as valid.
		fn validate_unsigned(source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			// Firstly let's check that we call the right function.
			if let Call::internal_lock_update { ref data, ref signature } = call {
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
				// I'm sure there is a way to do this without serialization but I can't spend so
				// much time fighting with rust
				let pub_key = data.public.encode();
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

				// most expensive bit last
				// Check whether a provided signature matches the public key used to sign the payload
				let signature_valid =
					SignedPayload::<T>::verify::<T::AuthorityId>(data, signature.clone()); // Verifying a Data with a Signature Returns a Public Key (if valid)
																	   // The provided signature does not match the public key used to sign the payload
				if !signature_valid {
					// Return TransactionValidityError if the call is not allowed.
					return InvalidTransaction::BadProof.into();
				}

				log::debug!("Sending frag lock update extrinsic");
				// Return ValidTransaction if the call is allowed
				ValidTransaction::with_tag_prefix("FragLockUpdate") // The tag prefix prevents other nodes to do the same transaction that have the same tag prefixes
					.and_provides((
						data.public.clone(),
						data.amount,
						data.lock_period,
						data.sender,
						data.signature.clone(),
						data.lock,
						data.block_number,
						pub_key,
					))
					.propagate(false)
					.build()
			} else {
				// Return TransactionValidityError if the call is not allowed.
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

		fn calculate_tickets_percentage_to_mint(amount: u128) -> <T as pallet_assets::Config>::Balance {
			// hard set to 20% for the moment
			//let percent = Percentage::from(50);
			//let percent_applied = percent.apply_to(amount);

			//percent_applied.saturated_into()
			amount.saturated_into()
		}

		/// Obtain all the recent (i.e since last checked by Clamor) logs of the event `Lock` or `Unlock` that were emitted from the FRAG Token Ethereum Smart Contract.
		/// Each event log will have either have the format `Lock(address indexed sender, bytes signature, uint256 amount)` or `Unlock(address indexed sender, bytes signature, uint256 amount)` (https://github.com/fragcolor-xyz/hasten-contracts/blob/clamor/contracts/FragToken.sol).
		///
		/// Then, for each of the event logs - send an unsigned transaction with a signed payload onto the Clamor Blockchain
		/// (NOTE: the signed payload consists of a payload and a signature.
		/// The payload is the information gained from the event log which is represented as an `EthLockUpdate`  struct
		/// and the signature is the signature obtained from signing the aforementioned payload using `Signer::<T, T::AuthorityId>::any_account()`) (问Gio)
		///
		/// NOTE: `Signer::<T, T::AuthorityId>::any_account()` uses any of the keys that was added using the RPC `author_insertKey` into Clamor (https://polkadot.js.org/docs/substrate/rpc/#insertkeykeytype-text-suri-text-publickey-bytes-bytes)
		fn sync_partner_contract(
			_block_number: T::BlockNumber,
			contract: &str,
			geth_uri: &str,
		) -> Result<(), &'static str> {
			let req = json!({
				"jsonrpc": "2.0",
				"method": "eth_blockNumber",
				"id": 1u64
			});

			let req = serde_json::to_string(&req).map_err(|_| "Invalid request")?;
			log::trace!("Request: {}", req);

			let response_body = http_json_post(geth_uri, req.as_bytes()); // Get the latest block number of the Ethereum Blockchain
			let response_body = if let Ok(response) = response_body {
				response
			} else {
				return Err("Failed to get response from geth");
			};

			let response = String::from_utf8(response_body).map_err(|_| "Invalid response")?;
			log::trace!("Response: {}", response);

			let v: Value =
				serde_json::from_str(&response).map_err(|_| "Invalid response - json parse")?;

			let current_block = v["result"].as_str().ok_or("Invalid response - no result")?; // Get the latest block number of the Ethereum Blockchain
			let current_block = u64::from_str_radix(&current_block[2..], 16)
				.map_err(|_| "Invalid response - invalid block number")?;
			log::trace!("Current block: {}", current_block);

			let last_block_ref = StorageValueRef::persistent(b"frag_sync_last_block"); // This key does not exist when the blockchain is first launched
			let last_block: Option<Vec<u8>> = last_block_ref.get().unwrap_or_default(); // If `last_block` doesn't exist, set it to `Vec<u8>`
			let last_block = if let Some(last_block) = last_block {
				// Convert `last_block` from `Vec<u8>` to `String`
				String::from_utf8(last_block).map_err(|_| "Invalid last block")?
			} else {
				String::from("0x0") // If `last_block` is None, set it to `String::from("0x0")`
			};

			let to_block = current_block.saturating_sub(T::EthConfirmations::get()); // The `to_block` is the latest block that is considered final
			let to_block = format!("0x{:x}", to_block);

			// This is basically a RPC query asking how much FRAG token was locked/unlocked by who all from block number `fromBlock` to block number `toBlock`
			let req = json!({
				"jsonrpc": "2.0",
				"method": "eth_getLogs", // i.e get the event logs of the smart contract (more info: https://docs.alchemy.com/alchemy/guides/eth_getlogs#what-are-logs)
				"id": "0",
				"params": [{
					"fromBlock": last_block,
					"toBlock": to_block, // Give us the event logs that were emitted (if any) from the block number `last_block` to the block number `to_block`, inclusive
					"address": contract,
					"topics": [
						// [] to OR
						[LOCK_EVENT, UNLOCK_EVENT]
					],
				}]
			});

			let req = serde_json::to_string(&req).map_err(|_| "Invalid request")?;
			log::trace!("Request: {}", req);

			let response_body = http_json_post(geth_uri, req.as_bytes()); // Make HTTP POST request with `req` to URL `get_uri`
			let response_body = if let Ok(response) = response_body {
				response
			} else {
				return Err("Failed to get response from geth");
			};

			let response = String::from_utf8(response_body).map_err(|_| "Invalid response")?;
			log::trace!("Response: {}", response);

			let v: Value =
				serde_json::from_str(&response).map_err(|_| "Invalid response - json parse")?;

			let logs = v["result"].as_array().ok_or_else(|| "Invalid response - no result")?; // `logs` is a list of event logs
			for log in logs {
				// `logs` is a list of event logs
				let block_number =
					log["blockNumber"].as_str().ok_or("Invalid response - no block number")?;
				let block_number = u64::from_str_radix(&block_number[2..], 16)
					.map_err(|_| "Invalid response - invalid block number")?;
				let topics =
					log["topics"].as_array().ok_or_else(|| "Invalid response - no topics")?;
				let topic = topics[0].as_str().ok_or_else(|| "Invalid response - no topic")?; // The topic can either be `LOCK_EVENT` or `UNLOCK_EVENT`
				let data = log["data"].as_str().ok_or_else(|| "Invalid response - no data")?;
				let data =
					hex::decode(&data[2..]).map_err(|_| "Invalid response - invalid data")?; // Convert the hexadecimal `data` from hexadecimal to binary (i.e raw bits)
				let data = ethabi::decode(
					&[ParamType::Bytes, ParamType::Uint(256), ParamType::Uint(256)],
					&data,
				) // First parameter is a signature, the second paramteter is the amount of FRAG token that was locked/unlocked, the third is the lock period (https://github.com/fragcolor-xyz/hasten-contracts/blob/clamor/contracts/FragToken.sol)
				.map_err(|_| "Invalid response - invalid eth data")?; // `data` is the decoded list of the params of the event log `topic`
				let locked = match topic {
					// Whether the event log type `topic` is a `LOCK_EVENT` or an `UNLOCK_EVENT`
					LOCK_EVENT => true,
					UNLOCK_EVENT => false,
					_ => return Err("Invalid topic"),
				};

				// Since the first parameter of the Lock or Unlock event is declared as indexed, they are treated like additional topics (https://medium.com/mycrypto/understanding-event-logs-on-the-ethereum-blockchain-f4ae7ba50378)
				let sender = topics[1].as_str().ok_or_else(|| "Invalid response - no sender")?; // `sender` is the account that locked/unlocked FRAG (i.e the first parameter of the event `Lock` or `Unlock`)
				let sender =
					hex::decode(&sender[2..]).map_err(|_| "Invalid response - invalid sender")?;
				let sender = H160::from_slice(&sender[12..]); // Hash the array slice `&sender[12..]`

				let eth_signature = data[0].clone().into_bytes().ok_or_else(|| "Invalid data")?; // (`data[0]` is actually a signature - https://github.com/fragcolor-xyz/hasten-contracts/blob/clamor/contracts/FragToken.sol btw )
				let eth_signature: ecdsa::Signature =
					(&eth_signature[..]).try_into().map_err(|_| "Invalid data")?;

				let amount = data[1].clone().into_uint().ok_or_else(|| "Invalid data")?; // Amount of FRAG token locked/unlocked (`data[1]`)

				// Lock period (`data[2]`). In case of Unlock event, it is zero.
				let lock_period = data[2].clone().into_uint().unwrap_or(U256::from(999));

				if locked {
					log::trace!(
						"Block: {}, sender: {}, locked: {}, amount: {}, lock_period: {}, signature: {:?}",
						block_number,
						sender,
						locked,
						amount,
						lock_period,
						eth_signature.clone(),
					);
				} else {
					// Unlock event
					log::trace!(
						"Block: {}, sender: {}, locked: {}, amount: {}, signature: {:?}",
						block_number,
						sender,
						locked,
						amount,
						eth_signature.clone(),
					);
				}

				// `send_unsigned_transaction` is returning a type of `Option<(Account<T>, Result<(), ()>)>`.
				//   The returned result means:
				//   - `None`: no account is available for sending transaction
				//   - `Some((account, Ok(())))`: transaction is successfully sent
				//   - `Some((account, Err(())))`: error occurred when sending the transaction
				Signer::<T, T::AuthorityId>::any_account() // `Signer::<T, T::AuthorityId>::any_account()` is the signer that signs the payload
					.send_unsigned_transaction(
						// this line is to prepare and return payload to be used
						|account| EthLockUpdate {
							// `account` is an account `Signer::<T, T::AuthorityId>::any_account()`
							public: account.public.clone(), // 问Gio what is account.public and why is it supposed to be in FragKey
							amount,
							lock_period,
							sender,
							signature: eth_signature.clone(),
							lock: locked,
							block_number,
						},
						// The second function closure returns the on-chain call with payload and signature passed in
						|payload, signature| Call::internal_lock_update {
							data: payload,
							signature,
						},
					)
					.ok_or_else(|| "Failed to sign transaction")?
					.1
					.map_err(|_| "Failed to send transaction")?;
			}

			last_block_ref.set(&to_block.as_bytes().to_vec()); // Recall that the `to_block` is the latest block that is considered final （问Gio）

			Ok(())
		}

		/// Obtain all the recent (i.e since last checked by Clamor) logs of the event `Lock` or `Unlock` that were emitted from the FRAG Token Ethereum Smart Contract.
		/// Each event log will have either have the format `Lock(address indexed sender, bytes signature, uint256 amount)` or `Unlock(address indexed sender, bytes signature, uint256 amount)` (https://github.com/fragcolor-xyz/hasten-contracts/blob/clamor/contracts/FragToken.sol).
		///
		/// Then, for each of the event logs - send an unsigned transaction with a signed payload onto the Clamor Blockchain
		/// (NOTE: the signed payload consists of a payload and a signature.
		/// The payload is the information gained from the event log which is represented as an `EthLockUpdate`  struct
		/// and the signature is the signature obtained from signing the aforementioned payload using `Signer::<T, T::AuthorityId>::any_account()`) (问Gio)
		///
		/// NOTE: `Signer::<T, T::AuthorityId>::any_account()` uses any of the keys that was added using the RPC `author_insertKey` into Clamor (https://polkadot.js.org/docs/substrate/rpc/#insertkeykeytype-text-suri-text-publickey-bytes-bytes)
		pub fn sync_partner_contracts(block_number: T::BlockNumber) {
			let geth_uri = if let Some(geth) = sp_clamor::clamor::get_geth_url() {
				String::from_utf8(geth).unwrap()
			} else {
				log::debug!("No geth url found, skipping sync");
				return; // It is fine to have a node not syncing with eth
			};

			let contracts = T::EthFragContract::get_partner_contracts();

			for contract in contracts {
				if let Err(e) = Self::sync_partner_contract(block_number, &contract, &geth_uri) {
					log::error!("Failed to sync partner contract with error: {}", e);
				}
			}
		}

		/// Unlink the **Clamor public account address `sender`** from **its linked EVM public
		/// account address `account`**
		fn unlink_account(sender: T::AccountId, account: H160) -> DispatchResult {
			if <EVMLinks<T>>::get(sender.clone()).ok_or(Error::<T>::AccountNotLinked)? != account {
				return Err(Error::<T>::DifferentAccountLinked.into());
			}
			if <EVMLinksReverse<T>>::get(account).ok_or(Error::<T>::AccountNotLinked)? != sender {
				return Err(Error::<T>::DifferentAccountLinked.into());
			}

			<EVMLinks<T>>::remove(sender.clone());
			<EVMLinksReverse<T>>::remove(account);
			// force dereferencing of protos and more
			<PendingUnlinks<T>>::append(sender.clone());

			// also emit event
			Self::deposit_event(Event::Unlinked { sender, eth_key: account });

			Ok(())
		}

		fn vote_count_and_write(data_hash: &H256, threshold: u64) -> DispatchResult {
			if threshold > 1 {
				let current_votes = <EVMLinkVoting<T>>::get(&data_hash);
				if let Some(current_votes) = current_votes {
					// Number of votes for the key `data_hash` in EVMLinkVoting
					if current_votes + 1u64 < threshold {
						// Current Votes has not passed the threshold
						<EVMLinkVoting<T>>::insert(&data_hash, current_votes + 1);
						return Ok(());
					} else {
						// Current votes passes the threshold, let's remove EVMLinkVoting perque perque non! (问Gio)
						// we are good to go, but let's remove the record
						<EVMLinkVoting<T>>::remove(&data_hash);
					}
				} else {
					// If key `data_hash` doesn't exist in EVMLinkVoting
					<EVMLinkVoting<T>>::insert(&data_hash, 1);
					return Ok(());
				}
			}
			Ok(())
		}
	}
}
