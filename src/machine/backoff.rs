use std::thread;
use std::time::Duration;


macro_rules! ifelse {
    ($test:expr => $true_expr:expr; $false_expr:expr) => {
        if $test {
            $true_expr
        }
        else {
            $false_expr
        }
    }
}

/// Exponential backoff implementation
pub fn exponential_backoff<F, E, T>(mut operation: F, data: &mut T, retries: Option<i32>) -> Result<(), E>
where
    F: FnMut(&mut T) -> Result<(), E>,
{
    let mut _retries = 0;
    let mut max_retries = 5;
    let mut delay = Duration::from_secs(1);
    ifelse!(retries.unwrap() > max_retries => println!("Provided number of retries can not be more than 5"); max_retries = retries.unwrap());

    while _retries < max_retries {
        match operation(data) {
            Ok(_) => return Ok(()), // Operation successful, exit early
            Err(_) => {
                println!("Operation failed, retrying ...");
                thread::sleep(delay);
                _retries += 1;
                delay *= 2; // Exponential backoff
            }
        }
    }

    Err(operation(data).err().unwrap_or_else(|| panic!("the operation could not be completed due to an unrecoverable error")))
}