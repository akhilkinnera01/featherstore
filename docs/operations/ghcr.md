# GHCR image publishing

The GitHub Actions workflow `.github/workflows/docker.yml` builds multi-architecture images and publishes them to GitHub Container Registry.

Published image names:

- `ghcr.io/<github-owner>/featherstore:sha-<short-sha>`
- `ghcr.io/<github-owner>/featherstore:main`
- `ghcr.io/<github-owner>/featherstore:vX.Y.Z` for release tags

## Manual local publish

```bash
docker buildx create --use --name featherstore-builder || docker buildx use featherstore-builder
echo "$GITHUB_TOKEN" | docker login ghcr.io -u <github-owner> --password-stdin
docker buildx build --platform linux/amd64,linux/arm64 \
  -t ghcr.io/<github-owner>/featherstore:<tag> \
  --push .
```

Required token permissions: package write plus repository access. Never put the token in a file committed to the repository.
