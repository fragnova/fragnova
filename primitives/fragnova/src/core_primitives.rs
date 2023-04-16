//! Core types that are used by the Fragnova blockchain.

#![cfg_attr(not(feature = "std"), no_std)]

use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	MultiSignature,
};

/// An index to a block.
pub type BlockNumber = u64;

/// An instant or duration in time.
pub type Moment = u64;

/// Alias to type for a signature for a transaction on the chain. This allows one of several
/// kinds of underlying crypto to be used, so isn't a fixed size when encoded.
pub type Signature = MultiSignature;

/// Alias to the public key used for this chain, actually a `MultiSigner`. Like the signature, this
/// also isn't a fixed size when encoded, as different cryptos have different size public keys.
pub type AccountPublic = <Signature as Verify>::Signer;

/// Alias to the opaque account ID type for this chain, actually a `AccountId32`. This is always
/// 32 bytes.
pub type AccountId = <AccountPublic as IdentifyAccount>::AccountId;

/// The type for looking up accounts. Related to Index pallet
pub type AccountIndex = u64;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// Index of a transaction in the chain.
pub type Index = u32;

/// Balance of an account.
///
/// 128-bits (or 38 significant decimal figures) will allow for 10 m currency (`10^7`) at a resolution
/// to all for one second's worth of an annualised 50% reward be paid to a unit holder (`10^11` unit
/// denomination), or `10^18` total atomic units, to grow at 50%/year for 51 years (`10^9` multiplier)
/// for an eventual total of `10^27` units (27 significant decimal figures).
/// We round denomination to `10^12` (12 SDF), and leave the other redundancy at the upper end so
/// that 32 bits may be multiplied with a balance in 128 bits without worrying about overflow.
pub type Balance = u128;
