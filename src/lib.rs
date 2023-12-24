/*!
This crate provides a simple idiomatic implementation of a state machine in rust.
The state machine offered by this crate is in close mimic of the AWS state-machine definition
style.
The crate is still in development and more features will be published to it as they
become available.

A state machine is comprised of steps which can be one of the following states
```text
pub enum State {
    Task,
    Choice(fn() -> bool),
    Sleep(u64),
    Pass,
    Parallel,
    Succeed,
    Fail,
    Map,
    CustomState,
}
```

A simple example of the usage is given below:

```text
use std::{error::Error, fmt::Debug};
use serde::{Deserialize, Serialize};
use sfn_machine::machine::
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
    let mut sfn_machine = StateMachine::new("MachineA011".to_string(), &mut shared_data, 3);

    sfn_machine.step("NodeA", State::Task, state_function_a, None, None, None, None);
    sfn_machine.step("NodeB", State::Task, state_function_b, None, None, None, None);
    sfn_machine.step("NodeC", State::Task, state_function_c, None, None, None, None);
    // The end attribute can be set optionally. When set, the node becomes the last step in the state machine
    sfn_machine.step("NodeD", State::Task, state_function_d, None, None, None, Some(true));

    // Validate node IDs
    sfn_machine.validate_node_ids();

    // execute state machine
    if let Err(err) = sfn_machine.execute() {
      println!("State machine execution failed: {}", err);
    }
  }
```

# Overview
The implementation is implemented as a linked-list, meaning the executions will follow
their order of definition, requiring no additional work to execute in a given order.

There is also the option to define the order of execution using the `next` attribute of the step function.

```text
fn state_function_a(data: &mut SharedData) -> Result<(), Box<dyn Error>> {
  data.counter += 1;
  Ok(())
}
  
fn state_function_b(data: &mut SharedData) -> Result<(), Box<dyn Error>> {
  data.counter += 100;
  Ok(())
}

let mut shared_data = SharedData { counter: shared_data.counter, id: shared_data.id };
let mut sfn_machine = StateMachine::new("MachineA011".to_string(), &mut shared_data, 3);

sfn_machine.step("NodeA", State::Task, state_function_a, state_function_b, None, None, None);
sfn_machine.step("NodeB", State::Task, state_function_b, None, None, None, None);
```

Same is also true for defining the last step in the state machine.

One can also define a set of errors to catch or retry, with corresponding actions to be taken when they are matched
Example
```text
sfn_machine.step("Node0", State::Task, StateMachine::error, None, None, Some(vec!["STATE.FAILED"]), Some(false));
```
*/

#![deny(missing_docs)]
#![warn(missing_debug_implementations)]


/// The state machine module defines a process for procedurally orchestrating a set of tasks
/// 
/// It is a minimalistic implementation that utilizes a linked-list such that the tasks already
/// execute is a given fashion with little work needed to defined the steps
pub mod machine;