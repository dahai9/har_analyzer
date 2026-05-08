# HAR Analyzer

Rust CLI tool that converts HAR (HTTP Archive) files into LLM-friendly structured output.

## Build & Run

```bash
just build          # debug build
just release        # release build
just overview       # run overview with sample HAR
just list --limit 5 # list requests
```

All `just` commands default to the sample HAR file (`StormSniffer-20260507-080821.har`).
Use `cargo run -- <file> <subcommand>` for arbitrary files.

## Architecture

- `src/har.rs` — HAR data structures. Parses `log.entries` as `Vec<serde_json::Value>` then deserializes each entry individually. Non-standard entries are skipped gracefully.
- `src/analyzer.rs` — Analysis: overview stats, filtering, search, domain grouping, body decoding (gzip/base64).
- `src/output.rs` — Markdown and JSON formatters for each subcommand.
- `src/main.rs` — CLI definition with clap. Subcommands: overview, list, detail, errors, search, headers, domains, timeline, slow.

## Key Design Decisions

- **Resilient parsing:** Real-world HAR files (StormSniffer, Charles, Fiddler) often have missing or non-standard fields. Each entry is parsed independently; failures are skipped with a count.
- **LLM-friendly output:** Default Markdown format is concise and structured. `--format json` for programmatic use.
- **Subcommand pattern:** Each subcommand outputs a focused view so LLMs can request exactly what they need without consuming the entire HAR.

## Conventions

- Language: Rust (edition 2024)
- CLI: clap derive
- Serialization: serde + serde_json
- Compression: flate2 (gzip), base64
- Command runner: just
- No external runtime dependencies (single static binary)
