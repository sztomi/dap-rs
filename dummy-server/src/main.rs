use std::fs::File;
use std::io::{BufWriter, BufReader};

use dap::adapter::Adapter;
use dap::client::{BasicClient};
use dap::requests::Request;
use dap::responses::{Response};
use dap::server::Server;

use anyhow::Result;

struct MyAdapter;

impl Adapter for MyAdapter {
  fn accept(&mut self, request: Request) -> Response {
    println!("accept {:?}", request);
    Response::make_ack(&request).unwrap()
  }
}

fn main() -> Result<()> {
  let adapter = MyAdapter{};
  let client = BasicClient::new(BufWriter::new(std::io::stdout()));
  let mut server = Server::new(adapter, client);
  let f = File::open("testinput.txt")?;
  let mut reader = BufReader::new(f);

  server.run(&mut reader)?;
  Ok(())
}
