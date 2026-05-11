from .client import FeatherstoreClient, FeatherstoreError
from .models import FeatureDef, FeatureSchema, FeatureRow, IngestSummary, LookupRequest, LookupRow, LookupResponse
from .synthetic import SyntheticDataset, generate_recommendation_dataset

__all__ = ["FeatherstoreClient", "FeatherstoreError", "FeatureDef", "FeatureSchema", "FeatureRow", "IngestSummary", "LookupRequest", "LookupRow", "LookupResponse", "SyntheticDataset", "generate_recommendation_dataset"]
