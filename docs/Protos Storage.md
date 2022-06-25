# Protos Storage
#proto #protos
## Structs
```rust
/// ¿
#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub enum LinkSource {
	// Generally we just store this data, we don't verify it as we assume auth service did it.
	// (Link signature, Linked block number, EIP155 Chain ID)
	Evm(ecdsa::Signature, u64, U256),
}

/// Types of Assets that are linked to a Proto-Fragment (e.g an ERC-721 Contract etc.)
#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub enum LinkedAsset {
	// Ethereum (ERC721 Contract address, Token ID, Link source)
	Erc721(H160, U256, LinkSource),
}

/// Types of Proto-Fragment Owner
#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
pub enum ProtoOwner<TAccountId> {
	// A regular account on this chain
	User(TAccountId),
	// An external asset not on this chain
	ExternalAsset(LinkedAsset),
}

#[derive(Encode, Decode, Clone, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct GetProtosParams<TAccountId, TString> {
	pub desc: bool,
	pub from: u64,
	pub limit: u64,
	pub metadata_keys: Vec<TString>,
	pub owner: Option<TAccountId>,
	pub return_owners: bool,
	pub categories: Vec<Categories>,
	pub tags: Vec<TString>,
}

#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug)]
pub struct ProtoPatch<TBlockNumber> {
	/// The block when this patch was created
	pub block: TBlockNumber,
	/// The hash of this patch data
	pub data_hash: Hash256,
	/// A patch can add references to other protos.
	pub references: Vec<Hash256>,
}

#[derive(Default, Encode, Decode, Clone, scale_info::TypeInfo, Debug)]
pub struct TicketsInfo {
	pub active_tickets: u128,
	pub liftime_tickets: u128,
}

/// Struct of a Proto-Fragment
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug)]
pub struct Proto<TAccountId, TBlockNumber> {
	/// Block number this proto was included in
	pub block: TBlockNumber,
	/// Patches to this proto
	pub patches: Vec<ProtoPatch<TBlockNumber>>,
	/// Include price of the proto.
	/// If None, this proto can't be included into other protos
	pub include_cost: Option<Compact<u64>>,
	/// The original creator of the proto.
	pub creator: TAccountId,
	/// The current owner of the proto.
	pub owner: ProtoOwner<TAccountId>,
	/// References to other protos.
	pub references: Vec<Hash256>,
	/// Categories associated with this proto
	pub category: Categories,
	/// tags associated with this proto
	pub tags: Vec<Compact<u64>>,
	/// Metadata attached to the proto.
	pub metadata: BTreeMap<Compact<u64>, Hash256>,
	/// Tickets information for this proto.
	pub tickets_info: TicketsInfo,
}
```
### LinkSource
### LinkedAsset
### ProtoOwner
A proto can be owned by a `User`, which would be a native Fragnova account or an `ExternalAsset`.
An `ExternalAsset` can represent anything external that can be unequivocally identified.
#### Types of `ExternalAsset`
* Ethereum NFTs, ERC721s.
### GetProtosParams
**This is used only in the `get_protos` RPC call.**
### ProtoPatch
### TicketsInfo
### Proto
## Storage
```rust
#[pallet::storage]
	pub type Tags<T: Config> = StorageMap<_, Twox64Concat, Vec<u8>, u64>;

	#[pallet::storage]
	pub type TagsIndex<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	pub type MetaKeys<T: Config> = StorageMap<_, Twox64Concat, Vec<u8>, u64>;

	#[pallet::storage]
	pub type MetaKeysIndex<T: Config> = StorageValue<_, u64, ValueQuery>;

	/// Storage Map of Proto-Fragments where the key is the hash of the data of the Proto-Fragment, and the value is the Proto struct of the Proto-Fragment
	#[pallet::storage]
	pub type Protos<T: Config> =
		StorageMap<_, Identity, Hash256, Proto<T::AccountId, T::BlockNumber>>;

	/// Storage Map which keeps track of the Proto-Fragments by Category type.
	/// The key is the Category type and the value is a list of the hash of a Proto-Fragment
	// Not ideal but to have it iterable...
	#[pallet::storage]
	pub type ProtosByCategory<T: Config> = StorageMap<_, Twox64Concat, Categories, Vec<Hash256>>;

	/// UploadAuthorities is a StorageValue that keeps track of the set of ECDSA public keys of the upload authorities
	/// * Note: An upload authority (also known as the off-chain validator) provides the digital signature needed to upload a Proto-Fragment
	#[pallet::storage]
	pub type ProtosByOwner<T: Config> =
		StorageMap<_, Twox64Concat, ProtoOwner<T::AccountId>, Vec<Hash256>>;

	// Staking management
	// (Amount staked, Last stake time)
	#[pallet::storage]
	pub type ProtoStakes<T: Config> = StorageDoubleMap<
		_,
		Identity,
		Hash256,
		Twox64Concat,
		T::AccountId,
		(T::Balance, T::BlockNumber),
	>;

	#[pallet::storage]
	pub type AccountStakes<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Vec<Hash256>>;
```
### Tags
### TagsIndex
### MetaKeys
### MetaKeysIndex
### Protos
### ProtosByCategory
### ProtosByOwner
### ProtoStakes
### AccountStakes