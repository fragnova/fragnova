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
        
        // let result = await api.rpc.protos.getMetadataBatch([""], ["image", "json_description"]);

        
    });

    it('getByTags should return correct output', async () => {
        
        const tags = [api.createType("Tags", "Code")]
        const owner = api.createType('AccountId', '5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy')
        const limit = 1000
        const from = 69
        const desc = false
    
        
        let listProtoHashes = await api.rpc.protos.getByTags(tags, null, limit, from, desc)

        
    });

    
})


