use trieve_server::ApiDoc;
use utoipa::OpenApi;

fn main() -> std::io::Result<()> {
    println!("{}", ApiDoc::openapi().to_pretty_json().unwrap());
    Ok(())
}
