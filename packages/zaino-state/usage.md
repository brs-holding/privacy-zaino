# Validator network adoption

`CommonBackendConfig.network` determines the expected chain. At startup the indexer reads the validator's upgrade schedule. `CustomTestnet` additionally fetches height zero and compares its hash to the configured genesis. A differing schedule or genesis stops startup. A custom profile using the public Zcash testnet genesis is rejected.

The verified profile is converted to Zebra's existing configured Testnet parameters under the network name `SwarmTestnet`. The indexer retains upstream transaction and proof handling. Use a separate index database for each genesis and restart the indexer alongside any validator network change.

See [network configuration](../zaino-common/usage.md) for the custom profile syntax. Clients using `GetLightdInfo` receive `swarm-testnet` for this network and `regtest` for a Regtest profile. The identity check itself is unchanged by that naming: the genesis and the full upgrade schedule are still compared against the validator before the index is opened, and either disagreement stops startup.
