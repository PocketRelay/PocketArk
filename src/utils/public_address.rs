use futures::TryFutureExt;
use std::net::Ipv4Addr;

/// Prints a list of possible urls that can be used to connect to
/// this Pocket relay server
pub async fn log_connection_urls(http_port: u16) {
    let mut output = String::new();
    if let Ok(local_address) = local_ip_address::local_ip() {
        output.push_str("LAN: ");
        output.push_str(&local_address.to_string());
        if http_port != 80 {
            output.push(':');
            output.push_str(&http_port.to_string());
        }
    }
    if let Some(public_address) = public_address().await {
        if !output.is_empty() {
            output.push_str(", ");
        }
        output.push_str(&format!("WAN: {public_address}"));
        if http_port != 80 {
            output.push(':');
            output.push_str(&http_port.to_string());
        }
    }

    if !output.is_empty() {
        output.push_str(", ");
    }

    output.push_str("LOCAL: 127.0.0.1");
    if http_port != 80 {
        output.push(':');
        output.push_str(&http_port.to_string());
    }

    tracing::info!("Connection URLS ({output})");
}

/// Retrieves the public address of the server either using the cached
/// value if its not expired or fetching the new value from the API using
/// `fetch_public_addr`
pub async fn public_address() -> Option<Ipv4Addr> {
    // Try fetch from cloudflare first
    if let Some(addr) = public_address_cloudflare().await {
        return Some(addr);
    }

    // API addresses for IP lookup
    const ADDRESSES: [&str; 2] = ["https://api.ipify.org/", "https://ipv4.icanhazip.com/"];

    // Try all addresses using the first valid value
    for address in ADDRESSES {
        let addr = match reqwest::get(address)
            // Read the response as text
            .and_then(reqwest::Response::text)
            .await
        {
            Ok(value) => value,
            Err(_) => continue,
        };

        let addr = addr
            // Trim whitespace and new lines
            .trim_matches(|c: char| c == '\n' || c.is_whitespace())
            // Attempt to parse as an IPv4 address
            .parse::<Ipv4Addr>();

        if let Ok(parsed) = addr {
            return Some(parsed);
        }
    }

    None
}

/// Retrieves the public address of the server using the cloudflare API
pub async fn public_address_cloudflare() -> Option<Ipv4Addr> {
    let response = match reqwest::get("https://cloudflare.com/cdn-cgi/trace")
        .and_then(reqwest::Response::text)
        .await
    {
        Ok(value) => value,
        Err(_) => return None,
    };

    // Find the line containing the IP address
    let (_, addr) = response
        .lines()
        .filter_map(|value| value.split_once('='))
        .find(|(key, _)| *key == "ip")?;

    // Attempt to parse it as an IPv4 address
    let addr = addr
        // Trim whitespace and new lines
        .trim_matches(|c: char| c == '\n' || c.is_whitespace())
        // Attempt to parse as an IPv4 address
        .parse::<Ipv4Addr>();

    addr.ok()
}
