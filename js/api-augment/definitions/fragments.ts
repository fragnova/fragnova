export default {
  rpc: {
    getDefinitions: {
      description: "Query and Return Fragment Definition(s) based on `params`",
      type: "String",
      params: [
        { name: "param", type: "GetDefinitionsParams" },
        { name: "at", type: "BlockHash", isOptional: true }
      ]
    },
    getInstances: {
      description: "Query and Return Fragment Instance(s) based on `params`",
      type: "String",
      params: [
        { name: "param", type: "GetInstancesParams" },
        { name: "at", type: "BlockHash", isOptional: true }
      ]
    },
    getInstanceOwner: {
      description: "Query the owner of a Fragment Instance. The return type is a String",
      type: "String",
      params: [
        { name: "param", type: "GetInstanceOwnerParams" },
        { name: "at", type: "BlockHash", isOptional: true }
      ]
    },
  },
  types: {
    DefinitionMetadata: {
      name: "Vec<u8>",
      currency: "Option<AssetId>",
      // Reserved for future use
      _reserved1: "Option<()>",
      _reserved2: "Option<()>",
      _reserved3: "Option<()>",
    },

    GetDefinitionsParams: {
      desc: "bool",
      from: "u64",
      limit: "u64",
      metadata_keys: "Vec<String>",
      owner: "Option<AccountId>",
      return_owners: "bool",
    },
    GetInstancesParams: {
      desc: "bool",
      from: "u64",
      limit: "u64",
      definition_hash: "String", // "[u8; 16]",  // using `String` because Polkadot-JS has a problem fixed-sized arrays: https://github.com/encointer/pallets/pull/86
      metadata_keys: "Vec<String>",
      owner: "Option<AccountId>",
      only_return_first_copies: "bool",
    },
    GetInstanceOwnerParams: {
      definition_hash: 'String', // "[u8; 16]", // using `String` because Polkadot-JS has a problem fixed-sized arrays: https://github.com/encointer/pallets/pull/86
      edition_id: "InstanceUnit",
      copy_id: "InstanceUnit",
    },
    "InstanceUnit": "u64",

  }

};



