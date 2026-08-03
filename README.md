*This project has been created as part of the 42 curriculum by zaperfish*

# ft_lgtm

## Description

ft_lgtm is an observability-focused platform combining Kubernetes, WebAssembly,
IPFS, and the LGTM observability stack (Loki, Grafana, Tempo, Mimir/Prometheus).

The goal of this project is to build a secure web playground where users can
write code, execute it inside a WebAssembly sandbox, and observe the complete
execution lifecycle through metrics, logs, and distributed traces.

The project demonstrates modern cloud-native concepts:

- **WebAssembly (WASM/WASI)** for secure code execution
- **Kubernetes** for application orchestration
- **IPFS** for decentralized content sharing
- **OpenTelemetry** for telemetry collection
- **Grafana LGTM stack** for observability

## Features

### Web Application

- Code editor playground
- Preloaded example code
- Execute button
- Execution status feedback
- Error handling
- Output visualization
- IPFS sharing link

### Backend

- Receive source code from frontend
- Compile source code to WebAssembly
- Execute WASM using WASI runtime
- Capture execution output
- Upload execution results to IPFS
- Return results to frontend
- OpenTelemetry instrumentation

### Security

The WASM execution environment applies restrictions:

- No network access
- Limited filesystem access
- Memory limit: `<memory-limit>`
- Execution timeout: `<timeout>`
- Output size limit: `<output-limit>`

## Requirements

Required software:

- docker
- k3d
- kubectl
- just

## Installation

Clone the repository:

```bash
git clone <repository-url>
cd <project-name>
```

Install dependencies and deploy the infrastructure:

```bash
make install-deps
```

This command installs:

- k3d
- kubectl
- just

> **Docker is assumed to be already installed by the user**

Deploy the complete stack:

```bash
just deploy
```

Add local DNS entries so that services are accessible through lgtm.local, grafana.lgtm.local and ipfs.lgtm.local:

```bash
just add-hosts
```

## Usage

### Access the application

Frontend:

```
http://lgtm.local
```

Grafana:

```
http://grafana.lgtm.local
```

IPFS Gateway:

```
http://ipfs.lgtm.local
```

## IPFS

Execution results are uploaded as an IPFS directory:

```
CID/
├── main.rs
└── run_result.json
```

Example:

```
http://ipfs.lgtm.local/ipfs/<CID>
```
