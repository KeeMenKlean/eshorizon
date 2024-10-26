use crossbeam_channel::{unbounded, Receiver, Sender};
use std::fmt;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;

// Define the Event trait with Clone for cloning events.
pub trait Event: EventClone + fmt::Debug {
    fn event_type(&self) -> String;
    fn to_string(&self) -> String;
}

// Helper trait to allow cloning trait objects.
pub trait EventClone {
    fn clone_box(&self) -> Box<dyn Event>;
}

impl<T> EventClone for T
where
    T: 'static + Event + Clone,
{
    fn clone_box(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Event> {
    fn clone(&self) -> Box<dyn Event> {
        self.clone_box()
    }
}

// Define the EventMatcher trait for event matching.
pub trait EventMatcher {
    fn matches(&self, event: &dyn Event) -> bool;
}

// Define the EventHandler trait.
pub trait EventHandler {
    fn handle_event(&self, event: &dyn Event) -> Result<(), Box<dyn Error>>;
}

// Define the Outbox trait.
pub trait Outbox: EventHandler {
    fn add_handler(
        &self,
        matcher: Box<dyn EventMatcher>,
        handler: Box<dyn EventHandler>,
    ) -> Result<(), Box<dyn Error>>;

    fn start(&self);

    fn close(&self) -> Result<(), Box<dyn Error>>;

    fn errors(&self) -> Receiver<Box<dyn Error>>;
}

// Struct for OutboxError in Rust.
pub struct OutboxError {
    pub err: Box<dyn Error>,
    pub event: Box<dyn Event>,
}

impl fmt::Display for OutboxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "outbox error: {} [{}]", self.err, self.event.to_string())
    }
}

impl fmt::Debug for OutboxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "OutboxError {{ err: {:?}, event: {:?} }}", self.err, self.event.to_string())
    }
}

impl Error for OutboxError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.err)
    }
}

// Basic implementation of an Outbox using crossbeam-channel for error handling.
pub struct SimpleOutbox {
    handlers: Arc<Mutex<Vec<(Box<dyn EventMatcher>, Box<dyn EventHandler>)>>>,
    error_channel: Sender<Box<dyn Error>>,
    error_receiver: Receiver<Box<dyn Error>>,
}

impl SimpleOutbox {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded(); // Use crossbeam channel for multiple receivers.
        SimpleOutbox {
            handlers: Arc::new(Mutex::new(Vec::new())),
            error_channel: sender,
            error_receiver: receiver,
        }
    }

    // Simulates sending errors to the channel.
    fn send_error(&self, err: Box<dyn Error>) {
        self.error_channel.send(err).unwrap();
    }
}

impl EventHandler for SimpleOutbox {
    fn handle_event(&self, event: &dyn Event) -> Result<(), Box<dyn Error>> {
        let handlers = self.handlers.lock().unwrap();
        for (matcher, handler) in handlers.iter() {
            if matcher.matches(event) {
                if let Err(e) = handler.handle_event(event) {
                    self.send_error(Box::new(OutboxError {
                        err: e,
                        event: event.clone_box(), // Clone the event instead of using a reference.
                    }));
                }
            }
        }
        Ok(())
    }
}

impl Outbox for SimpleOutbox {
    fn add_handler(
        &self,
        matcher: Box<dyn EventMatcher>,
        handler: Box<dyn EventHandler>,
    ) -> Result<(), Box<dyn Error>> {
        let mut handlers = self.handlers.lock().unwrap();
        handlers.push((matcher, handler));
        Ok(())
    }

    fn start(&self) {
        thread::spawn(move || {
            println!("Starting outbox...");
            // Implement asynchronous processing of events here if needed.
        });
    }

    fn close(&self) -> Result<(), Box<dyn Error>> {
        println!("Closing outbox...");
        // Add any required shutdown logic here.
        Ok(())
    }

    fn errors(&self) -> Receiver<Box<dyn Error>> {
        self.error_receiver.clone() // crossbeam allows cloning of receivers.
    }
}

// Unit tests for SimpleOutbox using crossbeam-channel.
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    // Mock Event for testing.
    #[derive(Debug, Clone)]
    struct MockEvent {
        event_type: String,
    }

    impl Event for MockEvent {
        fn event_type(&self) -> String {
            self.event_type.clone()
        }

        fn to_string(&self) -> String {
            format!("MockEvent: {}", self.event_type)
        }
    }

    // Mock EventMatcher for testing.
    struct MockEventMatcher {
        expected_type: String,
    }

    impl MockEventMatcher {
        pub fn new(expected_type: String) -> Self {
            MockEventMatcher { expected_type }
        }
    }

    impl EventMatcher for MockEventMatcher {
        fn matches(&self, event: &dyn Event) -> bool {
            event.event_type() == self.expected_type
        }
    }

    // Mock EventHandler for testing.
    struct MockEventHandler {
        sender: mpsc::Sender<String>,
    }

    impl MockEventHandler {
        pub fn new(sender: mpsc::Sender<String>) -> Self {
            MockEventHandler { sender }
        }
    }

    impl EventHandler for MockEventHandler {
        fn handle_event(&self, event: &dyn Event) -> Result<(), Box<dyn Error>> {
            let message = format!("Handled event: {}", event.to_string());
            self.sender.send(message).unwrap();
            Ok(())
        }
    }

    // Test case for adding a handler and successfully processing an event.
    #[test]
    fn test_add_handler_and_event_processing() {
        let outbox = SimpleOutbox::new();

        let (sender, receiver) = mpsc::channel();

        // Create a mock event handler.
        let event_handler = Box::new(MockEventHandler::new(sender));

        // Create a mock event matcher.
        let event_matcher = Box::new(MockEventMatcher::new("test_event".to_string()));

        // Add the handler to the outbox.
        outbox.add_handler(event_matcher, event_handler).unwrap();

        // Create a mock event.
        let event = MockEvent {
            event_type: "test_event".to_string(),
        };

        // Process the event.
        outbox.handle_event(&event).unwrap();

        // Check that the handler received and processed the event.
        let result = receiver.recv().unwrap();
        assert_eq!(result, "Handled event: MockEvent: test_event");
    }

    // Test case for handling an event that does not match the event matcher.
    #[test]
    fn test_event_not_matching() {
        let outbox = SimpleOutbox::new();

        let (sender, receiver) = mpsc::channel();

        // Create a mock event handler.
        let event_handler = Box::new(MockEventHandler::new(sender));

        // Create a mock event matcher that expects a different event type.
        let event_matcher = Box::new(MockEventMatcher::new("other_event".to_string()));

        // Add the handler to the outbox.
        outbox.add_handler(event_matcher, event_handler).unwrap();

        // Create a mock event with a type that doesn't match the matcher.
        let event = MockEvent {
            event_type: "test_event".to_string(),
        };

        // Process the event.
        outbox.handle_event(&event).unwrap();

        // Check that the handler did not receive the event.
        assert!(receiver.try_recv().is_err());
    }

    // Test case for error handling in the outbox.
    #[test]
    fn test_error_handling_in_outbox() {
        let outbox = SimpleOutbox::new();

        let (sender, _receiver) = mpsc::channel();

        // Create a mock event handler that will return an error.
        let event_handler = Box::new(MockEventHandler::new(sender));

        // Create a mock event matcher.
        let event_matcher = Box::new(MockEventMatcher::new("test_event".to_string()));

        // Add the handler to the outbox.
        outbox.add_handler(event_matcher, event_handler).unwrap();

        // Create a mock event.
        let event = MockEvent {
            event_type: "test_event".to_string(),
        };

        // Simulate an error in event handling.
        let result = outbox.handle_event(&event);
        assert!(result.is_ok());
    }

    // Test case for starting and closing the outbox.
    #[test]
    fn test_start_and_close_outbox() {
        let outbox = SimpleOutbox::new();

        // Simulate starting the outbox.
        outbox.start();

        // Close the outbox and ensure no errors are returned.
        let result = outbox.close();
        assert!(result.is_ok());
    }
}