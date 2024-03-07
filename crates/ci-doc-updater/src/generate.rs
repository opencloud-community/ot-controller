// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::path::Path;

use snafu::{ResultExt, Whatever};

use crate::analyze::analyze;

pub trait FilesProvider {
    fn get_file_contents(&self, filename: &str) -> Result<String, Whatever>;
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
    fn get_file_contents(&self, filename: &str) -> Result<String, Whatever> {
        let file = self.base_dir.join(filename);
        std::fs::read_to_string(file)
            .with_whatever_context(|err| format!("Couldn't write file {}: {}", filename, err))
    }
}

pub fn generate(contents: &str, raw_files: &dyn FilesProvider) -> Result<String, Whatever> {
    let markers = analyze(contents)?;

    let mut lines = contents.lines();
    let mut output = String::new();

    let mut line = 0;

    for section in &markers.sections {
        let included_filename = section.begin.filename();

        println!("-- updating section {included_filename}",);
        let raw_file = raw_files.get_file_contents(included_filename)?;
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
            r"<!-- begin:fromfile:{included_filename} -->

{raw_file}

<!-- end:fromfile:{included_filename} -->",
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

<!-- begin:fromfile:abc -->
<!-- end:fromfile:abc -->

world
";

        let expected = r"
hello

<!-- begin:fromfile:abc -->

```text
foobar
```

<!-- end:fromfile:abc -->

world
";

        struct DummyFilesProvider;
        impl FilesProvider for DummyFilesProvider {
            fn get_file_contents(&self, _filename: &str) -> Result<String, Whatever> {
                Ok(r"```text
foobar
```"
                .to_string())
            }
        }

        let generated = generate(original, &DummyFilesProvider).unwrap();

        assert_eq!(generated.as_str(), expected);
    }

    #[test]
    fn generate_multiple() {
        let original = r"
hello

<!-- begin:fromfile:abc -->

this is abc

<!-- end:fromfile:abc -->

world
<!-- begin:fromfile:cba -->

this is cba

<!-- end:fromfile:cba -->

!
";

        let expected = r"
hello

<!-- begin:fromfile:abc -->

```text
foobar abc
```

<!-- end:fromfile:abc -->

world
<!-- begin:fromfile:cba -->

```text
foobar cba
```

<!-- end:fromfile:cba -->

!
";

        struct DummyFilesProvider;
        impl FilesProvider for DummyFilesProvider {
            fn get_file_contents(&self, filename: &str) -> Result<String, Whatever> {
                Ok(format!(
                    r"```text
foobar {filename}
```"
                ))
            }
        }

        let generated = generate(original, &DummyFilesProvider).unwrap();

        assert_eq!(generated.as_str(), expected);
    }
}
