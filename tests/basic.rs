
use std::{error::Error, fmt::Debug};
use serde::{Deserialize, Serialize};
use state_machine::machine::
    {state::{StateMachine, State}, data::DeserializeStateData};

// Define the struct representing the shared data
#[derive(Debug, Serialize, Deserialize)]
struct SharedData {
  counter: i16,
  id: String,
}

// Implement the deserialization trait for the SharedData struct
impl DeserializeStateData for SharedData {
  fn from_json(json: &str) -> Result<Self, Box<dyn Error>> {
    let data: Self = serde_json::from_str(json)?;
    Ok(data)
  }
}

fn match_vecs<T: PartialEq + std::fmt::Debug>(a: &Vec<T>, b: &Vec<T>) -> bool {
    let mut matching = true;
    for index in 0..a.len() {
        if !b.contains(&a[index]) {
            matching = false;
            break
        }
    };

    matching
}

#[test]
pub fn main() {
    // JSON representation of the shared data
    let json_data = r#"{"counter": 5, "id": "come-id"}"#;
    // Deserialize the shared data
    let shared_data: SharedData = SharedData::from_json(json_data).expect("Failed to deserialize data");
  
    // Define state functions
    fn state_function_a(data: &mut SharedData) -> Result<(), Box<dyn Error>> {
      data.counter += 1;
      Ok(())
    }
  
    fn state_function_b(data: &mut SharedData) -> Result<(), Box<dyn Error>> {
      data.counter += 100;
      Ok(())
    }
  
    fn state_function_c(data: &mut SharedData) -> Result<(), Box<dyn Error>> {
      data.counter *= 1;
      Ok(())
    }

    fn state_function_d(data: &mut SharedData) -> Result<(), Box<dyn Error>> {
        data.counter *= 5;
        Ok(())
    }
  
    // Create a state machine
    let mut shared_data = SharedData { counter: shared_data.counter, id: shared_data.id };
    let mut state_machine = StateMachine::new("MachineA011".to_string(), &mut shared_data, 3);

    state_machine.step("NodeA", State::Task, state_function_a, None, None, None, None);
    state_machine.step("NodeB", State::Task, state_function_b, None, None, None, None);
    state_machine.step("NodeC", State::Task, state_function_c, None, None, None, None);
    // The end attribute can be set optionally. When set, the node becomes the last step in the state machine
    state_machine.step("NodeD", State::Task, state_function_d, None, None, None, Some(true));

    let ids = state_machine.get_node_ids();
    let set = vec!["NodeA", "NodeB", "NodeC", "NodeD"];

    assert_eq!(match_vecs(&ids, &set), true);

    // Validate node IDs
    state_machine.validate_node_ids();

    if let Err(err) = state_machine.execute() {
      println!("State machine execution failed: {}", err);
    }
  }