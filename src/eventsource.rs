use std::sync::Arc;

// Define the Event trait to mimic Go's Event interface
pub trait Event: Send + Sync + std::fmt::Display {}

// EventSource trait that contains uncommitted events logic
pub trait EventSource {
    // Returns a list of uncommitted events
    fn uncommitted_events(&self) -> Vec<Arc<dyn Event>>;

    // Clears the uncommitted events
    fn clear_uncommitted_events(&mut self);
}

// A basic implementation of EventSource for testing
pub struct BasicEventSource {
    uncommitted_events: Vec<Arc<dyn Event>>,
}

impl BasicEventSource {
    pub fn new() -> Self {
        Self {
            uncommitted_events: Vec::new(),
        }
    }
}

impl EventSource for BasicEventSource {
    fn uncommitted_events(&self) -> Vec<Arc<dyn Event>> {
        self.uncommitted_events.clone()
    }

    fn clear_uncommitted_events(&mut self) {
        self.uncommitted_events.clear();
    }
}

// Example test case for EventSource
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    struct TestEvent {
        name: String,
    }

    impl Event for TestEvent {}

    impl std::fmt::Display for TestEvent {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.name)
        }
    }

    #[test]
    fn test_event_source() {
        let mut event_source = BasicEventSource::new();

        let event = Arc::new(TestEvent {
            name: "TestEvent".to_string(),
        });

        // Simulate adding an event to uncommitted events
        event_source.uncommitted_events.push(event.clone());

        // Check that the event is in the uncommitted list
        let events = event_source.uncommitted_events();
        assert_eq!(events.len(), 1);
        assert_eq!(format!("{}", events[0]), "TestEvent");

        // Clear the uncommitted events
        event_source.clear_uncommitted_events();
        let events = event_source.uncommitted_events();
        assert_eq!(events.len(), 0);
    }
}