# Integration tests

This crate implements integration tests for dap-rs.

Currently the tests use jsonschema to verify the generated responses (only the responses at the 
moment). The schema is fetched from the [debug adapter protocol repository][1] and checked into
this one with a name containing the commit hash at the time of fetching it. For example,
`b01a8da52b83850c1a35e024bca09f7b285ac109_debugAdapterProtocol.json` is schema fetched from 
[this url][2]


[1]: https://github.com/microsoft/debug-adapter-protocol/
[2]: https://raw.githubusercontent.com/microsoft/debug-adapter-protocol/b01a8da52b83850c1a35e024bca09f7b285ac109/debugAdapterProtocol.json
