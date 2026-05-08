# Default HAR file for testing
sample := "StormSniffer-20260507-080821.har"

# List available commands
default:
    @just --list

# Build debug
build:
    cargo build

# Build release
release:
    cargo build --release

# Run with sample HAR
run *ARGS:
    cargo run -- {{sample}} {{ARGS}}

# Global overview
overview fmt="markdown":
    cargo run -- {{sample}} --format {{fmt}} overview

# List requests
list *ARGS:
    cargo run -- {{sample}} list {{ARGS}}

# Request detail by ID
detail id:
    cargo run -- {{sample}} detail {{id}}

# Error requests only
errors *ARGS:
    cargo run -- {{sample}} errors {{ARGS}}

# Search by keyword
search keyword *ARGS:
    cargo run -- {{sample}} search {{keyword}} {{ARGS}}

# Domain statistics
domains fmt="markdown":
    cargo run -- {{sample}} --format {{fmt}} domains

# Timeline view
timeline *ARGS:
    cargo run -- {{sample}} timeline {{ARGS}}

# Clean build artifacts
clean:
    cargo clean
