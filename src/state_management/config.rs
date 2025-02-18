use btc_heritage_wallet::bitcoin::Network;
use std::{path::PathBuf, str::FromStr, sync::OnceLock};

pub fn config() -> &'static Configuration {
    static CONFIGURATION: OnceLock<Configuration> = OnceLock::new();
    CONFIGURATION.get_or_init(|| {
        log::info!("Loading Configuration");
        let network = match std::env::var("BITCOIN_NETWORK").map(|s| Network::from_str(&s)) {
            Ok(Ok(net)) => net,
            _ => Network::Bitcoin,
        };
        log::debug!("network={network}");
        let datadir = match std::env::var("HERITAGE_WALLET_HOME").map(|s| PathBuf::from_str(&s)) {
            Ok(Ok(p)) => p,
            _ => {
                let mut home_path: PathBuf = dirs_next::home_dir().unwrap_or_default();
                home_path.push(".heritage-wallet");
                home_path
            }
        };
        log::debug!("datadir={}", datadir.to_str().expect("valid unicode"));
        let service_api_url = std::env::var("HERITAGE_SERVICE_API_URL")
            .unwrap_or("https://api.btcherit.com/v1".to_owned());
        log::debug!("service_api_url={service_api_url}");
        let auth_url = std::env::var("HERITAGE_AUTH_URL")
            .unwrap_or("https://device.crypto7.world/token".to_owned());
        log::debug!("auth_url={auth_url}");
        let auth_client_id = std::env::var("HERITAGE_AUTH_CLIENT_ID")
            .unwrap_or("cda6031ca00d09d66c2b632448eb8fef".to_owned());
        log::debug!("auth_client_id={auth_client_id}");
        let configuration = Configuration {
            network,
            datadir,
            heritage_service_config: HeritageServiceConfig {
                service_api_url,
                auth_url,
                auth_client_id,
            },
        };
        log::debug!("configuration={configuration:?}");
        configuration
    })
}

#[derive(Debug)]
pub struct Configuration {
    pub network: Network,
    pub datadir: PathBuf,
    pub heritage_service_config: HeritageServiceConfig,
}

#[derive(Debug)]
pub struct HeritageServiceConfig {
    pub service_api_url: String,
    pub auth_url: String,
    pub auth_client_id: String,
}
