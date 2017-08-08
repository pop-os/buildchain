# buildchain
Software for creating and managing a distributed and reproducible chain of builds

[![crates.io](https://img.shields.io/crates/v/buildchain.svg)](https://crates.io/crates/buildchain)
[![docs.rs](https://docs.rs/mio/badge.svg)](https://docs.rs/buildchain)
*Currently in development*

## Overview

Buildchain creates networks of several build, signing, and publishing nodes in order to securely and reliably deliver build artifacts to end users.

## Nodes

Note than one machine can participate in a network as more than one role. It is also possible to download network data as an observer, provided you have credentials.

### Building

Build nodes download input artifacts from a remote repository, usually a git source code repository. They then follow a set of build steps, usually a Dockerfile, to produce output artifacts. A manifest of the output artifacts and their shasum is created, which is then signed by either the build node or a signing node. Finally, the output artifacts and the signed manifest are published on the network, with a reference to the manifest placed in a secondary blockchain.

### Signing

Signing nodes are networkless systems that communicate over a very simple serial protocol. They initialize a key from a hardware random number generator, and store it in memory. They receive a manifest, sign the manifest, and output the signature as well as their own public key. They build a secondary blockchain of their own for auditing purposes.

### Publishing

Publishing nodes collect the signed build artifacts from the build servers on the network. Once enough build servers have produced identical builds, they publish the blockchain of the primary build server as the primary blockchain, along with all referenced artifacts.
