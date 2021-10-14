mod event;
mod git;
mod processor;

use crate::processor::{ProcessOptions, Processor};
use crate::{git::GitInfo, processor::Addon};
use clap::{App, Arg};
use log::LevelFilter;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
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

    let mut addons = Vec::<Box<dyn Addon<BufWriter<File>>>>::new();

    if !matches.is_present("no-git") {
        if let Some(git_path) = matches.value_of("git") {
            let required = matches.is_present("git");
            addons.push(Box::new(GitInfo::new(Path::new(&git_path), required)))
        }
    }

    TermLogger::init(
        LevelFilter::Debug,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )?;

    log::debug!("Reading from: {}", input);
    log::debug!("Writing to: {}", output);

    let input = File::open(input)?;
    let reader = BufReader::new(input);

    let output = File::create(output)?;
    let writer = BufWriter::new(output);

    let mut processor = Processor::new(
        writer,
        ProcessOptions {
            disable_front_matter,
            addons,
        },
    );

    for line in reader.lines() {
        processor.line(&line?)?;
    }

    Ok(())
}
