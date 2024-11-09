use clap::{Args, Parser};
use operators::{create_task::create_task, poll_task::poll_task};

pub mod operators;

#[derive(Parser)]
#[command(author, version)]
#[command(
    name = "tr-chunk",
    about = "Trieve File Chunker CLI - CLI for Trieve File Chunker",
    long_about = "Trieve File Chunker CLI is a CLI for the Trieve File Chunker. 
    
    It allows you to interact with the Trieve File Chunker from the command line by creating and polling tasks."
)]
#[command(arg_required_else_help(true))]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// The base URL of the Trieve File Chunker server
    #[arg(
        short,
        long,
        env = "TRIEVE_FILE_CHUNKER_BASE_URL",
        default_value = "http://localhost:8081"
    )]
    base_url: String,

    /// The API key to use for authentication
    #[arg(
        short,
        long,
        env = "TRIEVE_FILE_CHUNKER_API_KEY",
        default_value = "admin"
    )]
    api_key: String,
}

#[derive(Parser)]
enum Commands {
    #[command(name = "create", about = "Create a new chunking task")]
    Create(Create),

    #[command(name = "poll", about = "Poll a chunking task")]
    Poll(Poll),
}

#[derive(Args)]
struct Create {
    /// The path to the file to chunk
    #[arg(short, long)]
    file: String,
}

#[derive(Args)]
struct Poll {
    /// The task ID to poll
    #[arg(short, long)]
    task_id: String,
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Some(Commands::Create(create)) => {
            create_task(&create.file, &args.base_url, &args.api_key);
        }
        Some(Commands::Poll(poll)) => {
            poll_task(&poll.task_id, &args.base_url, &args.api_key);
        }
        None => {
            println!("No command provided");
        }
    }
}
