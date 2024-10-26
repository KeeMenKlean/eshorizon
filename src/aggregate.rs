use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use std::fmt;
use std::error::Error as StdError;
use lazy_static::lazy_static;

lazy_static! {
    static ref AGGREGATES:
    Arc<RwLock<HashMap<String, Box<dyn Fn(Uuid) ->
    Box<dyn Aggregate + Send + Sync> + Send + Sync>>>> = Arc::new(RwLock::new(HashMap::new()));
}

// Aggregate trait, representing a versioned entity.
pub trait Aggregate: Send + Sync {
    fn aggregate_type(&self) -> String;
    fn entity_id(&self) -> Uuid;
    fn handle_command(&self);
}

// A custom error for aggregate operations.
#[derive(Debug)]
pub struct AggregateError {
    err: Box<dyn StdError + Send + Sync>,
}

impl AggregateError {
    pub fn new<E>(err: E) -> Self
    where
        E: Into<Box<dyn StdError + Send + Sync>>,
    {
        AggregateError { err: err.into() }
    }
}

impl fmt::Display for AggregateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "aggregate error: {}", self.err)
    }
}

impl StdError for AggregateError {}

// Register an aggregate factory for a type.
pub fn register_aggregate(
    aggregate_type: String,
    factory: Box<dyn Fn(Uuid) -> Box<dyn Aggregate + Send + Sync> + Send + Sync>,
) {
    let mut aggregates = AGGREGATES.write().unwrap();
    if aggregates.contains_key(&aggregate_type) {
        panic!("Duplicate aggregate type registration for {}", aggregate_type);
    }
    aggregates.insert(aggregate_type, factory);
}

// Create an aggregate of a specific type using the registered factory.
pub fn create_aggregate(
    aggregate_type: &str,
    id: Uuid,
) -> Result<Box<dyn Aggregate + Send + Sync>, AggregateError> {
    let aggregates = AGGREGATES.read().unwrap();
    if let Some(factory) = aggregates.get(aggregate_type) {
        Ok(factory(id))
    } else {
        Err(AggregateError::new("Aggregate not registered"))
    }
}

// Example aggregate implementation
pub struct MyAggregate {
    id: Uuid,
}

impl MyAggregate {
    pub fn new(id: Uuid) -> Self {
        MyAggregate { id }
    }
}

impl Aggregate for MyAggregate {
    fn aggregate_type(&self) -> String {
        "MyAggregate".to_string()
    }

    fn entity_id(&self) -> Uuid {
        self.id
    }

    fn handle_command(&self) {
        println!("Handling command for aggregate with ID: {}", self.id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_create_aggregate() {
        let aggregate_type = "MyAggregate".to_string();

        // Register the aggregate factory.
        register_aggregate(
            aggregate_type.clone(),
            Box::new(|id| Box::new(MyAggregate::new(id))),
        );

        // Create an aggregate.
        let id = Uuid::new_v4();
        let aggregate = create_aggregate(&aggregate_type, id);
        assert!(aggregate.is_ok());

        let aggregate = aggregate.unwrap();
        assert_eq!(aggregate.aggregate_type(), "MyAggregate");
        assert_eq!(aggregate.entity_id(), id);
    }

    #[test]
    fn test_create_unregistered_aggregate() {
        let result = create_aggregate("UnregisteredAggregate", Uuid::new_v4());
        assert!(result.is_err());
    }
}