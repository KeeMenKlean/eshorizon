use async_trait::async_trait;
use std::any::Any;
use std::error::Error;
use std::fmt;
use std::sync::Arc;
use tokio::task::block_in_place;

// Event trait, representing a basic event.
pub trait Event: Send + Sync + fmt::Debug + Any {}

// Command trait, representing a basic command.
pub trait Command: Send + Sync + fmt::Debug + Any {}

// Error type for codec errors.
#[derive(Debug)]
pub struct CodecError(String);

impl fmt::Display for CodecError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "codec error: {}", self.0)
    }
}

impl Error for CodecError {}

#[async_trait]
pub trait EventCodec: Send + Sync {
    async fn marshal_event(&self,
                           ctx: Arc<tokio::sync::Mutex<()>>,
                           event: Arc<dyn Event>) -> Result<Vec<u8>, CodecError>;
    async fn unmarshal_event(&self,
                           ctx: Arc<tokio::sync::Mutex<()>>,
                           data: Vec<u8>) -> Result<(Arc<dyn Event>, Arc<tokio::sync::Mutex<()>>), CodecError>;
}

#[async_trait]
pub trait CommandCodec: Send + Sync {
    async fn marshal_command(&self,
                             ctx: Arc<tokio::sync::Mutex<()>>,
                             command: Arc<dyn Command>) -> Result<Vec<u8>, CodecError>;
    async fn unmarshal_command(&self,
                             ctx: Arc<tokio::sync::Mutex<()>>,
                             data: Vec<u8>) -> Result<(Arc<dyn Command>, Arc<tokio::sync::Mutex<()>>), CodecError>;
}

// A sample implementation of EventCodec for demonstration purposes.
pub struct MyEventCodec;

#[async_trait]
impl EventCodec for MyEventCodec {
    async fn marshal_event(&self,
                           _ctx: Arc<tokio::sync::Mutex<()>>,
                           event: Arc<dyn Event>) -> Result<Vec<u8>, CodecError> {
        block_in_place(|| {
            // Here you would implement the real serialization logic, for now we return an empty Vec.
            println!("Marshaling event: {:?}", event);
            Ok(vec![])
        })
    }

    async fn unmarshal_event(&self, _ctx:
    Arc<tokio::sync::Mutex<()>>, _data: Vec<u8>) -> Result<(Arc<dyn Event>, Arc<tokio::sync::Mutex<()>>), CodecError> {
        block_in_place(|| {
            // Here you would implement the real deserialization logic, for now we return an error.
            Err(CodecError("Unmarshaling not implemented".to_string()))
        })
    }
}

// A sample implementation of CommandCodec for demonstration purposes.
pub struct MyCommandCodec;

#[async_trait]
impl CommandCodec for MyCommandCodec {
    async fn marshal_command(&self, _ctx:
    Arc<tokio::sync::Mutex<()>>, command: Arc<dyn Command>) -> Result<Vec<u8>, CodecError> {
        block_in_place(|| {
            // Here you would implement the real serialization logic, for now we return an empty Vec.
            println!("Marshaling command: {:?}", command);
            Ok(vec![])
        })
    }

    async fn unmarshal_command(&self, _ctx:
    Arc<tokio::sync::Mutex<()>>, _data: Vec<u8>) -> Result<(Arc<dyn Command>, Arc<tokio::sync::Mutex<()>>), CodecError> {
        block_in_place(|| {
            // Here you would implement the real deserialization logic, for now we return an error.
            Err(CodecError("Unmarshaling not implemented".to_string()))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::Mutex as AsyncMutex;

    #[derive(Debug)]
    struct TestEvent;

    impl Event for TestEvent {}

    #[derive(Debug)]
    struct TestCommand;

    impl Command for TestCommand {}

    // Ensure multi-threaded runtime is used
    #[tokio::test(flavor = "multi_thread")]
    async fn test_marshal_event() {
        let codec = MyEventCodec;
        let ctx = Arc::new(AsyncMutex::new(()));
        let event = Arc::new(TestEvent);

        let result = codec.marshal_event(ctx.clone(), event).await;
        assert!(result.is_ok());
    }

    // Ensure multi-threaded runtime is used
    #[tokio::test(flavor = "multi_thread")]
    async fn test_marshal_command() {
        let codec = MyCommandCodec;
        let ctx = Arc::new(AsyncMutex::new(()));
        let command = Arc::new(TestCommand);

        let result = codec.marshal_command(ctx.clone(), command).await;
        assert!(result.is_ok());
    }
}