use uuid::Uuid;
use std::fmt;
use std::error::Error;

// Define the Entity trait and make it cloneable using a helper trait.
pub trait Entity: EntityClone + fmt::Debug {
    fn id(&self) -> Uuid;
}

// Helper trait to enable cloning for trait objects.
pub trait EntityClone {
    fn clone_box(&self) -> Box<dyn Entity>;
}

// Implement EntityClone for any type that implements Entity + Clone.
impl<T> EntityClone for T
where
    T: 'static + Entity + Clone,
{
    fn clone_box(&self) -> Box<dyn Entity> {
        Box::new(self.clone())
    }
}

// Implement Clone for Box<dyn Entity>.
impl Clone for Box<dyn Entity> {
    fn clone(&self) -> Box<dyn Entity> {
        self.clone_box()
    }
}

// Define the ReadRepo trait for reading entities.
pub trait ReadRepo {
    fn inner_repo(&self) -> Option<Box<dyn ReadRepo>>; // Return the inner repo if any.
    fn find(&self, id: Uuid) -> Result<Box<dyn Entity>, RepoError>;
    fn find_all(&self) -> Result<Vec<Box<dyn Entity>>, RepoError>;
    fn close(&self) -> Result<(), RepoError>;
}

// Define the WriteRepo trait for writing entities.
pub trait WriteRepo {
    fn save(&self, entity: Box<dyn Entity>) -> Result<(), RepoError>;
    fn remove(&self, id: Uuid) -> Result<(), RepoError>;
}

// Define the ReadWriteRepo trait combining read and write repositories.
pub trait ReadWriteRepo: ReadRepo + WriteRepo {}

// Define the Iter trait for iterating over entities.
pub trait Iter {
    fn next(&mut self) -> bool;
    fn value(&self) -> Option<&dyn Entity>;
    fn close(&mut self) -> Result<(), RepoError>;
}

// Error constants.
#[derive(Debug)]
pub struct RepoError {
    pub err: Option<Box<dyn Error>>,
    pub op: RepoOperation,
    pub entity_id: Option<Uuid>,
}

// Define the RepoOperation enum for operation names.
#[derive(Debug, Clone,PartialEq)]
pub enum RepoOperation {
    Find,
    FindAll,
    FindQuery,
    Save,
    Remove,
    Clear,
}

impl fmt::Display for RepoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut message = format!("repo: {:?}: ", self.op);
        if let Some(ref err) = self.err {
            message.push_str(&err.to_string());
        } else {
            message.push_str("unknown error");
        }
        if let Some(entity_id) = self.entity_id {
            message.push_str(&format!(" entity ID: {}", entity_id));
        }
        write!(f, "{}", message)
    }
}

impl Error for RepoError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.err.as_deref()
    }
}

// Implement a helper method to create a new RepoError.
impl RepoError {
    pub fn new(op: RepoOperation, err: Option<Box<dyn Error>>, entity_id: Option<Uuid>) -> Self {
        RepoError { err, op, entity_id }
    }
}



// Unit tests for repository functionalities.
#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    // Example implementation of an entity.
    #[derive(Debug, Clone)]
    pub struct MyEntity {
        pub id: Uuid,
        pub name: String,
    }

    impl Entity for MyEntity {
        fn id(&self) -> Uuid {
            self.id
        }
    }

    // Example implementation of a simple ReadRepo.
    pub struct SimpleReadRepo {
        entities: Vec<Box<dyn Entity>>,
    }

    impl SimpleReadRepo {
        pub fn new() -> Self {
            SimpleReadRepo { entities: Vec::new() }
        }

        pub fn add_entity(&mut self, entity: Box<dyn Entity>) {
            self.entities.push(entity);
        }
    }

    impl ReadRepo for SimpleReadRepo {
        fn inner_repo(&self) -> Option<Box<dyn ReadRepo>> {
            None
        }

        fn find(&self, id: Uuid) -> Result<Box<dyn Entity>, RepoError> {
            for entity in &self.entities {
                if entity.id() == id {
                    return Ok(entity.clone());
                }
            }
            Err(RepoError::new(RepoOperation::Find, None, Some(id)))
        }

        fn find_all(&self) -> Result<Vec<Box<dyn Entity>>, RepoError> {
            Ok(self.entities.clone())
        }

        fn close(&self) -> Result<(), RepoError> {
            Ok(())
        }
    }

    // Example of a ReadWriteRepo (combines both reading and writing).
    pub struct SimpleReadWriteRepo {
        pub read_repo: SimpleReadRepo,
    }

    impl ReadRepo for SimpleReadWriteRepo {
        fn inner_repo(&self) -> Option<Box<dyn ReadRepo>> {
            self.read_repo.inner_repo()
        }

        fn find(&self, id: Uuid) -> Result<Box<dyn Entity>, RepoError> {
            self.read_repo.find(id)
        }

        fn find_all(&self) -> Result<Vec<Box<dyn Entity>>, RepoError> {
            self.read_repo.find_all()
        }

        fn close(&self) -> Result<(), RepoError> {
            self.read_repo.close()
        }
    }

    impl WriteRepo for SimpleReadWriteRepo {
        fn save(&self, entity: Box<dyn Entity>) -> Result<(), RepoError> {
            // Logic to save entity.
            Ok(())
        }

        fn remove(&self, id: Uuid) -> Result<(), RepoError> {
            // Logic to remove entity.
            Ok(())
        }
    }

    #[test]
    fn test_find_entity() {
        let mut repo = SimpleReadRepo::new();
        let entity = Box::new(MyEntity {
            id: Uuid::new_v4(),
            name: "TestEntity".to_string(),
        });

        let entity_id = entity.id();
        repo.add_entity(entity);

        let found_entity = repo.find(entity_id).unwrap();
        assert_eq!(found_entity.id(), entity_id);
    }

    #[test]
    fn test_find_entity_not_found() {
        let repo = SimpleReadRepo::new();
        let entity_id = Uuid::new_v4();

        let result = repo.find(entity_id);
        assert!(result.is_err());
        if let Err(err) = result {
            assert_eq!(err.op, RepoOperation::Find);
            assert_eq!(err.entity_id, Some(entity_id));
        }
    }

    #[test]
    fn test_save_entity_in_read_write_repo() {
        let read_repo = SimpleReadRepo::new();
        let repo = SimpleReadWriteRepo { read_repo };

        let entity = Box::new(MyEntity {
            id: Uuid::new_v4(),
            name: "TestEntity".to_string(),
        });

        let result = repo.save(entity);
        assert!(result.is_ok());
    }
}