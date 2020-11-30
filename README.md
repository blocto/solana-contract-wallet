[![Build status][travis-image]][travis-url]
[![Gitpod Ready-to-Code](https://img.shields.io/badge/Gitpod-Ready--to--Code-blue?logo=gitpod)](https://gitpod.io/#https://github.com/portto/solana-contract-wallet)

[travis-image]: https://travis-ci.org/portto/solana-contract-wallet.svg?branch=master
[travis-url]: https://travis-ci.org/portto/solana-contract-wallet

# Solana Multi-Sig Wallet Program

The project comprises of:

* An on-chain multi-sig wallet program
* A client library that helps construct requests to the multi-sig wallet program

## Specs & Features

The multi-sig wallet program provides the following instructions:

* Initialize wallet and add initial key
* Add new owner public key with corresponding weight (requires authorization)
* Remove owner (requires authorization)
* Invoke any instruction to another program (requires authorization)

Each owner public key has its own weight (0~1000). Any authorized instruction requires the total signature weight to be at least 1000.

## Quick Start

[![Open in Gitpod](https://gitpod.io/button/open-in-gitpod.svg)](https://gitpod.io/#https://github.com/portto/solana-contract-wallet)

If you decide to open in Gitpod then refer to [README-gitpod.md](README-gitpod.md), otherwise continue reading.

The following dependencies are required to build and run this example,
depending on your OS, they may already be installed:

- Install node
- Install npm
- Install docker
- Install the latest Rust stable from https://rustup.rs/
- Install Solana v1.4.7 or later from https://docs.solana.com/cli/install-solana-cli-tools

If this is your first time using Docker or Rust, these [Installation Notes](README-installation-notes.md) might be helpful.

### Start local Solana cluster

This example connects to a local Solana cluster by default.

Enable on-chain program logs:
```bash
$ export RUST_LOG=solana_runtime::system_instruction_processor=trace,solana_runtime::message_processor=debug,solana_bpf_loader=debug,solana_rbpf=debug
```

Start a local Solana cluster:
```bash
$ npm run localnet:update
$ npm run localnet:up
```

View the cluster logs:
```bash
$ npm run localnet:logs
```

Note: To stop the local Solana cluster later:
```bash
$ npm run localnet:down
```

### Build the on-chain program

```bash
$ npm run build:program
```

### Run the client

```bash
$ npm run start
```

## Pointing to a public Solana cluster

Solana maintains three public clusters:
- `devnet` - Development cluster with airdrops enabled
- `testnet` - Tour De Sol test cluster without airdrops enabled
- `mainnet-beta` -  Main cluster
  
Use npm scripts to configure which cluster.

To point to `devnet`:
```bash
$ npm run cluster:devnet
```

To point back to the local cluster:
```bash
$ npm run cluster:localnet
```

## Information about Solana

For more information about programming on Solana, visit [Solana Docs](https://docs.solana.com/developing/programming-model/overview)
