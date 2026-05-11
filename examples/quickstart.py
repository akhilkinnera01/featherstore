from featherstore import FeatherstoreClient, generate_recommendation_dataset


def main() -> None:
    dataset = generate_recommendation_dataset(users=10, items=10, contexts=4, embedding_dims=2, seed=13)
    with FeatherstoreClient("http://localhost:8080") as client:
        for rows in dataset.iter_user_batches(5):
            client.ingest_batch(dataset.user_schema, rows)
        row = client.get_features("user", "3", ["age_bucket", "country_id"])
        print(row)

if __name__ == "__main__":
    main()
