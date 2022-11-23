import {ApiPromise} from "@polkadot/api";
import {connectToLocalNode} from "../index.js";
import Keyring from "@polkadot/keyring";
import {assert} from "chai";
import {KeyringPair} from "@polkadot/keyring/types";

const NORMAL_DISPATCH_RATIO = 0.75;
const MAXIMUM_BLOCK_LENGTH: number = 5 * 1024 * 1024;

// The extrinsic in question: https://polkadot.js.org/apps/#/extrinsics/decode/0xf5018400d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d0150cc530a8f70343680c46687ade61e1e4cdc0dfc6d916c3143828dc588938c1934030b530d9117001260426798d380306ea3a9d04fe7b525a33053a1c31bee86750200000b000000000000003448656c6c6f2c20576f726c6421
const ENCODED_EXTRINSIC_EXCL_DATA_PARAM: string = "f5018400d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d0150cc530a8f70343680c46687ade61e1e4cdc0dfc6d916c3143828dc588938c1934030b530d9117001260426798d380306ea3a9d04fe7b525a33053a1c31bee86750200000b00000000000000";
const EXTRINSIC_LENGTH_EXCL_DATA_PARAM: number = ENCODED_EXTRINSIC_EXCL_DATA_PARAM.length / 2;

// The data param (which is of type `Vec<u8>`) is prefixed with a Compact-encoding of the number of elements (https://docs.substrate.io/reference/scale-codec/).
// We are assuming it is Compact-encoded at "four-byte mode" (https://docs.substrate.io/reference/scale-codec/#fn-1) since
// 2**14 <= `NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_LENGTH - EXTRINSIC_LENGTH_EXCL_DATA_PARAM` <= 2**30.
//
// Therefore, the prefix will have a length of 4 bytes.
//
// Footnote: See https://substrate.stackexchange.com/questions/2830/compact-scale-decoding to understand how Compact-encoding works
const COMPACT_ENCODED_PREFIX_OF_DATA: number = 4;

describe("Testing Maximum Block Length for Normal Extrinsics", () => {

  let api: ApiPromise;
  let alice: KeyringPair;

  async function upload(caller: KeyringPair,
                        data: number[],
                        references=[],
                        category={text: "plain"},
                        tags=[],
                        linkedAsset=null,
                        license="Closed"): Promise<void> {

    const tx = api.tx.protos.upload(
      references,
      category,
      tags,
      linkedAsset,
      license,
      data
    );
    await tx.signAndSend(caller, ({ events = [], status }) => {
      console.log(`Transaction status: ${status.type}`);
      if (status.isInBlock) {
        console.log(`Included at block hash: ${status.asInBlock.toHex()}`);
        console.log('Events:');
        events.forEach(({ event: { data, method, section }, phase }) => {
          console.log(phase.toString());
          console.log(`${section}.${method}`);
          console.log(data.toString());
        });
      } else if (status.isFinalized) {
        console.log(`Finalized block hash: ${status.asFinalized.toHex()}`);
      }
    });

  };

  before(async function () {

    // the `beforeAll` hook should timeout after 20,000 ms (the default is 2000 ms). We do this to be safe
    this.timeout(200_000);

    api = await connectToLocalNode();

    const keyring = new Keyring({type: 'sr25519'});
    keyring.setSS58Format(93);
    alice = keyring.addFromUri('//Alice');

  });

  describe("protos.upload()", () => {

    it("should work if extrinsic call is ≤ `NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_LENGTH`", async function () {

      this.timeout(200_000);

      const data = [...new Uint8Array(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_LENGTH - EXTRINSIC_LENGTH_EXCL_DATA_PARAM - 10)];

      try {
        await upload(alice, data);
      } catch (e) {
        assert(false, `Proto-Fragment should have been uploaded successfully and NOT thrown the error: ${e}`,);
      }

    });

    it("should not work if extrinsic call is > `NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_LENGTH`", async function () {

      this.timeout(200_000);

      const data = [...new Uint8Array(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_LENGTH - EXTRINSIC_LENGTH_EXCL_DATA_PARAM - COMPACT_ENCODED_PREFIX_OF_DATA + 1)];


      try {
        await upload(alice, data);
        assert(false, "Proto-Fragment should NOT have been uploaded successfully!");
      } catch (e) { // the interface `RpcErrorInterface` of `e` is in `node_modules/@polkadot/rpc-provider/types.d.ts`
        assert.equal(e.code, 1010);
        assert.equal(e.data, "Transaction would exhaust the block limits");
        assert.equal(e.message, "1010: Invalid Transaction: Transaction would exhaust the block limits");
      }

    });

  });


});

