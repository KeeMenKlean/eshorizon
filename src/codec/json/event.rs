use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::error::Error;

// Event struct for internal usage in Rust
#[derive(Serialize, Deserialize, Debug)]
struct Event {
    event_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    raw_data: Option<Value>,
    #[serde(skip)]
    data: Option<Value>, // This will hold the deserialized event data
    timestamp: DateTime<Utc>,
    aggregate_type: String,
    aggregate_id: Uuid,
    version: i32,
    metadata: HashMap<String, Value>,
    context: HashMap<String, Value>,
}

// EventCodec responsible for encoding and decoding events in JSON format
pub struct EventCodec;

impl EventCodec {
    // Marshal the event into JSON bytes
    pub fn marshal_event(
        event_type: String,
        data: Option<Value>,
        timestamp: DateTime<Utc>,
        aggregate_type: String,
        aggregate_id: Uuid,
        version: i32,
        metadata: HashMap<String, Value>,
        context: HashMap<String, Value>,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let event = Event {
            event_type,
            raw_data: data.clone(),
            data,
            timestamp,
            aggregate_type,
            aggregate_id,
            version,
            metadata,
            context,
        };

        // Serialize the event struct into JSON bytes
        let json_bytes = serde_json::to_vec(&event)?;
        Ok(json_bytes)
    }

    // Unmarshal JSON bytes into an Event struct
    pub fn unmarshal_event(
        json_bytes: &[u8],
    ) -> Result<Event, Box<dyn Error>> {
        // Deserialize the event struct from the provided JSON bytes
        let mut event: Event = serde_json::from_slice(json_bytes)?;

        // Handle event data deserialization separately if needed
        if let Some(raw_data) = event.raw_data.take() {
            event.data = Some(raw_data);
        }

        Ok(event)
    }
}

// Test Command
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_marshal_unmarshal_event() {
        let aggregate_id = Uuid::new_v4();
        let timestamp = Utc::now();
        let metadata = HashMap::new();
        let context = HashMap::new();
        let data = Some(json!({"key": "value"}));

        // Marshal the event
        let serialized_event = EventCodec::marshal_event(
            "TestEvent".to_string(),
            data.clone(),
            timestamp,
            "TestAggregate".to_string(),
            aggregate_id,
            1,
            metadata.clone(),
            context.clone(),
        )
            .expect("Failed to serialize event");

        // Unmarshal the event
        let deserialized_event = EventCodec::unmarshal_event(&serialized_event)
            .expect("Failed to deserialize event");

        // Check that the event fields match the original values
        assert_eq!(deserialized_event.event_type, "TestEvent");
        assert_eq!(deserialized_event.aggregate_type, "TestAggregate");
        assert_eq!(deserialized_event.aggregate_id, aggregate_id);
        assert_eq!(deserialized_event.version, 1);
        assert_eq!(deserialized_event.data, data);
        assert_eq!(deserialized_event.metadata, metadata);
        assert_eq!(deserialized_event.context, context);
    }
}