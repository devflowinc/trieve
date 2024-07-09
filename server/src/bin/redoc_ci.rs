use trieve_server::ApiDoc;
use utoipa::OpenApi;

#[allow(clippy::print_stdout)]
fn main() -> std::io::Result<()> {
    println!("{}", ApiDoc::openapi().to_pretty_json().unwrap());
    Ok(())
}
