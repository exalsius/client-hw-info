<div align="center">

<p><img src="./docs/assets/logo_banner.png" alt="exalsius banner" width="250"></p>


<h1>Client Hardware Info Tool</h1>

[![CI Pipeline](https://github.com/exalsius/client-hw-info/actions/workflows/ci.yml/badge.svg)](https://github.com/exalsius/client-hw-info/actions/workflows/ci.yml)


</div>

The client hardware tool is a small utility that can be used to extract hardware information and optionally send a heartbeat and the gathered machine information to the [Exalsius API](https://github.com/exalsius/api).

## Features

- Read hardware information such as GPU, CPU, RAM, I/O from the machine
- Send the hardware information to the API server
- Continuously send a heartbeat to the API server to signal that the client is still alive

## Quickstart

Download the latest release from the [releases page](https://github.com/exalsius/client-hw-info/releases).  
Ensure that the binary is executable. If not, run ``chmod +x client-hw-info-v*``  
Execute ``./client-hw-info-v* --skip-heartbeat`` 

## How to run

Simply execute ``./target/release/client-hw-info-v*`` if you built it or execute the binary in the release folder if you downloaded a release.

There are 3 arguments that need to be passed *initially* when first running the application to send a periodical heartbeat to the server. 
Afterward, the application will fetch the env information from the config file and passing them is not necessary.
If you want to change the configuration, you can pass the arguments again and they will be overwritten.

`--api-url <url>`  Defines the base URL, e.g. https://api.exalsius.ai  
`--access-token <token>` The access/auth token  
`--node-id <id>`  The node id defined during initial connection

Optional execution:

`--skip-heartbeat`  If set, the heartbeat will not be sent and only hardware information will be fetched. No configuration file will be created.

The tool automatically creates a configuration file under $HOME/.config/exalsius/config.env from which the values are fetched.


## How to build

``cargo build --release`` automatically builds a static Linux 64 bit binary as defined in the config.toml file.   
Ensure that musl is installed on your system.

