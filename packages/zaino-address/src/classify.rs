//! The two classification entry points.
//!
//! Both parse the supplied string as a [`zcash_address::ZcashAddress`] and then
//! convert it for the queried network, so an address that is well-formed for a
//! *different* network is reported invalid rather than accepted.

use zcash_keys::{address::Address, encoding::AddressCodec as _};
use zcash_protocol::consensus::{BlockHeight, NetworkType, NetworkUpgrade, Parameters};
use zcash_transparent::address::TransparentAddress;

use crate::{
    sapling::sapling_key_bytes,
    validated::{ValidatedAddress, ZValidatedAddress, DEPRECATION_NOTICE},
};

/// A [`Parameters`] that carries a network type and nothing else.
///
/// The encoders take a `Parameters` in order to read a network type out of it, and
/// this crate is handed the network type directly. No activation height is read
/// while encoding or decoding an address, so there is none to supply.
#[derive(Clone, Copy)]
struct AddressNetwork(NetworkType);

impl Parameters for AddressNetwork {
    fn network_type(&self) -> NetworkType {
        self.0
    }

    fn activation_height(&self, _nu: NetworkUpgrade) -> Option<BlockHeight> {
        None
    }
}

/// Parses `raw_address` for `network`, returning `None` if it does not parse
/// or belongs to another network.
///
/// Shared by both entry points so the two RPCs cannot disagree about which
/// addresses exist.
fn parse_for_network(raw_address: &str, network: NetworkType) -> Option<Address> {
    let parsed = raw_address.parse::<zcash_address::ZcashAddress>().ok()?;

    match parsed.convert_if_network::<Address>(network) {
        Ok(address) => Some(address),
        Err(err) => {
            tracing::debug!(?err, "conversion error");
            None
        }
    }
}

/// Classifies an address for the `validateaddress` RPC.
///
/// Pure address parsing over `network`; no chain data required.
pub fn validate_address(raw_address: String, network: NetworkType) -> ValidatedAddress {
    match parse_for_network(&raw_address, network) {
        Some(Address::Transparent(taddr)) => ValidatedAddress::Transparent {
            is_script: matches!(taddr, TransparentAddress::ScriptHash(_)),
            address: raw_address,
        },
        _ => ValidatedAddress::Invalid,
    }
}

/// Classifies an address for the deprecated `z_validateaddress` RPC.
///
/// Pure address parsing over `network`; no chain data required.
///
/// # Deprecation
///
/// Emits [`DEPRECATION_NOTICE`] on every call.
pub fn z_validate_address(raw_address: String, network: NetworkType) -> ZValidatedAddress {
    tracing::warn!("{}", DEPRECATION_NOTICE);

    // The transparent arms echo the caller's string; the shielded arms
    // re-encode, because `convert_if_network` has already proved the address
    // belongs to this network and the canonical encoding is what the legacy full node
    // reports.
    match parse_for_network(&raw_address, network) {
        Some(Address::Transparent(TransparentAddress::PublicKeyHash(_))) => {
            ZValidatedAddress::P2pkh {
                address: raw_address,
            }
        }
        Some(Address::Transparent(TransparentAddress::ScriptHash(_))) => ZValidatedAddress::P2sh {
            address: raw_address,
        },
        Some(Address::Sapling(sapling)) => {
            let (diversifier, diversified_transmission_key) = sapling_key_bytes(&sapling);
            ZValidatedAddress::Sapling {
                address: sapling.encode(&AddressNetwork(network)),
                diversifier,
                diversified_transmission_key,
            }
        }
        Some(Address::Unified(unified)) => ZValidatedAddress::Unified {
            address: unified.encode(&AddressNetwork(network)),
        },
        // Sprout, and any address kind a future `Address` variant introduces.
        // Reporting "invalid" rather than guessing preserves the previous
        // behaviour and keeps Zaino from claiming to classify what it cannot.
        _ => ZValidatedAddress::Invalid,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zcash_protocol::consensus::NetworkType;

    const TEST: NetworkType = NetworkType::Test;
    const REGTEST: NetworkType = NetworkType::Regtest;
    const MAIN: NetworkType = NetworkType::Main;
    const SWARM_MAIN: NetworkType = NetworkType::SwarmMain;

    #[test]
    fn swarm_and_legacy_unified_addresses_classify_identically() {
        const LEGACY: &str = "utest10a8k6aw5w33kvyt7x6fryzu7vvsjru5vgcfnvr288qx2zm6p63ygcajtaze0px08t583dyrgr42vasazjhhnntus2tqrpkzu0dm2l4cgf3ld6wdqdrf3jv8mvfx9c80e73syer9l2wlgawjtf7yvj0eqwdf354trtelxnr0fhpw9792eaf49ghstkyftc9lwqqwy4ye0cleagp4nzyt";
        let parsed: zcash_address::ZcashAddress = LEGACY
            .parse()
            .expect("the published legacy vector is valid");
        let canonical = parsed.to_string();
        assert!(canonical.starts_with("swarm1"));
        let expected = ZValidatedAddress::Unified {
            address: canonical.clone(),
        };
        assert_eq!(z_validate_address(LEGACY.into(), TEST), expected);
        assert_eq!(z_validate_address(canonical.clone(), TEST), expected);
        assert_eq!(
            z_validate_address(canonical, MAIN),
            ZValidatedAddress::Invalid
        );
    }

    // Canonical source: zaino-serve wire::address::tests::served_vectors
    // Tracked for DRY consolidation: https://github.com/zingolabs/zaino/issues/988
    const TESTNET_P2PKH: &str = "tmVqEASZxBNKFTbmASZikGa5fPLkd68iJyx";
    const TESTNET_P2SH: &str = "t2MjoXQ2iDrjG9QXNZNCY9io8ecN4FJYK1u";
    const REGTEST_SAPLING: &str = "zregtestsapling1jalqhycwumq3unfxlzyzcktq3n478n82k2wacvl8gwfxk6ahshkxmtp2034qj28n7gl92ka5wca";
    const REGTEST_UNIFIED: &str = "uregtest1njwg60x0jarhyuuxrcdvw854p68cgdfe85822lmclc7z9vy9xqr7t49n3d97k2dwlee82skwwe0ens0rc06p4vr04tvd3j9ckl3qry83ckay4l4ngdq9atg7vuj9z58tfjs0mnsgyrnprtqfv8almu564z498zy6tp2aa569tk8fyhdazyhytel2m32awe4kuy6qq996um3ljaajj36";
    /// A Sprout address. Zaino does not classify these — see
    /// [`ZValidatedAddress`]'s Sprout note.
    const SPROUT: &str = "ztfhKyLouqi8sSwjRm4YMQdWPjTmrJ4QgtziVQ1Kd1e9EsRHYKofjoJdF438FwcUQnix8yrbSrzPpJJNABewgNffs5d4YZJ";

    #[test]
    fn unparseable_is_invalid() {
        assert_eq!(
            validate_address("not an address".into(), TEST),
            ValidatedAddress::Invalid
        );
        assert_eq!(
            z_validate_address("not an address".into(), TEST),
            ZValidatedAddress::Invalid
        );
    }

    /// A well-formed address for the wrong network must not validate. This is
    /// the whole reason classification takes a network rather than parsing in
    /// isolation.
    #[test]
    fn wrong_network_is_invalid() {
        assert_eq!(
            validate_address(TESTNET_P2PKH.into(), MAIN),
            ValidatedAddress::Invalid
        );
        assert_eq!(
            z_validate_address(TESTNET_P2PKH.into(), MAIN),
            ZValidatedAddress::Invalid
        );
    }

    #[test]
    fn p2pkh_and_p2sh_are_distinguished() {
        assert_eq!(
            validate_address(TESTNET_P2PKH.into(), TEST),
            ValidatedAddress::Transparent {
                address: TESTNET_P2PKH.into(),
                is_script: false,
            }
        );
        assert_eq!(
            validate_address(TESTNET_P2SH.into(), TEST),
            ValidatedAddress::Transparent {
                address: TESTNET_P2SH.into(),
                is_script: true,
            }
        );

        assert_eq!(
            z_validate_address(TESTNET_P2PKH.into(), TEST),
            ZValidatedAddress::P2pkh {
                address: TESTNET_P2PKH.into()
            }
        );
        assert_eq!(
            z_validate_address(TESTNET_P2SH.into(), TEST),
            ZValidatedAddress::P2sh {
                address: TESTNET_P2SH.into()
            }
        );
    }

    /// `validateaddress` describes transparent addresses only. A well-formed
    /// shielded address is reported invalid, matching the legacy full node.
    #[test]
    fn validate_address_rejects_shielded() {
        assert_eq!(
            validate_address(REGTEST_SAPLING.into(), REGTEST),
            ValidatedAddress::Invalid
        );
        assert_eq!(
            validate_address(REGTEST_UNIFIED.into(), REGTEST),
            ValidatedAddress::Invalid
        );
    }

    /// The Sapling arm carries key material; the byte-level vector for it is
    /// pinned in [`crate::sapling`]'s tests.
    #[test]
    fn z_validate_address_classifies_shielded() {
        assert!(matches!(
            z_validate_address(REGTEST_SAPLING.into(), REGTEST),
            ZValidatedAddress::Sapling { .. }
        ));
        assert!(matches!(
            z_validate_address(REGTEST_UNIFIED.into(), REGTEST),
            ZValidatedAddress::Unified { .. }
        ));
    }

    /// Sprout parses as a Zcash address but Zaino does not classify it, so both
    /// RPCs report invalid rather than describing it.
    #[test]
    fn sprout_is_invalid() {
        for network in [TEST, REGTEST, MAIN] {
            assert_eq!(
                validate_address(SPROUT.into(), network),
                ValidatedAddress::Invalid
            );
            assert_eq!(
                z_validate_address(SPROUT.into(), network),
                ZValidatedAddress::Invalid
            );
        }
    }

    /// The SWARM production network classifies its own encodings and refuses
    /// every other chain's, including SwarmTestnet's, whose addresses a
    /// zebra-derived network type would have let through.
    #[test]
    fn swarm_mainnet_classifies_only_its_own_addresses() {
        const SWARM_MAINNET_P2PKH: &str = "s1MCkDhVejM4RqDyRR1rEJkudd26FVWipPD";
        const SWARM_MAINNET_P2SH: &str = "s3Mtm9Ez6HFNovPfrY7WpjPGZmYNxztrxbb";

        assert_eq!(
            validate_address(SWARM_MAINNET_P2PKH.into(), SWARM_MAIN),
            ValidatedAddress::Transparent {
                is_script: false,
                address: SWARM_MAINNET_P2PKH.into(),
            }
        );
        assert_eq!(
            validate_address(SWARM_MAINNET_P2SH.into(), SWARM_MAIN),
            ValidatedAddress::Transparent {
                is_script: true,
                address: SWARM_MAINNET_P2SH.into(),
            }
        );

        for foreign in [
            TESTNET_P2PKH,
            TESTNET_P2SH,
            REGTEST_SAPLING,
            REGTEST_UNIFIED,
        ] {
            assert_eq!(
                validate_address(foreign.into(), SWARM_MAIN),
                ValidatedAddress::Invalid,
                "{foreign} is not a swarm-mainnet address",
            );
        }
        for network in [TEST, REGTEST, MAIN] {
            for ours in [SWARM_MAINNET_P2PKH, SWARM_MAINNET_P2SH] {
                assert_eq!(
                    validate_address(ours.into(), network),
                    ValidatedAddress::Invalid,
                    "{ours} is not a {network:?} address",
                );
            }
        }
    }

    /// The shielded arms re-encode, so the SWARM production HRPs have to come out
    /// of them rather than an upstream chain's.
    ///
    /// The address is the regtest vector's payment address under the SWARM
    /// production HRP: a payment address is a curve point, so it is taken from a
    /// known-good one rather than written out.
    #[test]
    fn swarm_mainnet_shielded_addresses_re_encode_under_their_own_hrps() {
        use zcash_keys::encoding::{decode_payment_address, encode_payment_address};
        use zcash_protocol::consensus::NetworkConstants as _;

        let payment =
            decode_payment_address(REGTEST.hrp_sapling_payment_address(), REGTEST_SAPLING)
                .expect("the regtest vector is a payment address");
        let swarm_mainnet_sapling =
            encode_payment_address(SWARM_MAIN.hrp_sapling_payment_address(), &payment);
        assert!(
            swarm_mainnet_sapling.starts_with("zswmsapling1"),
            "{swarm_mainnet_sapling}",
        );

        let classified = z_validate_address(swarm_mainnet_sapling.clone(), SWARM_MAIN);
        let ZValidatedAddress::Sapling { address, .. } = classified else {
            panic!("a swarm-mainnet sapling address classifies as sapling: {classified:?}");
        };
        assert_eq!(address, swarm_mainnet_sapling);
        assert_eq!(
            z_validate_address(swarm_mainnet_sapling, TEST),
            ZValidatedAddress::Invalid,
        );
        assert_eq!(
            z_validate_address(REGTEST_SAPLING.into(), SWARM_MAIN),
            ZValidatedAddress::Invalid,
        );
    }
}
