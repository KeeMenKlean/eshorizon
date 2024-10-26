// entity.rs

use uuid::Uuid;

// Entity trait defines an item that is identified by an ID.
pub trait Entity {
    fn entity_id(&self) -> Uuid;
}

// Versionable trait defines an item that has a version number.
pub trait Versionable {
    fn aggregate_version(&self) -> i32;
}

// A concrete struct implementing both Entity and Versionable traits.
pub struct MyEntity {
    id: Uuid,
    version: i32,
}

impl MyEntity {
    // Constructor for creating a new MyEntity.
    pub fn new(id: Uuid, version: i32) -> Self {
        MyEntity { id, version }
    }
}

impl Entity for MyEntity {
    fn entity_id(&self) -> Uuid {
        self.id
    }
}

impl Versionable for MyEntity {
    fn aggregate_version(&self) -> i32 {
        self.version
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_entity_id() {
        let id = Uuid::new_v4();
        let entity = MyEntity::new(id, 1);
        assert_eq!(entity.entity_id(), id);
    }

    #[test]
    fn test_aggregate_version() {
        let id = Uuid::new_v4();
        let entity = MyEntity::new(id, 10);
        assert_eq!(entity.aggregate_version(), 10);
    }
}