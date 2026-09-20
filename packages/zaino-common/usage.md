# Network configuration

The project fork adds an explicit `CustomTestnet` profile using standard Zcash testnet address encodings. Supply its expected genesis and every intended upgrade height. Startup verifies these assertions against the backing validator before indexing.

```toml
[network.CustomTestnet]
genesis_hash = "01d6e85dd3c1c128941a849c5025cd2e437258811a2551b82aefd68686c982e1"

[network.CustomTestnet.activation_heights]
BeforeOverwinter = 1
Overwinter = 1
Sapling = 1
Blossom = 1
Heartwood = 1
Canopy = 1
NU5 = 1
NU6 = 1
"NU6.1" = 1
"NU6.2" = 1
"NU6.3" = 1
```

The chain name exposed to light wallets is `privacy-testnet`. Clients must explicitly support this identity and pin the same genesis. The example identifies an engineering test chain, whose parameters remain subject to review before any public launch.

Existing `Mainnet`, `PubTestnet` (alias `Testnet`) and `Regtest` string configurations retain their meaning. Regtest lightwallet information reports `regtest`, preserving the distinction from Zebra's shared `test` RPC label.
