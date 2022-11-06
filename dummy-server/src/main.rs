use std::fs::File;
use std::io::{BufWriter, BufReader};

use dap_rs::adapter::Adapter;
use dap_rs::client::{BasicClient};
use dap_rs::requests::Request;
use dap_rs::responses::{Response, ResponseBody};
use dap_rs::server::Server;

use anyhow::Result;

struct MyAdapter;

impl Adapter for MyAdapter {
  fn accept(&mut self, request: Request) -> Response {
    println!("accept {:?}", request);
    Response::make_ack(&request).unwrap()
  }
}

fn main() -> Result<()> {
  let mut adapter = MyAdapter{};
  let mut client = BasicClient::new(BufWriter::new(std::io::stdout()));
  let mut server = Server::new(adapter, client);
  let f = File::open("testinput.txt")?;
  let mut reader = BufReader::new(f);

  server.run(&mut reader)?;
  Ok(())
}
