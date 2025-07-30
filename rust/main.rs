use std::io::{self, Write};
use ambler::{amble_from, Next, BoxFuture};

fn prompt_for_number() -> i32 {
    loop {
        print!("Enter a starting number: ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        match input.trim().parse::<i32>() {
            Ok(n) => return n,
            Err(_) => println!("Invalid number, please try again."),
        }
    }
}

fn prompt_number_node(_state: i32) -> BoxFuture<'static, Option<Next<'static>>> {
    Box::pin(async move {
        let number = prompt_for_number();
        Some(Next::new(move || start_node(number)))
    })
}

fn start_node(state: i32) -> BoxFuture<'static, Option<Next<'static>>> {
    Box::pin(async move {
        println!("Starting count from {}", state);
        Some(Next::new(move || step_node(state)))
    })
}

fn step_node(state: i32) -> BoxFuture<'static, Option<Next<'static>>> {
    Box::pin(async move {
        let new_state = state + 1;
        println!("Count: {}", new_state);
        if rand::random::<f64>() > 0.5 {
            Some(Next::new(move || step_node(new_state)))
        } else {
            Some(Next::new(move || stop_node(new_state)))
        }
    })
}

fn stop_node(_state: i32) -> BoxFuture<'static, Option<Next<'static>>> {
    Box::pin(async move {
        println!("Stopping count.");
        None
    })
}

#[tokio::main]
async fn main() {
    amble_from(|| prompt_number_node(0)).await;
}
