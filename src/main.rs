#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate clap;
extern crate traceroute;
extern crate hostname;
extern crate dns_lookup;

use clap::{App, Arg};

use std::net::IpAddr;
use std::time::Duration;
use std::thread::sleep;

#[derive(Debug, Serialize)]
struct TraceRoute {
    source: Option<String>,
    dest: String,
    hops: Vec<Option<Hop>>,
}

#[derive(Debug, Serialize)]
struct Hop {
    ttl: u8,
    ip: String,
    #[serde(skip_serializing_if="Option::is_none")]
    hostname: Option<String>,
    rtt: Duration,
}

fn validate_addr(addr: String) -> Result<(), String> {
    Ok(())
}

fn main() {
    let matches = App::new("xtraceroute")
        .arg(Arg::with_name("dest")
             .validator(validate_addr)
             .index(1)
             .required(true)
            .value_name("DESTINATION")
            .help("Specify the destination (IP address or hostname)")
            .takes_value(true))
        .get_matches();

    let wait_time = Duration::from_millis(200);
    let dest = matches.value_of_os("dest").unwrap().to_string_lossy().into_owned();
    let mut hops: Vec<Option<Hop>> = Vec::new();

    let tracert = traceroute::execute(&dest).unwrap(); // TODO: validate this/handle error

    for hop_result in tracert {
        hops.push(None);

        match hop_result {
            Ok(hop) => {
                let index  = (hop.ttl - 1) as usize;

                let hop = Hop {
                    ttl: hop.ttl,
                    ip: hop.host.ip().to_string(),
                    hostname: dns_lookup::lookup_addr(&hop.host.ip()).ok().and_then(|ip| Some(ip.to_string())),
                    rtt: Duration::from_millis(hop.rtt.num_milliseconds() as u64), // Because std::time::Duration and time::duration::Duration are two different things >.>
                };

                hops[index] = Some(hop);
            },
            Err(e) => {
                eprintln!("Error: {}", e);
            },
        }
        sleep(wait_time);
    }

    let traceroute = TraceRoute {
        source: hostname::get_hostname(),
        dest: dest,
        hops: hops,
    };

    println!("{}", serde_json::to_string_pretty(&traceroute).unwrap());
}
