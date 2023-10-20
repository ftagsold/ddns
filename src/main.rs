use crate::config::IpType;
use anyhow::Result;
use config::Config;
use lazy_static::lazy_static;
use log::{debug, info};
use reqwest::blocking::ClientBuilder;
use serde_json::{json, Value};
use std::net::UdpSocket;
use std::thread::sleep;
use std::time::Duration;

mod config;

lazy_static! {
    static ref CONFIG: Config = envy::from_env::<Config>().unwrap();
}

fn main() -> Result<()> {
    env_logger::init();

    debug!("Dotenv initialized");

    let zone_id = get_zone_id();

    debug!("Zone ID: {zone_id}");

    loop {
        let ip = get_ip().expect("Could not get IP");

        let records = get_records(&zone_id);

        CONFIG.sub_domains.clone().iter().for_each(|sub_domain| {
            let record = records.iter().find(|record| {
                record
                    .get("name")
                    .expect("Could not get record name")
                    .as_str()
                    .expect("Could not get record name as string")
                    == sub_domain
            });

            match record {
                Some(record) => update_record(record.clone(), &ip.clone()),
                None => create_record(sub_domain, &zone_id, &ip.clone()),
            }
        });

        sleep(Duration::from_secs(CONFIG.sleep));
    }
}

fn get_ip() -> Option<String> {
    let socket = match CONFIG.ip_type {
        IpType::A => {
            let socket = UdpSocket::bind("0.0.0.0:0").expect("Could not bind socket");
            socket
                .connect("8.8.8.8:80")
                .expect("Could not connect socket");

            socket
        }
        IpType::AAAA => {
            let socket = UdpSocket::bind("[::]:0").expect("Could not bind socket");
            socket
                .connect("[2001:4860:4860::8888]:80")
                .expect("Could not connect socket");

            socket
        }
    };

    let local_addr = socket.local_addr().expect("Could not get local address");

    match CONFIG.ip_type {
        IpType::A => {
            let ip = local_addr.ip().to_string();

            Some(ip)
        }
        IpType::AAAA => {
            let ip = local_addr.ip().to_string();

            Some(ip)
        }
    }
}

fn get_zone_id() -> String {
    let resp: Value = ClientBuilder::new()
        .use_rustls_tls()
        .build()
        .expect("Could not build client")
        .get(format!(
            "https://dns.hetzner.com/api/v1/zones?name={}",
            CONFIG.zone.clone()
        ))
        .header("Auth-API-Token", CONFIG.token.clone())
        .send()
        .expect("Could not get zone ID")
        .json()
        .expect("Could not parse zone ID");

    resp.get("zones")
        .expect("Could not get zones from value")
        .as_array()
        .expect("Could not get zones as array")
        .get(0)
        .expect("Could not get first zone")
        .get("id")
        .expect("Could not get zone ID")
        .as_str()
        .expect("Could not get zone ID as string")
        .to_string()
}

fn get_records(zone_id: &str) -> Vec<Value> {
    let resp: Value = ClientBuilder::new()
        .use_rustls_tls()
        .build()
        .expect("Could not build client")
        .get(format!(
            "https://dns.hetzner.com/api/v1/records?zone_id={}",
            zone_id
        ))
        .header("Auth-API-Token", CONFIG.token.clone())
        .send()
        .expect("Could not get records")
        .json()
        .expect("Could not parse records");

    resp.get("records")
        .expect("Could not get records from value")
        .as_array()
        .expect("Could not get records as array")
        .iter()
        .cloned()
        .filter_map(|record| {
            if CONFIG.sub_domains.contains(
                &record
                    .get("name")
                    .expect("Could not get record name")
                    .as_str()
                    .expect("Could not get record name as string")
                    .to_string(),
            ) {
                Some(record)
            } else {
                None
            }
        })
        .collect()
}

fn update_record(record: Value, ip: &str) {
    debug!("Record found: {:?}", record);

    if record
        .get("value")
        .expect("Could not get record value")
        .as_str()
        .expect("Could not get record value as string")
        == ip
    {
        debug!("Record value is already up to date");
    } else {
        let record_id = record
            .get("id")
            .expect("Could not get record ID")
            .as_str()
            .expect("Could not get record ID as string");

        let mut record = record
            .as_object()
            .expect("Could not get record as object")
            .clone();

        record.insert("value".to_string(), ip.to_string().into());

        ClientBuilder::new()
            .use_rustls_tls()
            .build()
            .expect("Could not build client")
            .put(format!(
                "https://dns.hetzner.com/api/v1/records/{record_id}"
            ))
            .header("Auth-API-Token", CONFIG.token.clone())
            .json(&record.clone())
            .send()
            .expect("Failed to update record");

        info!("Record updated: {:?}", record);
    }
}

fn create_record(sub_domain: &str, zone_id: &str, ip: &str) {
    let record = json!(
        {
            "value": ip,
            "ttl": CONFIG.ttl,
            "type": CONFIG.ip_type.to_string(),
            "name": sub_domain,
            "zone_id": zone_id
        }
    );

    ClientBuilder::new()
        .use_rustls_tls()
        .build()
        .expect("Could not build client")
        .post("https://dns.hetzner.com/api/v1/records".to_string())
        .header("Auth-API-Token", CONFIG.token.clone())
        .json(&record)
        .send()
        .expect("Failed to update record");

    info!("Record created: {:?}", record);
}
