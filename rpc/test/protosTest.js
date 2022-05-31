const assert = require('chai').assert;
const index = require("../index");


// const childProcess = require('child_process');

// const addDummyMetadata = async () => {
//     await new Promise((resolve, reject) => {
//         childProcess.exec("shards ../chains/test-protos-rpc.edn",
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

    describe('getProtos', () => {


        it('should return correct proto', async () => {

            let result = await api.rpc.protos.getProtos(api.createType("GetProtosParams", {desc: true, from: 0, limit: 10}))

            assert('b8a6d246ba4324f50e392a2675bfaedea16f23aea727e0454362f213b07eb9bc' in JSON.parse(result.toHuman()))

            result = await api.rpc.protos.getProtos(api.createType("GetProtosParams", {desc: true, from: 0, limit: 10,
                                                    owner: "5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy"}))


            assert('b8a6d246ba4324f50e392a2675bfaedea16f23aea727e0454362f213b07eb9bc' in JSON.parse(result.toHuman()))


        });

        it('should return no protos when filtering Category', async () => {

            let result = await api.rpc.protos.getProtos(api.createType("GetProtosParams", {desc: true, from: 0, limit: 10, categories: [{"text": "json"}]}))

            assert(result.toHuman() === "{}")

            result = await api.rpc.protos.getProtos(api.createType("GetProtosParams", {desc: true, from: 0, limit: 10, categories: [{"texture": "pngFile"}]}))

            assert(result.toHuman() === "{}")


        });

        it('should return correct owner', async () => {

            const params = api.createType("GetProtosParams", {desc: true, from: 0, limit: 10, metadata_keys: ['A', 'A'],
                                                              owner: "5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy", categories: [{"chain": ["generic", [888, 999]]}], return_owners: true});


            let result = await api.rpc.protos.getProtos(params)

            let json = JSON.parse(result.toHuman())


        });

        it('should return correct metadata', async () => {
            const params = api.createType("GetProtosParams", {desc: true, from: 0, limit: 10, metadata_keys: ['image', 'json_description'],
                                          owner: "5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy"});

            let result = await api.rpc.protos.getProtos(params)

            let json = JSON.parse(result.toHuman())

            assert(json['b8a6d246ba4324f50e392a2675bfaedea16f23aea727e0454362f213b07eb9bc']['image'] === 'fa99f4d939e6615bae7910a85689c5bebb2292f88572d8b90ba986200c401e30')

            assert(json['b8a6d246ba4324f50e392a2675bfaedea16f23aea727e0454362f213b07eb9bc']['json_description'] === 'b68b3f86cb5707e5ac8265086bdae2f62bc69287de329a4b8fe999c59528ca70')
        });

        it('should return null metadata', async () => {
            let result = await api.rpc.protos.getProtos(api.createType("GetProtosParams", {desc: true, from: 0, limit: 10, metadata_keys: ['A']}))

            let json = JSON.parse(result.toHuman())

            assert(json['b8a6d246ba4324f50e392a2675bfaedea16f23aea727e0454362f213b07eb9bc']['A'] === null)
        });

    });


})


