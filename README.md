## Client Hardware Info Tool

Provides information about a node's hardware.

## How to build

``cargo build --release`` Automatically build a static Linux 64 bit binary as defined in the config.toml file. 

## How to run

Simply execute ``./target/release/hwinfo`` if you built it or excute the binary in the release folder if you downloaded a release.

There are 3 arguments that need to be passed *initially* when first running the application in order to send a periodical heartbeart to the server:

`--api-url <url>`  Defines the base URL, e.g. https://api.exalsius.ai  
`--access-token <token>` The access/auth token  
`--node-id <id>`  The node id defined during initial connection

The tool automatically creates a configuration file from which the values are fetched.
