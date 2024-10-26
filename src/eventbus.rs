use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::{Arc, Mutex};
use std::fmt;
use std::error::Error;
use tokio::sync::oneshot;
use tokio::task;
use std::collections::HashMap;
use thiserror::Error;

// Define the Event trait to mimic Go's Event interface
pub trait Event: fmt::Display + Send + Sync {}

// EventHandler trait that every handler should implement.
pub trait EventHandler: Send + Sync {
    fn handle_event(&self, event: &dyn Event);
}

// EventMatcher trait, mimicking Go's matcher interface.
pub trait EventMatcher: Send + Sync {
    fn matches(&self, event: &dyn Event) -> bool;
}

// Custom errors to match Go's errors.
#[derive(Error, Debug)]
pub enum EventBusError {
    #[error("missing matcher")]
    MissingMatcher,

    #[error("missing handler")]
    MissingHandler,

    #[error("handler already added")]
    HandlerAlreadyAdded,

    #[error("event handling error: {0}")]
    HandlingError(String),
}

// Define the EventBus structure to hold handlers.
pub struct EventBus {
    handlers: Arc<Mutex<HashMap<usize, Arc<dyn EventHandler>>>>, // Using usize as a key for unique handler address.
    error_tx: Sender<EventBusError>,
}

impl EventBus {
    pub fn new() -> Self {
        let (error_tx, _): (Sender<EventBusError>, Receiver<EventBusError>) = mpsc::channel();
        Self {
            handlers: Arc::new(Mutex::new(HashMap::new())),
            error_tx,
        }
    }

    pub fn add_handler(&self, matcher: Arc<dyn EventMatcher>, handler: Arc<dyn EventHandler>) -> Result<(), EventBusError> {
        // Check for missing matcher or handler
        if Arc::strong_count(&matcher) == 0 {
            return Err(EventBusError::MissingMatcher);
        }
        if Arc::strong_count(&handler) == 0 {
            return Err(EventBusError::MissingHandler);
        }

        // Get the raw thin pointer and cast to usize.
        let handler_key = Arc::as_ptr(&handler) as *const () as usize;
        let mut handlers = self.handlers.lock().unwrap();
        if handlers.contains_key(&handler_key) {
            return Err(EventBusError::HandlerAlreadyAdded);
        }

        handlers.insert(handler_key, handler);
        Ok(())
    }

    pub fn errors(&self) -> Receiver<EventBusError> {
        let (_, rx): (Sender<EventBusError>, Receiver<EventBusError>) = mpsc::channel();
        rx
    }

    pub async fn close(&self) -> Result<(), EventBusError> {
        // Logic for shutting down the bus
        Ok(())
    }
}

// Test case for the EventBus.
#[cfg(test)]
mod tests {
    use super::*;

    struct TestEvent {
        name: String,
    }

    impl fmt::Display for TestEvent {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.name)
        }
    }

    impl Event for TestEvent {}

    struct TestHandler;

    impl EventHandler for TestHandler {
        fn handle_event(&self, event: &dyn Event) {
            println!("Handled event: {}", event);
        }
    }

    struct TestMatcher;

    impl EventMatcher for TestMatcher {
        fn matches(&self, _event: &dyn Event) -> bool {
            true
        }
    }

    #[tokio::test]
    async fn test_event_bus() {
        let event_bus = EventBus::new();
        let matcher = Arc::new(TestMatcher);
        let handler = Arc::new(TestHandler);

        assert!(event_bus.add_handler(matcher.clone(), handler.clone()).is_ok());

        let test_event = TestEvent { name: "Test Event".to_string() };
        handler.handle_event(&test_event);
    }
}