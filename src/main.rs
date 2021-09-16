mod event;

use crate::event::{suite, test, Record};
use chrono::Utc;
use clap::{App, Arg};
use log::LevelFilter;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::time::Duration;

struct Processor<W>
where
    W: Write,
{
    write: W,
    tests: Vec<test::Event>,
    test_count: Option<u64>,
    summary: Option<Summary>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Outcome {
    Ok,
    Failed,
}

impl Display for Outcome {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ok => f.write_str("✅"),
            Self::Failed => f.write_str("❌"),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Summary {
    outcome: Outcome,
    passed: u64,
    failed: u64,
    ignored: u64,
    filtered_out: u64,
    exec_time: Duration,
}

impl<W> Processor<W>
where
    W: Write,
{
    pub fn new(write: W) -> Self {
        Self {
            write,
            tests: Vec::new(),
            test_count: None,
            summary: None,
        }
    }

    fn write_header(&mut self, summary: &Summary) -> anyhow::Result<()> {
        let run_id = std::env::var("GITHUB_RUN_ID").ok();
        let repo = std::env::var("GITHUB_REPOSITORY").ok();

        let link = match (&repo, &run_id) {
            (Some(repo), Some(id)) => Some(format!(
                "https://github.com/{repo}/actions/runs/{id}",
                repo = repo,
                id = id
            )),
            _ => None,
        };

        let title = format!(
            "{} Test Result {}",
            summary.outcome,
            run_id.unwrap_or_else(|| "????".to_string())
        );

        writeln!(self.write, "---")?;
        writeln!(self.write, "title: \"{}\"", title)?;
        writeln!(self.write, "date: {}", Utc::now().to_rfc3339())?;
        writeln!(self.write, "categories: test-report")?;
        writeln!(self.write, "---")?;
        writeln!(self.write)?;

        let total = self
            .test_count
            .map(|total| total.to_string())
            .unwrap_or_else(|| "*unknown*".into());

        writeln!(
            self.write,
            r#"
| | Total | Passed | Failed | Ignored | Filtered | Duration |
| --- | ----- | -------| ------ | ------- | -------- | -------- |
| {} | {} | {} | {} | {} | {} | {:?} |
"#,
            summary.outcome,
            total,
            summary.passed,
            summary.failed,
            summary.ignored,
            summary.filtered_out,
            summary.exec_time
        )?;
        writeln!(self.write)?;

        if let Some(link) = link {
            writeln!(self.write, "**Job:** {}", link)?;
            writeln!(self.write)?;
        }

        Ok(())
    }

    pub fn line(&mut self, line: &str) -> anyhow::Result<()> {
        match serde_json::from_str(line) {
            Ok(record) => self.record(record)?,
            Err(err) => log::debug!("Ignoring line: {:?} -> {}", err, line),
        }

        Ok(())
    }

    fn record(&mut self, record: Record) -> anyhow::Result<()> {
        log::debug!("Record: {:?}", record);

        match record {
            Record::Test(test) => {
                self.tests.push(test);
            }

            Record::Suite(suite::Event::Started { test_count }) => {
                self.test_count = Some(test_count);
            }

            Record::Suite(suite::Event::Ok {
                passed,
                failed,
                ignored,
                filtered_out,
                exec_time,
                ..
            }) => {
                self.summary = Some(Summary {
                    outcome: Outcome::Ok,
                    passed,
                    failed,
                    ignored,
                    filtered_out,
                    exec_time,
                });
            }
            Record::Suite(suite::Event::Failed {
                passed,
                failed,
                ignored,
                filtered_out,
                exec_time,
                ..
            }) => {
                self.summary = Some(Summary {
                    outcome: Outcome::Failed,
                    passed,
                    failed,
                    ignored,
                    filtered_out,
                    exec_time,
                });
            }
        }

        Ok(())
    }

    fn make_name(&self, name: &str, outcome: &str) -> String {
        format!(
            "[{}](#{})",
            name,
            make_anchor(&self.make_heading(name, outcome))
        )
    }

    fn make_heading(&self, name: &str, outcome: &str) -> String {
        format!("{} {}", outcome, name)
    }

    fn render_index(&mut self) -> anyhow::Result<()> {
        writeln!(self.write)?;
        writeln!(self.write, "# Index")?;
        writeln!(self.write)?;
        writeln!(self.write, "| Name | Result | Duration |")?;
        writeln!(self.write, "| ---- | ------ | -------- |")?;

        for test in &self.tests {
            match test {
                test::Event::Started { .. } => {}
                test::Event::Ok { name, exec_time } => {
                    writeln!(
                        self.write,
                        "| {} | ✅ | {:?} | ",
                        self.make_name(&name, "✅"),
                        exec_time
                    )?;
                }

                test::Event::Failed {
                    name, exec_time, ..
                } => {
                    writeln!(
                        self.write,
                        "| {} | ❌ | {:?} | ",
                        self.make_name(&name, "❌"),
                        exec_time
                    )?;
                }
            }
        }

        Ok(())
    }

    fn render_details(&mut self) -> anyhow::Result<()> {
        writeln!(self.write)?;
        writeln!(self.write)?;
        writeln!(self.write, "# Details")?;

        for test in &self.tests {
            match test {
                test::Event::Started { .. } => {}
                test::Event::Ok { name, exec_time } => {
                    writeln!(self.write)?;
                    writeln!(self.write, "## {}", self.make_heading(name, "✅"))?;
                    writeln!(self.write)?;
                    writeln!(self.write, "**Duration**: {:?}", exec_time)?;
                }

                test::Event::Failed {
                    name,
                    exec_time,
                    stdout,
                } => {
                    writeln!(self.write)?;
                    writeln!(self.write, "## {}", self.make_heading(name, "❌"))?;
                    writeln!(self.write)?;
                    writeln!(self.write, "**Duration**: {:?}", exec_time)?;
                    if !stdout.is_empty() {
                        writeln!(self.write)?;
                        writeln!(self.write, "<details>")?;
                        writeln!(self.write)?;

                        writeln!(self.write, "<summary>Test output</summary>")?;
                        writeln!(self.write)?;

                        writeln!(self.write, "<pre>")?;
                        writeln!(self.write, "{}", stdout)?;
                        writeln!(self.write, "</pre>")?;

                        writeln!(self.write)?;
                        writeln!(self.write, "</details>")?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl<W> Drop for Processor<W>
where
    W: Write,
{
    fn drop(&mut self) {
        if let Some(summary) = self.summary.clone() {
            self.write_header(&summary).expect("Render header");
        }
        self.render_index().expect("Render index");
        self.render_details().expect("Render details");
    }
}

fn make_anchor(link: &str) -> String {
    let mut s = String::with_capacity(link.len());
    let mut was_dash = false;
    for c in link.chars() {
        if c == '_' {
            s.push(c);
            was_dash = false;
        } else if c == ' ' || c == '-' {
            // using c.is_whitespace() doesn't work, as markdown parsers
            // then to check for "space" and not for "is it a whitespace like character"
            if !was_dash {
                was_dash = true;
                s.push('-');
            }
        } else if c.is_alphanumeric() {
            s.push(c);
            was_dash = false;
        }
    }
    s
}

fn main() -> anyhow::Result<()> {
    let matches = App::new("Markdown Test Reporter")
        .author("Jens Reimann <ctron@dentrassi.de>")
        .arg(
            Arg::with_name("INPUT")
                .help("The filename of the JSON data. This file must may contain additional (non-JSON) lines, which will be ignored during processing.")
                .index(1)
                .default_value("test-output.json"),
        )
        .arg(Arg::with_name("output")
            .help("The name of the output file.")
            .short("o")
        )
        .get_matches();

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

    let mut processor = Processor::new(writer);

    for line in reader.lines() {
        processor.line(&line?)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anchors() {
        assert_eq!(make_anchor(""), "");
        assert_eq!(
            make_anchor("✅ tests::registry::test_registry_create_and_delete"),
            "-testsregistrytest_registry_create_and_delete"
        );
        assert_eq!(make_anchor("foo  bar"), "foo-bar");
    }
}
