## Traits

### Runnable
```clojure
(trait Runnable
       :setup {:Type Type.Shards}
       :loops [[{:Type Type.Wire
                 :Looped true}]])
```

### Fragnova-Node
```clojure
(trait Fragnova-Account
       :private-key Type.String)

(trait Fragnova-Node
       ; optional list of required/implemented traits first
       (implementing [Runnable])
       (requiring [Fragnova-Account])

       ; pallet accounts index
       :accounts-index Type.Int
       ; sponsor account call index
       :sponsor-account-index Type.Int

       ; pallet protos index
       :protos-index Type.Int

       :websocket-recv {:Type Type.Channel :Of Type.String}

       :make-calldata
       {:Type Type.Wire
        :Looped false
        :Inputs [Type.Bytes] ; scale encoded call arguments
        :Output Type.Bytes ; encoded data
        :Requires ; to call this wire you need to have those variables in context!
        [{:Type Type.Int   :Name "pallet-idx"}
         {:Type Type.Int   :Name "call-idx"}]}

       :send-signed-extrinsic
       {:Type Type.Wire
        :Looped false
        ; encoded call data from make-calldata
        :Inputs [Type.Bytes]}

       :get-storage-map
       {:Type Type.Wire
        :Looped false
        :Inputs [Type.Bytes] ; the key to query
        :Output Type.String ; TODO
        :Requires ; to call this wire you need to have those variables in context!
        [{:Type Type.String :Name "pallet-name"}
         {:Type Type.String :Name "map-name"}]}

       :get-header
       {:Type Type.Wire
        :Looped false
        ; the block hash to query
        :Inputs [Type.String]
        :Output Type.Table})

```
