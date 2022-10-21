pub mod errors;
pub mod events;
pub mod macros;
pub mod requests;
pub mod responses;
pub mod types;

/*
use responses::ResponseMessage;
use types::{Capabilities, InvalidatedAreas};

use anyhow::Result;

use serde::{de, Deserialize, Deserializer, Serialize};

#[derive(Serialize, Debug)]
struct FooBar {
  foo: InvalidatedAreas,
  bar: InvalidatedAreas,
}

fn main() -> Result<()> {
  let inv = FooBar {
    foo: types::InvalidatedAreas::Stacks,
    bar: types::InvalidatedAreas::String("hello".to_string()),
  };
  let j = serde_json::to_string(&inv)?;

  println!("{}", j);
  Ok(())
}
*/