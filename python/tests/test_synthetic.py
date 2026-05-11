from featherstore.synthetic import generate_recommendation_dataset


def test_same_seed_yields_same_rows():
    a = generate_recommendation_dataset(users=3, items=2, contexts=2, embedding_dims=2, seed=13)
    b = generate_recommendation_dataset(users=3, items=2, contexts=2, embedding_dims=2, seed=13)
    assert a.user_row(0) == b.user_row(0)
    assert a.item_row(1) == b.item_row(1)
    assert a.context_row(1) == b.context_row(1)


def test_counts_match_inputs():
    ds = generate_recommendation_dataset(users=5, items=4, contexts=3, embedding_dims=2)
    assert sum(len(batch) for batch in ds.iter_user_batches(2)) == 5
    assert sum(len(batch) for batch in ds.iter_item_batches(2)) == 4
    assert sum(len(batch) for batch in ds.iter_context_batches(2)) == 3


def test_schema_feature_count_matches_embedding_dims():
    ds = generate_recommendation_dataset(users=1, items=1, contexts=1, embedding_dims=4)
    assert len(ds.user_schema.features) == 6
    assert len(ds.item_schema.features) == 6
    assert len(ds.context_schema.features) == 6


def test_batch_iterators_cover_all_rows_exactly_once():
    ds = generate_recommendation_dataset(users=7, items=1, contexts=1, embedding_dims=1)
    ids = [row.id for batch in ds.iter_user_batches(3) for row in batch]
    assert ids == [str(i) for i in range(7)]
