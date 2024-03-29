const { ApiPromise, WsProvider } = require("@polkadot/api");

const connectToLocalNode = async () => {
  const wsProvider = new WsProvider("ws://127.0.0.1:9944");

  const api = await ApiPromise.create({
    provider: wsProvider,
    rpc: {
      protos: {
        getData: {
          description: "Query and Return Proto-Fragment data based on `proto_hash`. The **return type** is base64 encoded bytes.",
          type: "String",
          params: [
            { name: "proto_hash", type: "BlockHash" },
            { name: "at", type: "BlockHash", isOptional: true }
          ]
        },
        getProtos: {
          description: "Query and Return Proto-Fragment(s) based on `params`. The return type is a JSON string",
          type: "String",
          params: [
            { name: "param", type: "GetProtosParams" },
            { name: "at", type: "BlockHash", isOptional: true }
          ]
        },
        getGenealogy: {
          description: "Query the Genealogy of a Proto-Fragment based on `params`. The return type is a JSON string that represents an Adjacency List.",
          type: "String",
          params: [
            { name: "param", type: "GetGenealogyParams" },
            { name: "at", type: "BlockHash", isOptional: true }
          ]
        },
      },
      fragments: {
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
    },

    types: {
      Categories: {
        _enum: {
          "text": "TextCategories",
          "trait": "Option<ShardsTrait>",
          "shards": "ShardsScriptInfo",
          "audio": "AudioCategories",
          "texture": "TextureCategories",
          "vector": "VectorCategories",
          "video": "VideoCategories",
          "model": "ModelCategories",
          "binary": "BinaryCategories",
        }
      },
      AudioCategories: {
        _enum: [
          "oggFile",
          "mp3File",
        ]
      },
      ModelCategories: {
        _enum: [
          "gltfFile",
          "sdf",
          "physicsCollider"
        ]
      },
      TextureCategories: {
        _enum: [
          "pngFile",
          "jpgFile"
        ]
      },
      VectorCategories: {
        _enum: [
          "svgFile",
          "ttfFile",
          "otfFile"
        ]
      },
      VideoCategories: {
        _enum: [
          "mkvFile",
          "mp4File"
        ]
      },
      TextCategories: {
        _enum: [
          "plain",
          "json",
          "wgsl",
          "markdown"
        ]
      },
      BinaryCategories: {
        _enum: [
          "wasmProgram",
          "wasmReactor",
          "blendFile",
          "onnxModel",
          "safeTensors",
          "rareDomain"
        ]
      },
      ShardsScriptInfo: {
        format: "ShardsFormat",
        shardsVersion: "Compact<u32>",
        requiring: "Vec<ShardsTrait>",
        implementing: "Vec<ShardsTrait>"
      },
      ShardsTrait: "Vec<u16>", // TODO Fix - It should be `[u8; 8]` - but if I put that the RPC tests fails
      ShardsFormat: {
        _enum: [
          "edn",
          "binary",
        ]
      },

      UsageLicense: {
        _enum: {
          "Closed": null,
          "Open": null,
          "Contract": "AccountId",
        }
      },

      DefinitionMetadata: {
        name: "Vec<u8>",
        currency: "Option<AssetId>",
        // Reserved for future use
        _reserved1: "Option<()>",
        _reserved2: "Option<()>",
        _reserved3: "Option<()>",
      },

      GetProtosParams: {
        desc: 'bool',
        from: 'u32',
        limit: 'u32',
        metadata_keys: 'Vec<String>',
        owner: 'Option<AccountId>',
        return_owners: 'bool',
        categories: 'Vec<Categories>',
        tags: 'Vec<String>',
        exclude_tags: 'Vec<String>',
        available: 'Option<bool>',
      },
      GetGenealogyParams: {
        proto_hash: "String",
        get_ancestors: "bool",
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
  });

  return api;

};

module.exports.connectToLocalNode = connectToLocalNode;




