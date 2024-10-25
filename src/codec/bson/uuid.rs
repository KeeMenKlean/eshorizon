use mongodb::bson::{doc, to_bson, Bson};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

// Custom BSON serializer and deserializer for UUIDs, encoding them as strings.
#[derive(Debug, Serialize, Deserialize)]
#[serde_with::serde_as]
pub struct UUIDWrapper {
    #[serde_as(as = "uuid_as_binary::Uuid")]
    pub id: Uuid,
}

// BSON encoder: Convert UUID to BSON string.
pub fn encode_uuid_to_bson(uuid: Uuid) -> Bson {
    to_bson(&uuid.to_string()).unwrap()
}

// BSON decoder: Parse UUID from BSON string.
pub fn decode_uuid_from_bson(bson: Bson) -> Result<Uuid, fmt::Error> {
    if let Bson::String(s) = bson {
        Uuid::parse_str(&s).map_err(|_| fmt::Error)
    } else {
        Err(fmt::Error)
    }
}

// fn main() {
//     // Example usage of BSON encoding and decoding for UUIDs
//     let my_uuid = Uuid::new_v4();
//
//     // Encoding to BSON
//     let encoded_bson = encode_uuid_to_bson(my_uuid);
//     println!("Encoded BSON: {:?}", encoded_bson);
//
//     // Decoding from BSON
//     let decoded_uuid = decode_uuid_from_bson(encoded_bson).unwrap();
//     println!("Decoded UUID: {}", decoded_uuid);
// }