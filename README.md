# Markdown Test Reports

Converts `cargo test` results from JSON to Markdown.

## Install

Released versions:

    cargo install markdown-test-report

From Git:

    cargo install --git https://github.com/ctron/markdown-test-report

## Usage

```
Markdown generator for cargo test JSON files

Usage: markdown-test-report [OPTIONS] [INPUT]

Arguments:
  [INPUT]  The filename of the JSON test data. Unnecessary or unparsable lines will be ignored [default: test-output.json]

Options:
  -o, --output <OUTPUT>  The name of the output file
  -d, --no-front-matter  Disable report metadata
  -g, --git <GIT>        git top-level location [default: .]
  -s, --summary          Show only the summary section
  -q, --quiet            Be quiet
  -v, --verbose...       Be more verbose. May be repeated multiple times
  -n, --no-git           Disable extracting git information
  -h, --help             Print help information
  -V, --version          Print version information
```

## JSON output for `cargo test`

This tool requires the test data output in the JSON format. This can be achieved by running `cargo test` with additional options:

```shell
cargo test -- -Z unstable-options --report-time --format json
```

Currently, the JSON format option is unstable. Still it does work anyway with stable Rust and didn't change much so far.

Also, might the `cargo test` command output additional, non-JSON, messages, mixed into the JSON output. The markdown
reporter will simply filter out those lines.

## Examples

Used by:

  * https://drogue-iot.github.io/drogue-cloud-testing/

![Example Screenshot](docs/example1.png)
