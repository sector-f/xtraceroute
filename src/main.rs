#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate clap;
extern crate trust_dns_resolver;
extern crate traceroute;
extern crate hostname;

use trust_dns_resolver::Resolver;
use clap::{App, Arg};

// use std::net::{IpAddr, Ipv4Addr};
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
    ip: String,
    #[serde(skip_serializing_if="Option::is_none")]
    hostname: Option<String>,
    rtt: Duration,
}

fn main() {
    let matches = App::new("xtraceroute")
        .arg(Arg::with_name("dest")
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
    let resolver = Resolver::from_system_conf().unwrap(); // TODO: add more options to this (and make it async)

    for hop_result in tracert {
        match hop_result {
            Ok(hop) => {
                let index  = (hop.ttl - 1) as usize;
                hops.push(None);

                let hop = Hop {
                    ip: hop.host.ip().to_string(),
                    hostname: resolver.reverse_lookup(hop.host.ip()).ok().and_then(|ip| Some(ip.iter().nth(0).unwrap().to_string())),
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
