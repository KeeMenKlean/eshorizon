// This module provides an easy way to handle UUIDs, similar to the Go version.
// The uuid crate is used to generate and manipulate UUIDs.
use uuid::Uuid;

// A type alias for Uuid, similar to Go's UUID alias.
pub type UUID = Uuid;

// Nil represents an empty UUID (all zeros).
pub const NIL: UUID = Uuid::nil();

// Generates a new UUID (version 4, random-based).
pub fn new() -> UUID {
    Uuid::new_v4()
}

// Parses a UUID from a string and returns a Result.
pub fn parse(s: &str) -> Result<UUID, uuid::Error> {
    Uuid::parse_str(s)
}

// Parses a UUID from a string and panics if parsing fails.
pub fn must_parse(s: &str) -> UUID {
    Uuid::parse_str(s).expect("Failed to parse UUID")
}