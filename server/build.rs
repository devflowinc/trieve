use std::env;
use std::error::Error;

#[cfg(feature = "require-env")]
fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().expect("Failed to read .env file. Did you `cp .env.dist .env` ?");

    for (key, value) in env::vars() {
        println!("cargo:rustc-env={key}={value}");
    }

    println!("cargo:rerun-if-changed=.env");

    Ok(())
}

#[cfg(not(feature = "require-env"))]
fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::from_filename(".env.dist").expect("Failed to read from .env.dist file");

    for (key, value) in env::vars() {
        println!("cargo:rustc-env={key}={value}");
    }

    println!("cargo:rerun-if-changed=.env.dist");

    Ok(())

}
