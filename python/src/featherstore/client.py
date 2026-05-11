from __future__ import annotations
from dataclasses import dataclass
from typing import Any
from urllib.parse import quote
import httpx
from .models import FeatureRow, FeatureSchema, IngestSummary, LookupRequest, LookupResponse, LookupRow

@dataclass
class FeatherstoreError(RuntimeError):
    status_code: int
    body: str
    def __str__(self) -> str: return f"Featherstore request failed with HTTP {self.status_code}: {self.body}"

class FeatherstoreClient:
    def __init__(self, base_url: str = "http://localhost:8080", timeout: float = 5.0, client: httpx.Client | None = None):
        self.base_url = base_url.rstrip("/")
        self._owns_client = client is None
        self._client = client or httpx.Client(timeout=timeout)

    def close(self) -> None:
        if self._owns_client: self._client.close()

    def __enter__(self) -> "FeatherstoreClient": return self
    def __exit__(self, *exc: object) -> None: self.close()

    def health(self) -> dict[str, Any]: return self._request("GET", "/healthz")
    def ready(self) -> dict[str, Any]: return self._request("GET", "/readyz")

    def ingest_batch(self, schema: FeatureSchema, rows: list[FeatureRow], mode: str = "upsert") -> IngestSummary:
        payload = {"schema": schema.to_dict(), "rows": [r.to_dict() for r in rows], "mode": mode}
        return IngestSummary.from_dict(self._request("POST", "/v1/ingest/batch", json=payload))

    def lookup(self, requests: list[LookupRequest | dict[str, Any]], include_missing: bool = True) -> LookupResponse:
        payload_requests = [r.to_dict() if isinstance(r, LookupRequest) else r for r in requests]
        return LookupResponse.from_dict(self._request("POST", "/v1/features/lookup", json={"requests": payload_requests, "include_missing": include_missing}))

    def get_features(self, entity: str, entity_id: str, features: list[str] | None = None) -> LookupRow:
        path = f"/v1/features/{quote(entity, safe='')}/{quote(entity_id, safe='')}"
        params = {"features": ",".join(features)} if features else None
        return LookupRow.from_dict(self._request("GET", path, params=params))

    def _request(self, method: str, path: str, **kwargs: Any) -> Any:
        response = self._client.request(method, f"{self.base_url}{path}", **kwargs)
        if response.status_code // 100 != 2:
            raise FeatherstoreError(response.status_code, response.text)
        if not response.content:
            return {}
        return response.json()
