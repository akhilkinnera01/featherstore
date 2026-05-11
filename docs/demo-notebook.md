# Demo notebook

The notebook at `examples/demo_recommendation.ipynb` demonstrates the synthetic recommendation workflow through the Python SDK.

## What it does

1. Creates a deterministic synthetic dataset.
2. Connects to a local Featherstore server.
3. Loads user, item, and context features.
4. Checks readiness.
5. Runs single-row and batch lookups.
6. Shows a benchmark smoke command without presenting it as proof.

## Run it

```bash
cargo run -p featherstore-server --release
python -m pip install -e ./python[dev]
jupyter notebook examples/demo_recommendation.ipynb
```

If Jupyter is not installed, run the equivalent script:

```bash
python examples/full_recommendation.py
```

Notebook execution requires a running server at `http://localhost:8080`. It does not require private services or secrets.
