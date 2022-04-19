mod event;
mod git;
mod processor;

use crate::processor::{ProcessOptions, Processor};
use crate::{git::GitInfo, processor::Addon};
use clap::{App, Arg};
use log::LevelFilter;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use std::io::Write;
use std::ops::Deref;
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter},
    path::Path,
};

fn main() -> anyhow::Result<()> {
    let matches = App::new("Markdown Test Reporter")
        .author("Jens Reimann <ctron@dentrassi.de>")
        .arg(
            Arg::with_name("INPUT")
                .help("The filename of the JSON data. This file must may contain additional (non-JSON) lines, which will be ignored during processing")
                .index(1)
                .default_value("test-output.json"),
        )
        .arg(Arg::with_name("output")
            .help("The name of the output file")
            .short("o")
            .long("output")
            .takes_value(true)
        )
        .arg(Arg::with_name("no-front-matter")
            .long("no-front-matter")
            .help("Disable front matter generation")
        )
        .arg(Arg::with_name("git")
            .long("git")
            .help("Add information from the Git repository in the provided location")
            .default_value(".")
            .takes_value(true)
        )
        .arg (Arg::with_name("summary")
            .long("summary")
            .help ("Show only the summary section")
        )
        .arg(Arg::with_name("quiet")
            .long("quiet")
            .short("q")
            .help("Be quiet")
        )
        .arg(Arg::with_name("verbose")
            .long("verbose")
            .short("v")
            .help("Be more verbose. May be repeated multiple times.")
            .multiple(true)
            .conflicts_with("quiet")
        )
        .arg(Arg::with_name("no-git")
            .long("no-git")
            .help("Disable Git information extraction")
            .conflicts_with("git"))
        .get_matches();

    let disable_front_matter = matches.is_present("no-front-matter");
    let input = matches.value_of("INPUT").unwrap_or("test-output.json");
    let output = matches
        .value_of("output")
        .map(ToString::to_string)
        .unwrap_or_else(|| {
            if let Some(name) = input.strip_suffix(".json") {
                name.to_string() + ".md"
            } else {
                input.to_string() + ".md"
            }
        });

    let mut addons = Vec::<Box<dyn Addon>>::new();

    if !matches.is_present("no-git") {
        if let Some(git_path) = matches.value_of("git") {
            let required = matches.is_present("git");
            addons.push(Box::new(GitInfo::new(Path::new(&git_path), required)))
        }
    }

    let log_level = match (
        matches.is_present("quiet"),
        matches.occurrences_of("verbose"),
    ) {
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

    log::debug!("Reading from: {}", input);
    log::debug!("Writing to: {}", output);

    let input = File::open(input)?;
    let reader = BufReader::new(input);

    let output: Box<dyn Write> = match output.deref() {
        "-" => Box::new(std::io::stdout()),
        output => Box::new(File::create(output)?),
    };
    let writer = BufWriter::new(output);

    {
        let mut processor = Processor::new(
            writer,
            ProcessOptions {
                disable_front_matter,
                addons,
                summary: matches.is_present("summary"),
            },
        );

        for line in reader.lines() {
            processor.line(&line?)?;
        }
    }

    Ok(())
}
