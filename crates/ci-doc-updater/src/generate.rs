// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::path::Path;

use anyhow::Result;

use crate::analyze::analyze;

pub trait FilesProvider {
    fn get_file_contents(&self, filename: &str) -> Result<String>;
}

pub struct DirectoryFilesProvider<'a> {
    base_dir: &'a Path,
}

impl<'a> DirectoryFilesProvider<'a> {
    pub fn new(base_dir: &'a Path) -> Self {
        Self { base_dir }
    }
}

impl<'a> FilesProvider for DirectoryFilesProvider<'a> {
    fn get_file_contents(&self, filename: &str) -> Result<String> {
        let file = self.base_dir.join(filename);
        std::fs::read_to_string(file).map_err(From::from)
    }
}

pub fn generate(filename: &Path, contents: &str, raw_files: &dyn FilesProvider) -> Result<String> {
    let markers = analyze(contents)?;

    let mut lines = contents.lines();
    let mut output = String::new();

    let mut line = 0;

    for section in &markers.sections {
        let included_filename = section.begin.filename();
        let included_filetype = section.begin.filetype();

        println!("Updating file {included_filename} inside {filename:?}",);
        let raw_file = raw_files.get_file_contents(section.begin.filename())?;
        let raw_file = raw_file.trim();

        while line < section.begin.line() {
            output.push_str(lines.next().expect("too few lines found"));
            output.push('\n');
            line += 1;
        }

        while line <= section.end.line() {
            lines.next();
            line += 1;
        }

        // An empty line is required before and after the source code block to prevent
        // triggering "MD031 Fenced code blocks should be surrounded by blank lines" warnings
        // from markdownlint
        let new_section = format!(
            r"<!-- begin:fromfile:{included_filetype}:{included_filename} -->

```{included_filetype}
{raw_file}
```

<!-- end:fromfile:{included_filetype}:{included_filename} -->",
        );

        for new_line in new_section.lines() {
            output.push_str(new_line);
            output.push('\n');
        }
    }

    for line in lines {
        output.push_str(line);
        output.push('\n');
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn generate_single() {
        let original = r"
hello

<!-- begin:fromfile:text:abc -->
<!-- end:fromfile:text:abc -->

world
";

        let expected = r"
hello

<!-- begin:fromfile:text:abc -->

```text
foobar
```

<!-- end:fromfile:text:abc -->

world
";

        struct DummyFilesProvider;
        impl FilesProvider for DummyFilesProvider {
            fn get_file_contents(&self, _filename: &str) -> Result<String> {
                Ok("foobar".to_string())
            }
        }

        let generated = generate(Path::new("xyz"), original, &DummyFilesProvider).unwrap();

        assert_eq!(generated.as_str(), expected);
    }

    #[test]
    fn generate_multiple() {
        let original = r"
hello

<!-- begin:fromfile:text:abc -->

this is abc

<!-- end:fromfile:text:abc -->

world
<!-- begin:fromfile:text:cba -->

this is cba

<!-- end:fromfile:text:cba -->

!
";

        let expected = r"
hello

<!-- begin:fromfile:text:abc -->

```text
foobar abc
```

<!-- end:fromfile:text:abc -->

world
<!-- begin:fromfile:text:cba -->

```text
foobar cba
```

<!-- end:fromfile:text:cba -->

!
";

        struct DummyFilesProvider;
        impl FilesProvider for DummyFilesProvider {
            fn get_file_contents(&self, filename: &str) -> Result<String> {
                Ok(format!("foobar {filename}"))
            }
        }

        let generated = generate(Path::new("xyz"), original, &DummyFilesProvider).unwrap();

        assert_eq!(generated.as_str(), expected);
    }
}
