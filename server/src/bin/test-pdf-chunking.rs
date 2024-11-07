use std::env;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let args: Vec<String> = env::args().collect();
    // Get filename
    let filename = &args[1];
    println!("filename: {}", filename);

    // Make it a pathbuf
    let filename = std::path::PathBuf::from(filename);

    let results = trieve_server::operators::pdf_chunk_operator::chunk_pdf(filename).await;

    println!("results: {:?}", results);
}
