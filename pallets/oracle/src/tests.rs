use crate as pallet_oracle;
use crate::*;
use codec::Decode;
use ethabi::Token;
use frame_support::dispatch::DispatchResult;
use frame_support::inherent::BlockT;
use frame_support::{
	assert_ok, parameter_types,
	traits::{ConstU32, ConstU64},
};
use parking_lot::RwLock;
use sp_core::offchain::testing::{OffchainState, PoolState};
use sp_core::offchain::OffchainDbExt;
use sp_core::{
	ed25519::Signature,
	offchain::{testing, OffchainWorkerExt, TransactionPoolExt},
	H256, U256,
};
use std::sync::Arc;

use sp_keystore::{testing::KeyStore, KeystoreExt, SyncCryptoStore};
use sp_runtime::offchain::storage::StorageValueRef;
use sp_runtime::{
	testing::{Header, TestXt},
	traits::{BlakeTwo256, Extrinsic as ExtrinsicT, IdentifyAccount, IdentityLookup, Verify},
	RuntimeAppPublic,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// For testing the module, we construct a mock runtime.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Oracle: pallet_oracle::{Pallet, Call, Event<T>, ValidateUnsigned},
	}
);

parameter_types! {
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(frame_support::weights::Weight::from_ref_time(1024));
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
	pub StorageBytesMultiplier: u64 = 10;
	pub const IsTransferable: bool = false;
}
impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = sp_core::ed25519::Public;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = ConstU32<2>;
}

type Extrinsic = TestXt<RuntimeCall, ()>;
type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

impl frame_system::offchain::SigningTypes for Test {
	type Public = <Signature as Verify>::Signer;
	type Signature = Signature;
}

impl<LocalCall> frame_system::offchain::SendTransactionTypes<LocalCall> for Test
where
	RuntimeCall: From<LocalCall>,
{
	type Extrinsic = Extrinsic;
	type OverarchingCall = RuntimeCall;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Test
where
	RuntimeCall: From<LocalCall>,
{
	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
		call: RuntimeCall,
		_public: <Signature as Verify>::Signer,
		_account: AccountId,
		nonce: u64,
	) -> Option<(RuntimeCall, <Extrinsic as ExtrinsicT>::SignaturePayload)> {
		Some((call, (nonce, ())))
	}
}

impl Config for Test {
	type AuthorityId = crypto::FragAuthId;
	type RuntimeEvent = RuntimeEvent;
	type OracleContract = Test;
	type Threshold = ConstU64<1>;
}

impl OracleContract for Test {
	fn get_contract() -> &'static str {
		// https://docs.chain.link/docs/data-feeds/price-feeds/addresses/
		"0x547a514d5e3769680Ce22B2361c10Ea13619e8a9" // AAVE / USD on mainnet (this is just for testing purposes)
	}
}

fn test_pub() -> sp_core::sr25519::Public {
	sp_core::sr25519::Public::from_raw([1u8; 32])
}

fn hardcode_expected_request_and_response(state: &mut testing::OffchainState) {
	let geth_url = Some(String::from("https://www.dummywebsite.com/"));

	sp_clamor::init(geth_url);

	// example of response taken from ETH/BTC in mainnet
	/*
	curl --url https://mainnet.infura.io/v3/48a1226dccb4437f9f89005e62140779 -X POST -H "Content-Type: application/json" \
		-d '{"jsonrpc": 2,"method": "eth_call","params": [{"to": "0xAc559F25B1619171CbC396a50854A3240b6A4e99","data": "0xfeaf968c0000000000000000000000000000000000000000000000000000000000000000"},"latest"],"id":1}'
		{"jsonrpc":"2.0","id":1,
		"result":"0x00000000000000000000000000000000000000000000000100000000000025a20000000000000000000000000000000000000000000000000000000000762157000000000000000000000000
	*/
	state.expect_request(testing::PendingRequest {
		method: String::from("POST"),
		uri: String::from_utf8(sp_clamor::clamor::get_geth_url().unwrap()).unwrap(),
		headers: vec![(String::from("Content-Type"), String::from("application/json"))],
		body: json!({
				"jsonrpc": "2.0",
				"method": "eth_call", // https://ethereum.org/en/developers/docs/apis/json-rpc/#eth_call
				"params": [
					{
					"to": <
						<Test as pallet_oracle::Config>::OracleContract as pallet_oracle::OracleContract
					>::get_contract(),
					// first 4 bytes of keccak_256(latestRoundData()) function, padded - Use https://emn178.github.io/online-tools/keccak_256.html
					"data": "0xfeaf968c0000000000000000000000000000000000000000000000000000000000000000",
					},
					"latest"
				],
				 "id": 5, //goerli
			})
			.to_string()
			.into_bytes(),
		response: Some(
			json!({
					"id": 5,
					"jsonrpc": "2.0",
					"result": format!("0x{}", hex::encode(
								ethabi::encode(
									&[ Token::Tuple(vec![
											Token::Uint(U256::from(123)), //roundId: The round ID.
											Token::Int(U256::from(10000000)), //answer: The price.
											Token::Uint(U256::from(1667)), // startedAt: Timestamp of when the round started.
											Token::Uint(U256::from(1668)),// updatedAt: Timestamp of when the round was updated
											Token::Uint(U256::from(124)),// answeredInRound: The round ID of the round in which the answer was computed.
											])
									]
								),
							))
				})
				.to_string()
				.into_bytes(),
		),
		sent: true,
		..Default::default()
	});
}

pub fn new_test_ext_with_ocw() -> (
	sp_io::TestExternalities,
	Arc<RwLock<PoolState>>,
	Arc<RwLock<OffchainState>>,
	sp_core::ed25519::Public,
) {
	const PHRASE: &str =
		"news slush supreme milk chapter athlete soap sausage put clutch what kitten";

	let (offchain, offchain_state) = testing::TestOffchainExt::new();
	let (pool, pool_state) = testing::TestTransactionPoolExt::new();

	let keystore = KeyStore::new();

	SyncCryptoStore::ed25519_generate_new(
		&keystore,
		<crate::crypto::Public as RuntimeAppPublic>::ID,
		Some(&format!("{}", PHRASE)),
	)
	.unwrap();

	let ed25519_public_key =
		SyncCryptoStore::ed25519_public_keys(&keystore, crate::crypto::Public::ID)
			.get(0)
			.unwrap()
			.clone();

	let mut t = sp_io::TestExternalities::default();
	t.register_extension(OffchainDbExt::new(offchain.clone()));
	t.register_extension(OffchainWorkerExt::new(offchain));
	t.register_extension(TransactionPoolExt::new(pool));
	t.register_extension(KeystoreExt(Arc::new(keystore)));

	t.execute_with(|| System::set_block_number(1));
	(t, pool_state, offchain_state, ed25519_public_key)
}

pub fn store_price_(
	oracle_price: OraclePrice<
		<Test as SigningTypes>::Public,
		<Test as frame_system::Config>::BlockNumber,
	>,
) -> DispatchResult {
	Oracle::store_price(
		RuntimeOrigin::none(),
		oracle_price,
		sp_core::ed25519::Signature([69u8; 64]), // this can be anything
	)
}

#[test]
fn fetch_price_from_oracle_should_work() {
	let (mut t, pool_state, offchain_state, ed25519_public_key) = new_test_ext_with_ocw();

	hardcode_expected_request_and_response(&mut offchain_state.write());

	t.execute_with(|| {
		Oracle::fetch_price_from_oracle(1);

		let expected_data = OraclePrice {
			round_id: U256::from(123),
			price: U256::from(10000000),
			started_at: U256::from(1667),
			updated_at: U256::from(1668),
			answered_in_round: U256::from(124),
			block_number: System::block_number(),
			public: <Test as SigningTypes>::Public::from(ed25519_public_key),
		};

		let tx = pool_state.write().transactions.pop().unwrap();
		let tx = <Extrinsic as codec::Decode>::decode(&mut &*tx).unwrap();
		assert_eq!(tx.signature, None); // Because it's an **unsigned transaction** with a signed payload

		if let RuntimeCall::Oracle(crate::Call::store_price { oracle_price, signature }) = tx.call {
			assert_eq!(oracle_price.round_id, expected_data.round_id);
			assert_eq!(oracle_price.price, expected_data.price);
			assert_eq!(oracle_price.started_at, expected_data.started_at);
			assert_eq!(oracle_price.updated_at, expected_data.updated_at);
			assert_eq!(oracle_price.answered_in_round, expected_data.answered_in_round);
			assert_eq!(oracle_price.block_number, expected_data.block_number);
			assert_eq!(oracle_price.public, expected_data.public);

			let signature_valid =
				<OraclePrice<
					<Test as SigningTypes>::Public,
					<Test as frame_system::Config>::BlockNumber,
				> as SignedPayload<Test>>::verify::<crypto::FragAuthId>(&oracle_price, signature); // Notice in `pallet_accounts` that `EthLockUpdate<T::Public>` implements the trait `SignedPayload`

			assert!(signature_valid); // If `signature_valid` is true, it means `payload` and `signature` recovered the public address `data.public`
		}
	});
}

#[test]
fn fetch_from_oracle_should_work() {
	new_test_ext().execute_with(|| {
		let expected_data = OraclePrice {
			round_id: U256::from(123),
			price: U256::from(10000000),
			started_at: U256::from(1667),
			updated_at: U256::from(1668),
			answered_in_round: U256::from(124),
			block_number: System::block_number(),
			public: sp_core::ed25519::Public([69u8; 32]),
		};

		assert_ok!(store_price_(expected_data.clone()));
		let event = <frame_system::Pallet<Test>>::events()
			.pop()
			.expect("Expected one EventRecord to be found")
			.event;

		let price: u32 = expected_data.clone().price.try_into().unwrap();
		let block_number = expected_data.clone().block_number;

		assert_eq!(event, RuntimeEvent::from(Event::NewPrice { price, block_number }));
		assert_eq!(<Price<Test>>::get(), price);
	});
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	let mut ext = sp_io::TestExternalities::new(t);

	ext.execute_with(|| System::set_block_number(1)); // if we don't execute this line, Events are not emitted from extrinsics (I don't know why this is the case though)

	ext
}
