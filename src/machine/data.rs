use std::error::Error;


/// Define the trait for deserialization
pub trait DeserializeStateData: Sized {
    /// convert to json
    fn from_json(json: &str) -> Result<Self, Box<dyn Error>>;
}