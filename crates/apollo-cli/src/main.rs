//! Apollo CLI - Music library manager

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "apollo")]
#[command(author, version, about = "A modern music library manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new library
    Init {
        /// Path to the library database
        #[arg(short, long)]
        path: Option<String>,
    },
    /// Import music files
    Import {
        /// Directory to import from
        path: String,
    },
    /// List items in the library
    List {
        /// Filter by type (tracks, albums, artists)
        #[arg(short, long, default_value = "tracks")]
        type_: String,
    },
    /// Search the library
    Query {
        /// Search query
        query: String,
    },
    /// Start the web server
    Web {
        /// Host to bind to
        #[arg(short, long, default_value = "127.0.0.1")]
        host: String,
        /// Port to listen on
        #[arg(short, long, default_value = "8337")]
        port: u16,
    },
}

fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path } => {
            println!("Initializing library at {path:?}");
            // TODO: Implement
        }
        Commands::Import { path } => {
            println!("Importing from {path}");
            // TODO: Implement
        }
        Commands::List { type_ } => {
            println!("Listing {type_}");
            // TODO: Implement
        }
        Commands::Query { query } => {
            println!("Searching for: {query}");
            // TODO: Implement
        }
        Commands::Web { host, port } => {
            println!("Starting web server at {host}:{port}");
            // TODO: Implement
        }
    }
}
