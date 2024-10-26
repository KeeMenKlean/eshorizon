use mongodb::bson::{self, Bson, Binary, doc, from_bson, to_bson, spec::BinarySubtype};
use serde::{Deserialize, Serialize, Serializer, Deserializer};
use uuid::Uuid;
use std::fmt;

// Custom serializer for UUIDWrapper
#[derive(Debug, Clone, PartialEq)]
pub struct UUIDWrapper {
    pub id: Uuid,
}

impl Serialize for UUIDWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let binary = Bson::Binary(Binary {
            subtype: BinarySubtype::Uuid,
            bytes: self.id.as_bytes().to_vec(),
        });
        binary.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for UUIDWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bson = Bson::deserialize(deserializer)?;
        if let Bson::Binary(binary) = bson {
            Uuid::from_slice(&binary.bytes)
                .map(|id| UUIDWrapper { id })
                .map_err(serde::de::Error::custom)
        } else {
            Err(serde::de::Error::custom("Expected a binary BSON UUID"))
        }
    }
}

// BSON encoder: Convert UUID to BSON Binary.
pub fn encode_uuid_to_bson(uuid: Uuid) -> Bson {
    Bson::Binary(Binary {
        subtype: BinarySubtype::Uuid,
        bytes: uuid.as_bytes().to_vec(),
    })
}

// BSON decoder: Parse UUID from BSON Binary.
pub fn decode_uuid_from_bson(bson: Bson) -> Result<Uuid, fmt::Error> {
    if let Bson::Binary(binary) = bson {
        Uuid::from_slice(&binary.bytes).map_err(|_| fmt::Error)
    } else {
        Err(fmt::Error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mongodb::bson::{Bson, Binary, spec::BinarySubtype};

    #[test]
    fn test_encode_uuid_to_bson() {
        let uuid = Uuid::new_v4();
        let encoded_bson = encode_uuid_to_bson(uuid);

        // Ensure the BSON is a Binary containing the UUID
        if let Bson::Binary(binary) = encoded_bson {
            assert_eq!(binary.subtype, BinarySubtype::Uuid);
            assert_eq!(binary.bytes, uuid.as_bytes());
        } else {
            panic!("Expected Bson::Binary but got {:?}", encoded_bson);
        }
    }

    #[test]
    fn test_decode_uuid_from_bson() {
        let uuid = Uuid::new_v4();
        let encoded_bson = Bson::Binary(Binary {
            subtype: BinarySubtype::Uuid,
            bytes: uuid.as_bytes().to_vec(),
        });

        // Decode the BSON back to a UUID
        let decoded_uuid = decode_uuid_from_bson(encoded_bson).unwrap();

        // Ensure the decoded UUID matches the original
        assert_eq!(decoded_uuid, uuid);
    }

    #[test]
    fn test_decode_uuid_from_invalid_bson() {
        // Use a BSON value that is not a binary
        let invalid_bson = Bson::Int32(123);

        // Attempt to decode it, expecting an error
        let result = decode_uuid_from_bson(invalid_bson);

        assert!(result.is_err(), "Expected an error when decoding invalid BSON");
    }

    #[test]
    fn test_serialize_uuid_wrapper() {
        let uuid = Uuid::new_v4();
        let wrapper = UUIDWrapper { id: uuid };

        // Serialize the UUIDWrapper to BSON
        let bson = to_bson(&wrapper).unwrap();

        // Check if the resulting BSON contains the correct UUID as Binary
        if let Bson::Binary(binary) = bson {
            assert_eq!(binary.subtype, BinarySubtype::Uuid);
            assert_eq!(binary.bytes, uuid.as_bytes());
        } else {
            panic!("Expected a binary BSON UUID");
        }
    }

    #[test]
    fn test_deserialize_uuid_wrapper() {
        let uuid = Uuid::new_v4();
        let bson = Bson::Binary(Binary {
            subtype: BinarySubtype::Uuid,
            bytes: uuid.as_bytes().to_vec(),
        });

        // Deserialize the BSON into a UUIDWrapper struct
        let wrapper: UUIDWrapper = from_bson(bson).unwrap();

        // Ensure the deserialized UUID matches the original
        assert_eq!(wrapper.id, uuid);
    }
}