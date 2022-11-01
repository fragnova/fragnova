//! In Substrate, a chain specification is the collection of information that describes a Substrate-based blockchain network.
//! For example, the chain specification identifies the network that a blockchain node connects to,
//! the other nodes that it initially communicates with, and the initial state that nodes must agree on to produce blocks.
//!
//! The chain specification is defined using the ChainSpec struct. The ChainSpec struct separates the information required for a chain into two parts:
//!
//! - A client specification that contains information used by the Substrate outer node to communicate with network participants and send data to telemetry endpoints.
//!   Many of these chain specification settings can be overridden by command-line options when starting a node or can be changed after the blockchain has started.
//!
//! - The initial genesis state that all nodes in the network agree on.
//!   The genesis state must be established when the blockchain is first started and it cannot be changed thereafter without starting an entirely new blockchain.
//!
//! Source: https://docs.substrate.io/build/chain-spec/

// The imports from `clamor_runtime` that follow the pattern "<Pallet>Config" are the <Pallet>'s `GenesisConfig` struct
// See for more info: https://docs.substrate.io/reference/how-to-guides/basics/configure-genesis-state/
use clamor_runtime::{
	AccountId, AccountsConfig, AssetsConfig, AuraConfig, BalancesConfig, DetachConfig,
	GenesisConfig, GrandpaConfig, IndicesConfig, OracleConfig, Signature, SudoConfig, SystemConfig,
	WASM_BINARY,
};
use sc_service::ChainType;
use serde_json;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{ecdsa, ed25519, sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};

/// TODO: Documentation
pub type UploadId = ecdsa::Public;
/// TODO: Documentation
pub type EthId = ecdsa::Public;
/// TODO: Documentation
pub type DetachId = ed25519::Public;

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Generate a crypto pair from seed.
pub fn get_from_seed_to_eth(seed: &str) -> ecdsa::Public {
	let pair = ed25519::Pair::from_string(&format!("//{}", seed), None).unwrap();
	let mut message = b"fragments-frag-ecdsa-keys".to_vec();
	message.append(&mut pair.public().to_vec()); // salt it with the public key
	let signature = pair.sign(&message);
	let hash = sp_core::keccak_256(&signature.0[..]);
	let pair = ecdsa::Pair::from_seed_slice(&hash[..]).unwrap();
	let public = pair.public();
	public
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId, UploadId, EthId, DetachId) {
	(
		get_from_seed::<AuraId>(s),
		get_from_seed::<GrandpaId>(s),
		get_from_seed::<UploadId>(s),
		get_from_seed_to_eth(s),
		get_from_seed::<DetachId>(s),
	)
}

fn chain_spec_properties() -> serde_json::map::Map<String, serde_json::Value> {
	serde_json::json!({
		"ss58Format": 93,
		"tokenDecimals": 12,
		"tokenSymbol": "NOVA"
	})
	.as_object()
	.expect("Map given; qed")
	.clone()
}

/// Returns the `ChainSpec` struct used when for starting/joining a Clamor Development Network
pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice")],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
				],
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		None,
		// Properties
		Some(chain_spec_properties()),
		// Extensions
		None,
	))
}

/// Returns the `ChainSpec` struct used when for starting/joining a Clamor Testnet Network
pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
				],
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		None,
		// Properties
		Some(chain_spec_properties()),
		// Extensions
		None,
	))
}

/// Returns the `ChainSpec` struct used when for starting/joining a Clamor Mainnet Network
pub fn live_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Live Network",
		// ID
		"live",
		ChainType::Live,
		move || {
			GenesisConfig {
				system: SystemConfig {
					// Add Wasm runtime to storage.
					code: wasm_binary.to_vec(),
				},
				balances: BalancesConfig::default(),
				aura: AuraConfig::default(),
				grandpa: GrandpaConfig::default(),
				sudo: SudoConfig::default(),
				transaction_payment: Default::default(),
				indices: IndicesConfig { indices: vec![] },
				detach: DetachConfig::default(),
				assets: AssetsConfig::default(),
				accounts: AccountsConfig::default(),
				oracle: OracleConfig::default(),
			}
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		None,
		// Properties
		Some(chain_spec_properties()),
		// Extensions
		None,
	))
}

/// Configures the initial storage state for FRAME modules.
fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AuraId, GrandpaId, UploadId, EthId, DetachId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	_enable_println: bool,
) -> GenesisConfig {
	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		balances: BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, (1000000000u128 * (10u128.pow(12)))))
				.collect(),
		},
		aura: AuraConfig {
			authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
		},
		grandpa: GrandpaConfig {
			authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
		},
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(root_key.clone()),
		},
		transaction_payment: Default::default(),
		indices: IndicesConfig { indices: vec![] },
		detach: DetachConfig {
			eth_authorities: initial_authorities.iter().map(|x| (x.3.clone())).collect(),
			keys: initial_authorities.iter().map(|x| (x.4.clone())).collect(),
		},
		assets: AssetsConfig {
			assets: vec![(1337, root_key.clone(), true, 1, false)], // Genesis assets: id, owner, is_sufficient, min_balance, is_tradeable
			metadata: vec![(1337, "Fragnova Network Tickets".into(), "TICKET".into(), 0)], // Genesis metadata: id, name, symbol, decimals
			accounts: vec![], // Genesis accounts: id, account_id, balance
		},
		accounts: AccountsConfig {
			keys: initial_authorities.iter().map(|x| (x.4.clone())).collect(),
		},
		oracle: OracleConfig { keys: initial_authorities.iter().map(|x| (x.4.clone())).collect() },
	}
}
