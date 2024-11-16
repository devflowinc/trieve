use std::error::Error;

#[cfg(not(feature = "runtime-env"))]
fn main() -> Result<(), Box<dyn Error>> {
    use std::{env, process::Command};
    dotenvy::dotenv().expect("Failed to read .env file. Did you `cp .env.dist .env` ?");

    let output = Command::new("npx")
        .arg("tailwindcss")
        .arg("-i")
        .arg("./static/in.css")
        .arg("-o")
        .arg("./static/output.css")
        .output()?;

    // Stream output
    println!("{}", String::from_utf8_lossy(&output.stdout));

    for (key, value) in env::vars() {
        println!("cargo:rustc-env={key}={value}");
    }

    println!("cargo:rerun-if-changed=.env");

    minijinja_embed::embed_templates!("src/templates");
    Ok(())
}

#[cfg(feature = "runtime-env")]
fn main() -> Result<(), Box<dyn Error>> {
    minijinja_embed::embed_templates!("src/templates");
    Ok(())
}
