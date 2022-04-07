const { ApiPromise, WsProvider } = require('@polkadot/api');


const connectToLocalNode = async () => {
    const wsProvider = new WsProvider('ws://127.0.0.1:9944');

    api = await ApiPromise.create({
        provider: wsProvider,
        rpc: {
            protos: {
                getByTags: {description: "this is the description", type: "Vec<Hash256>",
                    params: [
                        {name: 'tags', type: 'Vec<Tags>'},
                        {name: 'owner', type: 'Option<AccountId>'},
                        {name: 'limit', type: 'u32'},
                        {name: 'from', type: 'u32'},
                        {name: 'desc', type: 'bool'},
                        {name: 'at', type: 'BlockHash', isOptional: true}
                    ]},
                getMetadataBatch: {description: "this is the description", type: "Vec<Option<Vec<Option<Hash256>>>>",
                    params: [
                        {name: 'batch', type: 'Vec<String>'},
                        {name: 'keys', type: 'Vec<String>'},
                        {name: 'at', type: 'BlockHash', isOptional: true}
                    ]},
            }
        },

        types: {
            Tags: {
                _enum: ['Code', 'Audio', 'Image']
            },

            BlockHash: 'Hash',

            Hash256: '[u8; 32]',


        }
    });

    return api


};







module.exports.connectToLocalNode = connectToLocalNode;




