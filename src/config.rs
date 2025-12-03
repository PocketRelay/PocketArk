use log::LevelFilter;
use serde::Deserialize;
use std::{
    env,
    fs::read_to_string,
    net::{IpAddr, Ipv4Addr},
    path::Path,
};

/// The server version extracted from the Cargo.toml
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Environment variable key to load the config from
const CONFIG_ENV_KEY: &str = "PA_CONFIG_JSON";

pub fn load_config() -> Option<Config> {
    // Attempt to load the config from the env
    if let Ok(env) = env::var(CONFIG_ENV_KEY) {
        let config: Config = match serde_json::from_str(&env) {
            Ok(value) => value,
            Err(err) => {
                eprintln!("Failed to load env config (Using default): {err:?}");
                return None;
            }
        };
        return Some(config);
    }

    // Attempt to load the config from disk
    let file = Path::new("config.json");
    if !file.exists() {
        return None;
    }

    let data = match read_to_string(file) {
        Ok(value) => value,
        Err(err) => {
            eprintln!("Failed to load config file (Using defaults): {err:?}");
            return None;
        }
    };

    let config: Config = match serde_json::from_str(&data) {
        Ok(value) => value,
        Err(err) => {
            eprintln!("Failed to load config file (Using default): {err:?}");
            return None;
        }
    };

    Some(config)
}

pub type Port = u16;

#[derive(Deserialize)]
#[serde(default)]
pub struct Config {
    pub host: IpAddr,
    pub port: Port,
    pub reverse_proxy: bool,
    pub logging: LevelFilter,
    pub tunnel: TunnelConfig,
    pub udp_tunnel: UdpTunnelConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            port: 80,
            reverse_proxy: false,
            logging: LevelFilter::Info,
            tunnel: Default::default(),
            udp_tunnel: Default::default(),
        }
    }
}

/// Configuration for how the server should use tunneling
///
/// This option applies to both the HTTP and UDP tunnels
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TunnelConfig {
    /// Only tunnel players with non "Open" NAT types if the QoS
    /// server is set to [`QosServerConfig::Disabled`] this is
    /// equivalent to [`TunnelConfig::Always`]
    #[default]
    Stricter,
    /// Always tunnel connections through the server regardless
    /// of NAT type
    Always,
    /// Never tunnel connections through the server
    Disabled,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct UdpTunnelConfig {
    /// Port to bind the UDP tunnel socket to, the socket is bound
    /// using the same host as the server
    pub port: Port,

    /// External facing port, only needed when the port visible to users
    /// is different to [UdpTunnelConfig::port]
    ///
    /// For cases such as different exposed port in docker or usage behind
    /// a reverse proxy such as NGINX
    pub external_port: Option<Port>,

    /// Optionally choose to disable the tunnel if you don't intend to use it
    /// default value is true
    pub enabled: bool,
}

impl Default for UdpTunnelConfig {
    fn default() -> Self {
        Self {
            port: 9032,
            external_port: None,
            enabled: true,
        }
    }
}

impl UdpTunnelConfig {
    /// Get the port the exposed to the clients for the UDP
    /// tunnel. This is [None] if the tunnel is disabled. Otherwise
    /// its [UdpTunnelConfig::external_port] if set otherwise its
    /// [UdpTunnelConfig::port]
    pub fn get_exposed_port(&self) -> Option<Port> {
        if !self.enabled {
            return None;
        }

        Some(self.external_port.unwrap_or(self.port))
    }
}
