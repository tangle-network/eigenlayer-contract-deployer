use alloy_primitives::Address;
use alloy_provider::{Provider, ProviderBuilder};

// Ensure our custom deploy path actually deploys code on-chain.
// This test is ignored by default because it requires a local `anvil` binary.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires anvil binary"]
async fn deploy_slashing_registry_coordinator_on_anvil() {
    // Skip fast if `anvil` binary is not available or the opt-in var is missing.
    if std::env::var("RUN_ANVIL_TESTS").is_err() {
        eprintln!("skipping: set RUN_ANVIL_TESTS=1 to run");
        return;
    }

    // Build a wallet-backed provider with the first dev key.
    let provider = ProviderBuilder::new().connect_anvil_with_wallet();

    // Deploy a minimal PauserRegistry (constructor requires non-zero address).
    let pauser = provider.get_accounts().await.expect("get accounts")[0];
    let pausers = vec![pauser];
    let pauser_registry =
        eigenlayer_contract_deployer::bindings::PauserRegistry::deploy(&provider, pausers, pauser)
            .await
            .expect("pauser registry deploy");
    let pauser_registry_addr = *pauser_registry.address();

    // Deploy SlashingRegistryCoordinator using our custom path.
    let deployed = eigenlayer_contract_deployer::slashing_registry_coordinator::deploy(
        &provider,
        Address::ZERO,        // _stakeRegistry
        Address::ZERO,        // _blsApkRegistry
        Address::ZERO,        // _indexRegistry
        Address::ZERO,        // _socketRegistry
        Address::ZERO,        // _allocationManager
        pauser_registry_addr, // _pauserRegistry (must be non-zero)
        "test-version".to_string(),
    )
    .await
    .expect("slashing registry coordinator deploy");

    let addr = *deployed.address();

    // Verify code exists at the deployed address.
    let code = provider.get_code_at(addr).await.expect("get code");
    assert!(
        !code.0.is_empty(),
        "expected non-empty code, got empty at {}",
        addr
    );
}
