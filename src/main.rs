mod event;

use crate::event::{suite, test, Record};
use log::LevelFilter;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

struct Processor<W>
where
    W: Write,
{
    write: W,
    tests: Vec<test::Event>,
}

impl<W> Processor<W>
where
    W: Write,
{
    pub fn new(write: W) -> Self {
        Self {
            write,
            tests: Vec::new(),
        }
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

            Record::Suite(suite::Event::Failed {
                passed,
                failed,
                ignored,
                filtered_out,
                ..
            }) => {
                writeln!(self.write, "# Summary")?;
                writeln!(self.write)?;
                writeln!(self.write, "| Passed | Failed | Ignored | Filtered |")?;
                writeln!(self.write, "| ------ | ------ | ------- | -------- |")?;
                writeln!(
                    self.write,
                    "| {} | {} | {} | {} |",
                    passed, failed, ignored, filtered_out
                )?;
                writeln!(self.write)?;
            }
            _ => {}
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

        writeln!(self.write)?;

        Ok(())
    }

    fn render_details(&mut self) -> anyhow::Result<()> {
        writeln!(self.write, "# Details")?;
        writeln!(self.write)?;

        for test in &self.tests {
            match test {
                test::Event::Started { .. } => {}
                test::Event::Ok { name, exec_time } => {
                    writeln!(self.write, "## {}", self.make_heading(name, "✅"))?;
                    writeln!(self.write, "**Duration**: {:?}", exec_time)?;
                }

                test::Event::Failed {
                    name,
                    exec_time,
                    stdout,
                } => {
                    writeln!(self.write, "## {}", self.make_heading(name, "❌"))?;
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
                        writeln!(self.write)?;
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
        } else if c.is_whitespace() || c == '-' {
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
    TermLogger::init(
        LevelFilter::Debug,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )?;

    let input = File::open("test-output.json")?;
    let reader = BufReader::new(input);

    let output = File::create("test-output.md")?;
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
