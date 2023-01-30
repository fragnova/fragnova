const index = require("../index");
const assert = require("chai").assert;
const { Keyring } = require('@polkadot/keyring');
const ethSignUtil = require('@metamask/eth-sig-util');
var Web3 = require('web3');


let api;
const web3 = new Web3(null);

const createEip712TypedDataForLinking = (chainId, verifyingContract, genesisBlockHash, fragnovaAccountIdHex) => {
  const typedData = {
    domain: {
      name: 'Fragnova Network',
      version: '1',
      chainId: chainId,
      verifyingContract: verifyingContract
    },
    message: {
      fragnovaGenesis: genesisBlockHash,
      op: "link",
      sender: fragnovaAccountIdHex,
    },
    primaryType:'Msg',
    types: {
      EIP712Domain: [
        {name: 'name', type: 'string'},
        {name: 'version', type: 'string'},
        {name: 'chainId', type: 'uint256'},
        {name: 'verifyingContract', type: 'address'}
      ],
      Msg: [
        {name: "fragnovaGenesis", type: "string"},
        {name: "op", type: "string"},
        {name: "sender", type: "string"},
      ]
    },
  };
  return typedData;
};

describe("Connection", () => {

  let alice;

  before(async function () {

    // the `beforeAll` hook should timeout after 20,000 ms (the default is 2000 ms). We do this to be safe
    this.timeout(200_000);

    api = await index.connectToLocalNode();

    const keyring = new Keyring({type: 'sr25519'});
    keyring.setSS58Format(93);
    alice = keyring.addFromUri('//Alice');

  });

  describe('accounts.link()', () => {

    it("should work", async function () {

      // the `it` hook should timeout after 20,000 ms (the default is 2000 ms). We do this because it takes time to get the nonce and then call `accounts.link()`
      this.timeout(200_000);

      const callLinkAndAssertEventEmitted = async (caller, nonce, signature, correctEthereumAddress) => {

        return new Promise((resolve, reject) => {
          // Inspired from https://polkadot.js.org/docs/api/examples/promise/transfer-events/
          api.tx.accounts.link(signature).signAndSend(
            caller,
            {nonce},
            ({events = [], status}) => {
              console.log('Transaction status:', status.type);

              if (status.isInBlock) {
                console.log('Included at block hash', status.asInBlock.toHex());
                console.log('Events:');

                events.forEach(({event: {data, method, section}, phase}) => {
                  console.log('\t', phase.toString(), `: ${section}.${method}`, data.toString());
                  if (section === "accounts" && method === "Linked" && data.toString() === JSON.stringify([caller.address, correctEthereumAddress])) {
                    resolve();
                  }
                });

              } else if (status.isFinalized) {
                console.log('Finalized block hash', status.asFinalized.toHex());

                reject(); // if the `Linked` event was emitted, `resolve()` would have been called. Therefore, this line would never have been executed
              }
            });
        });

      }

      const chainId = 5;
      const verifyingContract = "0xf5a0af5a0af5a0af5a0af5a0af5a0af5a0af5a0a";
      const genesisBlockHash = api.genesisHash.toHex();
      const aliceHex = `0x${Buffer.from(alice.publicKey).toString('hex')}`;
      let typedData = createEip712TypedDataForLinking(chainId, verifyingContract, genesisBlockHash, aliceHex);
      const ecdsaPrivateKey = new Buffer.from("4f3adf983ac636a65a842ce7c78d9aa706d3b113bce9c46f30d7d21715b23b1d", 'hex');
      const signature = ethSignUtil.signTypedData({privateKey: ecdsaPrivateKey, data: typedData, version: ethSignUtil.SignTypedDataVersion.V4});

      const { nonce } = await api.query.system.account(alice.publicKey);

      let ethereumAddress = web3.eth.accounts.privateKeyToAccount(ecdsaPrivateKey.toString('hex')).address.toLowerCase();

      try {
        await callLinkAndAssertEventEmitted(alice, nonce, signature, ethereumAddress);
      } catch (e) {
        assert(false, "the `Linked` Event was not emitted, Monsieur!");
      }

    });

  });


});
