// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::str::FromStr;

use anyhow::{anyhow, bail, ensure, Context, Result};
use once_cell::sync::Lazy;
use regex::{Captures, Regex};

static RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^\s*<!-- (?<marker>(begin|end)):fromfile:(?<filetype>[-_.a-zA-Z]+):(?<filename>[-_./a-zA-Z]+) -->$")
        .unwrap()
});

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarkerKind {
    Begin,
    End,
}

impl FromStr for MarkerKind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "begin" => Ok(Self::Begin),
            "end" => Ok(Self::End),
            s => bail!("Unknown marker kind: {s}"),
        }
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct MarkerLine<'a> {
    kind: MarkerKind,
    line: usize,
    filename: &'a str,
    filetype: &'a str,
}

#[allow(dead_code)]
impl<'a> MarkerLine<'a> {
    pub fn kind(&self) -> MarkerKind {
        self.kind
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn filename(&self) -> &'a str {
        self.filename
    }

    pub fn filetype(&self) -> &'a str {
        self.filetype
    }
}

impl<'a> MarkerLine<'a> {
    fn try_from_captures(captures: &Captures<'a>, line: usize) -> Result<Self> {
        let filename = captures
            .name("filename")
            .context("filename not found")?
            .as_str();

        let filetype = captures
            .name("filetype")
            .context("filetype not found")?
            .as_str();

        let kind = captures
            .name("marker")
            .context("marker not found")?
            .as_str();
        let kind: MarkerKind = kind.parse()?;

        Ok(MarkerLine {
            kind,
            line,
            filename,
            filetype,
        })
    }
}

pub struct MarkedSection<'a> {
    pub begin: MarkerLine<'a>,
    pub end: MarkerLine<'a>,
}

impl<'a> MarkedSection<'a> {
    fn retrieve_from_slice(lines: &[MarkerLine<'a>]) -> Result<MarkedSection<'a>> {
        let begin = lines.first().context("No more marker lines left")?.clone();

        ensure!(
            begin.kind() == MarkerKind::Begin,
            "Found end marker line for file {:?} without matching begin",
            begin.filename
        );

        let end = lines
            .get(1)
            .context(anyhow!("Missing end marker for file {:?}", begin.filename))?
            .clone();

        ensure!(
            end.kind() == MarkerKind::End,
            "Found begin marker for file {:?} instead of end marker for file {:?}",
            end.filename(),
            begin.filename()
        );

        ensure!(
            begin.filename() == end.filename(),
            "Found begin marker for filename {:?}, but end marker for filename {:?}",
            begin.filename(),
            end.filename(),
        );

        Ok(MarkedSection { begin, end })
    }
}

pub struct FileMarkers<'a> {
    pub sections: Vec<MarkedSection<'a>>,
}

pub fn analyze(text: &str) -> Result<FileMarkers<'_>> {
    let marker_lines = text
        .lines()
        .enumerate()
        .filter_map(|(line, contents)| {
            RE.captures(contents)
                .map(|captures| MarkerLine::try_from_captures(&captures, line))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let sections = marker_lines
        .chunks(2)
        .map(MarkedSection::retrieve_from_slice)
        .collect::<Result<Vec<_>>>()?;

    Ok(FileMarkers { sections })
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn check_regex_line(line: &str, marker: &str, filename: &str) {
        assert!(RE.is_match(line));

        let captures = RE.captures(line).unwrap();
        assert_eq!(captures.name("marker").unwrap().as_str(), marker);
        assert_eq!(captures.name("filename").unwrap().as_str(), filename);
    }

    #[test]
    fn regex_line() {
        check_regex_line(
            "<!-- begin:fromfile:text:opentalk-controller-help -->",
            "begin",
            "opentalk-controller-help",
        );

        check_regex_line(
            "<!-- end:fromfile:text:opentalk_controller_help -->",
            "end",
            "opentalk_controller_help",
        );
    }

    #[test]
    fn regex_text() {
        let text: &str = r"blah
blub
<!-- begin:fromfile:text:opentalk-controller-help -->
```text
hello

world
```
 <!-- end:fromfile:text:opentalk-controller-help -->

The end.";
        println!("{}", text);

        let captures = RE.captures_iter(text).collect::<Vec<_>>();

        assert_eq!(captures.len(), 2);

        {
            let c = captures.first().unwrap();

            let marker = c.name("marker").unwrap().as_str();
            let filename = c.name("filename").unwrap().as_str();

            assert_eq!(marker, "begin");
            assert_eq!(filename, "opentalk-controller-help");
        }

        {
            let c = captures.get(1).unwrap();

            let marker = c.name("marker").unwrap().as_str();
            let filename = c.name("filename").unwrap().as_str();

            assert_eq!(marker, "end");
            assert_eq!(filename, "opentalk-controller-help");
        }
    }
}
