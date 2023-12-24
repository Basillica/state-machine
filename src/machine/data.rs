use std::error::Error;


/// The shared data between the steps of the state machine implements this trait.
/// 
/// The trail has as of yet of single function to serialize the struct to json
pub trait DeserializeStateData: Sized {
    /// A method within the trait to deserialize json from a string
    fn from_json(json: &str) -> Result<Self, Box<dyn Error>>;
}