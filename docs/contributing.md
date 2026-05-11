# Contributing and release notes

## Local development checks

Run from the repository root:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python -m pip install -e ./python[dev]
pytest python/tests
python -m pip install -r docs/requirements.txt
python -m mkdocs build --strict
```

## Documentation standard

- Document current behavior only.
- Mark future work as planned or proposed.
- Do not claim benchmark proof without a report that meets the proof standard.
- Keep security limitations visible.
- Include exact commands and expected outputs for runbooks.

## Release notes checklist

Every MVP release note should include:

- Git tag or commit.
- Docker/GHCR image name, if published.
- Helm chart version or path.
- Rust and Python test status.
- Docs build status.
- Docker build status.
- Helm lint/template or install status.
- Benchmark report path and proof status.
- Known limitations.

## Image convention

`ghcr.io/<github-owner>/featherstore:<tag-or-sha>`

The owner is intentionally not hardcoded in repository docs.

## Current MVP release caveats

- Server has no auth or TLS.
- Store is single-process and in-memory.
- Snapshot persistence is local reproducibility support.
- Performance target is not proven unless an attached benchmark report says `proven`.
