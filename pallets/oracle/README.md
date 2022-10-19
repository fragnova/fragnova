## Oracle pallet

An offchain worker fetches the current price from Chainlink's Price Feed smart contract on Ethereum and prepare either unsigned transaction to feed the result back on chain.
The on-chain logic will simply aggregate the results and store last `n` values to compute the average price.

License: BUSL-1.1
