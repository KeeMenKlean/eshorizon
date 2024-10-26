use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use std::collections::HashMap;
use std::error::Error;

// Command struct used for internal transport
#[derive(Serialize, Deserialize, Debug)]
struct Command {
    command_type: String,
    command: Value, // We use serde_json::Value to store raw JSON
    context: HashMap<String, Value>,
}

// CommandCodec responsible for encoding and decoding commands in JSON format
pub struct CommandCodec;

impl CommandCodec {
    // Marshal a command into JSON bytes
    pub fn marshal_command<T: Serialize>(
        command_type: String,
        cmd: &T,
        context: HashMap<String, Value>,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        // Create the command object to wrap everything
        let serialized_cmd = serde_json::to_value(cmd)?;
        let command = Command {
            command_type,
            command: serialized_cmd,
            context,
        };

        // Serialize the entire command struct into JSON bytes
        let json_bytes = serde_json::to_vec(&command)?;
        Ok(json_bytes)
    }

    // Unmarshal JSON bytes into a Command struct
    pub fn unmarshal_command<T: for<'de> Deserialize<'de>>(
        json_bytes: &[u8],
    ) -> Result<(String, T, HashMap<String, Value>), Box<dyn Error>> {
        // Deserialize the command struct
        let command: Command = serde_json::from_slice(json_bytes)?;

        // Deserialize the inner command based on the provided generic type T
        let deserialized_cmd: T = serde_json::from_value(command.command)?;

        Ok((command.command_type, deserialized_cmd, command.context))
    }
}

// Example test for serialization and deserialization
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestCommand {
        field1: String,
        field2: i32,
    }

    #[test]
    fn test_marshal_unmarshal_command() {
        let command = TestCommand {
            field1: "test".to_string(),
            field2: 42,
        };
        let command_type = "TestCommand".to_string();
        let mut context = HashMap::new();
        context.insert("user".to_string(), json!("test_user"));

        // Marshal the command
        let serialized = CommandCodec::marshal_command(command_type.clone(), &command, context.clone())
            .expect("Failed to serialize command");

        // Unmarshal the command
        let (deserialized_command_type, deserialized_command, deserialized_context) =
            CommandCodec::unmarshal_command::<TestCommand>(&serialized).expect("Failed to deserialize command");

        // Check that the command type, command, and context are the same
        assert_eq!(deserialized_command_type, command_type);
        assert_eq!(deserialized_command, command);
        assert_eq!(deserialized_context, context);
    }
}