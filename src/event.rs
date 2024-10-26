use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use uuid::Uuid;
use std::fmt;
use lazy_static::lazy_static;

// EventData trait represents data attached to an event.
pub trait EventData: Send + Sync {}

// MyEventData struct implements EventData for demonstration.
#[derive(Debug)]
pub struct MyEventData {
    pub field: String,
}

impl EventData for MyEventData {}

lazy_static! {
    static ref EVENT_DATA_FACTORIES:
    Arc<RwLock<HashMap<String, Box<dyn Fn() ->
    Box<dyn EventData + Send + Sync> + Send + Sync>>>> = Arc::new(RwLock::new(HashMap::new()));
}

pub trait Event {
    fn event_type(&self) -> String;
    fn data(&self) -> &dyn EventData;
    fn timestamp(&self) -> SystemTime;
    fn aggregate_type(&self) -> String;
    fn aggregate_id(&self) -> Uuid;
    fn version(&self) -> i32;
    fn metadata(&self) -> &HashMap<String, String>;
    fn to_string(&self) -> String;
}

pub struct MyEvent {
    event_type: String,
    data: Box<dyn EventData + Send + Sync>,
    timestamp: SystemTime,
    aggregate_type: String,
    aggregate_id: Uuid,
    version: i32,
    metadata: HashMap<String, String>,
}

impl MyEvent {
    pub fn new(event_type: String, data:
    Box<dyn EventData + Send + Sync>, aggregate_type: String, aggregate_id: Uuid, version: i32) -> Self {
        MyEvent {
            event_type,
            data,
            timestamp: SystemTime::now(),
            aggregate_type,
            aggregate_id,
            version,
            metadata: HashMap::new(),
        }
    }
}

impl Event for MyEvent {
    fn event_type(&self) -> String {
        self.event_type.clone()
    }

    fn data(&self) -> &dyn EventData {
        &*self.data
    }

    fn timestamp(&self) -> SystemTime {
        self.timestamp
    }

    fn aggregate_type(&self) -> String {
        self.aggregate_type.clone()
    }

    fn aggregate_id(&self) -> Uuid {
        self.aggregate_id
    }

    fn version(&self) -> i32 {
        self.version
    }

    fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    fn to_string(&self) -> String {
        format!("Event type: {}", self.event_type)
    }
}

// Error for unregistered event data
#[derive(Debug, Clone)]
pub struct EventDataNotRegistered;

impl fmt::Display for EventDataNotRegistered {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "event data not registered")
    }
}

// Register event data factory
pub fn register_event_data(event_type: String, factory: Box<dyn Fn() -> Box<dyn EventData + Send + Sync> + Send + Sync>) {
    let mut factories = EVENT_DATA_FACTORIES.write().unwrap();
    if factories.contains_key(&event_type) {
        panic!("Duplicate event type registration for {}", event_type);
    }
    factories.insert(event_type, factory);
}

// Create event data
pub fn create_event_data(event_type: &str) -> Result<Box<dyn EventData + Send + Sync>, EventDataNotRegistered> {
    let factories = EVENT_DATA_FACTORIES.read().unwrap();
    if let Some(factory) = factories.get(event_type) {
        Ok(factory())
    } else {
        Err(EventDataNotRegistered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event_data = Box::new(MyEventData { field: "Some data".to_string() });
        let event = MyEvent::new(
            "TestEvent".to_string(),
            event_data,
            "TestAggregate".to_string(),
            Uuid::new_v4(),
            1,
        );
        assert_eq!(event.event_type(), "TestEvent");
        assert_eq!(event.aggregate_type(), "TestAggregate");
        assert_eq!(event.version(), 1);
    }

    #[test]
    fn test_register_and_create_event_data() {
        register_event_data("TestEvent".to_string(), Box::new(|| Box::new(MyEventData { field: "Test".to_string() })));
        let event_data = create_event_data("TestEvent");
        assert!(event_data.is_ok());
    }

    #[test]
    fn test_event_data_not_registered() {
        let result = create_event_data("UnregisteredEvent");
        assert!(result.is_err());
    }
}