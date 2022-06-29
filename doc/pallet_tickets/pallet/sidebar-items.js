window.SIDEBAR_ITEMS = {"enum":[["Call","Add a Clamor Account ID to `FragKeys`. "],["Error","Custom dispatch errors of this pallet."],["Event","The event emitted by this pallet."]],"struct":[["GenesisConfig","Can be used to configure the genesis state of this pallet."],["Pallet","The pallet implementing the on-chain logic."]],"trait":[["Config","Configure the pallet by specifying the parameters and types on which it depends."]],"type":[["EVMLinkVoting","StorageMap that maps a FRAG token locking or unlocking event to a number of votes ().  The key for this map is: `blake2_256(encoded(<Amount of FRAG token that was locked/unlocked, Signature written by the owner of the FRAG token on a determinstic message,  Whether it was locked or unlocked, Ethereum Block Number where it was locked/unlocked>))`"],["EVMLinkVotingClosed","StorageMap that maps a FRAG token locking or unlocking event to a boolean indicating whether voting on the aforementioned event has ended**."],["EVMLinks","StorageMap that maps a Clamor Account ID to an Ethereum Account ID,  where both accounts are owned by the same owner."],["EVMLinksReverse","This StorageMap is the reverse mapping of `EVMLinks`."],["EthLockedFrag","StorageMap that maps an Ethereum Account ID to a to an Ethlock struct of the aforementioned Ethereum Account Id (the struct contains the amount of FRAG token locked, amongst other things)"],["FragKeys","StorageValue that equals the List of Clamor Account IDs that both validate and send unsigned transactions with signed payload"],["FragUsage","StorageMap that maps a Clamor Account ID to the Amount of FRAG token staked by the aforementioned Clamor Account ID"],["Module","Type alias to `Pallet`, to be used by `construct_runtime`."],["PendingUnlinks","List of Clamor Accounts whose (FRAG staking)-related Storage Items are yet to be cleared"]]};