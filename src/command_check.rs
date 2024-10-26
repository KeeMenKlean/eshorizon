use std::error::Error;
use std::fmt;
use uuid::Uuid;
use std::time::SystemTime;

// Custom error for missing command.
#[derive(Debug)]
pub struct CommandCheckError {
    field: String,
}

impl CommandCheckError {
    pub fn new(field: &str) -> Self {
        CommandCheckError {
            field: field.to_string(),
        }
    }
}

impl fmt::Display for CommandCheckError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "missing field: {}", self.field)
    }
}

impl Error for CommandCheckError {}

// Error constants.
pub const ERR_MISSING_COMMAND: &str = "missing command";
pub const ERR_MISSING_AGGREGATE_ID: &str = "missing aggregate ID";

// Command trait representing a domain command.
pub trait Command {
    fn aggregate_id(&self) -> Uuid;
}

// A helper trait that can be implemented to check if a value is zero.
pub trait IsZero {
    fn is_zero(&self) -> bool;
}

// Implement IsZero for Uuid to check if it is zero-valued (nil).
impl IsZero for Uuid {
    fn is_zero(&self) -> bool {
        *self == Uuid::nil()
    }
}

// Implement IsZero for SystemTime to check if it is the UNIX epoch.
impl IsZero for SystemTime {
    fn is_zero(&self) -> bool {
        *self == SystemTime::UNIX_EPOCH
    }
}

// Check a command for missing or zero values.
pub fn check_command(cmd: &dyn Command) -> Result<(), Box<dyn Error>> {
    if cmd.aggregate_id().is_zero() {
        return Err(Box::new(CommandCheckError::new(ERR_MISSING_AGGREGATE_ID)));
    }

    // Add other field checks as necessary.
    // This is where you can implement additional logic for checking non-zero fields.

    Ok(())
}


// Test cases.
#[cfg(test)]
mod tests {
    use super::*;

    // Example command struct.
    pub struct MyCommand {
        pub id: Uuid,
    }

    impl Command for MyCommand {
        fn aggregate_id(&self) -> Uuid {
            self.id
        }
    }

    #[test]
    fn test_missing_aggregate_id() {
        let command = MyCommand { id: Uuid::nil() };
        let result = check_command(&command);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            CommandCheckError::new(ERR_MISSING_AGGREGATE_ID).to_string()
        );
    }

    #[test]
    fn test_valid_command() {
        let command = MyCommand { id: Uuid::new_v4() };
        let result = check_command(&command);
        assert!(result.is_ok());
    }
}