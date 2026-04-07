<div align="center">

<p><img src="./docs/assets/logo_banner.png" alt="exalsius banner" width="250"></p>


<h1>Client Hardware Info Tool</h1>

[![CI Pipeline](https://github.com/exalsius/client-hw-info/actions/workflows/ci.yml/badge.svg)](https://github.com/exalsius/client-hw-info/actions/workflows/ci.yml)


</div>

The client hardware tool is a small utility that can be used to extract hardware information and optionally send a heartbeat and the gathered machine information to the [Exalsius API](https://github.com/exalsius/api).

## Features

- Read hardware information such as GPU, CPU, RAM, I/O from the machine
- Collect software and operating system details
- Send the hardware information to the API server
- Continuously send a heartbeat to the API server to signal that the client is still alive
- Self-register a new node and optionally install a systemd timer for periodic heartbeats

## Quickstart

Download the latest release from the [releases page](https://github.com/exalsius/client-hw-info/releases).  
Ensure that the binary is executable. If not, run `chmod +x ./client-hw-info*`  
Execute `./client-hw-info* --skip-heartbeat`

## How to build

Run `cargo build --release`.

The compiled binary will be available at `./target/release/client-hw-info`.

## How to run

Execute `./target/release/client-hw-info` if you built the project locally, or run the downloaded release binary.

There are three main execution modes.

### 1. Collect hardware information only

Use this mode when you only want to inspect the machine locally.

`./target/release/client-hw-info --skip-heartbeat`

This collects hardware, software, and OS details and exits without contacting the API. No configuration file is created.

### 2. Send heartbeats for an already registered node

For the first heartbeat-enabled run, provide:

`--api-url <url>` Base API URL, for example `https://api.exalsius.ai`  
`--access-token <token>` Access token used for authentication  
`--node-id <id>` Node id of the existing node

Example:

```bash
./target/release/client-hw-info \
  --api-url https://api.exalsius.ai \
  --access-token <token> \
  --node-id <node-id>
```

After the first successful run, the tool stores the values in `$HOME/.config/exalsius/config.env` and reuses them automatically on later executions.

If you pass `--api-url`, `--access-token`, or `--node-id` again later, the stored configuration will be updated.

### 3. Self-register a new node

Use self-registration when the node does not exist yet in the Exalsius backend.

Required arguments:

`--self-register` Start the self-registration workflow  
`--api-url <url>` Base API URL  
`--register-token <token>` Registration token from the user profile  
`--username <name>` SSH username used to access the node  
`--private-key-id <id>` Private key id configured for node access  
`--hostname <name>` Node name you want to assign to the node
`--ip-addr <ip>` Public or reachable IP address of the node  
`--port <port>` SSH port of the node

Example:

```bash
./target/release/client-hw-info \
  --self-register \
  --api-url https://api.exalsius.ai \
  --register-token <register-token> \
  --username ubuntu \
  --private-key-id <private-key-id> \
  --hostname gpu-node-01 \
  --ip-addr 203.0.113.10 \
  --port 22
```

The self-register request sends the collected hardware, software, and system information to the API, creates `$HOME/.config/exalsius/config.env`, and stores the returned `node_id` and access token for later heartbeat runs.

Self-registration is intended for the initial setup of a node. Afterward, run the tool normally without `--self-register`.

### systemd behavior during self-registration

By default, a successful self-registration also creates and enables:

`/etc/systemd/system/client-hw-info.service`  
`/etc/systemd/system/client-hw-info.timer`

The timer triggers the tool every 15 minutes.

This requires permission to write to `/etc/systemd/system` and to execute `systemctl`. Run the self-registration command with sufficient privileges if you want the timer to be installed automatically.

If you do not want the tool to create systemd units, add:

`--skip-systemd`

Because the generated service uses the current binary path as `ExecStart`, run self-registration from the final binary location you want systemd to use, e.g., /usr/local/bin.

## Configuration file

The tool stores its runtime configuration in:

`$HOME/.config/exalsius/config.env`

The file contains:

`NODE_ID=<node id>`  
`API_URL=<api url>`  
`AUTH_TOKEN=<access token>`

## Version

Use `--version` or `-V` to print the current version.


