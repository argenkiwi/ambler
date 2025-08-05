mod ambler;
mod lead;

use std::io::{self, Write};
use std::thread;
use std::time::Duration;
use ambler::amble;
use lead::Lead;

async fn start(state: i32) -> (i32, Option<Lead>) {
    print!("Enter a starting number (or press Enter for default): ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim();

    if input.is_empty() {
        println!("Using default starting number.");
        (state, Some(Lead::Count))
    } else {
        match input.parse::<i32>() {
            Ok(number) => (number, Some(Lead::Count)),
            Err(_) => {
                println!("Invalid number, please try again.");
                (state, Some(Lead::Start))
            }
        }
    }
}

async fn count(state: i32) -> (i32, Option<Lead>) {
    println!("Count: {}", state);
    thread::sleep(Duration::from_secs(1));
    let new_state = state + 1;
    if rand::random::<f64>() > 0.5 {
        (new_state, Some(Lead::Count))
    } else {
        (new_state, Some(Lead::Stop))
    }
}

async fn stop(state: i32) -> (i32, Option<Lead>) {
    println!("Stopping count at {}.", state);
    (state, None)
}

#[tokio::main]
async fn main() {
    let initial_state = 0;
    let initial_lead = Lead::Start;

    amble(initial_state, initial_lead, |lead, state| async move {
        match lead {
            Lead::Start => start(state).await,
            Lead::Count => count(state).await,
            Lead::Stop => stop(state).await,
        }
    }).await;
}
