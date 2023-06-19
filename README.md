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

## Stability

This crate is in a fairly early stage and breakages will be frequent. Any version before
1.0 might be a breaking version.


## License

This library is dual-licensed as MIT and Apache 2.0. That means users may choose either of these
licenses. In general, these are non-restrictive, non-viral licenses, a.k.a. *"do what you want
but no guarantees from me"*.

## Credit

This is a fork of [dap-rs](https://github.com/sztomi/dap-rs) by [Tamás Szelei](https://github.com/sztomi).
They made most of the code, i just fixed it up for my own purposes.

[1]: https://microsoft.github.io/debug-adapter-protocol/
[2]: https://microsoft.github.io/language-server-protocol/
[3]: https://microsoft.github.io/debug-adapter-protocol/specification#Requests