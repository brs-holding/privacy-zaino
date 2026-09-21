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

The genesis hash above is an example. Use the one published for the network you are indexing; the value shown belongs to an earlier engineering chain and is kept only so the syntax is concrete.

The chain name exposed to light wallets is **`swarm-testnet`**, and the profile's display name — also the `network_name` of the zebra parameters built for it — is **`SwarmTestnet`**. Both live in one place, `CUSTOM_TESTNET_CHAIN_NAME` and `CUSTOM_TESTNET_DISPLAY_NAME` in `config::network`. Clients must explicitly support this identity and pin the same genesis: a wallet built for `swarm-testnet` will refuse an indexer reporting anything else, which is the intended behaviour when it has been pointed at the wrong chain. The parameters of that chain remain subject to review before any public launch.

Existing `Mainnet`, `PubTestnet` (alias `Testnet`) and `Regtest` string configurations retain their meaning. Regtest lightwallet information reports `regtest`, preserving the distinction from Zebra's shared `test` RPC label.

## Changing the label

`Network::lightwallet_chain_name` owns the one string per network kind that `GetLightdInfo.chain_name` reports. It is a published interface: changing the `CustomTestnet` value re-identifies the chain to every wallet in the field, so it changes only alongside a new genesis. The unit tests in `config/network.rs` pin each label and assert that no two kinds share one.
