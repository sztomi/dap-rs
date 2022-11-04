use std::io::BufRead;

use crate::adapter::Adapter;
use crate::errors::ServerError;

enum InputState {
  Header,
  Content,
}

pub struct Server<A: Adapter> {
  adapter: A,
}

impl<A: Adapter> Server<A> {
  pub fn new(adapter: A) -> Self {
    Self { adapter }
  }

  pub fn run<Buf: BufRead>(&mut self, input: &mut Buf) -> Result<(), ServerError> {
    let mut state = InputState::Header;
    let mut buffer = String::new();

    loop {
      match state {
        InputState::Header => {
          if let Ok(read_size) = input.read_line(&mut buffer) {
            let parts: Vec<&str> = buffer.split(':').collect();
            if parts.len() == 2 && parts[0] == "Content-length" {
              let content_length: usize = parts[1].parse().unwrap(); // TODO
              buffer.clear();
              buffer.reserve(content_length);
            }
          }
        },
        InputState::Content => todo!(),
      }
    }
  }
}
