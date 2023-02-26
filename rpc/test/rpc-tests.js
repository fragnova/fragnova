const assert = require("chai").assert;
const index = require("../index");
const { Keyring } = require('@polkadot/keyring');
const {blake2AsU8a} = require('@polkadot/util-crypto');
const fs = require('fs').promises;

let api;

const sleep = ms => new Promise(r => setTimeout(r, ms));

const upload = async (signer, data, references=[], category={text: "plain"}, tags=[], linkedAsset=null, license="Closed", cluster=null) => {
  const txHash = await api.tx.protos.upload(
    references,
    category,
    tags,
    linkedAsset,
    license,
    cluster,
    {local: data},
  ).signAndSend(signer);
  console.log(arguments.callee.name, 'sent with transaction hash', txHash.toHex());
  await sleep(6000);
  return blake2AsU8a(data);
};
const setMetadata = async (signer, protoHash, metadata_key, data) => {
  const txHash = await api.tx.protos.setMetadata(
    protoHash,
    metadata_key,
    data
  ).signAndSend(signer);
  console.log(arguments.callee.name, 'sent with transaction hash', txHash.toHex());
  await sleep(6000);
  return blake2AsU8a(data);
};
const create = async (signer, protoHash, metadata, permissions={"bits": 0b111}, unique=null, max_supply=null) => {
  const txHash = await api.tx.fragments.create(
    protoHash,
    metadata,
    permissions,
    unique,
    max_supply
  ).signAndSend(signer);
  console.log(arguments.callee.name, 'sent with transaction hash', txHash.toHex());
  await sleep(6000);
  return blake2AsU8a([...protoHash, ...metadata.toU8a()], 128);
};
const mint = async (signer, definitionHash, options, amount=null) => {
  const txHash = await api.tx.fragments.mint(
    definitionHash,
    options,
    amount
  ).signAndSend(signer);
  console.log(arguments.callee.name, 'sent with transaction hash', txHash.toHex());
  await sleep(6000);
};

describe("RPCs", () => {

  let protoOwner;
  let protoOwnerSs58;
  let protoHash;
  let jsonDescriptionHash;
  let imageHash;
  let definitionHash;
  let protoHashChild;
  let protoHashGrandchild;

  before(async function () {

    // the `beforeAll` hook should timeout after 20,000 ms (the default is 2000 ms). We do this because it takes time to connect to the local node, since the node was just launched immediately prior.
    this.timeout(200_000);

    api = await index.connectToLocalNode();

    const keyring = new Keyring({type: 'sr25519'});
    keyring.setSS58Format(93);
    const alice = keyring.addFromUri('//Alice');

    protoHash = await upload(alice, [...Buffer.from('Proto-Indo-European')], []);
    jsonDescriptionHash = await setMetadata(alice, protoHash, "json_description", '{"name": "monalisa", "description": "iconic, priceless, renaissance art"}');
    const imageData = "0x" + (await fs.readFile("monalisa.jpeg")).toString("hex");
    imageHash = await setMetadata(alice, protoHash, "image", imageData);
    definitionHash = await create(alice, protoHash, api.createType("FragmentMetadata", {name: "Dummy Name", currency: null}));
    await mint(alice, definitionHash, {Quantity: 1});

    protoHashChild = await upload(alice, [...Buffer.from('Proto-Italic')], [protoHash]);
    protoHashGrandchild = await upload(alice, [...Buffer.from('Latin')], [protoHashChild]);

    protoOwner = Buffer.from(alice.publicKey).toString('hex');
    protoOwnerSs58 = alice.address;

    protoHash = Buffer.from(protoHash).toString('hex');
    jsonDescriptionHash = Buffer.from(jsonDescriptionHash).toString('hex');
    imageHash = Buffer.from(imageHash).toString('hex');
    definitionHash = Buffer.from(definitionHash).toString('hex');
    protoHashChild = Buffer.from(protoHashChild).toString('hex');
    protoHashGrandchild = Buffer.from(protoHashGrandchild).toString('hex');

  });

  describe("protos.getProtos()", () => {

    it("should return proto", async () => {
      const params = api.createType("GetProtosParams", {desc: true, from: 0, limit: 10});
      const result = await api.rpc.protos.getProtos(params);
      const obj = JSON.parse(result.toHuman());
      assert(protoHash in obj);
    });

    it("should return owner", async () => {
      const params = api.createType("GetProtosParams", {desc: true, from: 0, limit: 10, return_owners: true});
      const result = await api.rpc.protos.getProtos(params);
      const obj = JSON.parse(result.toHuman());
      assert.equal(obj[protoHash]["owner"]["type"], "internal");
      assert.equal(obj[protoHash]["owner"]["value"], protoOwner);
    });

    it("should return no protos when filtering by a wrong category", async () => {
      const params = api.createType("GetProtosParams", {desc: true, from: 0, limit: 10, categories: [{ "text": "json" }]});
      const result = await api.rpc.protos.getProtos(params);
      const obj = JSON.parse(result.toHuman());
      assert(Object.keys(obj).length === 0 && obj.constructor === Object);
    });

    it("should return no protos when filtering by a non-existent Category Trait", async () => {
      const params = api.createType("GetProtosParams", { desc: true, from: 0, limit: 10, categories: [{ "trait": [1, 1, 1, 1, 1, 1, 1, 1] }] });
      const result = await api.rpc.protos.getProtos(params);
      const obj = JSON.parse(result.toHuman());
      assert(Object.keys(obj).length === 0 && obj.constructor === Object);
    });

    it("should return no protos when filtering by a non-existent Category Shards", async () => {
      const params = api.createType("GetProtosParams", { desc: true, from: 0, limit: 10, categories: [{ "shards": {format: "edn", requiring: [[0, 0, 0, 0, 0, 0, 0, 0]], implementing: [[0, 0, 0, 0, 0, 0, 0, 0]]} }] });
      const result = await api.rpc.protos.getProtos(params);
      const obj = JSON.parse(result.toHuman());
      assert(Object.keys(obj).length === 0 && obj.constructor === Object);
    });

    it("should return correct metadata", async () => {
      const params = api.createType("GetProtosParams", {
        desc: true, from: 0, limit: 10,
        metadata_keys: ["image", "json_description"],
        owner: protoOwnerSs58
      });
      const result = await api.rpc.protos.getProtos(params);
      const obj = JSON.parse(result.toHuman());
      assert.equal(obj[protoHash]["metadata"]["json_description"], jsonDescriptionHash);
      assert.equal(obj[protoHash]["metadata"]["image"], imageHash);
    });

  });

  describe("protos.getGenealogy()", () => {
    it("should return descendents", async () => {
      const params = api.createType("GetGenealogyParams", {proto_hash: protoHash, get_ancestors: false});
      const result = await api.rpc.protos.getGenealogy(params);
      const obj = JSON.parse(result.toHuman());
      assert.deepEqual(
        obj,
        {
          [protoHash]: [protoHashChild],
          [protoHashChild]: [protoHashGrandchild],
          [protoHashGrandchild]: [],
        }
      );
    });

    it("should return ancestors", async () => {
      const params = api.createType("GetGenealogyParams", {proto_hash: protoHashGrandchild, get_ancestors: true});
      const result = await api.rpc.protos.getGenealogy(params);
      const obj = JSON.parse(result.toHuman());
      assert.deepEqual(
        obj,
        {
          [protoHash]: [],
          [protoHashChild]: [protoHash],
          [protoHashGrandchild]: [protoHashChild],
        }
      );
    });
  })

  describe("fragments.getDefinitions()", () => {
    it("should return FD", async () => {
      const params = api.createType("GetDefinitionsParams", {desc: true, from: 0, limit: 10});
      const result = await api.rpc.fragments.getDefinitions(params);
      const obj = JSON.parse(result.toHuman());
      assert(definitionHash in obj);
    });

    it("should return num_instances", async () => {
      const params = api.createType("GetDefinitionsParams", {desc: true, from: 0, limit: 10});
      const result = await api.rpc.fragments.getDefinitions(params);
      const obj = JSON.parse(result.toHuman());
      assert.equal(obj[definitionHash]["num_instances"], 1);
    });

    it("should return owner", async () => {
      const params = api.createType("GetDefinitionsParams", {desc: true, from: 0, limit: 10, return_owners: true});
      const result = await api.rpc.fragments.getDefinitions(params);
      const obj = JSON.parse(result.toHuman());
      assert.equal(obj[definitionHash]["owner"]["type"], "internal");
      assert.equal(obj[definitionHash]["owner"]["value"], protoOwner);
    });

  });

  describe("fragments.getInstances()", () => {
    it("should return correct FI", async () => {
      const params = api.createType("GetInstancesParams", {desc: true, from: 0, limit: 10, definition_hash: definitionHash});
      const result = await api.rpc.fragments.getInstances(params);
      const obj = JSON.parse(result.toHuman());
      assert("1.1" in obj);
    });
  });

  describe("fragments.getInstanceOwner()", () => {
    it("should return FI owner", async () => {
      const params = api.createType("GetInstanceOwnerParams", {definition_hash: definitionHash, edition_id: 1, copy_id: 1});
      const result = await api.rpc.fragments.getInstanceOwner(params);
      assert.equal(result, protoOwner);
    });
  });

  describe("protos.getData()", () => {

    it("should work", async function () {

      this.timeout(200_000);

      const data = "Proto-Austronesian";
      const protoHash = blake2AsU8a(data);

      const keyring = new Keyring({type: 'sr25519'});
      keyring.setSS58Format(93);
      const alice = keyring.addFromUri('//Alice');

      await new Promise((resolve) => {
        api.tx.protos.upload(
          [],
          {text: 'plain'},
          [],
          null,
          "Closed",
          null,
          {local: data},
        ).signAndSend(alice, (result) => {
          if (result.status.isFinalized) {
            resolve();
          }
        });
      });

      const result = await api.rpc.protos.getData(protoHash);
      assert.equal(btoa(data), result.toHuman()); // `btoa()` encodes binary data to base64 encoding
    });
  });

});
