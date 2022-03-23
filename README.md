![Build](https://github.com/ElementsProject/elements-miniscript/workflows/Continuous%20integration/badge.svg)

**Minimum Supported Rust Version:** 1.41.1

*This crate uses "2015" edition

# Elements Miniscript
This library is a fork of [rust-miniscript](https://github.com/rust-bitcoin/rust-miniscript) for elements.


## High-Level Features

This library supports

* [Output descriptors](https://github.com/bitcoin/bitcoin/blob/master/doc/descriptors.md)
including embedded Miniscripts
* Parsing and serializing descriptors to a human-readable string format
* Compilation of abstract spending policies to Miniscript (enabled by the
`compiler` flag)
* Semantic analysis of Miniscripts and spending policies, with user-defined
public key types
* Encoding and decoding Miniscript as Bitcoin Script, given key types that
are convertible to `bitcoin::PublicKey`
* Determining satisfiability, and optimal witnesses, for a given descriptor;
completing an unsigned `elements::TxIn` with appropriate data
* Determining the specific keys, hash preimages and timelocks used to spend
coins in a given Bitcoin transaction

More information can be found in [the documentation](https://docs.rs/elements-miniscript)
or in [the `examples/` directory](https://github.com/ElementsProject/elements-miniscript/tree/master/examples)


## Minimum Supported Rust Version (MSRV)
This library should always compile with any combination of features on **Rust 1.41.1**.


## Contributing
Contributions are generally welcome. If you intend to make larger changes please
discuss them in an issue before PRing them to avoid duplicate work and
architectural mismatches. If you have any questions or ideas you want to discuss
please join us in
[##miniscript](https://web.libera.chat/?channels=##miniscript) on Libera.

# Release Notes

See [CHANGELOG.md](CHANGELOG.md).
