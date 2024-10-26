use std::collections::HashMap;
use std::any::Any;
use std::fmt;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::error::Error;

// Wrapper for clonable Any types
#[derive(Debug)]
pub struct CloneableAny(Box<dyn Any + Send + Sync>);

impl Clone for CloneableAny {
    fn clone(&self) -> Self {
        if let Some(cloned_value) = self.0.downcast_ref::<i32>() {
            CloneableAny(Box::new(cloned_value.clone()))
        } else if let Some(cloned_value) = self.0.downcast_ref::<String>() {
            CloneableAny(Box::new(cloned_value.clone()))
        } else {
            panic!("Attempted to clone unsupported type in CloneableAny");
        }
    }
}

impl CloneableAny {
    pub fn new<T: Any + Clone + Send + Sync>(value: T) -> Self {
        CloneableAny(Box::new(value))
    }
}

// Event trait using CloneableAny for metadata and data
pub trait Event: Send + Sync {
    fn event_type(&self) -> String;
    fn data(&self) -> Arc<CloneableAny>;
    fn timestamp(&self) -> DateTime<Utc>;
    fn aggregate_type(&self) -> String;
    fn aggregate_id(&self) -> Uuid;
    fn version(&self) -> u32;
    fn metadata(&self) -> HashMap<String, CloneableAny>;
}

// Struct to hold configuration for comparing events.
pub struct CompareConfig {
    ignore_timestamp: bool,
    ignore_version: bool,
    ignore_position: bool,
}

impl CompareConfig {
    pub fn new() -> Self {
        CompareConfig {
            ignore_timestamp: false,
            ignore_version: false,
            ignore_position: false,
        }
    }
}

// Type alias for a comparison option function.
pub type CompareOption = Box<dyn Fn(&mut CompareConfig)>;

// Ignore timestamp option setter.
pub fn ignore_timestamp() -> CompareOption {
    Box::new(|config: &mut CompareConfig| {
        config.ignore_timestamp = true;
    })
}

// Ignore version option setter.
pub fn ignore_version() -> CompareOption {
    Box::new(|config: &mut CompareConfig| {
        config.ignore_version = true;
    })
}

// Ignore position metadata option setter.
pub fn ignore_position_metadata() -> CompareOption {
    Box::new(|config: &mut CompareConfig| {
        config.ignore_position = true;
    })
}

// Custom error for event comparison.
#[derive(Debug)]
pub struct CompareError {
    details: String,
}

impl CompareError {
    fn new(msg: &str) -> CompareError {
        CompareError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for CompareError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Compare error: {}", self.details)
    }
}

impl Error for CompareError {}

// Helper function to compare metadata.
fn compare_metadata(
    m1: &HashMap<String, CloneableAny>,
    m2: &HashMap<String, CloneableAny>,
    ignore_position: bool,
) -> bool {
    let mut m1_filtered = m1.clone();
    let mut m2_filtered = m2.clone();

    if ignore_position {
        m1_filtered.remove("position");
        m2_filtered.remove("position");
    }

    if m1_filtered.len() != m2_filtered.len() {
        return false;
    }

    for (key, value1) in m1_filtered.iter() {
        if let Some(value2) = m2_filtered.get(key) {
            let ptr1: *const () = &*value1.0 as *const _ as *const ();
            let ptr2: *const () = &*value2.0 as *const _ as *const ();
            if ptr1 != ptr2 {
                return false;
            }
        } else {
            return false;
        }
    }

    true
}

// Function to compare two events.
pub fn compare_events(
    e1: &dyn Event,
    e2: &dyn Event,
    options: &[CompareOption],
) -> Result<(), Box<dyn Error>> {
    let mut config = CompareConfig::new();

    // Apply comparison options.
    for option in options {
        option(&mut config);
    }

    if e1.event_type() != e2.event_type() {
        return Err(Box::new(CompareError::new(&format!(
            "Event type mismatch: {} (should be {})",
            e1.event_type(),
            e2.event_type()
        ))));
    }

    if e1.data().type_id() != e2.data().type_id() {
        return Err(Box::new(CompareError::new("Event data mismatch")));
    }

    if !config.ignore_timestamp && e1.timestamp() != e2.timestamp() {
        return Err(Box::new(CompareError::new(&format!(
            "Timestamp mismatch: {} (should be {})",
            e1.timestamp(),
            e2.timestamp()
        ))));
    }

    if e1.aggregate_type() != e2.aggregate_type() {
        return Err(Box::new(CompareError::new(&format!(
            "Aggregate type mismatch: {} (should be {})",
            e1.aggregate_type(),
            e2.aggregate_type()
        ))));
    }

    if e1.aggregate_id() != e2.aggregate_id() {
        return Err(Box::new(CompareError::new(&format!(
            "Aggregate ID mismatch: {} (should be {})",
            e1.aggregate_id(),
            e2.aggregate_id()
        ))));
    }

    if !config.ignore_version && e1.version() != e2.version() {
        return Err(Box::new(CompareError::new(&format!(
            "Version mismatch: {} (should be {})",
            e1.version(),
            e2.version()
        ))));
    }

    if !compare_metadata(&e1.metadata(), &e2.metadata(), config.ignore_position) {
        return Err(Box::new(CompareError::new("Metadata mismatch")));
    }

    Ok(())
}

// Function to compare two slices of events.
pub fn compare_event_slices(
    evts1: Vec<Arc<dyn Event>>,
    evts2: Vec<Arc<dyn Event>>,
    options: &[CompareOption],
) -> bool {
    if evts1.len() != evts2.len() {
        return false;
    }

    for i in 0..evts1.len() {
        if compare_events(evts1[i].as_ref(), evts2[i].as_ref(), options).is_err() {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use chrono::Utc;

    // Sample Event implementation for testing.
    #[derive(Clone)]
    pub struct TestEvent {
        pub event_type: String,
        pub data: Arc<CloneableAny>,
        pub timestamp: DateTime<Utc>,
        pub aggregate_type: String,
        pub aggregate_id: Uuid,
        pub version: u32,
        pub metadata: HashMap<String, CloneableAny>,
    }

    impl Event for TestEvent {
        fn event_type(&self) -> String {
            self.event_type.clone()
        }

        fn data(&self) -> Arc<CloneableAny> {
            self.data.clone()
        }

        fn timestamp(&self) -> DateTime<Utc> {
            self.timestamp
        }

        fn aggregate_type(&self) -> String {
            self.aggregate_type.clone()
        }

        fn aggregate_id(&self) -> Uuid {
            self.aggregate_id
        }

        fn version(&self) -> u32 {
            self.version
        }

        fn metadata(&self) -> HashMap<String, CloneableAny> {
            self.metadata.clone()
        }
    }

    // Test when two events are completely equal
    #[test]
    fn test_compare_events_equal() {
        let metadata: HashMap<String, CloneableAny> = HashMap::new();
        let event1 = TestEvent {
            event_type: "TestEvent".to_string(),
            data: Arc::new(CloneableAny::new(42)),
            timestamp: Utc::now(),
            aggregate_type: "TestAggregate".to_string(),
            aggregate_id: Uuid::new_v4(),
            version: 1,
            metadata,
        };

        let event2 = event1.clone();

        let result = compare_events(&event1, &event2, &[]);
        assert!(result.is_ok(), "Expected events to be equal, but they were not.");
    }

    // Test when two events have different data
    #[test]
    fn test_compare_events_different_data() {
        let metadata: HashMap<String, CloneableAny> = HashMap::new();
        let event1 = TestEvent {
            event_type: "TestEvent".to_string(),
            data: Arc::new(CloneableAny::new(42)),
            timestamp: Utc::now(),
            aggregate_type: "TestAggregate".to_string(),
            aggregate_id: Uuid::new_v4(),
            version: 1,
            metadata,
        };

        let event2 = TestEvent {
            event_type: "TestEvent".to_string(),
            data: Arc::new(CloneableAny::new(43)), // Different data
            timestamp: Utc::now(),
            aggregate_type: "TestAggregate".to_string(),
            aggregate_id: event1.aggregate_id,
            version: 1,
            metadata: HashMap::new(),
        };

        let result = compare_events(&event1, &event2, &[]);
        assert!(result.is_err(), "Expected events to be different, but they were equal.");
    }

    // Test when two events have different metadata
    #[test]
    fn test_compare_events_different_metadata() {
        let mut metadata1: HashMap<String, CloneableAny> = HashMap::new();
        metadata1.insert("key1".to_string(), CloneableAny::new("value1".to_string()));

        let mut metadata2: HashMap<String, CloneableAny> = HashMap::new();
        metadata2.insert("key1".to_string(), CloneableAny::new("value2".to_string())); // Different metadata

        let event1 = TestEvent {
            event_type: "TestEvent".to_string(),
            data: Arc::new(CloneableAny::new(42)),
            timestamp: Utc::now(),
            aggregate_type: "TestAggregate".to_string(),
            aggregate_id: Uuid::new_v4(),
            version: 1,
            metadata: metadata1,
        };

        let event2 = TestEvent {
            event_type: "TestEvent".to_string(),
            data: Arc::new(CloneableAny::new(42)),
            timestamp: Utc::now(),
            aggregate_type: "TestAggregate".to_string(),
            aggregate_id: event1.aggregate_id,
            version: 1,
            metadata: metadata2,
        };

        let result = compare_events(&event1, &event2, &[]);
        assert!(result.is_err(), "Expected events to have different metadata, but they were equal.");
    }

    // Test when two events have different aggregate types
    #[test]
    fn test_compare_events_different_aggregate_type() {
        let metadata: HashMap<String, CloneableAny> = HashMap::new();
        let event1 = TestEvent {
            event_type: "TestEvent".to_string(),
            data: Arc::new(CloneableAny::new(42)),
            timestamp: Utc::now(),
            aggregate_type: "TestAggregateA".to_string(),
            aggregate_id: Uuid::new_v4(),
            version: 1,
            metadata,
        };

        let event2 = TestEvent {
            event_type: "TestEvent".to_string(),
            data: Arc::new(CloneableAny::new(42)),
            timestamp: Utc::now(),
            aggregate_type: "TestAggregateB".to_string(), // Different aggregate type
            aggregate_id: event1.aggregate_id,
            version: 1,
            metadata: HashMap::new(),
        };

        let result = compare_events(&event1, &event2, &[]);
        assert!(result.is_err(), "Expected events to have different aggregate types, but they were equal.");
    }

    // Test when two events have different timestamps
    #[test]
    fn test_compare_events_different_timestamp() {
        let metadata: HashMap<String, CloneableAny> = HashMap::new();
        let event1 = TestEvent {
            event_type: "TestEvent".to_string(),
            data: Arc::new(CloneableAny::new(42)),
            timestamp: Utc::now(),
            aggregate_type: "TestAggregate".to_string(),
            aggregate_id: Uuid::new_v4(),
            version: 1,
            metadata,
        };

        let event2 = TestEvent {
            event_type: "TestEvent".to_string(),
            data: Arc::new(CloneableAny::new(42)),
            timestamp: Utc::now() + chrono::Duration::seconds(1), // Different timestamp
            aggregate_type: "TestAggregate".to_string(),
            aggregate_id: event1.aggregate_id,
            version: 1,
            metadata: HashMap::new(),
        };

        let result = compare_events(&event1, &event2, &[]);
        assert!(result.is_err(), "Expected events to have different timestamps, but they were equal.");
    }

    // Test when two events have different versions
    #[test]
    fn test_compare_events_different_version() {
        let metadata: HashMap<String, CloneableAny> = HashMap::new();
        let event1 = TestEvent {
            event_type: "TestEvent".to_string(),
            data: Arc::new(CloneableAny::new(42)),
            timestamp: Utc::now(),
            aggregate_type: "TestAggregate".to_string(),
            aggregate_id: Uuid::new_v4(),
            version: 1,
            metadata,
        };

        let event2 = TestEvent {
            event_type: "TestEvent".to_string(),
            data: Arc::new(CloneableAny::new(42)),
            timestamp: Utc::now(),
            aggregate_type: "TestAggregate".to_string(),
            aggregate_id: event1.aggregate_id,
            version: 2, // Different version
            metadata: HashMap::new(),
        };

        let result = compare_events(&event1, &event2, &[]);
        assert!(result.is_err(), "Expected events to have different versions, but they were equal.");
    }

    // Test ignoring timestamps in comparison
    #[test]
    fn test_compare_events_ignore_timestamp() {
        let metadata: HashMap<String, CloneableAny> = HashMap::new();
        let event1 = TestEvent {
            event_type: "TestEvent".to_string(),
            data: Arc::new(CloneableAny::new(42)),
            timestamp: Utc::now(),
            aggregate_type: "TestAggregate".to_string(),
            aggregate_id: Uuid::new_v4(),
            version: 1,
            metadata,
        };

        let event2 = TestEvent {
            event_type: "TestEvent".to_string(),
            data: Arc::new(CloneableAny::new(42)),
            timestamp: Utc::now() + chrono::Duration::seconds(1), // Different timestamp
            aggregate_type: "TestAggregate".to_string(),
            aggregate_id: event1.aggregate_id,
            version: 1,
            metadata: HashMap::new(),
        };

        let result = compare_events(&event1, &event2, &[ignore_timestamp()]);
        assert!(result.is_ok(), "Expected events to be equal when ignoring timestamps, but they were not.");
    }
}