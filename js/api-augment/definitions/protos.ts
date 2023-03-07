export default {
  rpc: {
    getData: {
      description: "Query and Return Proto-Fragment data based on `proto_hash`. The **return type** is base64 encoded bytes.",
      type: "String",
      params: [
        { name: "proto_hash", type: "Hash" },
        { name: "at", type: "Hash", isOptional: true }
      ]
    },
    getProtos: {
      description: "Query and Return Proto-Fragment(s) based on `params`. The return type is a JSON string",
      type: "String",
      params: [
        { name: "param", type: "GetProtosParams" },
        { name: "at", type: "Hash", isOptional: true }
      ]
    },
    getGenealogy: {
      description: "Query the Genealogy of a Proto-Fragment based on `params`. The return type is a JSON string that represents an Adjacency List.",
      type: "String",
      params: [
        { name: "param", type: "GetGenealogyParams" },
        { name: "at", type: "Hash", isOptional: true }
      ]
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
        "rareDomain",
      ]
    },
    ShardsScriptInfo: {
      format: "ShardsFormat",
      requiring: "Vec<ShardsTrait>",
      implementing: "Vec<ShardsTrait>"
    },
    ShardsTrait: "Vec<u16>", // TODO Review - It should be `[u8; 8]` - but if I put that the RPC tests fails
    ShardsFormat: {
      _enum: [
        "edn",
        "binary",
      ]
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

  }
};
