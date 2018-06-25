#rpc-lib

This is an RPC 2.0 library for making requests to a server, it has been modified to handle some non-standard server responses such as, string error return value.

#### Why not strict RPC 2.0 responses
Because I am supporting some legacy Go RPC server that are slightly off spec, however, this will also support the proper RPC 2.0 spec.

## Requirements

On Linux:

- OpenSSL 1.0.1, 1.0.2, or 1.1.0 with headers (see https://github.com/sfackler/rust-openssl)

On Windows and macOS:

- Nothing.


Usage
-----
First, add this to your `Cargo.toml`:
```toml
[dependencies]
rpc-lib = "1.0.0"
```

Here is a quite example to get you going:
```rust
extern crate rpc_lib;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use rpc_lib::client::prelude::*;
use std::error::Error;

#[derive(Serialize)]
struct Input {
    id: String,
}

#[derive(Deserialize, Debug)]
struct Project {
    id: String,
    url: String,
    enabled: bool,
}

fn main() -> Result<(), Box<Error>> {
    let input = Input {
        id: "x0x0x0x0".to_string(),
    };
    let client = Client::new("http://localhost:4000/rpc");
    let res: Project = client.call("1", "Project.Find", input)?;
    println!("{:?}", res);
    Ok(())
}
```