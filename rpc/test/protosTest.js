const assert = require('chai').assert;
const index = require("../index");


// const childProcess = require('child_process');

// const addDummyMetadata = async () => {
//     await new Promise((resolve, reject) => {
//         childProcess.exec("cbl ../chains/test-protos-rpc.edn",
//         (error, stdout, stderr) => {
//             if (error) {
//                 reject(error);
//             } else {
//                 resolve(stdout); 
//             }
//         });
//     });
// }



describe('Protos RPCs', () => {

    before(async () => {
        await index.connectToLocalNode();  
    });

    it('getMetadataBatch should return correct output', async () => {
        
        let protoId = 'b8a6d246ba4324f50e392a2675bfaedea16f23aea727e0454362f213b07eb9bc'
        
        let result = await api.rpc.protos.getMetadataBatch([protoId], ["image", "json_description"]);

        assert(JSON.stringify(result.toHuman()) === '[["0xfa99f4d939e6615bae7910a85689c5bebb2292f88572d8b90ba986200c401e30","0xb68b3f86cb5707e5ac8265086bdae2f62bc69287de329a4b8fe999c59528ca70"]]')

        
    });

    it('getByTags should return correct output', async () => {
        
        const tags = [api.createType("Tags", "Code")]
        const owner = api.createType('AccountId', '5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy')
        const limit = 1000
        const from = 0
        const desc = false
    
        
        let listProtoHashes = await api.rpc.protos.getByTags(tags, null, limit, from, desc)

        assert(JSON.stringify(listProtoHashes.toHuman()) === '["0xb8a6d246ba4324f50e392a2675bfaedea16f23aea727e0454362f213b07eb9bc"]')

        
    });

    
})


