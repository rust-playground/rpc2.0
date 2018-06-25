#rpc-cli

This is an RPC 2.0 cli.

## Requirements

On Linux:

- OpenSSL 1.0.1, 1.0.2, or 1.1.0 with headers (see https://github.com/sfackler/rust-openssl)

On Windows and macOS:

- Nothing.

Usages
------
```shell
USAGE:
    rpc-cli [FLAGS] --host <host> --method <method> [INPUT]

FLAGS:
        --help       Prints help information
        --raw        Return the raw results instesad of pretty printed
    -V, --version    Prints version information

OPTIONS:
    -h, --host <host>        The host to hit, eg. http:://localhost:4000/rpc
    -m, --method <method>    The RPC Method, eg. Calc.Add

ARGS:
    <INPUT>    Sets the input or file to be used

```