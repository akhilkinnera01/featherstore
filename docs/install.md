# Install

## Prerequisites

- Rust 1.78 or newer.
- Python 3.10 or newer.
- Optional: Docker for image builds.
- Optional: Helm and a Kubernetes cluster for deployment checks.

## Clone and enter the repository

```bash
git clone https://github.com/<owner>/featherstore.git
cd featherstore
```

If you are working from a local prepared repository, run commands from the repository root.

## Install the Python SDK

```bash
python -m pip install -e ./python
```

For tests and local development:

```bash
python -m pip install -e ./python[dev]
```

## Build and run the Rust server

```bash
cargo build --workspace
cargo run -p featherstore-server --release
```

The server listens on `0.0.0.0:8080` by default.

## Install docs dependencies

```bash
python -m pip install -r docs/requirements.txt
python -m mkdocs build --strict
```

## Optional tool checks

```bash
docker build -t featherstore:local .
helm lint deploy/helm/featherstore
helm template featherstore deploy/helm/featherstore
```
