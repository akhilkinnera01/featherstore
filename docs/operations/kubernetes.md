# Kubernetes and Helm

Featherstore includes a Helm chart at `deploy/helm/featherstore`. The chart deploys one server pod by default with liveness/readiness probes and service exposure.

Defaults are for functional validation. They are not benchmark-proof production sizing.

## Validate the chart

```bash
helm lint deploy/helm/featherstore
helm template featherstore deploy/helm/featherstore
```

## Install

```bash
helm install featherstore deploy/helm/featherstore   --set image.repository=ghcr.io/<github-owner>/featherstore   --set image.tag=<tag-or-sha>
```

For a locally built image, use the repository/tag available to your cluster. For kind or minikube, load the local image into the cluster before install.

## Local kind fallback

If no external Kubernetes cluster is available, a local Colima + kind deployment can validate the image, chart, probes, and metrics endpoint on macOS:

```bash
brew install docker colima kind helm kubernetes-cli
HOME=$HOME colima start --cpu 2 --memory 4 --disk 20
docker build -t featherstore:local .
kind create cluster --name featherstore
kind load docker-image featherstore:local --name featherstore
helm upgrade --install featherstore deploy/helm/featherstore \
  --namespace featherstore --create-namespace \
  --set image.repository=featherstore \
  --set image.tag=local \
  --set image.pullPolicy=IfNotPresent \
  --wait
kubectl -n featherstore rollout status deploy/featherstore
kubectl -n featherstore port-forward svc/featherstore 8080:8080
curl -fsS http://127.0.0.1:8080/healthz
curl -fsS http://127.0.0.1:8080/readyz
curl -fsS http://127.0.0.1:8080/metrics
```

In long-path sandboxed shells, Colima/Lima can exceed the Unix socket path length when `$HOME` points at a nested tool profile directory. Set `HOME` to the real user home before starting Colima, for example `HOME=/Users/<user> colima start ...`.

## Check rollout

```bash
kubectl get pods -l app.kubernetes.io/name=featherstore
kubectl port-forward svc/featherstore 8080:8080
curl -fsS http://localhost:8080/healthz
curl -fsS http://localhost:8080/readyz
```

## Metrics options

If your cluster has Prometheus Operator installed, enable ServiceMonitor support:

```bash
helm upgrade --install featherstore deploy/helm/featherstore   --set metrics.serviceMonitor.enabled=true
```

If you use kind or minikube, label benchmark reports as local fallback. Local single-node clusters are not production proof for `10k QPS p99 < 5 ms`.

## Uninstall

```bash
helm uninstall featherstore
```
