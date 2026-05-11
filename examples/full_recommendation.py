from __future__ import annotations

from featherstore import FeatherstoreClient, LookupRequest, generate_recommendation_dataset


def main() -> None:
    dataset = generate_recommendation_dataset(users=100, items=100, contexts=24, embedding_dims=4, seed=13)

    with FeatherstoreClient("http://localhost:8080") as client:
        print("health:", client.health())

        for rows in dataset.iter_user_batches(25):
            client.ingest_batch(dataset.user_schema, rows)
        for rows in dataset.iter_item_batches(25):
            client.ingest_batch(dataset.item_schema, rows)
        for rows in dataset.iter_context_batches(24):
            client.ingest_batch(dataset.context_schema, rows)

        print("ready:", client.ready())
        print("user:", client.get_features("user", "42", ["age_bucket", "country_id", "u_emb_0"]))
        print("item:", client.get_features("item", "42", ["category_id", "price_bucket", "i_emb_0"]))
        print("context:", client.get_features("context", "12", ["hour_bucket", "device_type", "c_emb_0"]))

        batch = client.lookup([
            LookupRequest("user", "7", ["age_bucket", "country_id"]),
            LookupRequest("item", "7", ["category_id", "price_bucket"]),
            LookupRequest("context", "7", ["hour_bucket", "device_type"]),
        ])
        print("batch:", batch)


if __name__ == "__main__":
    main()
