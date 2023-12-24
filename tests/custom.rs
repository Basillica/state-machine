
use std::error::Error;
use serde::{Deserialize, Serialize};
use state_machine::machine::
    {state::{StateMachine, State, ErrorBlock}, data::DeserializeStateData};

// Define the struct representing the shared data
#[derive(Debug, Serialize, Deserialize)]
struct SharedData {
  counter: i16,
  id: String,
}

// Implement the deserialization trait for SharedData
impl DeserializeStateData for SharedData {
  fn from_json(json: &str) -> Result<Self, Box<dyn Error>> {
    let data: Self = serde_json::from_str(json)?;
    Ok(data)
  }
}

#[test]
pub fn main() {
    // JSON representation of the shared data
    let json_data = r#"{"counter": 5, "id": "frigging-id"}"#;
    // Deserialize the shared data
    let shared_data: SharedData = SharedData::from_json(json_data).expect("Failed to deserialize");
  
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

    fn cond() -> bool {
        true
    }
  
    // Create a state machine
    let mut shared_data = SharedData { counter: shared_data.counter, id: shared_data.id };
    let mut state_machine = StateMachine::new("MachineA011".to_string(), &mut shared_data, 3);
   
    // Add nodes to the state machine
    let err = vec![ErrorBlock {
        error_equals: vec![String::from("STATE.FAILED")], next: state_function_a
    },];

    state_machine.step("Node0", State::Task, StateMachine::error, None, None, Some(vec!["STATE.FAILED"]), Some(false));
    state_machine.step("NodeA", State::Task, state_function_a, None, None, None, Some(false));
    state_machine.step("NodeE", State::Choice(cond), StateMachine::okay, None, None, None, None);
    state_machine.step("NodeB", State::Task, state_function_b, None, Some(err), None, None);
    state_machine.step("NodeC", State::Sleep(1), StateMachine::okay, None, None, None, None);
    state_machine.step("NodeD", State::Choice(cond), StateMachine::choice, None, None, None, None);
    state_machine.step("NodeF", State::Task, state_function_c, None, None, None, None);
    state_machine.step("NodeG", State::Task, state_function_d, None, None, None, None);

    // Validate node IDs
    state_machine.validate_node_ids();
    // execute  a step by its id
    let _ = state_machine.execute_by_id("NodeG");
    // Execute the state machine
    if let Err(err) = state_machine.execute() {
      println!("State machine execution failed: {}", err);
    }
  
    // Print the final state of the shared data after executing the functions
    println!("Final Shared Data: {:?}", shared_data);

  }