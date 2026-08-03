*This project has been created as part of the 42 curriculum by zaperfish

# FT-LGTM

## Description

LGTM is an observability-focused platform combining Kubernetes, WebAssembly,
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

Deploy the complete stack:

```bash
just deploy
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

## Observability

The project uses the LGTM stack:

### Metrics

Collected using OpenTelemetry Metrics API.

Displayed in Grafana:

- Successful executions
- Failed executions
- Total executions
- Execution duration

### Logs

Collected using Loki.

Application logs are correlated with traces.

### Traces

Collected using OpenTelemetry tracing.

Each execution contains:

- HTTP request span
- Compilation span
- WASM execution span
- IPFS upload span

Example trace:

```
HTTP Request
 |
 +-- Compilation
 |
 +-- WASM Execution
 |
 +-- IPFS Upload
```

## Dashboards

Grafana dashboards are located in:

```
dashboards/
├── metrics.json
└── traces.json
```

Available dashboards:

### Execution Metrics

Shows:

- Total executions
- Successful executions
- Failed executions
- Execution trends

### Execution Traces

Shows:

- Execution duration
- Slow executions
- Average execution time
- Longest execution

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

## Resources

### Documentation

- Kubernetes documentation:
  <https://kubernetes.io/docs/>

- Kubernetes local clusters:
  <https://k3d.io/>

- WebAssembly:
  <https://webassembly.org/>

- WASI:
  <https://wasi.dev/>

- IPFS:
  <https://docs.ipfs.tech/>

- OpenTelemetry:
  <https://opentelemetry.io/docs/>

- Grafana:
  <https://grafana.com/docs/>

## License

This project is part of the 42 curriculum.
