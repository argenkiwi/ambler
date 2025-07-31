use std::io::{self, Write};
use std::thread;
use std::time::Duration;
use ambler::{amble, Next, BoxFuture};

fn start(state: i32) -> BoxFuture<'static, Option<Next<'static>>> {
    Box::pin(async move {
        print!("Enter a starting number (or press Enter for default): ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.is_empty() {
            println!("Using default starting number.");
            Some(Next::new(move || count(state)))
        } else {
            match input.parse::<i32>() {
                Ok(number) => Some(Next::new(move || count(number))),
                Err(_) => {
                    println!("Invalid number, please try again.");
                    Some(Next::new(move || start(state)))
                }
            }
        }
    })
}

fn count(state: i32) -> BoxFuture<'static, Option<Next<'static>>> {
    Box::pin(async move {
        println!("Count: {}", state);
        thread::sleep(Duration::from_secs(1));
        let new_state = state + 1;
        if rand::random::<f64>() > 0.5 {
            Some(Next::new(move || count(new_state)))
        } else {
            Some(Next::new(move || stop(new_state)))
        }
    })
}

fn stop(state: i32) -> BoxFuture<'static, Option<Next<'static>>> {
    Box::pin(async move {
        println!("Stopping count at {}.", state);
        None
    })
}

#[tokio::main]
async fn main() {
    amble(Some(Next::new(|| start(0)))).await;
}
