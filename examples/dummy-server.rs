use std::fs::File;
use std::io::{BufReader, BufWriter};

use thiserror::Error;

use dap::prelude::*;

#[derive(Error, Debug)]
enum MyAdapterError {
  #[error("Unhandled command")]
  UnhandledCommandError,

  #[error("Missing command")]
  MissingCommandError,
}
type DynResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> DynResult<()> {
  let output = BufWriter::new(std::io::stdout());
  let f = File::open("testinput.txt")?;
  let input = BufReader::new(f);
  let mut server = Server::new(input, output);

  let req = match server.poll_request()? {
    Some(req) => req,
    None => return Err(Box::new(MyAdapterError::MissingCommandError)),
  };
  if let Command::Initialize(_) = req.command {
    let rsp = req.success(ResponseBody::Initialize(Some(types::Capabilities {
      ..Default::default()
    })));

    // When you call respond, send_event etc. the message will be wrapped
    // in a base message with a appropriate seq number, so you don't have to keep track of that yourself
    server.respond(rsp)?;

    server.send_event(Event::Initialized)?;
  } else {
    return Err(Box::new(MyAdapterError::UnhandledCommandError));
  }
  Ok(())
}
