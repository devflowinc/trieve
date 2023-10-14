use std::error::Error;

#[cfg(not(feature = "runtime-env"))]
fn main() -> Result<(), Box<dyn Error>> {
    use std::env;
    dotenvy::dotenv().expect("Failed to read .env file. Did you `cp .env.dist .env` ?");

    for (key, value) in env::vars() {
        println!("cargo:rustc-env={key}={value}");
    }

    println!("cargo:rerun-if-changed=.env");

    Ok(())
}

#[cfg(feature = "runtime-env")]
fn main() -> Result<(), Box<dyn Error>> {
    Ok(())
}
