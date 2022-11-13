# dap-rs, a Rust implementation of the Debug Adapter Protocol

## Introduction

This crate is a Rust implementation of the [Debug Adapter Protocol][1] (or DAP for short).

The best way to think of DAP is to compare it to [LSP][2] (Language Server Protocol) but
for debuggers. The core idea is the same: a protocol that serves as *lingua franca*
for editors and debuggers to talk to each other. This means that an editor that implements
DAP can use a debugger that also implements DAP.

In practice, the adapter might be separate from the actual debugger. For example, one could
implement an adapter that writes commands to the stdin of a gdb subprocess, then parses
the output it receives (this is why it's called an "adapter" - it adapts the debugger to
editors that know DAP).

## License

This library is dual-licensed as MIT and Apache 2.0. That means users may choose either of these
licenses. In general, these are non-restrictive, non-viral licenses, a.k.a. *"do what you want
but no guarantees from me"*.

Commercial support is available on a contract basis (contact me szelei.t@gmail.com).

## Getting started

To get started, create a binary project and add `dap` to your Cargo.toml:

```toml
[package]
name = "dummy-server"
version = "0.1.0"
edition = "2021"

[dependencies]
dap = "0.1.0"
```

Import the following types:

```rust
use dap::{Adapter, BasicClient, Request, Response, Server, Context};
```

Create your `Adapter` which is going to be the heart of your implementation.
Its `accept` function will be called for each incoming request, and each return type will be
returned to the client in its serialized form.

```rust
struct MyAdapter;

impl Adapter for MyAdapter {
  fn accept(&mut self, request: Request, _ctx: &mut dyn Context) -> Response {
    println!("accept {:?}", request);
    Response::make_ack(&request).unwrap()
  }
}
```

The `request` will be deserialized and its `command` field will be one of the [requests][3]
variants. In practice, this function will likely contain a large `match` expression or
some other means of dispatching the requests to code that can handle them.

The currently unused `_ctx` parameter could be used to send events and reverse requests to the
client.

What about error handling?

As you may have noticed, we are unwrapping a result in the above example, which may cause
a panic. In real-life application code, you should seldom unwrap a Result this way.
For DAP clients, and ultimately, users, it is important to see the error that is happening,
instead of the adapter suddenly dying on them. The DAP protocol allows returning response messages
that contain errors. This is why the `accept` function does not return a result. Implementors
of the `Adapter` trait should map their errors to error responses.

After this, you will want to create the infrastructure for communicating with the client. First,
an instance of your adapter:

```rust
let adapter = MyAdapter{};
```
Then, a client. In this crate, the Client is responsible for sending the responses, events and
reverse requests to the actual client that is connected.

```rust
let client = BasicClient::new(BufWriter::new(std::io::stdout()));
```

`BasicClient` is a builtin implementation that takes a `BufWriter` where the serialized
responses, event and reverse requests are written. It is easy and typical to write to the
standard output, but some implementations may want to write to a socket instead.

The `Client` and `Context` traits can be implemented to provide different behavior.

Next, we create the `Server`. The `Server` ties together the `Adapter` and the `Client`. Most
importantly, it is the server's responsibility to deserialize the incoming JSON requests,
pass them to the `Adapter`, then take the return value and pass it to the `Client` (which
in turn will serialize it and write it to real client's buffer).

```rust
let mut server = Server::new(adapter, client);
```
Finally, we create a `BufReader` for the server which serves as the input mechanism and run the
server.

```rust
let mut reader = BufReader::new(std::io::stdin());
server.run(&mut reader)?;
```

The server will run until EOF is encountered.

[1]: https://microsoft.github.io/debug-adapter-protocol/
[2]: https://microsoft.github.io/language-server-protocol/
[3]: https://microsoft.github.io/debug-adapter-protocol/specification#Requests