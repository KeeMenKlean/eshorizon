use chrono::{DateTime, Utc};
use mongodb::bson::{self, doc, Bson};
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
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, Bson>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
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


// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let aggregate_id = Uuid::new_v4();
//     let timestamp = Utc::now();
//
//     // Create an example event
//     let event = Event::new(
//         "ExampleEvent".to_string(),
//         None, // No data for simplicity
//         timestamp,
//         "ExampleAggregate".to_string(),
//         aggregate_id,
//         1,
//         HashMap::new(), // Empty metadata for simplicity
//         HashMap::new(), // Empty context for simplicity
//     );
//
//     // Marshal the event to BSON
//     let serialized_event = EventCodec::marshal_event(&event)?;
//     println!("Serialized Event: {:?}", serialized_event);
//
//     // Unmarshal the event back to an Event struct
//     let deserialized_event = EventCodec::unmarshal_event(&serialized_event)?;
//     println!("Deserialized Event: {:?}", deserialized_event);
//
//     Ok(())
// }