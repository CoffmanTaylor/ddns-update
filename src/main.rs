use pnet::datalink::interfaces;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[derive(Serialize, Deserialize)]
struct Config {
    username: String,
    password: String,
    domain: String,
}

fn main() {
    // Readin the config file.
    let config_file = File::open("/etc/gddns.conf").expect("Failed to open file.");
    let reader = BufReader::new(config_file);
    let config: Config = serde_json::from_reader(reader).expect("Invaild config file.");
    assert_eq!(
        16,
        config.username.len(),
        "The username must be exactly 16 chars long."
    );
    assert_eq!(
        16,
        config.password.len(),
        "The password must be exactly 16 chars long."
    );

    // Get a vector with all network interfaces found
    let all_interfaces = interfaces();

    // Search for the default interface - the one that is
    // up, not loopback and has an IP.
    let default_interface = all_interfaces
        .iter()
        .find(|e| e.is_up() && !e.is_loopback() && !e.ips.is_empty())
        .expect("Unable to find valid an interface.");

    // Search for the default IPv6 address, the one with the largest prefix.
    let ip_address = default_interface
        .ips
        .iter()
        .filter(|ip| ip.is_ipv6())
        .max_by_key(|ip| ip.prefix())
        .expect("Unable to find an IPv6 address.");

    assert_eq!(
        128,
        ip_address.prefix(),
        "The IP address is not a complete address."
    );

    // format the ip address
    let ip_address = ip_address.network().to_string();

    let client = reqwest::blocking::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()
        .expect("Failed to build the client");

    let res = client
        .post(&format!(
            "https://{}:{}@domains.google.com/nic/update",
            config.username, config.password
        ))
        .form(&vec![("hostname", config.domain), ("myip", ip_address)])
        .send()
        .expect("Failed to post");

    let res = res.text().expect("Failed to get text from response.");

    if !(res.starts_with("good") || res.starts_with("nochg")) {
        panic!(format!("Bad response: {:?}", res));
    }
}
