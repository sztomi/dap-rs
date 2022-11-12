use std::fs::File;
use std::io::{BufReader, BufWriter};

use dap::responses::ResponseBody::Initialize;
use dap::types::Capabilities;
use dap::{Adapter, BasicClient, Context, Command, Request, Response, Server};

use anyhow::Result;

struct MyAdapter;

impl Adapter for MyAdapter {
  fn accept(&mut self, request: Request, _ctx: &mut dyn Context) -> Response {
    eprintln!("Accept {:?}\n", request.command);

    match &request.command {
      Command::Initialize(args) => {
        eprintln!(
          "> Client '{}' requested initialization.",
          args.client_name.as_ref().unwrap()
        );
        Response::make_success(&request, Initialize(Some(Capabilities {
            supports_configuration_done_request: Some(true),
            supports_evaluate_for_hovers: Some(true),
            ..Default::default()
        })))
      }
      Command::Next(_) => Response::make_ack(&request).unwrap(),
      _ => todo!(),
    }
  }
}

fn main() -> Result<()> {
  let adapter = MyAdapter {};
  let client = BasicClient::new(BufWriter::new(std::io::stdout()));
  let mut server = Server::new(adapter, client);

  let f = File::open("testinput.txt")?;
  let mut reader = BufReader::new(f);

  server.run(&mut reader)?;
  Ok(())
}
