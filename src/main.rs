// #![deny(missing_docs)]
mod event;
mod git;
mod processor;

use crate::processor::{ProcessOptions, Processor};
use crate::{git::GitInfo, processor::Addon};
use clap::Parser;
use log::LevelFilter;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use std::io::Write;
use std::ops::Deref;
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter},
    path::Path,
};

#[derive(Debug, Parser)]
#[command(name = "Markdown Test Reporter", version, about, author, long_about = None)]
struct Cli {
    /// The filename of the JSON test data. Unnecessary or unparsable lines will be ignored
    #[arg(value_parser, default_value = "test-output.json")]
    input: String,
    /// The name of the output file
    #[arg(short, long, value_parser)]
    output: Option<String>,
    /// Disable report metadata
    #[arg(short='d', long, action = clap::ArgAction::SetTrue)]
    no_front_matter: bool,
    /// git top-level location [default: .]
    #[arg(short, long, value_parser)]
    git: Option<String>,
    /// Show only the summary section
    #[arg(short, long, action)]
    summary: bool,
    /// Be quiet
    #[arg(short, long, conflicts_with = "verbose")]
    quiet: bool,
    /// Be more verbose. May be repeated multiple times
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    /// Disable extracting git information
    #[arg(short, long, action = clap::ArgAction::SetTrue, conflicts_with = "git")]
    no_git: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Parse filepaths
    let input_path = Path::new(&cli.input);

    let file_stem = input_path
        .file_stem()
        .ok_or_else(|| anyhow::anyhow!("unable to parse input filename"))
        .unwrap()
        .to_str()
        .unwrap();

    let output_file = match cli.output {
        Some(o) => o,
        None => String::from(file_stem) + ".md",
    };

    let mut addons = Vec::<Box<dyn Addon>>::new();

    if !cli.no_git {
        let required = cli.git.is_some();
        addons.push(Box::new(GitInfo::new(
            Path::new(&cli.git.as_deref().unwrap_or(".")),
            required,
        )));
    }

    let log_level = match (cli.quiet, cli.verbose) {
        (true, _) => LevelFilter::Off,
        (_, 0) => LevelFilter::Warn,
        (_, 1) => LevelFilter::Info,
        (_, 2) => LevelFilter::Debug,
        (_, _) => LevelFilter::Trace,
    };

    TermLogger::init(
        log_level,
        Config::default(),
        TerminalMode::Stderr,
        ColorChoice::Auto,
    )?;

    log::debug!("input_path: {}", input_path.display());
    log::debug!("file_stem: {}", file_stem);

    log::debug!("Reading from: {}", input_path.display());
    log::debug!("Writing to: {}", output_file);

    let input = File::open(input_path)?;
    let reader = BufReader::new(input);

    let output: Box<dyn Write> = match output_file.deref() {
        "-" => Box::new(std::io::stdout()),
        output => Box::new(File::create(output)?),
    };
    let writer = BufWriter::new(output);

    {
        let mut processor = Processor::new(
            writer,
            ProcessOptions {
                disable_front_matter: cli.no_front_matter,
                addons,
                summary: cli.summary,
            },
        );

        for line in reader.lines() {
            processor.line(&line?)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert()
    }

    #[test]
    fn test_git_not_present() {
        let cli: Cli = Parser::parse_from(vec!["markdown-test-report"]);
        assert_eq!(cli.git.as_deref(), None);
    }

    #[test]
    fn test_git_present_with_default() {
        let cli: Cli = Parser::parse_from(vec!["markdown-test-report", "--git", "."]);
        assert_eq!(cli.git.as_deref(), Some("."));
    }

    #[test]
    fn test_git_present_with_other() {
        let cli: Cli = Parser::parse_from(vec!["markdown-test-report", "--git", "foo"]);
        assert_eq!(cli.git.as_deref(), Some("foo"));
    }
}
