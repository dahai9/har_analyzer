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

## LLM Agent Skill Setup

This repo ships a reusable skill file that teaches LLM coding agents how to use `har_analyzer` for HAR analysis. All three tools use the same `SKILL.md` format (YAML frontmatter + markdown), just in different directories.

### Claude Code

Skill is already in place at `.claude/skills/har-analyze/SKILL.md`. Auto-discovered, no setup needed.

**Global install (for all projects):**

```bash
mkdir -p ~/.claude/skills/har-analyze
cp .claude/skills/har-analyze/SKILL.md ~/.claude/skills/har-analyze/SKILL.md
```

### OpenAI Codex CLI

Codex scans `.agents/skills/` for skill directories.

**Project-level:**

```bash
mkdir -p .agents/skills/har-analyze
cp .claude/skills/har-analyze/SKILL.md .agents/skills/har-analyze/SKILL.md
```

**Global install:**

```bash
mkdir -p ~/.agents/skills/har-analyze
cp .claude/skills/har-analyze/SKILL.md ~/.agents/skills/har-analyze/SKILL.md
```

### Google Gemini CLI

Gemini scans `.gemini/skills/` (also accepts `.agents/skills/` as an alias).

**Project-level:**

```bash
mkdir -p .gemini/skills/har-analyze
cp .claude/skills/har-analyze/SKILL.md .gemini/skills/har-analyze/SKILL.md
```

**Global install:**

```bash
mkdir -p ~/.gemini/skills/har-analyze
cp .claude/skills/har-analyze/SKILL.md ~/.gemini/skills/har-analyze/SKILL.md
```

### Shared across all tools

Since both Codex and Gemini recognize `.agents/skills/`, you can use a single location for both:

```bash
mkdir -p .agents/skills/har-analyze
cp .claude/skills/har-analyze/SKILL.md .agents/skills/har-analyze/SKILL.md

# Claude Code: symlink to its expected path
mkdir -p .claude/skills
ln -s ../../.agents/skills/har-analyze .claude/skills/har-analyze
```

| Tool | Skill directory | Install command |
|---|---|---|
| Claude Code | `.claude/skills/` | Already in repo |
| Codex CLI | `.agents/skills/` | `mkdir -p .agents/skills/har-analyze && cp ...` |
| Gemini CLI | `.gemini/skills/` or `.agents/skills/` | `mkdir -p .gemini/skills/har-analyze && cp ...` |

## License

MIT
