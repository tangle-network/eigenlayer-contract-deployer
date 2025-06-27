#![allow(
    non_camel_case_types,
    non_snake_case,
    clippy::pub_underscore_fields,
    clippy::missing_errors_doc,
    clippy::too_many_arguments,
    clippy::used_underscore_binding,
    clippy::cast_possible_truncation
)]
use std::collections::BTreeMap;

use alloy_provider::Network;

use crate::bindings::SlashingRegistryCoordinator::constructorCall;
use crate::bindings::core::{
    quorum_bitmap_history_lib, signature_checker_lib, slashing_registry_coordinator,
};

pub const ARTIFACT: &str = include_str!(
    "../dependencies/eigenlayer-middleware-1.3.1/out/SlashingRegistryCoordinator.sol/SlashingRegistryCoordinator.json"
);

fn artifact() -> std::io::Result<serde_json::Value> {
    serde_json::from_str(ARTIFACT).map_err(std::io::Error::other)
}

fn contract_bytecode()
-> std::io::Result<(String, serde_json::value::Map<String, serde_json::Value>)> {
    let artifact = artifact()?;
    let bytecode = artifact["deployedBytecode"]
        .as_str()
        .ok_or_else(|| std::io::Error::other("No deployed bytecode found"))?;
    let link_references = artifact["linkReferences"]
        .as_object()
        .cloned()
        .unwrap_or_default();
    Ok((bytecode.to_string(), link_references))
}

fn link_all_fully_qualified(
    bytecode: &str,
    link_references: &serde_json::Map<String, serde_json::Value>,
    libs: &BTreeMap<&str, alloy_sol_types::private::Address>,
) -> String {
    let mut replacements = Vec::new();

    // Iterate over link references
    for (file_name, references) in link_references {
        // Check if we have a corresponding library address
        if let Some(&lib_address) = libs.get(file_name.as_str()) {
            // Get the references object for this file
            if let Some(refs_obj) = references.as_object() {
                // Iterate through each contract's references in this file
                for (_contract_name, positions) in refs_obj {
                    if let Some(positions_array) = positions.as_array() {
                        // Process each position
                        for position in positions_array {
                            if let (Some(start), Some(length)) =
                                (position["start"].as_u64(), position["length"].as_u64())
                            {
                                // Convert address to hex string and prepare replacement
                                let addr_hex = format!("{:x}", lib_address);
                                let addr_hex = addr_hex.strip_prefix("0x").unwrap_or(&addr_hex);

                                // Pad with zeros or trim to match required length
                                let replacement = if addr_hex.len() < length as usize {
                                    format!("{:0>width$}", addr_hex, width = length as usize)
                                } else {
                                    addr_hex[..length as usize].to_string()
                                };

                                replacements.push((
                                    start as usize * 2,
                                    length as usize * 2,
                                    replacement,
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    // Sort replacements by start index in descending order to preserve offsets
    replacements.sort_by(|a, b| b.0.cmp(&a.0));

    // Apply replacements to the bytecode
    let mut result = bytecode.to_string();
    for (start, length, replacement) in replacements {
        if start + length <= result.len() {
            result.replace_range(start..start + length, &replacement);
        }
    }

    result
}

pub async fn deploy_builder<P: alloy_contract::private::Provider<N> + Clone, N: Network>(
    provider: P,
    _stakeRegistry: alloy::sol_types::private::Address,
    _blsApkRegistry: alloy::sol_types::private::Address,
    _indexRegistry: alloy::sol_types::private::Address,
    _socketRegistry: alloy::sol_types::private::Address,
    _allocationManager: alloy::sol_types::private::Address,
    _pauserRegistry: alloy::sol_types::private::Address,
    _version: alloy::sol_types::private::String,
) -> alloy_contract::Result<alloy_contract::RawCallBuilder<P, N>> {
    let (bytecode, link_references) = contract_bytecode().expect("Failed to get contract bytecode");
    // Deploy library contracts if needed.
    let quourm_bitmap_history_lib =
        quorum_bitmap_history_lib::QuorumBitmapHistoryLib::deploy(provider.clone()).await?;
    let signature_checker_lib =
        signature_checker_lib::SignatureCheckerLib::deploy(provider.clone()).await?;
    let libs = BTreeMap::from([
        (
            "src/libraries/QuorumBitmapHistoryLib.sol",
            *quourm_bitmap_history_lib.address(),
        ),
        (
            "src/libraries/SignatureCheckerLib.sol",
            *signature_checker_lib.address(),
        ),
    ]);
    let linked_bytecode =
        alloy::hex::decode(link_all_fully_qualified(&bytecode, &link_references, &libs))
            .expect("Failed to decode linked bytecode");
    Ok(alloy_contract::RawCallBuilder::<P, N>::new_raw_deploy(
        provider,
        [
            &linked_bytecode[..],
            &alloy_sol_types::SolConstructor::abi_encode(&constructorCall {
                _stakeRegistry,
                _blsApkRegistry,
                _indexRegistry,
                _socketRegistry,
                _allocationManager,
                _pauserRegistry,
                _version,
            })[..],
        ]
        .concat()
        .into(),
    ))
}

pub async fn deploy<P: alloy_contract::private::Provider<N> + Clone, N: Network>(
    provider: P,
    _stakeRegistry: alloy::sol_types::private::Address,
    _blsApkRegistry: alloy::sol_types::private::Address,
    _indexRegistry: alloy::sol_types::private::Address,
    _socketRegistry: alloy::sol_types::private::Address,
    _allocationManager: alloy::sol_types::private::Address,
    _pauserRegistry: alloy::sol_types::private::Address,
    _version: alloy::sol_types::private::String,
) -> alloy_contract::Result<
    slashing_registry_coordinator::SlashingRegistryCoordinator::SlashingRegistryCoordinatorInstance<
        P,
        N,
    >,
> {
    let builder = deploy_builder(
        provider.clone(),
        _stakeRegistry,
        _blsApkRegistry,
        _indexRegistry,
        _socketRegistry,
        _allocationManager,
        _pauserRegistry,
        _version,
    )
    .await?;
    let deployed = builder.deploy().await?;
    Ok(slashing_registry_coordinator::SlashingRegistryCoordinator::new(deployed, provider))
}
