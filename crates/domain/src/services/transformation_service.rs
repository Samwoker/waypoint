use std::sync::Arc;
use sqlx::PgPool;
use uuid::Uuid;
use relay_core::error::CoreError;

use crate::dto::{
    CreateTransformationInput, TransformationRule, TransformationView, UpdateTransformationInput,
};
use data::repositories::{SubscriptionRepository, TransformationRepository};

pub struct TransformationService {
    pool: Arc<PgPool>,
}

impl TransformationService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    fn validate_rules(rules: &[TransformationRule]) -> Result<(), CoreError> {
        if rules.is_empty() {
            return Err(CoreError::Validation(
                "Transformation rules cannot be empty".to_string(),
            ));
        }

        for (i, rule) in rules.iter().enumerate() {
            // Validate JSONPath: must start with '$'
            if !rule.source_path.starts_with('$') {
                return Err(CoreError::Validation(format!(
                    "Rule {i}: source_path '{}' is not a valid JSONPath (must start with '$')",
                    rule.source_path
                )));
            }
            if !rule.dest_path.starts_with('$') {
                return Err(CoreError::Validation(format!(
                    "Rule {i}: dest_path '{}' is not a valid JSONPath (must start with '$')",
                    rule.dest_path
                )));
            }
        }

        Ok(())
    }

    fn rules_to_json(rules: &[TransformationRule]) -> serde_json::Value {
        serde_json::to_value(rules).unwrap_or(serde_json::Value::Array(vec![]))
    }

    fn json_to_rules(json: &serde_json::Value) -> Vec<TransformationRule> {
        serde_json::from_value(json.clone()).unwrap_or_default()
    }

    pub async fn list_transformations(
        &self,
        tenant_id: Uuid,
        subscription_id: Uuid,
    ) -> Result<Vec<TransformationView>, CoreError> {
        // Verify subscription belongs to tenant
        let sub_repo = SubscriptionRepository::new(&self.pool);
        if sub_repo.find_by_tenant_and_id(tenant_id, subscription_id).await?.is_none() {
            return Err(CoreError::NotFound(format!("Subscription '{subscription_id}' not found")));
        }

        let repo = TransformationRepository::new(&self.pool);
        let transformations = repo.list_by_subscription(tenant_id, subscription_id).await?;

        Ok(transformations
            .into_iter()
            .map(|t| TransformationView {
                id: t.id,
                tenant_id: t.tenant_id,
                subscription_id: t.subscription_id,
                rules: Self::json_to_rules(&t.rules),
                created_at: t.created_at,
                updated_at: t.updated_at,
            })
            .collect())
    }

    pub async fn create_transformation(
        &self,
        tenant_id: Uuid,
        input: CreateTransformationInput,
    ) -> Result<TransformationView, CoreError> {
        // Verify subscription belongs to tenant
        let sub_repo = SubscriptionRepository::new(&self.pool);
        if sub_repo.find_by_tenant_and_id(tenant_id, input.subscription_id).await?.is_none() {
            return Err(CoreError::NotFound(format!(
                "Subscription '{}' not found",
                input.subscription_id
            )));
        }

        Self::validate_rules(&input.rules)?;

        let rules_json = Self::rules_to_json(&input.rules);
        let repo = TransformationRepository::new(&self.pool);
        let t = repo.create(tenant_id, input.subscription_id, rules_json).await?;

        Ok(TransformationView {
            id: t.id,
            tenant_id: t.tenant_id,
            subscription_id: t.subscription_id,
            rules: Self::json_to_rules(&t.rules),
            created_at: t.created_at,
            updated_at: t.updated_at,
        })
    }

    pub async fn update_transformation(
        &self,
        tenant_id: Uuid,
        transformation_id: Uuid,
        input: UpdateTransformationInput,
    ) -> Result<TransformationView, CoreError> {
        // Verify transformation belongs to tenant
        let repo = TransformationRepository::new(&self.pool);
        if repo.find_by_tenant_and_id(tenant_id, transformation_id).await?.is_none() {
            return Err(CoreError::NotFound(format!(
                "Transformation '{transformation_id}' not found"
            )));
        }

        Self::validate_rules(&input.rules)?;

        let rules_json = Self::rules_to_json(&input.rules);
        let t = repo.update(tenant_id, transformation_id, rules_json).await?;

        Ok(TransformationView {
            id: t.id,
            tenant_id: t.tenant_id,
            subscription_id: t.subscription_id,
            rules: Self::json_to_rules(&t.rules),
            created_at: t.created_at,
            updated_at: t.updated_at,
        })
    }

    pub async fn delete_transformation(
        &self,
        tenant_id: Uuid,
        transformation_id: Uuid,
    ) -> Result<(), CoreError> {
        let repo = TransformationRepository::new(&self.pool);
        repo.delete(tenant_id, transformation_id).await
    }

    pub fn extract_jsonpath(payload: &serde_json::Value, path: &str) -> Option<serde_json::Value> {
        let clean_path = path.strip_prefix('$').unwrap_or(path);
        let clean_path = clean_path.strip_prefix('.').unwrap_or(clean_path);

        if clean_path.is_empty() {
            return Some(payload.clone());
        }

        let mut current = payload;
        for segment in clean_path.split('.') {
            if segment.is_empty() {
                continue;
            }
            match current {
                serde_json::Value::Object(map) => {
                    if let Some(val) = map.get(segment) {
                        current = val;
                    } else {
                        return None;
                    }
                }
                serde_json::Value::Array(arr) => {
                    if let Ok(idx) = segment.parse::<usize>() {
                        if let Some(val) = arr.get(idx) {
                            current = val;
                        } else {
                            return None;
                        }
                    } else {
                        return None;
                    }
                }
                _ => return None,
            }
        }

        Some(current.clone())
    }

    pub fn transform_value(template: &serde_json::Value, payload: &serde_json::Value) -> serde_json::Value {
        match template {
            serde_json::Value::String(s) if s.starts_with('$') => {
                Self::extract_jsonpath(payload, s).unwrap_or(serde_json::Value::Null)
            }
            serde_json::Value::Object(map) => {
                let mut new_map = serde_json::Map::new();
                for (k, v) in map {
                    new_map.insert(k.clone(), Self::transform_value(v, payload));
                }
                serde_json::Value::Object(new_map)
            }
            serde_json::Value::Array(arr) => {
                serde_json::Value::Array(arr.iter().map(|item| Self::transform_value(item, payload)).collect())
            }
            other => other.clone(),
        }
    }

    pub async fn test_transformation(
        &self,
        template_str: &str,
        payload: &serde_json::Value,
    ) -> Result<serde_json::Value, CoreError> {
        let template_json: serde_json::Value = match serde_json::from_str(template_str) {
            Ok(v) => v,
            Err(_) => {
                // If template string is a simple path like "$.data.id"
                if template_str.starts_with('$') {
                    return Ok(Self::extract_jsonpath(payload, template_str).unwrap_or(serde_json::Value::Null));
                }
                return Err(CoreError::Validation("Invalid JSON template string".to_string()));
            }
        };

        Ok(Self::transform_value(&template_json, payload))
    }
}
