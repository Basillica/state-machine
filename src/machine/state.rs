use std::collections::HashSet;
use std::error::Error;
use std::{thread, time::Duration};
use crate::machine::{error, backoff};
use crate::machine::data;
use log::{error, info, LevelFilter};
use env_logger::Builder;
use std::env;


/// A logger method
pub fn init_logger() {
    // Check if the RUST_LOG environment variable is set
    if let Ok(log_var) = env::var("RUST_LOG") {
        // If set, use the specified log level
        Builder::from_env(log_var.as_str())
            .target(env_logger::Target::Stdout)
            .init();
    } else {
        // Default to info level if RUST_LOG is not set
        Builder::new()
            .filter_level(LevelFilter::Info)
            .target(env_logger::Target::Stdout)
            .init();
    }
}


/// The states of the state machine
/// 
/// They define the possible states that a step in the state machine could be in
#[derive(Debug)]
pub enum State {
    /// The task state is the state a regular step would be in, a step that performs
    /// an operation
    Task,
    /// choice state is only executed if it the condition is true
    Choice(fn() -> bool),
    /// sleep state does nothing but put the main thread to sleep for a while
    Sleep(u64),
    /// pass state does absolutely nothing
    Pass,
    /// parallel state would perform a set of instructions in parallel
    Parallel,
    /// succeed state defines a successful execution of the state machine.
    /// It is also the end of the execution and terminates the state machine.
    Succeed,
    /// fail state defines when the execution has failed.
    /// It terminates the state machine and exist the program.
    /// The error can be retried depending on its error type
    Fail,
    /// map state executes an operation on a a given map
    Map,
    /// custom state
    CustomState,
}

// Define the function signature for the state nodes
type StateFunction<T> = fn(&mut T) -> Result<(), Box<dyn Error>>;


/// error block
#[derive(Debug)]
pub struct ErrorBlock<T: data::DeserializeStateData>  {
    /// error strings
    pub error_equals: Vec<String>,
    /// next method
    pub next: StateFunction<T>,
}

/// Define the data structure for each element in the linked list
#[derive(Debug)]
pub struct StateNode<'a, T: data::DeserializeStateData> {
    id: String,
    state: State,
    state_function: StateFunction<T>,
    next: Option<StateFunction<T>>,
    catch: Option<Vec<ErrorBlock<T>>>,
    retry: Option<Vec<&'a str>>,
    invocation_count: i8,
    end: Option<bool>
}

impl<'a, T: data::DeserializeStateData> StateNode<'a, T> {
    fn new(id: &str, state: State, state_function: StateFunction<T>, next: Option<StateFunction<T>>, catch: Option<Vec<ErrorBlock<T>>>, retry: Option<Vec<&'a str>>, end: Option<bool>) -> Self {
        StateNode {
        id: id.to_string(),
        state,
        state_function,
        invocation_count: 0,
        catch,
        retry,
        next,
        end,
        }
    }

    fn execute(&mut self, data: &mut T) -> Result<(), Box<dyn Error>> {
        // Perform actions specific to each state if needed
        match self.state {
            State::Task => {
                // Execute the assigned function for the state
                match (self.state_function)(data) {
                    Ok(_) => (),
                    Err(e) => {
                        return Err(e);
                    },
                };
            }
            State::Choice(func) => {
                if func() {
                    // Execute the assigned function for the state
                    match (self.state_function)(data) {
                        Ok(_) => (),
                        Err(e) => {
                            return Err(e);
                        },
                    };
                }
            }
            State::Sleep(v) => {
                thread::sleep(Duration::from_secs(v));
            }
            State::Pass => {}
            State::Parallel => {}
            State::Succeed => {}
            State::Fail => {}
            State::Map => {}
            State::CustomState => {}
        }
        Ok(())
    }
}

/// Define the StateMachine struct
#[derive(Debug)]
pub struct StateMachine<'a, T: data::DeserializeStateData> {
    id: String,
    nodes: Vec<StateNode<'a, T>>,
    node_ids: HashSet<String>,
    retries: i32,
    shared_data: &'a mut T,
    error_string: Option<String>
}

impl<'a, T: data::DeserializeStateData> StateMachine<'a, T> {
    /// Initialize the state machine with an empty list of nodes and an empty set of node IDs
    pub fn new(id: String, shared_data: &'a mut T, retries: i32) -> Self {
        info!("Executing state machine: {} ........", id);
        StateMachine {
            id,
            nodes: Vec::new(),
            node_ids: HashSet::new(),
            retries,
            shared_data,
            error_string: None,
        }
    }

    /// Add a new node to the state machine
    pub fn step(&mut self, id: &str, state: State, state_function: StateFunction<T>, next: Option<StateFunction<T>>, catch: Option<Vec<ErrorBlock<T>>>, retry: Option<Vec<&'a str>>, end: Option<bool>) {
        // Check for duplicate node IDs
        if !self.node_ids.insert(id.to_string()) {
        panic!("Duplicate node ID found: {}", id);
        }

        // Create and add the new node
        let new_node = StateNode::new(id, state, state_function, next, catch, retry, end);
        self.nodes.push(new_node);
    }

    /// Validate the uniqueness of node IDs
    pub fn validate_node_ids(&self) {
        if self.nodes.len() != self.node_ids.len() {
            panic!("Duplicate node IDs found in the state machine");
        }
    }

    /// get node ids
    pub fn get_node_ids(&self) -> Vec<&str> {
        let v: Vec<&str> = self.node_ids.iter().map(|v| v.as_str()).collect();
        v
    }

    /// execute by id
    pub fn execute_by_id(&mut self, node_id: &str) -> Result<(), error::StateMachineError> {
        for node in &mut self.nodes {
            if node.id == node_id {
                if let Err(err) = node.execute(self.shared_data) {
                    error!("Error: {}", err);
                    return Err(error::StateMachineError {
                        message: err.to_string(),
                    });
                }
                break
            }
        }
        Ok(())
    }

    /// okay step
    pub fn okay(_: &mut T) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    /// pass step
    pub fn pass(_: &mut T) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    /// choice step
    pub fn choice(_: &mut T) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    /// error step
    pub fn error(_: &mut T) -> Result<(), Box<dyn Error>> {
        Err(Box::new(error::StateMachineError {
            message: String::from("STATE.FAILED"),
        }))
    }

    /// Execute the state machine and handle errors
    pub fn execute(&mut self) -> Result<(), error::StateMachineError> {
        for node in &mut self.nodes {
            // break if the last node/step
            if node.end.is_some() && node.end.unwrap() {
                break
            }
            // check for invocations more than three times
            if node.invocation_count == 2 {
                let error = format!("state machine {} failed for step {}. Step have been invoked upto three times", self.id, node.id);
                return Err(error::StateMachineError {
                    message: error,
                });
            }

            // if there is an error in the state and the current node is to catch some errors
            if self.error_string.is_some() && !node.catch.is_some() {
                return Err(error::StateMachineError {
                    message: format!("{:?}", self.error_string),
                });
            }
            
            if self.error_string.is_some() && node.catch.is_some() {
                let catch = node.catch.as_ref().unwrap();
                for val in catch.iter() {
                    if  val.error_equals.contains(&self.error_string.as_ref().unwrap()) {
                        match (val.next)(self.shared_data) {
                            Ok(_) => (),
                            Err(e) => {
                                self.error_string = Some(e.to_string());
                                return Err(error::StateMachineError {
                                    message: format!("{:?}", self.error_string),
                                });
                            },
                        };
                    }
                }
            }

            if node.next.is_some() {
                match Some(node.next) {
                    Some(v) => {
                        let fffn = v.unwrap();
                        match fffn(self.shared_data) {
                            Ok(_) => (),
                            Err(e) => {
                                self.error_string = Some(e.to_string());
                                return Err(error::StateMachineError {
                                    message: format!("{:?}", self.error_string),
                                })
                            }
                        };
                    },
                    None => (),
                }
            }


            if let Err(err) = node.execute(self.shared_data) {
                // Propagate errors when they occur, and the current node becomes the exit
                if node.retry.is_some() {
                    // if  node.retry.as_ref().unwrap().contains(&self.error_string.as_ref().unwrap().as_str()) {
                    if  node.retry.as_ref().unwrap().contains(&err.to_string().as_str()) {
                        match backoff::exponential_backoff(|x| node.execute(x), self.shared_data, Some(self.retries)) {
                            Ok(_) => println!("Operation completed successfully"),
                            Err(_) => println!("Operation failed for step {} after multiple retries", node.id),
                        };
                    }
                }

                return Err(error::StateMachineError {
                    message: err.to_string(),
                });
            }

            // break if the last node/step
            if node.end.is_some() && node.end.unwrap() {
                break
            }
        }

        Ok(())
    }
}