from __future__ import annotations
import argparse, time
from .client import FeatherstoreClient
from .synthetic import generate_recommendation_dataset

def load_synthetic_main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Load deterministic synthetic recommendation data into Featherstore")
    parser.add_argument("--server", default="http://localhost:8080"); parser.add_argument("--users", type=int, default=1000); parser.add_argument("--items", type=int, default=1000); parser.add_argument("--contexts", type=int, default=24); parser.add_argument("--embedding-dims", type=int, default=8); parser.add_argument("--batch-size", type=int, default=5000); parser.add_argument("--seed", type=int, default=13)
    args = parser.parse_args(argv)
    dataset = generate_recommendation_dataset(args.users, args.items, args.contexts, args.embedding_dims, args.seed)
    started = time.perf_counter(); accepted = 0
    with FeatherstoreClient(args.server, timeout=30.0) as client:
        for rows in dataset.iter_user_batches(args.batch_size): accepted += client.ingest_batch(dataset.user_schema, rows).accepted_rows
        for rows in dataset.iter_item_batches(args.batch_size): accepted += client.ingest_batch(dataset.item_schema, rows).accepted_rows
        for rows in dataset.iter_context_batches(max(1, min(args.batch_size, args.contexts or 1))): accepted += client.ingest_batch(dataset.context_schema, rows).accepted_rows
    print(f"accepted_rows={accepted} elapsed_seconds={time.perf_counter() - started:.3f}")
    return 0

if __name__ == "__main__": raise SystemExit(load_synthetic_main())
