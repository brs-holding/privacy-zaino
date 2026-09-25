//! Zcash address parsing and classification.
//!
//! Zaino serves two address-validation RPCs, `validateaddress` and the
//! deprecated `z_validateaddress`. Neither reads the chain: both are pure
//! functions of an address string and a network. This crate is where that
//! logic lives, so both the indexing library and the serving layer can reach
//! it without either owning it.
//!
//! # Why a separate crate
//!
//! The `librustzcash` address stack (`zcash_address`, `zcash_keys`,
//! `zcash_transparent`, `sapling-crypto`) is a dependency set no other Zaino
//! crate wants. It cannot go in `zaino-primitives`, whose whole dependency
//! list is `thiserror` — that minimalism is what lets every other crate depend
//! on it. It does not belong in `zaino-common` either, which is configuration,
//! logging and networking infrastructure rather than domain logic. So it is a
//! leaf: nothing in Zaino depends on it except the consumers of these two
//! RPCs.
//!
//! # No serialization
//!
//! The types here are domain types. They carry raw key material as bytes, not
//! hex, and they do not derive `Serialize`. The legacy-compatible JSON shapes
//! these RPCs return — field names, hex encoding, the `type` / `address_type`
//! duplication — are the serving layer's concern and live in `zaino-serve`.
//!
//! # Network parameterisation
//!
//! Entry points take a [`zcash_protocol::consensus::NetworkType`], the network
//! an address encoding actually belongs to, rather than a
//! [`Parameters`](zcash_protocol::consensus::Parameters) to read one out of.
//! `zebra_chain::parameters::Network` implements `Parameters`, but it has two
//! kinds and answers `Test` for every configured testnet, so a caller serving
//! the SWARM production network and passing its zebra network would classify
//! its own addresses as another chain's. The caller names the network type,
//! and this crate stays free of any dependency on Zebra.

mod classify;
mod sapling;
mod validated;

pub use classify::{validate_address, z_validate_address};
pub use sapling::sapling_key_bytes;
pub use validated::{ValidatedAddress, ZValidatedAddress, DEPRECATION_NOTICE};
