from __future__ import annotations
from dataclasses import dataclass
from typing import Any, Literal

FeatureDType = Literal["int64", "float64", "bool", "string"]
FeatureValue = int | float | bool | str | None

@dataclass(frozen=True)
class FeatureDef:
    name: str
    dtype: FeatureDType
    nullable: bool = False
    def to_dict(self) -> dict[str, Any]: return {"name": self.name, "dtype": self.dtype, "nullable": self.nullable}
    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "FeatureDef": return cls(str(data["name"]), data["dtype"], bool(data.get("nullable", False)))

@dataclass(frozen=True)
class FeatureSchema:
    entity: str
    version: int
    features: list[FeatureDef]
    def to_dict(self) -> dict[str, Any]: return {"entity": self.entity, "version": self.version, "features": [f.to_dict() for f in self.features]}
    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "FeatureSchema": return cls(str(data["entity"]), int(data["version"]), [FeatureDef.from_dict(f) for f in data["features"]])

@dataclass(frozen=True)
class FeatureRow:
    id: str
    values: dict[str, FeatureValue]
    event_ts_ms: int | None = None
    def to_dict(self) -> dict[str, Any]: return {"id": self.id, "values": self.values, "event_ts_ms": self.event_ts_ms}
    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "FeatureRow": return cls(str(data["id"]), dict(data.get("values", {})), data.get("event_ts_ms"))

@dataclass(frozen=True)
class IngestSummary:
    entity: str
    accepted_rows: int
    rejected_rows: int
    total_rows: int
    schema_version: int
    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "IngestSummary": return cls(data["entity"], int(data["accepted_rows"]), int(data["rejected_rows"]), int(data["total_rows"]), int(data["schema_version"]))

@dataclass(frozen=True)
class LookupRequest:
    entity: str
    id: str
    features: list[str] | None = None
    def to_dict(self) -> dict[str, Any]: return {"entity": self.entity, "id": self.id, "features": self.features}
    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "LookupRequest": return cls(str(data["entity"]), str(data["id"]), data.get("features"))

@dataclass(frozen=True)
class LookupRow:
    entity: str
    id: str
    found: bool
    values: dict[str, FeatureValue]
    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "LookupRow": return cls(str(data["entity"]), str(data["id"]), bool(data["found"]), dict(data.get("values", {})))

@dataclass(frozen=True)
class LookupResponse:
    rows: list[LookupRow]
    elapsed_us: int
    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "LookupResponse": return cls([LookupRow.from_dict(r) for r in data.get("rows", [])], int(data.get("elapsed_us", 0)))
