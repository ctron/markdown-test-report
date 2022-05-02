# Markdown Test Reports

Converts `cargo test` results from JSON to Markdown.

## Install

Released versions:

    cargo install markdown-test-report

From Git:

    cargo install --git https://github.com/ctron/markdown-test-report

## Usage

~~~
Markdown Test Reporter 
Jens Reimann <ctron@dentrassi.de>

USAGE:
    markdown-test-report [FLAGS] [OPTIONS] [INPUT]

FLAGS:
    -h, --help               Prints help information
        --no-front-matter    Disable front matter generation.
    -V, --version            Prints version information

OPTIONS:
        --git <git>          Add information from Git
    -o, --output <output>    The name of the output file.

ARGS:
    <INPUT>    The filename of the JSON data. This file must may contain additional (non-JSON) lines, which will be
               ignored during processing. [default: test-output.json]
~~~

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
