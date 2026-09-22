# Validator network adoption

`CommonBackendConfig.network` determines the expected chain. At startup the indexer reads the validator's upgrade schedule. `CustomTestnet` additionally fetches height zero and compares its hash to the configured genesis. A differing schedule or genesis stops startup. A custom profile using the public Zcash testnet genesis is rejected.

The verified profile is converted to Zebra's existing configured Testnet parameters under the network name `SwarmTestnet`. The indexer retains upstream transaction and proof handling. Use a separate index database for each genesis and restart the indexer alongside any validator network change.

See [network configuration](../zaino-common/usage.md) for the custom profile syntax. Clients using `GetLightdInfo` receive `swarm-testnet` for this network and `regtest` for a Regtest profile. The identity check itself is unchanged by that naming: the genesis and the full upgrade schedule are still compared against the validator before the index is opened, and either disagreement stops startup.

## What else this branch carries

Besides the SwarmTestnet identity, `swarm-testnet-support` cherry-picks the
upstream fix for a crash this project hit in production.

| Upstream | What |
| --- | --- |
| [#1551](https://github.com/zingolabs/zaino/issues/1551) | `handle_reorg`'s `recursion_count` is `u8` but the depth guard it feeds is `u32` |
| [#1584](https://github.com/zingolabs/zaino/pull/1584) | chain-head-service: walk reorgs iteratively, and trim after the handoff |

Both reorg walks in `zaino-chain-head-service` used to recurse once per block.
`Box::pin` bounds the future's type but not the call chain, so depth cost stack:
upstream measured a 2 MiB tokio worker dying at about 150 levels. On
2026-09-22 the SwarmTestnet indexer aborted exactly that way, with
`fatal runtime error: stack overflow`, once a second miner had been producing
competing tips on a chain several hundred blocks long.

Two commits are taken, the minimum that applies: `0e01934a` adds
`tests/reorg_depth.rs`, and `433657de` is the fix, which edits those tests and
therefore needs them. The follow-up doc commit `f0a644ed` is deliberately not
taken. Nothing is rebased: the branch keeps its original base so the identity
changes stay reviewable against it.

Run those tests the way upstream did, with a worker-sized stack, or a large
default stack will let the old recursive walk pass:

```sh
RUST_MIN_STACK=2097152 cargo test -p zaino-chain-head-service reorg_depth
```
