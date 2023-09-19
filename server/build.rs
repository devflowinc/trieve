use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().expect("Failed to read .env file. Did you `cp .env.dist .env` ?");

    for (key, value) in env::vars() {
        println!("cargo:rustc-env={key}={value}");
    }

    println!("cargo:rerun-if-changed=.env");

    Ok(())
}
