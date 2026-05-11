from featherstore.report import percentile, proof_status
from featherstore.benchmark import _scenario_request


def test_proof_status_requires_all_thresholds():
    assert proof_status(10_000, 4.99, 0.0, True) == "proven"
    assert proof_status(9_999, 4.0, 0.0, True) == "not_proven"
    assert proof_status(10_000, 5.0, 0.0, True) == "not_proven"
    assert proof_status(10_000, 4.0, 0.01, True) == "not_proven"
    assert proof_status(10_000, 4.0, 0.0, False) == "invalid"


def test_percentile_calculation_for_known_samples():
    samples = [1, 2, 3, 4, 5]
    assert percentile(samples, 50) == 3
    assert percentile(samples, 0) == 1
    assert percentile(samples, 100) == 5


def test_batch_scenario_reports_multiple_entity_lookups_per_request():
    path, payload, lookups = _scenario_request("batch_post_request_qps", 0, users=100, items=100, contexts=24)
    assert path == "/v1/features/lookup"
    assert payload is not None
    assert len(payload["requests"]) == lookups == 8


def test_unknown_scenario_fails():
    try:
        _scenario_request("bad", 0, 1, 1, 1)
    except ValueError as exc:
        assert "unknown scenario" in str(exc)
    else:
        raise AssertionError("expected ValueError")
