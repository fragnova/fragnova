# Fragments Storage
#fragment #fragments
## Structs
```rust
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq)]
pub struct FragmentMetadata<TFungibleAsset> {
	pub name: Vec<u8>,
	pub currency: Option<TFungibleAsset>, // Where None is NOVA
}

#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq)]
pub struct UniqueOptions {
	pub mutable: bool,
}

/// Struct of a Fragment Class
#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq)]
pub struct FragmentClass<TFungibleAsset, TAccountId, TBlockNum> {
	/// The Proto-Fragment that was used to create this Fragment Class
	pub proto_hash: Hash256,
	/// The metadata of the Fragment Class
	pub metadata: FragmentMetadata<TFungibleAsset>,
	/// The next owner permissions
	pub permissions: FragmentPerms,
	/// If Fragments must contain unique data when created (injected by buyers, validated by the system)
	pub unique: Option<UniqueOptions>,
	/// If scarce, the max supply of the Fragment
	pub max_supply: Option<Compact<Unit>>,
	/// The creator of this class
	pub creator: TAccountId,
	/// The block number when the item was created
	pub created_at: TBlockNum,
}

#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq)]
pub struct InstanceData<TBlockNum> {
	/// Next owner permissions, owners can change those if they want to more restrictive ones, never more permissive
	pub permissions: FragmentPerms,
	/// The block number when the item was created
	pub created_at: TBlockNum,
	/// Custom data, if unique, this is the hash of the data that can be fetched using bitswap directly on our nodes
	pub custom_data: Option<Hash256>,
	/// Expiring at block, if not None, the item will be removed after this date
	pub expiring_at: Option<TBlockNum>,
}

#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq)]
pub struct PublishingData<TBlockNum> {
	pub price: Compact<u128>,
	pub units_left: Option<Compact<Unit>>,
	pub expiration: Option<TBlockNum>,
}

#[derive(Encode, Decode, Clone, scale_info::TypeInfo, Debug, PartialEq)]
pub enum FragmentBuyOptions {
	Quantity(u64),
	UniqueData(Vec<u8>),
}
```
### FragmentMetadata
Proto-hash + this metadata compose the Fragment class unique id.
#### Remarks
* #immutable - once created there is no way to edit, intentionally.
### FragmentClass
Fragments classes are the DNA of fragments. This is the way to program fragments distribution, expiration and starting permissions before even creating any fragment yet.
#### Remarks
* #immutable - once created there is no way to edit, intentionally.
### InstanceData
#### Remarks
* On purpose not storing owner because:
  * Big, 32 bytes
  * Most of use cases will definitely already have the owner available when using this structure, as likely going thru `Inventory` etc.
### PublishingData
### FragmentBuyOptions
When buying fragments if they are not unique, and so there is no need to have extra data attached, users will be able to buy in bulk. If not this will be the data, which is indexed and fully stored #immutable on chain for IPFS retrieval.
## Storage Mapping
```rust
// proto-hash to fragment-hash-sequence
/// Storage Map that keeps track of the number of Fragments that were created using a Proto-Fragment.
/// The key is the hash of the Proto-Fragment, and the value is the list of hash of the Fragments
#[pallet::storage]
pub type Proto2Fragments<T: Config> = StorageMap<_, Identity, Hash256, Vec<Hash128>>;

// fragment-hash to fragment-data
/// Storage Map of Fragments where the key is the hash of the concatenation of its corresponding Proto-Fragment and the name of the Fragment, and the value is the Fragment struct of the Fragment
#[pallet::storage]
pub type Classes<T: Config> =
	StorageMap<_, Identity, Hash128, FragmentClass<T::AssetId, T::AccountId>>;

#[pallet::storage]
pub type Publishing<T: Config> =
	StorageMap<_, Identity, Hash128, PublishingData<T::BlockNumber>>;

#[pallet::storage]
pub type EditionsCount<T: Config> = StorageMap<_, Identity, Hash128, Compact<Unit>>;

#[pallet::storage]
pub type CopiesCount<T: Config> =
	StorageMap<_, Twox64Concat, (Hash128, Compact<u64>), Compact<Unit>>;

#[pallet::storage]
pub type Fragments<T: Config> = StorageNMap<
	_,
	// Keys are using Identity for compression, as we deteministically create fragments
	(
		storage::Key<Identity, Hash128>,
		// Editions
		storage::Key<Identity, Unit>,
		// Copies
		storage::Key<Identity, Unit>,
	),
	InstanceData<T::BlockNumber>,
>;

#[pallet::storage]
pub type Owners<T: Config> = StorageDoubleMap<
	_,
	Identity,
	Hash128,
	Twox64Concat,
	T::AccountId,
	Vec<(Compact<Unit>, Compact<Unit>)>,
>;

#[pallet::storage]
pub type Inventory<T: Config> = StorageDoubleMap<
	_,
	Twox64Concat,
	T::AccountId,
	Identity,
	Hash128,
	Vec<(Compact<Unit>, Compact<Unit>)>,
>;

#[pallet::storage]
pub type Expirations<T: Config> =
	StorageMap<_, Twox64Concat, T::BlockNumber, Vec<(Hash128, Compact<Unit>, Compact<Unit>)>>;
```
### Proto2Fragments
Self-explanatory, a way to find all the fragments made out of a proto.
### Classes
### Publishing
### EditionsCount
### CopiesCount
### Fragments
#### Keys hashing reasoning
Very long key, means takes a lot of redundant storage (because we will have **many** Instances!), we try to limit the  damage by using `Identity` so that the final key will be:
`[16 bytes of Fragment class hash]+[8 bytes of u64, edition]+[8 bytes of u64, copy id]` for a total of 32 bytes.
### Owners
*Notice this pulls from memory (and deserializes (scale)) the whole `Vec<_,_>`, this is on purpose as it should not be too big.*

A shortcut to map from Class to owners.
### Inventory
*Notice this pulls from memory (and deserializes (scale)) the whole `Vec<_,_>`, this is on purpose as it should not be too big.*

A shortcut to map from owners to Class and finally instances.
### Expirations
Fragments can expire, we process expirations every `on_finalize`
