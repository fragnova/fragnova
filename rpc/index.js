const { ApiPromise, WsProvider } = require('@polkadot/api');

const { Text } = require('@polkadot/types')

const connectToLocalNode = async () => {
    const wsProvider = new WsProvider('ws://127.0.0.1:9944');

    api = await ApiPromise.create({
        provider: wsProvider,
        rpc: {
            protos: {
                getProtos: {
                    description: "this is the description", type: "String",
                    params: [
                        { name: 'params', type: 'GetProtosParams' },
                        { name: 'at', type: 'BlockHash', isOptional: true }
                    ]
                },
            }
        },

        types: {
            ShardsFormat: {
                _enum: [
                    "edn",
                    "binary"
                ]
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
                    "ttfFile"
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
                    "json"
                ]
            },

            BinaryCategories: {
                _enum: [
                    "wasmProgram",
                    "wasmReactor",
                    "blendFile",
                ]
            },

            ShardsScriptInfo: {
                format: 'ShardsFormat',
                requiring: 'Vec<ShardsTrait>',
                implementing: 'Vec<ShardsTrait>'
            },

            ShardsTrait: "Vec<u16>",

            ShardsFormat: {
                _enum: [
                    "edn",
                    "binary",
                ]
            },

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

            BlockHash: 'Hash',

            GetProtosParams: {
                desc: 'bool',
                from: 'u32',
                limit: 'u32',
                metadata_keys: 'Vec<String>',
                owner: 'Option<AccountId>',
                return_owners: 'bool',
                categories: 'Vec<Categories>',
                tags: 'Vec<String>',
                available: 'Option<bool>',
            }


        }
    });

    return api


};



// (async () => {
//     const api = await connectToLocalNode();

//     const params = api.createType("GetProtosParams", {categories: ["Code"], owner: "5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy", limit: 10, from: 0, desc: true,
//         metadata_keys: ['A', 'A'], return_owners: true});


//     let string_json = await api.rpc.protos.getProtos(params)

//     console.log('string_json is', string_json)

// })()



module.exports.connectToLocalNode = connectToLocalNode;




