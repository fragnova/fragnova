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
                        {name: 'owner', type: 'AccountId', isOptional: true},
                        {name: 'limit', type: 'u32'},
                        {name: 'from', type: 'u32'},
                        {name: 'desc', type: 'bool'},
                        {name: 'at', type: 'BlockHash', isOptional: true}
                    ]},
                getMetadataBatch: {description: "this is the description", type: "Vec<Option<Vec<Option<Hash256>>>>",
                    params: [
                        {name: 'batch', type: 'Vec<Hash256>'},
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


const main = async () => {

    let api = await connectToLocalNode()

    const tags = [api.createType("Tags", "Code")]
    const owner = api.createType('AccountId', '5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy')
    const limit = 1000
    const from = 69
    const desc = false


    let listProtoHashes = await api.rpc.protos.getByTags(tags, owner, limit, from, desc)

    console.log('list proto hashes are:', listProtoHashes.toHuman())

}


main()





