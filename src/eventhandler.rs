use std::any::Any;
use std::error::Error;
use std::fmt;
use std::sync::Arc;
use thiserror::Error;
use tokio::task;

// Define the Event trait to mimic Go's Event interface
pub trait Event: fmt::Display + Send + Sync {}

// EventHandlerType as a string for identification
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EventHandlerType(String);

impl fmt::Display for EventHandlerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// EventHandler trait: similar to the Go version with `HandleEvent` and `HandlerType` methods
pub trait EventHandler: Send + Sync {
    fn handle_event(&self,
                    ctx: &task::JoinHandle<()>,
                    event: Arc<dyn Event>) -> Result<(), EventHandlerError>;
    fn handler_type(&self) -> EventHandlerType;
}

// Functional event handler, similar to Go's EventHandlerFunc
pub struct EventHandlerFunc<F: Fn(&task::JoinHandle<()>,
    Arc<dyn Event>) -> Result<(), EventHandlerError> + Send + Sync> {
    handler_fn: F,
    handler_type: EventHandlerType,
}

impl<F> EventHandlerFunc<F>
where
    F: Fn(&task::JoinHandle<()>, Arc<dyn Event>) -> Result<(), EventHandlerError> + Send + Sync
{
    pub fn new(handler_type: String, handler_fn: F) -> Self {
        Self {
            handler_fn,
            handler_type: EventHandlerType(handler_type),
        }
    }
}

impl<F> EventHandler for EventHandlerFunc<F>
where
    F: Fn(&task::JoinHandle<()>, Arc<dyn Event>) -> Result<(), EventHandlerError> + Send + Sync
{
    fn handle_event(&self, ctx: &task::JoinHandle<()>, event: Arc<dyn Event>) -> Result<(), EventHandlerError> {
        (self.handler_fn)(ctx, event)
    }

    fn handler_type(&self) -> EventHandlerType {
        self.handler_type.clone()
    }
}

// Custom errors related to event handling
#[derive(Error, Debug)]
pub enum EventHandlerError {
    #[error("missing event")]
    MissingEvent,

    #[error("could not handle event: {0}")]
    HandlingError(String),
}

// Example test case for EventHandler

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::task;

    struct TestEvent {
        name: String,
    }

    impl fmt::Display for TestEvent {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.name)
        }
    }

    impl Event for TestEvent {}

    #[tokio::test]
    async fn test_event_handler_func() {
        let handler = EventHandlerFunc::new(
            "test_handler".to_string(),
            |_, event| {
                println!("Handled event: {}", event);
                Ok(())
            },
        );

        let test_event = Arc::new(TestEvent {
            name: "Test Event".to_string(),
        });

        let ctx = task::spawn(async {});
        assert!(handler.handle_event(&ctx, test_event.clone()).is_ok());
    }
}