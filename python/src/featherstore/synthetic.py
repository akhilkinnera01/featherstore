from __future__ import annotations
from dataclasses import dataclass
from collections.abc import Iterator
from .models import FeatureDef, FeatureRow, FeatureSchema

@dataclass(frozen=True)
class SyntheticDataset:
    users: int
    items: int
    contexts: int
    embedding_dims: int
    seed: int
    user_schema: FeatureSchema
    item_schema: FeatureSchema
    context_schema: FeatureSchema

    def iter_user_batches(self, batch_size: int = 10_000) -> Iterator[list[FeatureRow]]:
        yield from _batch_rows((self.user_row(i) for i in range(self.users)), batch_size)
    def iter_item_batches(self, batch_size: int = 10_000) -> Iterator[list[FeatureRow]]:
        yield from _batch_rows((self.item_row(i) for i in range(self.items)), batch_size)
    def iter_context_batches(self, batch_size: int = 24) -> Iterator[list[FeatureRow]]:
        yield from _batch_rows((self.context_row(i) for i in range(self.contexts)), batch_size)

    def user_row(self, i: int) -> FeatureRow:
        values = {"age_bucket": (i + self.seed) % 10, "country_id": 1 + ((i * 17 + self.seed) % 250)}
        values.update({f"u_emb_{d}": _score(i, d, self.seed, 997) for d in range(self.embedding_dims)})
        return FeatureRow(str(i), values, None)
    def item_row(self, i: int) -> FeatureRow:
        values = {"category_id": (i * 7 + self.seed) % 128, "price_bucket": (i * 11 + self.seed) % 20}
        values.update({f"i_emb_{d}": _score(i, d, self.seed + 31, 991) for d in range(self.embedding_dims)})
        return FeatureRow(str(i), values, None)
    def context_row(self, i: int) -> FeatureRow:
        values = {"hour_bucket": i % 24, "device_type": (i + self.seed) % 5}
        values.update({f"c_emb_{d}": _score(i, d, self.seed + 53, 983) for d in range(self.embedding_dims)})
        return FeatureRow(str(i), values, None)

def generate_recommendation_dataset(users: int = 100_000, items: int = 100_000, contexts: int = 24, embedding_dims: int = 8, seed: int = 13) -> SyntheticDataset:
    if min(users, items, contexts, embedding_dims) < 0: raise ValueError("dataset dimensions must be non-negative")
    user_features = [FeatureDef("age_bucket", "int64"), FeatureDef("country_id", "int64")] + [FeatureDef(f"u_emb_{d}", "float64") for d in range(embedding_dims)]
    item_features = [FeatureDef("category_id", "int64"), FeatureDef("price_bucket", "int64")] + [FeatureDef(f"i_emb_{d}", "float64") for d in range(embedding_dims)]
    context_features = [FeatureDef("hour_bucket", "int64"), FeatureDef("device_type", "int64")] + [FeatureDef(f"c_emb_{d}", "float64") for d in range(embedding_dims)]
    return SyntheticDataset(users, items, contexts, embedding_dims, seed, FeatureSchema("user", 1, user_features), FeatureSchema("item", 1, item_features), FeatureSchema("context", 1, context_features))

def _score(i: int, d: int, seed: int, mod: int) -> float:
    return round((((i + 1) * (d + 3) * (seed + 7)) % mod) / mod, 6)

def _batch_rows(rows: Iterator[FeatureRow], batch_size: int) -> Iterator[list[FeatureRow]]:
    if batch_size <= 0: raise ValueError("batch_size must be positive")
    batch: list[FeatureRow] = []
    for row in rows:
        batch.append(row)
        if len(batch) >= batch_size:
            yield batch; batch = []
    if batch: yield batch
