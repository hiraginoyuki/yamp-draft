use std::{env, process};
use yamp::Args;

#[tokio::main]
async fn main() {
    let args = Args::parse(env::args()).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    if let Err(err) = yamp::run(args).await {
        println!("Problem while running: {}", err);
        process::exit(1);
    };
}
