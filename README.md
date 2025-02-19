# Git Repository Concatenator

Converts Git repositories into a single Markdown document for AI analysis. Generates a structured overview with syntax-highlighted code content, which you can feed into AI models for full context analysis.

## Installation

clone this repository and build the project with Cargo:
```bash
git clone [repository-url]
```

```bash
cargo build --release
```

## Usage

Local repository:
```bash
cargo run -- /path/to/local/repo
```

Remote repository:
```bash
# HTTPS
cargo run -- https://gitlab.com/username/repo.git
cargo run -- https://github.com/username/repo.git
cargo run -- https://bitbucket.org/username/repo.git

# SSH
cargo run -- git@gitlab.com:username/repo.git
cargo run -- git@github.com:username/repo.git
cargo run -- git@bitbucket.org:username/repo.git

# Generic Git URL
cargo run -- ssh://git@your-git-server:port/repo.git
```

Output will be saved to `./output/[repository-name].md`

## Features

- Works with any Git repository (GitHub, GitLab, Bitbucket, self-hosted, etc.)
- Generates JSON repository structure
- Includes all text-based files with syntax highlighting
- Skips binary files and build artifacts
- UTF-8 encoding support

## License

[MIT License](LICENSE)
