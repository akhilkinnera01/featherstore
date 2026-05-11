import httpx
import pytest
from featherstore import FeatherstoreClient, FeatherstoreError, FeatureDef, FeatureRow, FeatureSchema, LookupRequest


def test_model_dict_serialization_matches_api_shape():
    schema = FeatureSchema("user", 1, [FeatureDef("age_bucket", "int64"), FeatureDef("score", "float64")])
    row = FeatureRow("42", {"age_bucket": 3, "score": 0.5})
    assert schema.to_dict() == {"entity": "user", "version": 1, "features": [{"name": "age_bucket", "dtype": "int64", "nullable": False}, {"name": "score", "dtype": "float64", "nullable": False}]}
    assert row.to_dict() == {"id": "42", "values": {"age_bucket": 3, "score": 0.5}, "event_ts_ms": None}


def test_client_builds_expected_urls_and_payloads():
    requests = []
    def handler(request: httpx.Request) -> httpx.Response:
        requests.append(request)
        if request.url.path == "/v1/ingest/batch":
            data = request.read().decode()
            assert '"entity":"user"' in data
            return httpx.Response(200, json={"entity":"user","accepted_rows":1,"rejected_rows":0,"total_rows":1,"schema_version":1})
        if request.url.path == "/v1/features/user/42":
            assert request.url.params["features"] == "age_bucket"
            return httpx.Response(200, json={"entity":"user","id":"42","found":True,"values":{"age_bucket":3}})
        if request.url.path == "/v1/features/lookup":
            return httpx.Response(200, json={"rows":[],"elapsed_us":1})
        return httpx.Response(404, text="missing")
    transport = httpx.MockTransport(handler)
    client = FeatherstoreClient("http://testserver", client=httpx.Client(transport=transport))
    schema = FeatureSchema("user", 1, [FeatureDef("age_bucket", "int64")])
    assert client.ingest_batch(schema, [FeatureRow("42", {"age_bucket": 3})]).accepted_rows == 1
    assert client.get_features("user", "42", ["age_bucket"]).found
    assert client.lookup([LookupRequest("user", "42")]).rows == []
    assert [r.url.path for r in requests] == ["/v1/ingest/batch", "/v1/features/user/42", "/v1/features/lookup"]


def test_non_2xx_raises_featherstore_error():
    transport = httpx.MockTransport(lambda request: httpx.Response(400, json={"error":"bad","kind":"bad_request"}))
    client = FeatherstoreClient("http://testserver", client=httpx.Client(transport=transport))
    with pytest.raises(FeatherstoreError) as exc:
        client.health()
    assert exc.value.status_code == 400
    assert "bad_request" in exc.value.body
