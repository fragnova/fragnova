const assert = require("chai").assert;
const index = require("../index");


// const childProcess = require("child_process");

// const addDummyMetadata = async () => {
//     await new Promise((resolve, reject) => {
//         childProcess.exec("shards ../shards/test-protos-rpc.edn",
//         (error, stdout, stderr) => {
//             if (error) {
//                 reject(error);
//             } else {
//                 resolve(stdout);
//             }
//         });
//     });
// }


const PROTO = "95036581e1cd69cd1796f0fb62f0d1dcbb360c806aefa4398a3c890514042465";
const PROTO_OWNER = "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";
const PROTO_OWNER_SS58 = "fRmUiaJxhGEsYzjHzXP5wAAXxchGRpF1hi5JfCPNoLRb8B2F1";
const PROTO_IMAGE = "fa99f4d939e6615bae7910a85689c5bebb2292f88572d8b90ba986200c401e30";
const PROTO_JSON_DESCRIPTION = "b68b3f86cb5707e5ac8265086bdae2f62bc69287de329a4b8fe999c59528ca70";

const DEFINITION = "f8289150f3845d05869f308534d92c44";
const DEFINITION_OWNER = PROTO_OWNER;

const INSTANCE = "{1.1}";

describe("Test Protos RPCs", () => {

  before(async () => {
    await index.connectToLocalNode();
  });

  describe("Test protos_getProtos", () => {

    it("should return correct proto", async () => {
      const params = api.createType("GetProtosParams", {desc: true, from: 0, limit: 10});
      let result = await api.rpc.protos.getProtos(params);
      let json = JSON.parse(result.toHuman());
      assert(PROTO in json);
    });

    it("should return correct owner", async () => {
      const params = api.createType("GetProtosParams", {desc: true, from: 0, limit: 10, return_owners: true});
      let result = await api.rpc.protos.getProtos(params);
      let json = JSON.parse(result.toHuman());
      assert(json[PROTO]["owner"]["value"] === PROTO_OWNER && json[PROTO]["owner"]["type"] === "internal");
    });

    it("should return no protos when filtering by wrong category", async () => {
      const params = api.createType("GetProtosParams", {desc: true, from: 0, limit: 10, categories: [{ "text": "json" }]});
      let result = await api.rpc.protos.getProtos(params);
      let json = JSON.parse(result.toHuman());
      assert(Object.keys(json).length === 0 && json.constructor === Object);
    });

    it("should return no protos when filtering for non-existing Category Trait", async () => {
      const params = api.createType("GetProtosParams", { desc: true, from: 0, limit: 10, categories: [{ "trait": [1, 1, 1, 1, 1, 1, 1, 1] }] });
      let result = await api.rpc.protos.getProtos(params);
      let json = JSON.parse(result.toHuman());
      assert(Object.keys(json).length === 0 && json.constructor === Object);
    });

    it("should return no protos when filtering for non-existing Category Shards", async () => {
      const params = api.createType("GetProtosParams", { desc: true, from: 0, limit: 10, categories: [{ "shards": {format: "edn", requiring: [[0, 0, 0, 0, 0, 0, 0, 0]], implementing: [[0, 0, 0, 0, 0, 0, 0, 0]]} }] });
      let result = await api.rpc.protos.getProtos(params);
      let json = JSON.parse(result.toHuman());
      assert(Object.keys(json).length === 0 && json.constructor === Object);
    });

    it("should return correct metadata", async () => {
      const params = api.createType("GetProtosParams", {
          desc: true, from: 0, limit: 10, metadata_keys: ["image", "json_description"],
          owner: PROTO_OWNER_SS58
      });
      let result = await api.rpc.protos.getProtos(params);
      let json = JSON.parse(result.toHuman());
      console.log(json);
      assert.equal(json[PROTO]["image"], PROTO_IMAGE);
      assert.equal(json[PROTO]["json_description"], PROTO_JSON_DESCRIPTION);
    });

  });


})

describe("Test Fragments RPCs", () => {
  before(async () => {
    await index.connectToLocalNode();
  })

  describe("Test fragments_getDefinitions", () => {
    it("should return correct FD", async () => {
      const params = api.createType("GetDefinitionsParams", {desc: true, from: 0, limit: 10});
      let result = await api.rpc.fragments.getDefinitions(params);
      let json = JSON.parse(result.toHuman());
      assert(DEFINITION in json);
    });

    it("should return correct num_instances", async () => {
      const params = api.createType("GetDefinitionsParams", {desc: true, from: 0, limit: 10});
      let result = await api.rpc.fragments.getDefinitions(params);
      let json = JSON.parse(result.toHuman());
      assert.equal(json[DEFINITION]["num_instances"], 1);
    });

    it("should return correct owner", async () => {
      const params = api.createType("GetDefinitionsParams", {desc: true, from: 0, limit: 10, return_owners: true});
      let result = await api.rpc.fragments.getDefinitions(params);
      let json = JSON.parse(result.toHuman());
      assert(json[DEFINITION]["owner"]["value"] === PROTO_OWNER && json[DEFINITION]["owner"]["type"] === "internal");
    });

  })

  describe("Test fragments_getInstances", () => {
    it("should return correct FI", async () => {
      const params = api.createType("GetInstancesParams", {desc: true, from: 0, limit: 10, definition_hash: DEFINITION});
      let result = await api.rpc.fragments.getInstances(params);
      let json = JSON.parse(result.toHuman());
      assert("1.1" in json);
    });
  })


})


