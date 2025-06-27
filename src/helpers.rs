use std::str::FromStr;

use crate::bindings::{EmptyContract, ProxyAdmin, TransparentUpgradeableProxy};
use alloy::{
    network::EthereumWallet,
    signers::local::PrivateKeySigner,
    transports::{TransportErrorKind, http::reqwest::Url},
};
use alloy_primitives::{Address, Bytes, FixedBytes};
use alloy_provider::{
    PendingTransactionBuilder, PendingTransactionError, ProviderBuilder, RootProvider,
};
use color_eyre::eyre::eyre;
use tracing::error;

/// Helper function to deploy an empty proxy
///
/// # Errors
/// Returns an error if the deployment fails or if there are issues with the transaction receipt.
pub async fn deploy_empty_proxy(
    wallet: &RootProvider,
    proxy_admin: Address,
) -> color_eyre::eyre::Result<Address> {
    let data = Bytes::new();

    let empty_contract = EmptyContract::deploy(wallet).await?;
    let &empty_contract_addr = empty_contract.address();

    let proxy =
        TransparentUpgradeableProxy::deploy(wallet, empty_contract_addr, proxy_admin, data).await?;

    Ok(*proxy.address())
}

/// Helper function to upgrade a proxy with an implementation
///
/// # Errors
/// Returns an error if the upgrade fails or if there are issues with the transaction receipt.
pub async fn upgrade_proxy(
    wallet: &RootProvider,
    proxy_admin_addr: Address,
    proxy_addr: Address,
    implementation_addr: Address,
    data: Bytes,
) -> color_eyre::eyre::Result<()> {
    let proxy_admin = ProxyAdmin::new(proxy_admin_addr, wallet.clone());

    let receipt = if data.is_empty() {
        let call = proxy_admin.upgrade(proxy_addr, implementation_addr);
        get_receipt(call).await?
    } else {
        let call = proxy_admin.upgradeAndCall(proxy_addr, implementation_addr, data);
        get_receipt(call).await?
    };

    if !receipt.status() {
        return Err(eyre!("Failed to upgrade proxy"));
    }

    Ok(())
}

// use crate::error::Error;
// use crate::state::{AnvilState, get_default_state_json};
use alloy_contract::{CallBuilder, CallDecoder};
use alloy_provider::Provider;
use alloy_provider::network::Ethereum;
use alloy_rpc_types_eth::TransactionReceipt;
// use blueprint_core::{error, info};
// use std::fs;
// use tempfile::TempDir;
// use testcontainers::{
//     ContainerAsync, GenericImage, ImageExt,
//     core::{ExecCommand, IntoContainerPort, WaitFor},
//     runners::AsyncRunner,
// };
// use tokio::io::AsyncBufReadExt;

#[allow(clippy::missing_errors_doc)]
pub async fn get_receipt<P, D>(
    call: CallBuilder<P, D, Ethereum>,
) -> Result<TransactionReceipt, color_eyre::eyre::Error>
where
    P: Provider<Ethereum>,
    D: CallDecoder,
{
    let pending_tx = match call.send().await {
        Ok(tx) => tx,
        Err(e) => {
            error!("Failed to send transaction: {:?}", e);
            return Err(e.into());
        }
    };

    let receipt = match pending_tx.get_receipt().await {
        Ok(receipt) => receipt,
        Err(e) => {
            error!("Failed to get transaction receipt: {:?}", e);
            return Err(e.into());
        }
    };

    Ok(receipt)
}

#[allow(clippy::type_complexity)]
/// Get the provider for an http endpoint with the [`Wallet`](EthereumWallet) for the specified private key
///
/// # Returns
/// - [`RootProvider`] - The provider
///
/// # Panics
/// - If the provided http endpoint is not a valid URL
#[must_use]
pub fn get_provider_from_signer(key: &str, rpc_url: &str) -> RootProvider {
    let signer = PrivateKeySigner::from_str(key).expect("wrong key ");
    let wallet = EthereumWallet::from(signer);
    let url = Url::parse(rpc_url).expect("Wrong rpc url");
    ProviderBuilder::new()
        .wallet(wallet.clone())
        .connect_http(url)
        .root()
        .clone()
}

/// Wait for a transaction to finish and return its receipt.
///
/// # Arguments
///
/// `rpc_url` - The RPC URL.
/// `tx_hash` - The hash of the transaction.
///
/// # Returns
///
/// A [`TransportResult`] containing the transaction hash.
#[allow(clippy::missing_errors_doc)]
pub async fn wait_transaction(
    rpc_url: &str,
    tx_hash: FixedBytes<32>,
) -> Result<TransactionReceipt, PendingTransactionError> {
    let url = Url::parse(rpc_url).map_err(|_| TransportErrorKind::custom_str("Invalid RPC URL"))?;
    let root_provider = ProviderBuilder::new()
        .disable_recommended_fillers()
        .connect_http(url);
    let pending_tx = PendingTransactionBuilder::new(root_provider, tx_hash);
    pending_tx.get_receipt().await
}
