// const {ApiPromise} = require("@polkadot/api");
// const { Vec, U8aFixed } = require('@polkadot/types-codec');
//
// const { Keyring } = require('@polkadot/keyring');
//
// const { createFragnovaApi } = require("@fragnova/api");
// const fs = require("fs").promises;
// const assert = require("chai").assert;
//
// describe("Transactions Per Second", () => {
//
//   let alice;
//
//   before(async function () {
//
//     // the `beforeAll` hook should timeout after 20,000 ms (the default is 2000 ms). We do this to be safe
//     this.timeout(200_000);
//
//     const keyring = new Keyring({type: 'sr25519'});
//     keyring.setSS58Format(93);
//     alice = keyring.addFromUri('//Alice');
//
//   });
//
//   describe("All `protos.upload()` transactions (of images) should execute in the same block", () => {
//     it("should work", async function () {
//
//       // the `it` hook should timeout after 20,000 ms (the default is 2000 ms). We do this to be safe
//       this.timeout(200_000);
//
//       const api = await createFragnovaApi("ws://127.0.0.1:9944");
//
//       const blockHashes = await new Promise((resolve, reject) => {
//
//         const blockHashes = [];
//
//         const totalIterations = 10;
//         for (let nonce = 0; nonce < totalIterations; nonce++) {
//
//           const data = [...new Uint8Array(3 * 1024 * 1024).fill(nonce)];
//
//           api.tx.protos.upload(
//             [],
//             {Binary: 'BlendFile'},
//             [],
//             null,
//             "Closed",
//             null,
//             {Local: data}
//           ).signAndSend(alice, {nonce: nonce}, (result) => {
//             console.log('Transaction status:', result.status.type);
//
//             if (result.status.isInBlock) {
//               console.log('Included at block hash', result.status.asInBlock.toHex());
//             } else if (result.status.isFinalized) {
//               console.log('Finalized block hash', result.status.asFinalized.toHex());
//               blockHashes.push(result.status.asFinalized.toHex());
//               if (blockHashes.length == totalIterations) {
//                 resolve(blockHashes);
//               }
//             }
//           });
//         }
//       });
//
//       assert(blockHashes.every(val => val === blockHashes[0]), "All the transactions were not executed in the same block 😢");
//
//     });
//   });
//
//
// });
