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
            AudioCategories: {
                _enum: [
                    /// An audio file of the supported formats (mp3, ogg, wav, etc.)
                    "file",
                    /// A chainblocks script that returns an effect chain that requires an input, validated
                    "effect",
                    /// A chainblocks script that returns an instrument chain (no audio input), validated
                    "instrument"]
            },

            ModelCategories: {
                _enum: [
                    /// A GLTF binary model
                    "gltf",
                    /// ???
                    "sdf",
                    /// A physics collision model
                    "physicsCollider"]
            },

            ShaderCategories: {
                _enum: [
                    /// A chainblocks script that returns a shader chain (we validate that)
                    "generic",
                    /// A chainblocks script that returns a shader chain constrained to be a compute shader (we validate that)
                    "compute",
                    /// A chainblocks script that returns a shader chain constrained to be a screen post effect shader (we validate that)
                    "postEffect"]
            },

            TextureCategories: {
                _enum: [
                    "pngFile",
                    "jpgFile"]
            },

            VectorCategories: {
                _enum: [
                    "svgFile",
                    "fontFile"]
            },

            VideoCategories: {
                _enum: [
                    "mp4File"]
            },

            TextCategories: {
                _enum: [
                    "plain",
                    "json"]
            },

            BinaryCategories: {
                _enum: [
                    "wasmModule"]
            },

            ChainCategories: {
                _enum: [
                    /// A chainblocks script that returns a generic chain (we validate that)
                    "generic",
                    /// An animation sequence in chainblocks edn
                    "animation",
                    /// A chainblocks script that returns a chain constrained to be used as particle fx (we validate that)
                    "particle"]
            },

            Categories: {
                _enum: {
                    "chain": "ChainCategories",
                    "audio": "AudioCategories",
                    "texture": "TextureCategories",
                    "vector": "VectorCategories",
                    "video": "VideoCategories",
                    "model": "ModelCategories",
                    "shader": "ShaderCategories",
                    "text": "TextCategories",
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




