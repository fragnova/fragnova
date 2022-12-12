use crate::*;

use pallet_detach::SupportedChains;

// pub use pallet_protos::dummy_data::ProtoFragment;
pub use copied_from_pallet_protos::ProtoFragment;

mod copied_from_pallet_protos {

	use super::*;

	pub fn compute_data_hash(data: &Vec<u8>) -> Hash256 {
		blake2_256(&data)
	}

	// use base58::ToBase58;
	// pub fn compute_data_cid(data: &Vec<u8>) -> Vec<u8> {

	// 	let hash = compute_data_hash(data);

	// 	let cid = [&CID_PREFIX[..], &hash[..]].concat();
	// 	let cid = cid.to_base58();
	// 	let cid = [&b"z"[..], cid.as_bytes()].concat();

	// 	cid
	// }

	#[derive(Clone)]
	pub struct ProtoFragment {
		pub references: Vec<Hash256>,
		pub category: Categories,
		pub tags: Vec<Vec<u8>>,
		pub linked_asset: Option<pallet_protos::LinkedAsset>,
		pub include_cost: Option<u64>,
		pub data: Vec<u8>,
	}
	impl ProtoFragment {
		pub fn get_proto_hash(&self) -> Hash256 {
			compute_data_hash(&self.data)
		}

		// pub fn get_proto_cid(&self) -> Vec<u8> {
		// 	compute_data_cid(&self.data)
		// }
	}
}

use protos::permissions::FragmentPerms;

use sp_clamor::Hash256;

use protos::categories::{Categories, TextCategories};

#[derive(Clone)]
pub struct Definition {
	// "Definition" is short for "Fragment Definition"
	pub proto_fragment: ProtoFragment,

	pub metadata: DefinitionMetadata<u64>,
	pub permissions: FragmentPerms,

	pub unique: Option<UniqueOptions>,

	pub max_supply: Option<u64>,
}

impl Definition {
	pub fn get_definition_id(&self) -> Hash128 {
		blake2_128(
			&[
				&self.proto_fragment.get_proto_hash()[..],
				&self.metadata.name.encode(),
				&self.metadata.currency.encode(),
			]
				.concat(),
		)
	}

	pub fn get_vault_account_id(&self) -> sp_core::ed25519::Public {
		let hash = blake2_256(&[&b"fragments-vault"[..], &self.get_definition_id()].concat());
		sp_core::ed25519::Public::from_raw(hash)
	}
}

#[derive(Clone)]
pub struct Publish {
	pub definition: Definition,

	pub price: u128, // price per instance

	/// * `quantity` (*optional*) - **Maximum amount of Fragment Instances** that **can be bought**
	pub quantity: Option<u64>,

	pub expires: Option<u64>,

	/// If the Fragment instance represents a **stack of stackable items** (for e.g gold coins or arrows - https://runescape.fandom.com/wiki/Stackable_items),
	/// the **number of items** to **top up** in the **stack of stackable items**
	pub stack_amount: Option<u64>,
}

#[derive(Clone)]
pub struct Mint {
	pub definition: Definition,
	pub buy_options: FragmentBuyOptions,
	pub amount: Option<u64>,
}

impl Mint {
	pub fn get_quantity(&self) -> u64 {
		match &self.buy_options {
			FragmentBuyOptions::Quantity(q) => q.clone(),
			FragmentBuyOptions::UniqueData(_) => 1,
		}
	}
}

#[derive(Clone)]
pub struct Buy {
	pub publish: Publish,
	pub buy_options: FragmentBuyOptions,
}

#[derive(Clone)]
pub struct Give {
	pub mint: Mint,
	pub edition_id: InstanceUnit,
	pub copy_id: InstanceUnit,
	pub to: sp_core::ed25519::Public,
	pub new_permissions: Option<FragmentPerms>,
	pub expiration: Option<u64>,
}

pub struct CreateAccount {
	// Creates Account for a Fragment Instance
	pub mint: Mint,
	pub edition_id: InstanceUnit,
	pub copy_id: InstanceUnit,
}

#[derive(Clone)]
pub struct Resell {
	pub mint: Mint,
	pub edition_id: InstanceUnit,
	pub copy_id: InstanceUnit,
	pub new_permissions: Option<FragmentPerms>,
	pub expiration: Option<u64>,
	pub secondary_sale_type: SecondarySaleType,
}
#[derive(Clone)]
pub struct EndResale {
	pub resell: Resell
}
#[derive(Clone)]
pub struct SecondaryBuy {
	pub resell: Resell,
	pub options: SecondarySaleBuyOptions
}

pub struct Detach {
	pub mint: Mint,
	pub edition_id: u64,
	pub copy_id: u64,
	pub target_chain: SupportedChains,
	pub target_account: Vec<u8>,
}

/// NOTE: All `ProtoFragment`-type fields found in `DummyData` have no references
pub struct DummyData {
	pub definition: Definition,

	pub publish: Publish,
	pub publish_with_max_supply: Publish,

	pub mint_non_unique: Mint,
	pub mint_unique: Mint,
	pub mint_non_unique_with_max_supply: Mint,

	pub buy_non_unique: Buy,
	pub buy_unique: Buy,
	pub buy_non_unique_with_limited_published_quantity: Buy,

	pub give_no_copy_perms: Give,
	pub give_copy_perms: Give,

	pub create_account: CreateAccount,

	pub resell_normal: Resell,

	pub end_resale: EndResale,

	pub secondary_buy: SecondaryBuy,
	pub secondary_buy_no_copy_perms: SecondaryBuy,
	pub secondary_buy_copy_perms: SecondaryBuy,

	pub detach: Detach,

	pub account_id: sp_core::ed25519::Public,
	pub account_id_second: sp_core::ed25519::Public,
	pub account_id_third: sp_core::ed25519::Public,
}

impl DummyData {
	pub fn new() -> Self {
		let definition = Definition {
			proto_fragment: ProtoFragment {
				references: Vec::new(),
				category: Categories::Text(TextCategories::Plain),
				tags: Vec::new(),
				linked_asset: None,
				include_cost: Some(111),
				data: "0x111".as_bytes().to_vec(),
			},
			metadata: DefinitionMetadata { name: b"Il Nome".to_vec(), currency: Currency::Native },
			permissions: FragmentPerms::EDIT | FragmentPerms::TRANSFER,
			unique: Some(UniqueOptions { mutable: true }),
			max_supply: None,
		};
		let definition_unique = definition.clone();
		let definition_non_unique = Definition { unique: None, ..definition.clone() };

		let publish = Publish {
			definition: definition.clone(),
			price: 2,
			quantity: None,
			expires: None,
			stack_amount: None,
		};
		let publish_with_max_supply = Publish {
			definition: Definition {
				max_supply: Some(1234), // with max supply!
				..definition.clone()
			},
			..publish.clone()
		};

		let mint_non_unique = Mint {
			definition: definition_non_unique.clone(),
			buy_options: FragmentBuyOptions::Quantity(1), // 1 ensures `quantity` is never above `definition.max_supply`!
			amount: None,
		};
		let mint_unique = Mint {
			definition: definition_unique.clone(),
			buy_options: FragmentBuyOptions::UniqueData(b"I Dati".to_vec()),
			amount: None,
		};
		let mint_non_unique_with_max_supply: Mint = {
			let mut mint_non_unique = mint_non_unique.clone();
			mint_non_unique.definition.max_supply = Some(1234);
			mint_non_unique
		};

		let buy_non_unique = Buy {
			publish: Publish { definition: definition_non_unique.clone(), ..publish.clone() },
			buy_options: FragmentBuyOptions::Quantity(1), // 1 ensures `quantity` is never above `definition.max_supply`!
		};
		let buy_unique = Buy {
			publish: Publish { definition: definition_unique.clone(), ..publish.clone() },
			buy_options: FragmentBuyOptions::UniqueData(b"I Dati".to_vec()),
		};
		let buy_non_unique_with_limited_published_quantity: Buy = {
			let mut buy_non_unique = buy_non_unique.clone();
			buy_non_unique.publish.quantity = Some(1234); // with limited published quantity
			buy_non_unique
		};

		let give_no_copy_perms = Give {
			mint: Mint {
				definition: Definition {
					permissions: FragmentPerms::TRANSFER, // no copy perms
					..mint_unique.definition.clone()
				},
				..mint_unique.clone()
			},
			edition_id: 1,
			copy_id: 1,
			to: sp_core::ed25519::Public::from_raw([69u8; 32]),
			new_permissions: Some(FragmentPerms::NONE),
			expiration: Some(999),
		};
		let give_copy_perms = Give {
			mint: Mint {
				definition: Definition {
					permissions: FragmentPerms::TRANSFER | FragmentPerms::COPY, // copy perms
					..mint_unique.definition.clone()
				},
				..mint_unique.clone()
			},
			edition_id: 1,
			copy_id: 1,
			to: sp_core::ed25519::Public::from_raw([69u8; 32]),
			new_permissions: Some(FragmentPerms::NONE),
			expiration: Some(999),
		};

		let create_account = CreateAccount { mint: mint_unique.clone(), edition_id: 1, copy_id: 1 };

		let resell_normal = Resell {
			mint: mint_unique.clone(),
			edition_id: 1,
			copy_id: 1,
			new_permissions: Some(FragmentPerms::NONE),
			expiration: Some(999),
			secondary_sale_type: SecondarySaleType::Normal(777),
		};

		let end_resale = EndResale {
			resell: resell_normal.clone(),
		};

		let secondary_buy = SecondaryBuy {
			resell: resell_normal.clone(),
			options: SecondarySaleBuyOptions::Normal,
		};

		let secondary_buy_no_copy_perms = {
			let mut secondary_buy = secondary_buy.clone();
			secondary_buy.resell.mint.definition.permissions = FragmentPerms::TRANSFER; // no copy perms
			secondary_buy
		};
		let secondary_buy_copy_perms = {
			let mut secondary_buy = secondary_buy.clone();
			secondary_buy.resell.mint.definition.permissions = FragmentPerms::TRANSFER | FragmentPerms::COPY; // no copy perms
			secondary_buy
		};

		let detach = Detach {
			mint: mint_unique.clone(),
			edition_id: 1,
			copy_id: 1,
			target_chain: SupportedChains::EthereumMainnet,
			target_account: [7u8; 20].to_vec(),
		};

		Self {
			definition,

			publish,
			publish_with_max_supply,

			mint_non_unique,
			mint_unique,
			mint_non_unique_with_max_supply,

			buy_non_unique,
			buy_unique,
			buy_non_unique_with_limited_published_quantity,

			give_no_copy_perms,
			give_copy_perms,

			create_account,

			resell_normal,

			end_resale,

			secondary_buy,
			secondary_buy_no_copy_perms,
			secondary_buy_copy_perms,

			detach,

			account_id: sp_core::ed25519::Public::from_raw([1u8; 32]),
			account_id_second: sp_core::ed25519::Public::from_raw([2u8; 32]),
			account_id_third: sp_core::ed25519::Public::from_raw([3u8; 32]),
		}
	}
}
