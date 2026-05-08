---
name: har-analyze
description: Analyze HAR (HTTP Archive) files to understand API flows, debug network issues, and reverse-engineer app logic. TRIGGER when: user mentions HAR files, network traffic analysis, API reverse engineering, or wants to understand how an app communicates with its backend.
---

# HAR Analysis

Analyze HAR files using the `har_analyzer` CLI tool.

### install

```bash
cargo install --git https://github.com/dahai9/har_analyzer
```

## Commands

Given a HAR file, use subcommands to extract focused views:

```bash
# Global overview — domains, methods, status codes, size, time range
har_analyzer <file> overview

# Request list — compact table, supports filters
har_analyzer <file> list [--domain <d>] [--method GET|POST] [--status <code>] [--type json|image] [--limit N]

# Single request detail — headers, body, timings, cookies
har_analyzer <file> detail <id>

# Errors only — status >= 400
har_analyzer <file> errors [--limit N]

# Search — across URL, headers, response/request body
har_analyzer <file> search <keyword> [--limit N]

# Headers — find requests containing a specific header name
har_analyzer <file> headers <name> [--limit N]

# Domain stats — per-domain request count, avg time, errors, size
har_analyzer <file> domains

# Timeline — chronological request order
har_analyzer <file> timeline [--limit N]

# Slow requests — sorted by duration descending
har_analyzer <file> slow [--limit N]
```

All commands default to Markdown output. Add `--format json` for JSON.

## Analysis Workflow

1. `overview` — understand the landscape
2. `list --domain <target>` — focus on the target API
3. `timeline` — see request order
4. `detail <id>` — examine key requests (auth, business logic)
5. `search <keyword>` / `headers <name>` — find specific patterns
