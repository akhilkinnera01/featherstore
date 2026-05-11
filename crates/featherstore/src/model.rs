use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub type EntityType = String;
pub type EntityId = String;
pub type FeatureName = String;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FeatureDef {
    pub name: FeatureName,
    pub dtype: FeatureDType,
    pub nullable: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FeatureSchema {
    pub entity: EntityType,
    pub version: u32,
    pub features: Vec<FeatureDef>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FeatureDType {
    Int64,
    Float64,
    Bool,
    String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum FeatureValue {
    Int64(i64),
    Float64(f64),
    Bool(bool),
    String(String),
    Null,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ApiFeatureRow {
    pub id: EntityId,
    pub values: BTreeMap<FeatureName, FeatureValue>,
    pub event_ts_ms: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dtype_json_names_are_snake_case() {
        assert_eq!(
            serde_json::to_string(&FeatureDType::Int64).unwrap(),
            "\"int64\""
        );
        assert_eq!(
            serde_json::to_string(&FeatureDType::Float64).unwrap(),
            "\"float64\""
        );
    }

    #[test]
    fn feature_value_round_trips_json_primitives() {
        let values = vec![
            FeatureValue::Int64(1),
            FeatureValue::Float64(1.25),
            FeatureValue::Bool(true),
            FeatureValue::String("x".to_string()),
            FeatureValue::Null,
        ];
        for value in values {
            let json = serde_json::to_string(&value).unwrap();
            let decoded: FeatureValue = serde_json::from_str(&json).unwrap();
            assert_eq!(decoded, value);
        }
    }

    #[test]
    fn schema_feature_order_is_preserved() {
        let schema = FeatureSchema {
            entity: "user".into(),
            version: 1,
            features: vec![
                FeatureDef {
                    name: "a".into(),
                    dtype: FeatureDType::Int64,
                    nullable: false,
                },
                FeatureDef {
                    name: "b".into(),
                    dtype: FeatureDType::Float64,
                    nullable: false,
                },
            ],
        };
        let json = serde_json::to_string(&schema).unwrap();
        let decoded: FeatureSchema = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.features[0].name, "a");
        assert_eq!(decoded.features[1].name, "b");
    }
}
