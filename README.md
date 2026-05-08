# HAR Analyzer

LLM-friendly HAR file analyzer. Converts large HAR JSON into concise Markdown/JSON for LLM consumption.

## Install

### Download release binary

```bash
# Linux x86_64
curl -sL https://github.com/<owner>/har_analyzer/releases/latest/download/har_analyzer-linux-amd64 -o har_analyzer && chmod +x har_analyzer

# macOS arm64
curl -sL https://github.com/<owner>/har_analyzer/releases/latest/download/har_analyzer-macos-arm64 -o har_analyzer && chmod +x har_analyzer
```

### Build from source

```bash
git clone <repo_url> && cd har_analyzer
cargo build --release
# binary: target/release/har_analyzer
```

### cargo install

```bash
cargo install --git <repo_url>
```

## Usage

```bash
har_analyzer <file.har> <command> [options]
```

### Commands

```
overview              Global summary: domains, methods, status codes, size
list [--domain D]     Request table. Filters: --method, --status, --type, --limit
detail <id>           Full request/response: headers, body, timings
errors [--limit N]    Error responses (status >= 400)
search <keyword>      Search URL, headers, body
headers <name>        Find requests with a specific header
domains               Per-domain stats: count, avg time, errors, size
timeline [--limit N]  Chronological request order
slow [--limit N]      Slowest requests by duration
```

Add `--format json` to any command for JSON output.

### Examples

```bash
har_analyzer capture.har overview
har_analyzer capture.har list --domain api.example.com --type json
har_analyzer capture.har detail 42
har_analyzer capture.har headers Authorization
har_analyzer capture.har slow --limit 5
har_analyzer capture.har search "login"
```

## License

MIT
