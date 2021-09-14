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
}

impl<W> Processor<W>
where
    W: Write,
{
    pub fn new(write: W) -> Self {
        Self { write }
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
            Record::Test(test::Event::Ok { name, exec_time }) => {
                writeln!(self.write, "## ✅ {}", name)?;
            }

            Record::Test(test::Event::Failed {
                name,
                exec_time,
                stdout,
            }) => {
                writeln!(self.write, "## ❌ {}", name)?;
                writeln!(self.write, "Duration: {:?}", exec_time)?;
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

            Record::Suite(suite::Event::Failed {
                passed,
                failed,
                ignored,
                filtered_out,
                ..
            }) => {
                writeln!(self.write, "# Summary")?;
                writeln!(self.write, "| Passed | Failed | Ignored | Filtered |")?;
                writeln!(self.write, "| --- | --- | --- | --- |")?;
                writeln!(
                    self.write,
                    "| {} | {} | {} | {} |",
                    passed, failed, ignored, filtered_out
                )?;
            }
            _ => {}
        }

        Ok(())
    }
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
