use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use uuid::Uuid;
use std::fmt;
use std::any::Any;

// Trait for Snapshotable entities.
pub trait Snapshotable {
    fn create_snapshot(&self) -> Snapshot;
    fn apply_snapshot(&mut self, snapshot: &Snapshot);
}

// Struct for Snapshot.
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub version: i32,
    pub aggregate_type: AggregateType,
    pub timestamp: SystemTime,
    pub state: Box<dyn SnapshotData>,
}

// Define the SnapshotData trait for the state in snapshots.
pub trait SnapshotData: SnapshotDataClone + fmt::Debug + AsAny {}

// Helper trait for enabling cloning of SnapshotData trait objects.
pub trait SnapshotDataClone {
    fn clone_box(&self) -> Box<dyn SnapshotData>;
}

impl<T> SnapshotDataClone for T
where
    T: 'static + SnapshotData + Clone,
{
    fn clone_box(&self) -> Box<dyn SnapshotData> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn SnapshotData> {
    fn clone(&self) -> Box<dyn SnapshotData> {
        self.clone_box()
    }
}

// A trait that allows downcasting of SnapshotData to concrete types.
pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

impl<T: 'static + SnapshotData> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

// A struct representing the type of an aggregate (similar to Go's AggregateType).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AggregateType(String);

// Snapshot factory registry for different aggregate types.
pub struct SnapshotFactoryRegistry {
    factories: Arc<RwLock<HashMap<AggregateType, Box<dyn Fn(Uuid) -> Box<dyn SnapshotData>>>>>,
}

impl SnapshotFactoryRegistry {
    pub fn new() -> Self {
        SnapshotFactoryRegistry {
            factories: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // Registers a snapshot factory for a specific aggregate type.
    pub fn register_snapshot_data<F>(&self, aggregate_type: AggregateType, factory: F)
    where
        F: 'static + Fn(Uuid) -> Box<dyn SnapshotData>,
    {
        if aggregate_type.0.is_empty() {
            panic!("attempt to register empty aggregate type");
        }

        let mut factories = self.factories.write().unwrap();
        if factories.contains_key(&aggregate_type) {
            panic!(
                "registering duplicate types for {}",
                aggregate_type.0
            );
        }
        factories.insert(aggregate_type, Box::new(factory));
    }

    // Creates a concrete instance using the registered snapshot factories.
    pub fn create_snapshot_data(
        &self,
        aggregate_id: Uuid,
        aggregate_type: AggregateType,
    ) -> Result<Box<dyn SnapshotData>, String> {
        let factories = self.factories.read().unwrap();
        if let Some(factory) = factories.get(&aggregate_type) {
            Ok(factory(aggregate_id))
        } else {
            Err("snapshot data not registered".to_string())
        }
    }
}

// Unit tests for the Snapshot functionality.
#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use std::time::SystemTime;

    #[derive(Debug, Clone)]
    struct MySnapshotData {
        pub id: Uuid,
        pub value: String,
    }

    impl SnapshotData for MySnapshotData {}

    #[test]
    fn test_snapshot_creation() {
        let data = MySnapshotData {
            id: Uuid::new_v4(),
            value: "snapshot_data".to_string(),
        };

        let snapshot = Snapshot {
            version: 1,
            aggregate_type: AggregateType("MyAggregate".to_string()),
            timestamp: SystemTime::now(),
            state: Box::new(data.clone()),
        };

        assert_eq!(snapshot.version, 1);
        assert_eq!(snapshot.aggregate_type.0, "MyAggregate");
    }

    #[test]
    fn test_snapshot_factory_registration() {
        let registry = SnapshotFactoryRegistry::new();
        let aggregate_type = AggregateType("MyAggregate".to_string());

        registry.register_snapshot_data(aggregate_type.clone(), |id| {
            Box::new(MySnapshotData {
                id,
                value: "test_value".to_string(),
            })
        });

        let result = registry.create_snapshot_data(Uuid::new_v4(), aggregate_type);
        assert!(result.is_ok());

        let snapshot_data = result.unwrap();
        let my_data = snapshot_data
            .as_any()
            .downcast_ref::<MySnapshotData>()
            .unwrap();

        assert_eq!(my_data.value, "test_value");
    }
}