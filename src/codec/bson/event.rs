use chrono::{DateTime, Utc};
use mongodb::bson::{self, Bson};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// Event struct to match the bson event format
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    pub event_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Bson>,
    pub timestamp: DateTime<Utc>,
    pub aggregate_type: String,
    pub aggregate_id: Uuid,
    pub version: i32,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, Bson>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub context: HashMap<String, Bson>,
}

// EventCodec responsible for encoding and decoding events to and from BSON
pub struct EventCodec;

impl EventCodec {
    // Marshal the event into BSON bytes
    pub fn marshal_event(event: &Event) -> Result<Vec<u8>, bson::ser::Error> {
        bson::to_vec(event)
    }

    // Unmarshal BSON bytes into an Event struct
    pub fn unmarshal_event(data: &[u8]) -> Result<Event, bson::de::Error> {
        bson::from_slice(data)
    }
}

// Example of creating a new event and serializing/deserializing
impl Event {
    pub fn new(
        event_type: String,
        data: Option<Bson>,
        timestamp: DateTime<Utc>,
        aggregate_type: String,
        aggregate_id: Uuid,
        version: i32,
        metadata: HashMap<String, Bson>,
        context: HashMap<String, Bson>,
    ) -> Self {
        Event {
            event_type,
            data,
            timestamp,
            aggregate_type,
            aggregate_id,
            version,
            metadata,
            context,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_marshal_event() {
        let event = Event::new(
            "TestEvent".to_string(),
            Some(Bson::String("TestData".to_string())),
            Utc::now(),
            "TestAggregate".to_string(),
            Uuid::new_v4(),
            1,
            HashMap::new(), // Empty metadata
            HashMap::new(), // Empty context
        );

        // Marshal the event into BSON
        let serialized_event = EventCodec::marshal_event(&event).unwrap();
        assert!(!serialized_event.is_empty(), "Serialized event should not be empty");
    }

    #[test]
    fn test_unmarshal_event() {
        let event = Event::new(
            "TestEvent".to_string(),
            Some(Bson::String("TestData".to_string())),
            Utc::now(),
            "TestAggregate".to_string(),
            Uuid::new_v4(),
            1,
            HashMap::new(), // Empty metadata
            HashMap::new(), // Empty context
        );

        // Marshal the event into BSON
        let serialized_event = EventCodec::marshal_event(&event).unwrap();

        // Unmarshal the BSON data back into an Event struct
        let deserialized_event = EventCodec::unmarshal_event(&serialized_event).unwrap();

        // Ensure the deserialized event matches the original event
        assert_eq!(deserialized_event.event_type, event.event_type);
        assert_eq!(deserialized_event.aggregate_type, event.aggregate_type);
        assert_eq!(deserialized_event.aggregate_id, event.aggregate_id);
        assert_eq!(deserialized_event.version, event.version);
        assert_eq!(deserialized_event.data, event.data);
    }

    #[test]
    fn test_marshal_unmarshal_event_with_metadata_context() {
        // Create sample metadata and context
        let mut metadata = HashMap::new();
        metadata.insert("meta_key".to_string(), Bson::String("meta_value".to_string()));

        let mut context = HashMap::new();
        context.insert("ctx_key".to_string(), Bson::String("context_value".to_string()));

        let event = Event::new(
            "TestEvent".to_string(),
            Some(Bson::String("TestData".to_string())),
            Utc::now(),
            "TestAggregate".to_string(),
            Uuid::new_v4(),
            1,
            metadata.clone(),
            context.clone(),
        );

        // Marshal the event into BSON
        let serialized_event = EventCodec::marshal_event(&event).unwrap();

        // Unmarshal the BSON data back into an Event struct
        let deserialized_event = EventCodec::unmarshal_event(&serialized_event).unwrap();

        // Ensure the deserialized event matches the original event
        assert_eq!(deserialized_event.event_type, event.event_type);
        assert_eq!(deserialized_event.aggregate_type, event.aggregate_type);
        assert_eq!(deserialized_event.aggregate_id, event.aggregate_id);
        assert_eq!(deserialized_event.version, event.version);
        assert_eq!(deserialized_event.data, event.data);

        // Check that metadata and context are correctly deserialized
        assert_eq!(deserialized_event.metadata, event.metadata);
        assert_eq!(deserialized_event.context, event.context);
    }

    #[test]
    fn test_event_without_data() {
        let event = Event::new(
            "NoDataEvent".to_string(),
            None, // No data
            Utc::now(),
            "TestAggregate".to_string(),
            Uuid::new_v4(),
            1,
            HashMap::new(), // Empty metadata
            HashMap::new(), // Empty context
        );

        // Marshal the event into BSON
        let serialized_event = EventCodec::marshal_event(&event).unwrap();

        // Unmarshal the BSON data back into an Event struct
        let deserialized_event = EventCodec::unmarshal_event(&serialized_event).unwrap();

        // Ensure the deserialized event matches the original event
        assert_eq!(deserialized_event.event_type, event.event_type);
        assert_eq!(deserialized_event.aggregate_type, event.aggregate_type);
        assert_eq!(deserialized_event.aggregate_id, event.aggregate_id);
        assert_eq!(deserialized_event.version, event.version);
        assert!(deserialized_event.data.is_none());
    }
}