const { ApiPromise, WsProvider } = require("@polkadot/api");
const { Keyring } = require('@polkadot/keyring');

const assert = require("chai").assert;

describe("Extrinsics Per Block", () => {

  describe("3 `contracts.uploadCode()` extrinsics of length 3 MiB each should execute in the same block", () => {
    it("should work", async function () {

      // the `it` hook should timeout after 20,000 ms (the default is 2000 ms). We do this to be safe
      this.timeout(200_000);

      const api = await ApiPromise.create({
        provider: new WsProvider("ws://127.0.0.1:9944")
      });
      const keyring = new Keyring({type: 'sr25519'});
      keyring.setSS58Format(93);
      const alice = keyring.addFromUri('//Alice');

      const blockHashes = await new Promise((resolve, reject) => {

        const blockHashes = [];

        const totalIterations = 3;
        for (let nonce = 0; nonce < totalIterations; nonce++) {

          const data = [...new Uint8Array(3 * 1024 * 1024).fill(nonce)];

          api.tx.contracts.uploadCode(
            data,
            null,
            "Deterministic"
          ).signAndSend(alice, {nonce: nonce}, (result) => {
            console.log('Transaction status:', result.status.type);

            if (result.status.isInBlock) {
              console.log('Included at block hash', result.status.asInBlock.toHex());
              blockHashes.push(result.status.asInBlock.toHex());
              if (blockHashes.length == totalIterations) {
                resolve(blockHashes);
              }
            } else if (result.status.isFinalized) {
              console.log('Finalized block hash', result.status.asFinalized.toHex());
            }
          });
        }
      });

      assert(blockHashes.every(val => val === blockHashes[0]), "All the extrinsics were not executed in the same block 😢");

    });
  });


});
