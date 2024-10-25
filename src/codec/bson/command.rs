use bson::{self, Bson};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

// Command struct, similar to the internal command structure in Go
#[derive(Debug, Serialize, Deserialize,PartialEq)]
pub struct Command {
    pub command_type: String,
    pub command: Bson,
    pub context: HashMap<String, Bson>,
}

// CommandCodec struct for handling BSON serialization and deserialization
pub struct CommandCodec;

impl CommandCodec {
    // MarshalCommand serializes a command to BSON bytes
    pub fn marshal_command(cmd: &Command) -> Result<Vec<u8>, bson::ser::Error> {
        bson::to_vec(cmd)
    }

    // UnmarshalCommand deserializes BSON bytes into a Command struct
    pub fn unmarshal_command(data: &[u8]) -> Result<Command, bson::de::Error> {
        bson::from_slice(data)
    }
}

// Unit tests for CommandCodec
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marshal_command() {
        let command = Command {
            command_type: "TestCommand".to_string(),
            command: Bson::String("Test data".to_string()),
            context: HashMap::new(),
        };

        // Marshal the command into BSON
        let serialized_command = CommandCodec::marshal_command(&command).unwrap();
        assert!(!serialized_command.is_empty(), "Serialized command should not be empty");
    }

    #[test]
    fn test_unmarshal_command() {
        let command = Command {
            command_type: "TestCommand".to_string(),
            command: Bson::String("Test data".to_string()),
            context: HashMap::new(),
        };

        // Marshal the command into BSON
        let serialized_command = CommandCodec::marshal_command(&command).unwrap();

        // Unmarshal the BSON data back into a Command struct
        let deserialized_command = CommandCodec::unmarshal_command(&serialized_command).unwrap();

        // Ensure the deserialized command is the same as the original
        assert_eq!(deserialized_command, command);
    }

    #[test]
    fn test_marshal_unmarshal_command() {
        let mut context = HashMap::new();
        context.insert("key".to_string(), Bson::String("value".to_string()));

        let command = Command {
            command_type: "TestCommand".to_string(),
            command: Bson::String("Test data".to_string()),
            context,
        };

        // Marshal the command into BSON
        let serialized_command = CommandCodec::marshal_command(&command).unwrap();

        // Unmarshal the BSON data back into a Command struct
        let deserialized_command = CommandCodec::unmarshal_command(&serialized_command).unwrap();

        // Ensure the deserialized command is the same as the original
        assert_eq!(deserialized_command, command);
    }
}