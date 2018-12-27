#[macro_use]
extern crate clap;
extern crate failure;
extern crate openssl_probe;
extern crate rpc_lib;
extern crate serde;
extern crate serde_json;

use clap::{App, Arg};
use failure::{Error, ResultExt};
use rpc_lib::client::prelude::*;
use serde_json::Value;
use std::io;
use std::io::Read;

fn main() -> Result<(), Error> {
    const HOST: &str = "host";
    const METHOD: &str = "method";
    const INPUT: &str = "INPUT";
    const RAW: &str = "raw";

    let matches = App::new("rpc-cli")
        .version(crate_version!())
        .author("Dean Karn <dean.karn@gmail.com>")
        .about("Allows making RPC 2.0 requests")
        .arg(
            Arg::with_name(HOST)
                .short("h")
                .long("host")
                .help("The host to hit, eg. http:://localhost:4000/rpc")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name(METHOD)
                .short("m")
                .long("method")
                .help("The RPC Method, eg. Calc.Add")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name(RAW)
                .long("raw")
                .help("Return the raw results instesad of pretty printed"),
        )
        .arg(
            Arg::with_name(INPUT)
                .help("Sets the input or file to be used")
                .index(1),
        )
        .get_matches();

    // we rpc_lib uses reqwest, which requires openssl, which is installed in different locations
    // per os, this determines the location and sets the ENV vars; why does it matter? because
    // we want to statically compile this program.
    openssl_probe::init_ssl_cert_env_vars();

    let input = matches.value_of(INPUT);
    let data: String = match input {
        Some("-") => {
            let mut buffer = String::new();
            let stdin = io::stdin();
            let mut handle = stdin.lock();
            handle.read_to_string(&mut buffer)?;
            buffer
        }
        _ => match input {
            Some(s) => s.to_owned(),
            None => "".to_string(),
        },
    };
    let v: Value = serde_json::from_str(data.as_ref()).context("failed to parse INPUT to JSON")?;
    let host = matches.value_of(HOST).unwrap();
    let method = matches.value_of(METHOD).unwrap();
    let client =
        Client::new(host).with_user_agent(format!("rpc-cli/{}", env!("CARGO_PKG_VERSION")));
    let res: Value = client.call("1", method, v)?;

    let json = if matches.is_present(RAW) {
        serde_json::to_string(&res)?
    } else {
        serde_json::to_string_pretty(&res)?
    };
    print!("{}", json);
    Ok(())
}
