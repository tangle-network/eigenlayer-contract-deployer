![Tangle Network Banner](https://raw.githubusercontent.com/tangle-network/tangle/refs/heads/main/assets/Tangle%20%20Banner.png)

<h1 align="center">Eigenlayer Contract Deployer</h1>

<p align="center"><em>A Rust library for generating bindings and deploying smart contracts for Eigenlayer AVSs.</em></p>

<p align="center">
  <a href="https://github.com/tangle-network/eigenlayer-contract-deployer/actions"><img src="https://img.shields.io/github/actions/workflow/status/tangle-network/eigenlayer-contract-deployer/ci.yml?branch=main&logo=github" alt="Build Status"></a>
  <a href="https://github.com/tangle-network/eigenlayer-contract-deployer/releases"><img src="https://img.shields.io/github/v/release/tangle-network/eigenlayer-contract-deployer?sort=semver&display_name=release" alt="Latest Release"></a>
  <a href="https://github.com/tangle-network/eigenlayer-contract-deployer/blob/main/LICENSE"><img src="https://img.shields.io/github/license/tangle-network/eigenlayer-contract-deployer" alt="License"></a>
  <a href="https://discord.com/invite/cv8EfJu3Tn"><img src="https://img.shields.io/discord/833784453251596298?label=Discord" alt="Discord"></a>
  <a href="https://t.me/tanglenet"><img src="https://img.shields.io/endpoint?color=neon&url=https%3A%2F%2Ftg.sumanjay.workers.dev%2Ftanglenet" alt="Telegram"></a>
</p>

## Overview

The Eigenlayer Contract Deployer is a Rust library that provides bindings for smart contracts commonly used in blueprints created with the Blueprint SDK. It focuses on EigenLayer/EVM blueprints and includes tools for deploying contracts and setting up environments for blueprints.

The library contains:

1. **Rust Bindings for Smart Contracts**: Automatically generated Rust bindings for specified contracts.
2. **Deployment Tools**: Helper functions for deploying contracts and setting up permissions.
3. **Environment Setup**: Utilities for setting up environments for blueprints, specifically for EigenLayer AVSs.

Currently, the library includes specific support for the EigenLayer Incredible Squaring blueprint, with plans to make it more general-purpose for any AVS.

## Features

- **Automatic Binding Generation**: Uses a build script to generate Rust bindings from Solidity contracts.
- **Core Contract Deployment**: Functions to deploy EigenLayer core contracts.
- **AVS Contract Deployment**: Functions to deploy AVS-specific contracts for EigenLayer.
- **Permission Management**: Utilities to set up permissions between aforementioned contracts.

## Usage

### Prerequisites

- Rust 1.86
- Forge (for contract compilation and binding generation)

### Installation

Add the library to your Cargo.toml:

```toml
[dependencies]
eigenlayer-contract-deployer = { git = "https://github.com/tangle-network/eigenlayer-contract-deployer" }
```

### Example: Deploying EigenLayer AVS Contracts

```rust
use eigenlayer_contract_deployer::deploy::deploy_avs_contracts;
use eigenlayer_contract_deployer::core::deploy_core_contracts;
use alloy_primitives::Address;

async fn deploy_my_avs() -> Result<(), Box<dyn std::error::Error>> {
    // Deploy core contracts first
    let core_contracts = deploy_core_contracts(
        "http://localhost:8545",
        "private_key",
        deployer_address,
        config_data,
        None,
        None,
    ).await?;
    
    // Deploy AVS contracts
    let avs_contracts = deploy_avs_contracts(
        "http://localhost:8545",
        "private_key",
        deployer_address,
        1, // num_quorums
        core_contracts.permission_controller,
        core_contracts.allocation_manager,
        core_contracts.avs_directory,
        core_contracts.delegation_manager,
        core_contracts.pauser_registry,
        core_contracts.rewards_coordinator,
        core_contracts.strategy_factory,
        task_generator_addr,
        aggregator_addr,
        100, // task_response_window_block
    ).await?;
    
    // Setup permissions
    setup_avs_permissions(
        &core_contracts,
        &avs_contracts,
        &wallet,
        deployer_address,
        "metadata_uri".to_string(),
    ).await?;
    
    Ok(())
}
```

## Architecture

The library is structured as follows:

- **build.rs**: Handles the automatic generation of Rust bindings from Solidity contracts.
- **src/bindings/**: Contains the generated Rust bindings for contracts.
- **src/core.rs**: Functions for deploying EigenLayer core contracts.
- **src/deploy.rs**: Functions for deploying AVS-specific contracts.
- **src/helpers.rs**: Utility functions for contract deployment.
- **src/permissions.rs**: Functions for setting up permissions between contracts.

## 📜 License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your discretion.
