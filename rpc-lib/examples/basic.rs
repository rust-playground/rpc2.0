extern crate rpc_lib;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate failure;

use failure::Error;
use rpc_lib::client::prelude::*;

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

fn main() -> Result<(), Error> {
    let input = Input {
        id: "x0x0x0x0".to_string(),
    };
    let client = Client::new("http://localhost:4000/rpc");
    let res: Project = client.call("1", "Project.Find", input)?;
    println!("{:?}", res);
    Ok(())
}
