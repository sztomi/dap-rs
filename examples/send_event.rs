use std::error;
use std::fs::File;
use std::io::{BufReader, BufWriter};

use thiserror::Error;

use dap::prelude::*;

#[derive(Error, Debug)]
enum MyAdapterError {
  #[error("Error while sending error")]
  SendEventError(#[from] ClientError),
}

struct MyAdapter;

impl Adapter for MyAdapter {
  type Error = MyAdapterError;

  fn accept(&mut self, _request: Request, ctx: &mut dyn Context) -> Result<Response, Self::Error> {
    ctx
      .send_event(Event::Initialized)
      .map_err(MyAdapterError::SendEventError)?;
    Ok(Response::empty())
  }
}

type DynResult<T> = std::result::Result<T, Box<dyn error::Error>>;

fn main() -> DynResult<()> {
  let adapter = MyAdapter {};
  let client = BasicClient::new(BufWriter::new(std::io::stdout()));
  let mut server = Server::new(adapter, client);

  let f = File::open("testinput.txt")?;
  let mut reader = BufReader::new(f);

  server.run(&mut reader)?;
  Ok(())
}
