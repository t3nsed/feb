# feb: Git Commit Analyzer

A Rust-based tool that analyzes Git commit histories and rates each committer's coding style based on performance and maintainability metrics using the Deepseek AI API.

Just a fun project to re-learn Rust and use Deepseek-r1, but afaik for something like this it doesn't really matter which model you use.

## Features

- Analyzes both local and public Git repositories
- Generates performance and maintainability scores for each committer
- Processes entire commit history
- Provides detailed per-committer statistics

## Prerequisites

- Rust (latest stable version)
- Git
- Deepseek API key

## Installation

1. Clone this repository:
```bash
git clone https://github.com/t3nsed/feb.git
cd feb
```

2. Build the project:
```bash
cargo build --release
```

The compiled binary will be available at `target/release/feb`

## Configuration

Set your Deepseek API key using one of these methods:

1. Environment variable:
```bash
export DEEPSEEK_API_KEY=your_api_key_here
```

2. Command-line argument:
```bash
feb /path/to/repo --api-key your_api_key_here
```

Note: Command-line argument takes precedence over environment variable.

## Usage

Basic usage:
```bash
feb /path/to/repository
```

The program will:
1. Analyze each commit in the repository's history
2. Extract code changes and commit messages
3. Send the data to Deepseek API for analysis
4. Calculate and display average scores for each committer

## Output Format

The program outputs statistics for each committer in the following format:
```
Committer: John Doe <john@example.com>
  Performance Score: 8.45
  Maintainability Score: 7.92

Committer: Jane Smith <jane@example.com>
  Performance Score: 9.12
  Maintainability Score: 8.78
```

Scores are on a scale of 0-10, where:
- Performance Score: Indicates code efficiency and optimization
- Maintainability Score: Indicates code readability and maintainability

## License

MIT