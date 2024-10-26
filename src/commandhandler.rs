use std::error::Error;
use std::fmt;
use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::Mutex;
use uuid::Uuid;

// Define the Command trait, which represents a command.
pub trait Command: Send + Sync {
    fn aggregate_id(&self) -> Uuid;
}

// Custom error for command handling operations.
#[derive(Debug)]
pub struct CommandHandlerError {
    details: String,
}

impl CommandHandlerError {
    pub fn new(msg: &str) -> CommandHandlerError {
        CommandHandlerError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for CommandHandlerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CommandHandler error: {}", self.details)
    }
}

impl Error for CommandHandlerError {}

// Define the CommandHandler trait.
#[async_trait]
pub trait CommandHandler: Send + Sync {
    async fn handle_command(&self, ctx: Arc<Mutex<()>>, cmd: Arc<dyn Command>) -> Result<(), Box<dyn Error>>;
}

// Define a function type CommandHandlerFn that can be used as a command handler.
pub type CommandHandlerFn = Arc<dyn Fn(Arc<Mutex<()>>, Arc<dyn Command>) -> Result<(), Box<dyn Error>> + Send + Sync>;

// Implement CommandHandler for CommandHandlerFn.
#[async_trait]
impl CommandHandler for CommandHandlerFn {
    async fn handle_command(&self, ctx: Arc<Mutex<()>>, cmd: Arc<dyn Command>) -> Result<(), Box<dyn Error>> {
        self(ctx, cmd)
    }
}

// Test cases.
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::Mutex;


    // Example implementation of a Command.
    pub struct MyCommand {
        pub id: Uuid,
    }

    impl Command for MyCommand {
        fn aggregate_id(&self) -> Uuid {
            self.id
        }
    }


    #[tokio::test]
    async fn test_command_handler_func() {
        let handler: CommandHandlerFn = Arc::new(|_ctx, cmd| {
            println!("Handling command with ID: {}", cmd.aggregate_id());
            Ok(())
        });

        let command = Arc::new(MyCommand { id: Uuid::new_v4() });
        let context = Arc::new(Mutex::new(()));

        let result = handler.handle_command(context.clone(), command.clone()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_command_handler_error() {
        let handler: CommandHandlerFn = Arc::new(|_ctx, _cmd| {
            Err(Box::new(CommandHandlerError::new("Command handling failed")))
        });

        let command = Arc::new(MyCommand { id: Uuid::new_v4() });
        let context = Arc::new(Mutex::new(()));

        let result = handler.handle_command(context.clone(), command.clone()).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "CommandHandler error: Command handling failed");
    }
}