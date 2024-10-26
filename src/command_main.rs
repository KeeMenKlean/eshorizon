use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use std::error::Error;
use std::fmt;
use lazy_static::lazy_static;

// Trait representing a Command.
pub trait Command: Send + Sync {
    fn aggregate_id(&self) -> Uuid;
    fn aggregate_type(&self) -> String;
    fn command_type(&self) -> String;
}

// Custom error for command operations.
#[derive(Debug)]
pub struct CommandError {
    err: Box<dyn Error + Send + Sync>,
}

impl CommandError {
    pub fn new<E>(err: E) -> Self
    where
        E: Into<Box<dyn Error + Send + Sync>>,
    {
        CommandError { err: err.into() }
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "command error: {}", self.err)
    }
}

impl Error for CommandError {}

// Thread-safe storage for command factories.
lazy_static! {
    static ref COMMANDS: Arc<RwLock<HashMap<String, Box<dyn Fn() ->
    Box<dyn Command + Send + Sync> + Send + Sync>>>> = Arc::new(RwLock::new(HashMap::new()));
}

// Register a command factory for a type.
pub fn register_command(
    command_type: String,
    factory: Box<dyn Fn() -> Box<dyn Command + Send + Sync> + Send + Sync>,
) {
    let mut commands = COMMANDS.write().unwrap();
    if commands.contains_key(&command_type) {
        panic!("Duplicate command type registration for {}", command_type);
    }
    commands.insert(command_type, factory);
}

// Create a command of a specific type using the registered factory.
pub fn create_command(command_type: &str) -> Result<Box<dyn Command + Send + Sync>, CommandError> {
    let commands = COMMANDS.read().unwrap();
    if let Some(factory) = commands.get(command_type) {
        Ok(factory())
    } else {
        Err(CommandError::new("Command not registered"))
    }
}

// Example command implementation.
pub struct MyCommand {
    id: Uuid,
}

impl MyCommand {
    pub fn new() -> Self {
        MyCommand {
            id: Uuid::new_v4(),
        }
    }
}

impl Command for MyCommand {
    fn aggregate_id(&self) -> Uuid {
        self.id
    }

    fn aggregate_type(&self) -> String {
        "MyAggregate".to_string()
    }

    fn command_type(&self) -> String {
        "MyCommand".to_string()
    }
}

// Test cases.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_create_command() {
        let command_type = "MyCommand".to_string();

        // Register the command factory.
        register_command(
            command_type.clone(),
            Box::new(|| Box::new(MyCommand::new())),
        );

        // Create a command.
        let command = create_command(&command_type);
        assert!(command.is_ok());

        let command = command.unwrap();
        assert_eq!(command.command_type(), "MyCommand");
    }

    #[test]
    fn test_create_unregistered_command() {
        let result = create_command("UnregisteredCommand");
        assert!(result.is_err());
    }
}