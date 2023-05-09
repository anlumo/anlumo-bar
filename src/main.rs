use std::{
    io::Write,
    net::IpAddr,
    time::{Duration, SystemTime},
};

use chrono::Local;
use mpris::PlayerFinder;
use serde::Serialize;
use tokio::time::sleep;

#[derive(Default, Debug, Serialize)]
struct Header {
    version: u32,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    click_events: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    cont_signal: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_signal: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
#[allow(dead_code)]
enum IntegerOrString {
    Integer(u32),
    String(String),
}

#[derive(Default, Debug, Serialize)]
struct Body {
    full_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    short_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    background: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    border: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    border_top: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    border_bottom: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    border_left: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    border_right: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    min_width: Option<IntegerOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    align: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    instance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    urgent: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    separator: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    separator_block_width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    markup: Option<String>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::io::Result<()> {
    let player_finder = PlayerFinder::new().unwrap();
    let header = Header {
        version: 1,
        click_events: false,
        cont_signal: None,
        stop_signal: None,
    };
    let mut stdout = std::io::stdout().lock();
    serde_json::to_writer(&mut stdout, &header)?;
    writeln!(stdout, "\n[[]")?;
    loop {
        let mut body = player_finder
            .find_active()
            .ok()
            .and_then(|player| player.get_metadata().ok())
            .map(|metadata| {
                vec![Body {
                    full_text: metadata.title().map(|s| s.to_owned()).unwrap_or_default(),
                    color: Some("#97a891".to_owned()),
                    separator_block_width: Some(20),
                    ..Default::default()
                }]
            })
            .unwrap_or_default();

        // Get a list of network interfaces
        let interfaces = pnet::datalink::interfaces();

        // Filter the list to get only non-loopback interfaces and IPv4 addresses
        let ips: Vec<_> = interfaces
            .into_iter()
            .flat_map(|interface| {
                interface
                    .ips
                    .into_iter()
                    .filter(|ip_network| match ip_network.ip() {
                        IpAddr::V4(ip) => !ip.is_loopback(),
                        IpAddr::V6(ip) => !ip.is_loopback() && ip.segments()[0] != 0xfe80,
                    })
                    .map(|ip_network| ip_network.ip().to_string())
            })
            .collect();

        body.push(Body {
            full_text: ips.join(" "),
            color: Some("#91a4a8".to_owned()),
            separator_block_width: Some(20),
            ..Default::default()
        });
        body.push(Body {
            full_text: format!("{}", Local::now().format("%F %T")),
            separator_block_width: Some(20),
            ..Default::default()
        });

        write!(stdout, ",")?;
        serde_json::to_writer(&mut stdout, &body)?;
        write!(stdout, "\n")?;
        stdout.flush()?;

        let time_since_the_epoch = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let time_until_next_second =
            Duration::new(1, 0) - Duration::new(0, time_since_the_epoch.subsec_nanos());
        sleep(time_until_next_second).await;
    }
}
