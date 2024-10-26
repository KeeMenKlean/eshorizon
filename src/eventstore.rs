use std::sync::Arc;
use async_trait::async_trait;
use std::error::Error;
use std::fmt;
use ::uuid::Uuid;

// Define the Event trait and add Debug to it
pub trait Event: Send + Sync + fmt::Display {}

// EventStore trait, analogous to the Go EventStore interface
#[async_trait]
pub trait EventStore {
    // Save appends events to the store
    async fn save(&self, events: Vec<Arc<dyn Event>>, original_version: i32) -> Result<(), EventStoreError>;

    // Load retrieves all events for a given aggregate ID
    async fn load(&self, aggregate_id: Uuid) -> Result<Vec<Arc<dyn Event>>, EventStoreError>;

    // LoadFrom retrieves events starting from a specific version
    async fn load_from(&self, aggregate_id: Uuid, version: i32) -> Result<Vec<Arc<dyn Event>>, EventStoreError>;

    // Close the event store
    async fn close(&self) -> Result<(), Box<dyn Error + Send + Sync>>;
}

// SnapshotStore trait
#[async_trait]
pub trait SnapshotStore {
    async fn load_snapshot(&self, aggregate_id: Uuid) -> Result<Snapshot, Box<dyn Error + Send + Sync>>;
    async fn save_snapshot(&self, aggregate_id: Uuid, snapshot: Snapshot) -> Result<(), Box<dyn Error + Send + Sync>>;
}

// Snapshot struct placeholder (you can customize this as needed)
pub struct Snapshot;

// Define custom EventStoreError for handling event store errors
pub struct EventStoreError {
    pub err: Option<Box<dyn Error + Send + Sync>>,
    pub op: Option<String>,
    pub aggregate_type: Option<String>,
    pub aggregate_id: Option<Uuid>,
    pub aggregate_version: Option<i32>,
    pub events: Vec<Arc<dyn Event>>,  // Now Event must implement Debug
}

// Manual implementation of Debug for EventStoreError
impl fmt::Debug for EventStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut message = String::new();

        if let Some(op) = &self.op {
            message += &format!("Operation: {}, ", op);
        }

        if let Some(err) = &self.err {
            message += &format!("Error: {}, ", err);
        }

        if let Some(aggregate_id) = &self.aggregate_id {
            let aggregate_type = self.aggregate_type.as_deref().unwrap_or("Aggregate");
            message += &format!("{}, ID: {}, Version: {}, ", aggregate_type, aggregate_id, self.aggregate_version.unwrap_or(0));
        }

        // We can't print the debug for `dyn Event` directly, but we can format it
        message += "Events: [";
        for event in &self.events {
            message += &format!("{}, ", event);
        }
        message += "]";

        write!(f, "{}", message)
    }
}

impl fmt::Display for EventStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut message = String::from("event store: ");

        if let Some(op) = &self.op {
            message += &format!("{}: ", op);
        }

        if let Some(err) = &self.err {
            message += &format!("{}", err);
        } else {
            message += "unknown error";
        }

        if let Some(aggregate_id) = &self.aggregate_id {
            let aggregate_type = self.aggregate_type.as_deref().unwrap_or("Aggregate");
            message += &format!(", {}({}, v{})", aggregate_type, aggregate_id, self.aggregate_version.unwrap_or(0));
        }

        if !self.events.is_empty() {
            let event_strings: Vec<String> = self.events.iter().map(|e| format!("{}", e)).collect();
            message += &format!(" [{}]", event_strings.join(", "));
        }

        write!(f, "{}", message)
    }
}



impl Error for EventStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        // Use `map` to downcast the error from `Send + Sync` to `Error + 'static`
        self.err.as_deref().map(|err| err as &(dyn Error + 'static))
    }
}

impl EventStoreError {
    pub fn new(
        err: Option<Box<dyn Error + Send + Sync>>,
        op: Option<String>,
        aggregate_type: Option<String>,
        aggregate_id: Option<Uuid>,
        aggregate_version: Option<i32>,
        events: Vec<Arc<dyn Event>>,
    ) -> Self {
        Self {
            err,
            op,
            aggregate_type,
            aggregate_id,
            aggregate_version,
            events,
        }
    }
}


// Test cases
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::Mutex;


    // A simple in-memory event store for testing
    pub struct InMemoryEventStore {
        store: Mutex<Vec<(Uuid, Vec<Arc<dyn Event>>, i32)>>, // (aggregate ID, events, version)
    }

    impl InMemoryEventStore {
        pub fn new() -> Self {
            Self {
                store: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl EventStore for InMemoryEventStore {
        async fn save(&self, events: Vec<Arc<dyn Event>>, original_version: i32) -> Result<(), EventStoreError> {
            if events.is_empty() {
                return Err(EventStoreError::new(
                    Some(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, "missing events"))),
                    Some("save".to_string()),
                    None,
                    None,
                    None,
                    Vec::new(),
                ));
            }

            let aggregate_id = Uuid::new_v4();
            let mut store = self.store.lock().await;

            // Insert events with version
            store.push((aggregate_id, events, original_version));
            Ok(())
        }

        async fn load(&self, aggregate_id: Uuid) -> Result<Vec<Arc<dyn Event>>, EventStoreError> {
            let store = self.store.lock().await;
            for (id, events, _) in store.iter() {
                if *id == aggregate_id {
                    return Ok(events.clone());
                }
            }

            Err(EventStoreError::new(
                Some(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "aggregate not found"))),
                Some("load".to_string()),
                None,
                Some(aggregate_id),
                None,
                Vec::new(),
            ))
        }

        async fn load_from(&self, aggregate_id: Uuid, version: i32) -> Result<Vec<Arc<dyn Event>>, EventStoreError> {
            let store = self.store.lock().await;
            for (id, events, stored_version) in store.iter() {
                if *id == aggregate_id && *stored_version >= version {
                    return Ok(events.clone());
                }
            }

            Err(EventStoreError::new(
                Some(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "aggregate not found or version mismatch"))),
                Some("load_from".to_string()),
                None,
                Some(aggregate_id),
                Some(version),
                Vec::new(),
            ))
        }

        async fn close(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
            Ok(())
        }
    }

    // Define a simple test event
    #[derive(Debug)]
    struct TestEvent {
        name: String,
    }

    impl Event for TestEvent {}

    impl fmt::Display for TestEvent {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.name)
        }
    }



    #[tokio::test]
    async fn test_event_store_save_and_load() {
        let store = InMemoryEventStore::new();

        let event1 = Arc::new(TestEvent {
            name: "Event1".to_string(),
        });
        let event2 = Arc::new(TestEvent {
            name: "Event2".to_string(),
        });

        // Convert Vec<Arc<TestEvent>> to Vec<Arc<dyn Event>>
        let events: Vec<Arc<dyn Event>> = vec![event1.clone(), event2.clone()]
            .into_iter()
            .map(|event| event as Arc<dyn Event>)
            .collect();

        // Save events (now using Vec<Arc<dyn Event>>)
        assert!(store.save(events.clone(), 1).await.is_ok());

        let aggregate_id = Uuid::new_v4(); // Simulating loading with a new random UUID
        let result = store.load(aggregate_id).await;
        assert!(result.is_err()); // Load should fail because we used a non-matching aggregate ID
    }

    #[tokio::test]
    async fn test_save_missing_events_error() {
        let store = InMemoryEventStore::new();

        // Attempt to save with no events
        let result = store.save(vec![], 1).await;
        assert!(result.is_err());

        if let Err(error) = result {
            assert_eq!(format!("{}", error), "event store: save: missing events");
        }
    }

    #[tokio::test]
    async fn test_load_aggregate_not_found() {
        let store = InMemoryEventStore::new();
        let aggregate_id = Uuid::new_v4(); // Random UUID

        // Try loading from an empty store
        let result = store.load(aggregate_id).await;
        assert!(result.is_err());

        if let Err(error) = result {
            assert_eq!(format!("{}", error), format!("event store: load: aggregate not found, Aggregate({}, v0)", aggregate_id));
        }
    }

    #[tokio::test]
    async fn test_load_from_version_mismatch() {
        let store = InMemoryEventStore::new();
        let aggregate_id = Uuid::new_v4(); // Random UUID

        // Try loading from version with no events
        let result = store.load_from(aggregate_id, 5).await;
        assert!(result.is_err());

        if let Err(error) = result {
            assert_eq!(format!("{}", error), format!("event store: load_from: aggregate not found or version mismatch, Aggregate({}, v5)", aggregate_id));
        }
    }

    #[tokio::test]
    async fn test_debug_implementation() {
        let store = InMemoryEventStore::new();

        let event = Arc::new(TestEvent {
            name: "TestDebugEvent".to_string(),
        });

        // Convert Vec<Arc<TestEvent>> to Vec<Arc<dyn Event>> by mapping
        let events: Vec<Arc<dyn Event>> = vec![event.clone()]
            .into_iter()
            .map(|event| event as Arc<dyn Event>)
            .collect();

        let save_result = store.save(events.clone(), 1).await;
        assert!(save_result.is_ok());

        let aggregate_id = Uuid::new_v4(); // Random aggregate ID
        let load_result = store.load(aggregate_id).await;

        if let Err(error) = load_result {
            println!("{:?}", error); // Test that Debug works
        }
    }
}