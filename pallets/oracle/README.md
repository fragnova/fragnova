## Oracle pallet

The pallet uses **eth_call** on Ethereum to call the selected oracle provider.

 For **Chainlink** it encodes the call to `latestRoundData()` function from ChainLink Feed Price contract.
 It uses the first 4 bytes of `keccak_256(latestRoundData())`, padded - Use https:emn178.github.io/online-tools/keccak_256.html.

 For **Uniswap** it encodes the call to `quoteExactInputSingle` function in the Quoter smart contracts that _returns the amount out received for a given exact input but for a swap of a single pool_:
 https:docs.uniswap.org/contracts/v3/reference/periphery/lens/Quoter#quoteexactinputsingle.

 The pool used at this moment is ETH/USDT. (TODO: it needs to be changed when the FRAG pool will be available).

 ```
 function quoteExactInputSingle(
 		address tokenIn,
 		address tokenOut,
 		uint24 fee,
 		uint256 amountIn,
 		uint160 sqrtPriceLimitX96
 ) public returns (uint256 amountOut)
 ```
 Using web3 library we can obtain the function encoding as follows:
 ```
 web3.eth.abi.encodeFunctionCall({
    name: 'quoteExactInputSingle',
    type: 'function',
    inputs: [{
          type: 'address',
          name: 'tokenIn'
        },{
          type: 'address',
          name: 'tokenOut'
        },{
          type: 'uint24',
          name: 'fee'
        },{
          type: 'uint256',
          name: 'amountIn'
          },{
          type: 'uint160',
          name: 'sqrtPriceLimitX96'
        }]
      }, [
      "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",   ETH
      "0xdac17f958d2ee523a2206206994597c13d831ec7",  USDT
      "500",  fee. There are three fee tiers: 500, 3000, 10000.
      "1000000000000000000",  1 ETH (expressed with 18 decimals)
      "0"
 ]);
 ```

 The result is: `0xf7729d43000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2000000000000000000000000dac17f958d2ee523a2206206994597c13d831ec700000000000000000000000000000000000000000000000000000000000001f40000000000000000000000000000000000000000000000000de0b6b3a76400000000000000000000000000000000000000000000000000000000000000000000`
 The first 4 bytes are the function selector, the rest are the parameters.

 Using the above result and knowing the address of the Quoter contracts on ethereum mainnet, we can call `eth_call`:

 ```
 curl --url https:mainnet.infura.io/v3/<API-TOKEN> -X POST -H "Content-Type: application/json" \
  -d '{"jsonrpc": 2,"method": "eth_call","params": \
    [{\
      "to": "0xb27308f9F90D607463bb33eA1BeBb41C27CE5AB6",\  Quoter smart contract address in mainnet
      "data": "<the result above>"},\
      latest"],"id":1}'
```
 Response:
 ```
 {"jsonrpc":"2.0","id":1,"result":"0x000000000000000000000000000000000000000000000000000000004c0fbc35"}
 ```

 Using web3 library we can decode the result as follows:
 ```
 ethers.utils.formatUnits(
    0x000000000000000000000000000000000000000000000000000000004c0fbc35,  the response above
      6)  the decimals of the tokenOut (USDT)
 ```

 `1276.099637`  the price of 1 ETH in USDT


License: BUSL-1.1
