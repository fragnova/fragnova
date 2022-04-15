const { ApiPromise, WsProvider } = require('@polkadot/api');

const {Text} = require('@polkadot/types')

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

                getProtos: {description: "this is the description", type: "Text",
                    params: [
                        // {name: 'params', type: 'GetProtosParams'},

                        {name: 'desc', type: 'bool'},
                        {name: 'from', type: 'u32'},
                        {name: 'limit', type: 'u32'},
                        {name: 'metadata_keys', type: 'Vec<String>', isOptional: true},
                        {name: 'owner', type: 'AccountId', isOptional: true},
                        {name: 'return_owners', type: 'bool'},
                        {name: 'tags', type: 'Vec<Tags>', isOptional: true},


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

            GetProtosParams: {
                tags: 'Option<Vec<Tags>>',
                owner: 'Option<AccountId>',
                limit: 'u32',
                from: 'u32',
                desc: 'bool',
                metadata_keys: 'Option<Vec<Text>>',
                return_owners: 'bool'
            }


        }
    });

    return api


};



(async () => {
    const api = await connectToLocalNode();   

    // const params = api.createType("GetProtosParams", {tags: ["Code"], owner: "5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy", limit: 10, from: 0, desc: true,
    //     metadata_keys: ['A', 'A'], return_owners: true});


    let string_json = await api.rpc.protos.getProtos({tags: ["Code"], owner: "5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy", limit: 10, from: 0, desc: true,
    metadata_keys: ['A', 'A'], return_owners: true})

    console.log('string_json is', string_json)

})()



module.exports.connectToLocalNode = connectToLocalNode;




