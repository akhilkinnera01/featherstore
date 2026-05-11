# Release and rollback runbook

Image convention: `ghcr.io/<github-owner>/featherstore:<tag-or-sha>`. CI publishes SHA, branch, and semver tag variants from `.github/workflows/docker.yml` after the repository is pushed to GitHub with package permissions enabled.

## Pre-release gates

- Rust: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` pass.
- Python SDK: `python -m pip install -e ./python[dev]` and `pytest python/tests` pass.
- Docker: `docker build -t featherstore:local .` passes.
- Helm: `helm lint deploy/helm/featherstore` and `helm template featherstore deploy/helm/featherstore` pass.
- Docs: operations docs and Grafana dashboard exist.
- Benchmark smoke: short CI benchmark completes and is labeled as smoke, not proof.
- Production approval: explicit human approval is required before any production deploy.

## Publish to GitHub and GHCR

```bash
gh repo create featherstore --public --source=. --remote=origin --push
# or, for an existing empty repo:
git remote add origin https://github.com/<github-owner>/featherstore.git
git push -u origin main
```

GHCR publishing is handled by the `docker-ghcr` workflow on `main`, semver tags, or manual dispatch:

```bash
gh workflow run docker.yml --ref main
```

If `gh` is not authenticated, run `gh auth login` or provide a GitHub token with `repo`, `workflow`, and package write permissions. Do not commit tokens.

## Staging deploy

```bash
helm upgrade --install featherstore deploy/helm/featherstore \
  --namespace featherstore --create-namespace \
  --set image.repository=ghcr.io/<github-owner>/featherstore \
  --set image.tag=<tag-or-sha>
kubectl -n featherstore rollout status deploy/featherstore
kubectl -n featherstore port-forward svc/featherstore 8080:8080
curl -fsS http://127.0.0.1:8080/healthz
curl -fsS http://127.0.0.1:8080/metrics
```

## Production gate

Production deploys require a durable approval artifact with approver, timestamp, target version, environment, rollback owner, and expected rollback window. Do not deploy production from unreviewed local state.

## Rollback

For stateless/in-memory deployments, rollback is a Helm revision rollback plus cache/data reload from upstream sources:

```bash
helm -n featherstore history featherstore
helm -n featherstore rollback featherstore <REVISION>
kubectl -n featherstore rollout status deploy/featherstore
curl -fsS http://127.0.0.1:8080/healthz
```

If snapshot persistence is enabled, confirm whether the snapshot schema/data remains compatible before rolling back. If not compatible, disable `server.snapshot.loadOnStart`, deploy the rollback, and reload features from source-of-truth data.

## MVP limitations for release notes

- No authentication or TLS in the server.
- In-memory single-process store.
- Snapshot is local reproducibility support, not durable production storage.
- Benchmark target is not proven unless the attached benchmark report says `proven`.
