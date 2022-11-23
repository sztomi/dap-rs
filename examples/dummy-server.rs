use std::fs::File;
use std::io::{BufReader, BufWriter};

use thiserror::Error;

use dap::prelude::*;


struct MyAdapter;

#[derive(Error, Debug)]
enum MyAdapterError {
  #[error("Unhandled command")]
  UnhandledCommandError,
}

impl Adapter for MyAdapter {
  type Error = MyAdapterError;

  fn accept(
    &mut self,
    request: Request,
    _ctx: &mut dyn Context,
  ) -> Result<Option<Response>, Self::Error> {
    eprintln!("Accept {:?}\n", request.command);

    match &request.command {
      Command::Initialize(args) => {
        eprintln!(
          "> Client '{}' requested initialization.",
          args.client_name.as_ref().unwrap()
        );
        Ok(Some(Response::make_success(
          &request,
          ResponseBody::Initialize(Some(types::Capabilities {
            supports_configuration_done_request: Some(true),
            supports_evaluate_for_hovers: Some(true),
            ..Default::default()
          })),
        )))
      }
      Command::Next(_) => Ok(Some(Response::make_ack(&request).unwrap())),
      _ => Err(MyAdapterError::UnhandledCommandError)
    }
  }
}

type DynResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> DynResult<()> {
  let adapter = MyAdapter {};
  let client = BasicClient::new(BufWriter::new(std::io::stdout()));
  let mut server = Server::new(adapter, client);

  let f = File::open("testinput.txt")?;
  let mut reader = BufReader::new(f);

  server.run(&mut reader)?;
  Ok(())
}
