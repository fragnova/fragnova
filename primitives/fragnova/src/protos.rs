// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use crate::{Hash128, Hash256};
use codec::{Compact, Decode, Encode};
use sp_std::{
	collections::btree_map::BTreeMap,
	vec::Vec
};
use sp_core::{ecdsa, H160, U256};
use protos::{categories::Categories};

/// TODO: Documentation
#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub enum LinkSource {
	// Generally we just store this data, we don't verify it as we assume auth service did it.
	// (Link signature, Linked block number, EIP155 Chain ID)
	/// TODO: Documentation
	Evm(ecdsa::Signature, u64, U256),
}

/// **Types** of **Assets that are linked to a Proto-Fragment** (e.g an ERC-721 Contract etc.)
#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub enum LinkedAsset {
	/// Ethereum (ERC721 Contract address, Token ID, Link source)
	Erc721(H160, U256, LinkSource),
}

/// **Types** of **Proto-Fragment Owners**
#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub enum ProtoOwner<TAccountId> {
	/// A **regular account** on **this chain**
	User(TAccountId),
	/// An **external asset** not on this chain
	ExternalAsset(LinkedAsset),
}

/// **Struct** of a **Proto-Fragment Patch**
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq, Eq)]
pub struct ProtoPatch<TBlockNumber> {
	/// **Block Number** in which the **patch was created**
	pub block: TBlockNumber,
	/// **Hash** of patch data
	pub data_hash: Hash256,
	/// **List of New Proto-Fragments** that was **used** to **create** the **patch** (INCDT)
	pub references: Vec<Hash256>,
	/// **Data** of the **patch** (Only valid if not Local)
	pub data: ProtoData,
}

/// Struct that represents the account information of a Proto-Fragment
#[derive(Default, Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq, Eq)]
pub struct AccountsInfo {
	/// TODO: Documentation
	pub active_accounts: u128,
	/// TODO: Documentation
	pub lifetime_accounts: u128,
}

/// **Enum** that indicates **how a Proto-Fragment can be used**
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq, Eq)]
pub enum UsageLicense<TContractAddress> {
	/// Proto-Fragment is not available for use (owners can always use it)
	Closed,
	/// Proto-Fragment is available for use freely
	Open,
	/// Proto-Fragment is available for use if a custom contract returns true
	Contract(TContractAddress),
}

/// **Enum** that indicates **how a Proto-Fragment data can be fetched**
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq, Eq)]
pub enum ProtoData {
	/// Data is stored on this chain's blocks directly, this is the most safe way of storing data.
	Local(Vec<u8>),
	/// Data is stored on the Arweave chain, store tx hash, no way to guarantee exact 100% uniqueness of data.
	Arweave(Hash256),
	/// Data is maybe somewhere on the IPFS network, this is unsafe cos the IPFS network is all about caching and content delivery, offering no guarantee of permanent storage
	/// With that said there are some ways to guarantee the data is stored on IPFS via stuff like FileCoin + Lighthouse but they are relatively not as popular as Arweave so it's not recommended.
	Ipfs([u8; 64]),
}

/// **Struct** of a **Proto-Fragment**
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq, Eq)]
pub struct Proto<TAccountId, TBlockNumber> {
	/// **Block Number** in which the **Proto-Fragment was minted in**
	pub block: TBlockNumber,
	/// **List of *ProtoPatch* structs** of the **Proto-Fragment**
	pub patches: Vec<ProtoPatch<TBlockNumber>>,
	/// **License** details of the **Proto-Fragment**
	pub license: UsageLicense<TAccountId>,
	/// **Original Creator** of the **Proto-Fragment**
	pub creator: TAccountId,
	/// *Current Owner** of the **Proto-Fragment**
	pub owner: ProtoOwner<TAccountId>,
	/// **List of other Proto-Fragments** used to create the **Proto-Fragment**
	pub references: Vec<Hash256>,
	/// **Category** of the **Proto-Fragment**
	pub category: Categories,
	/// **List of Tags** associated with the **Proto-Fragment**
	pub tags: Vec<Compact<u64>>,
	/// **Map** that maps the **Key of a Proto-Fragment's Metadata Object** to the **Hash of the
	/// aforementioned Metadata Object**
	pub metadata: BTreeMap<Compact<u64>, Hash256>,
	/// Accounts information for this proto.
	pub accounts_info: AccountsInfo,
	/// **Data** of the **Proto-Fragment** (valid only if not Local)
	pub data: ProtoData,
	/// **Cluster** ID where the Proto belongs to (Optional)
	pub cluster: Option<Hash128>,
}
