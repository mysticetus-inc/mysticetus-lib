use std::collections::HashMap;

use protos::pubsub::Topic;

pub(crate) fn make_qualified_topic_name(project_id: &str, topic_name: &str) -> String {
    format!("projects/{project_id}/topics/{topic_name}")
}

pub(crate) fn make_qualified_subscription_name(
    project_id: &str,
    subscription_name: &str,
) -> String {
    format!("projects/{project_id}/subscriptions/{subscription_name}")
}

pub(crate) fn make_default_topic(project_id: &str, topic: &str) -> Topic {
    Topic {
        state: protos::pubsub::topic::State::Active as i32,
        ingestion_data_source_settings: None,
        name: make_qualified_topic_name(project_id, topic),
        labels: HashMap::new(),
        message_storage_policy: None,
        kms_key_name: String::new(),
        schema_settings: None,
        satisfies_pzs: false,
        message_retention_duration: None,
        message_transforms: Vec::new(),
    }
}
