mod analyzer;
mod har;
mod output;

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "har_analyzer", about = "LLM-friendly HAR file analyzer")]
struct Cli {
    /// HAR file path
    file: PathBuf,

    /// Output format
    #[arg(short, long, default_value = "markdown", global = true)]
    format: OutputFormat,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Markdown,
    Json,
}

#[derive(Subcommand)]
enum Commands {
    /// Global overview: request count, domains, status codes, methods
    Overview,
    /// List all requests in a compact table
    List {
        /// Filter by domain (substring match)
        #[arg(short, long)]
        domain: Option<String>,
        /// Filter by HTTP method
        #[arg(short, long)]
        method: Option<String>,
        /// Filter by status code
        #[arg(short, long)]
        status: Option<u16>,
        /// Max entries to show
        #[arg(short, long)]
        limit: Option<usize>,
    },
    /// Show details of a single request by ID
    Detail {
        /// Request ID (0-based index from list)
        id: usize,
    },
    /// Show only error responses (status >= 400)
    Errors {
        /// Max entries to show
        #[arg(short, long)]
        limit: Option<usize>,
    },
    /// Search requests by keyword in URL, headers, or body
    Search {
        /// Search keyword
        keyword: String,
        /// Max entries to show
        #[arg(short, long)]
        limit: Option<usize>,
    },
    /// Per-domain statistics
    Domains,
    /// Requests in chronological order
    Timeline {
        /// Max entries to show
        #[arg(short, long)]
        limit: Option<usize>,
    },
}

fn main() {
    let cli = Cli::parse();
    let (har, entries) = match analyzer::load_har(&cli.file) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };
    let entries = &entries;

    let out = match &cli.command {
        Commands::Overview => {
            let mut ov = analyzer::analyze_overview(entries);
            ov.creator = format!("{} {}", har.log.creator.name, har.log.creator.version);
            match cli.format {
                OutputFormat::Markdown => output::overview_md(&ov, &ov.creator.clone()),
                OutputFormat::Json => output::overview_json(&ov, &ov.creator.clone()),
            }
        }
        Commands::List {
            domain,
            method,
            status,
            limit,
        } => {
            let filtered = analyzer::filter_entries(
                entries,
                domain.as_deref(),
                method.as_deref(),
                *status,
            );
            let slice = match limit {
                Some(n) => &filtered[..filtered.len().min(*n)],
                None => &filtered,
            };
            match cli.format {
                OutputFormat::Markdown => output::list_md(slice),
                OutputFormat::Json => output::list_json(slice),
            }
        }
        Commands::Detail { id } => {
            if *id >= entries.len() {
                eprintln!("Error: ID {id} out of range (0..{})", entries.len() - 1);
                std::process::exit(1);
            }
            match cli.format {
                OutputFormat::Markdown => output::detail_md(*id, &entries[*id]),
                OutputFormat::Json => output::detail_json(*id, &entries[*id]),
            }
        }
        Commands::Errors { limit } => {
            let filtered = analyzer::filter_errors(entries);
            let slice = match limit {
                Some(n) => &filtered[..filtered.len().min(*n)],
                None => &filtered,
            };
            match cli.format {
                OutputFormat::Markdown => output::list_md(slice),
                OutputFormat::Json => output::list_json(slice),
            }
        }
        Commands::Search { keyword, limit } => {
            let filtered = analyzer::search_entries(entries, keyword);
            let slice = match limit {
                Some(n) => &filtered[..filtered.len().min(*n)],
                None => &filtered,
            };
            match cli.format {
                OutputFormat::Markdown => output::list_md(slice),
                OutputFormat::Json => output::list_json(slice),
            }
        }
        Commands::Domains => {
            let stats = analyzer::analyze_domains(entries);
            match cli.format {
                OutputFormat::Markdown => output::domains_md(&stats),
                OutputFormat::Json => output::domains_json(&stats),
            }
        }
        Commands::Timeline { limit } => {
            let mut sorted: Vec<(usize, &crate::har::Entry)> =
                entries.iter().enumerate().collect();
            sorted.sort_by(|a, b| a.1.started_date_time.cmp(&b.1.started_date_time));
            let slice = match limit {
                Some(n) => &sorted[..sorted.len().min(*n)],
                None => &sorted,
            };
            match cli.format {
                OutputFormat::Markdown => output::timeline_md(slice),
                OutputFormat::Json => output::timeline_json(slice),
            }
        }
    };

    print!("{out}");
}
