use std::fmt::Display;

#[derive(Debug, Clone)]
pub(crate) struct CommandLine {
    lines: Vec<Line>,
}

#[derive(Debug, Clone)]
pub(crate) struct Line {
    words: Vec<String>,
}

impl CommandLine {
    pub fn new() -> Self {
        Self { lines: vec![] }
    }

    pub fn push_line(&mut self, line: Line) {
        self.lines.push(line);
    }

    pub fn iter_words(&self) -> impl Iterator<Item = &str> {
        self.lines.iter().map(|line| line.iter()).flatten()
    }
}

impl Display for CommandLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for line in &self.lines {
            writeln!(f, "{}", line)?;
        }
        Ok(())
    }
}

const INDENTATION_WIDTH: usize = 4;

impl Line {
    fn indent_word(word: impl AsRef<str>, indentation_depth: usize) -> String {
        let mut indented_word =
            String::with_capacity(INDENTATION_WIDTH * indentation_depth + word.as_ref().len());
        for _ in 0..(indentation_depth * INDENTATION_WIDTH) {
            indented_word.push(' ');
        }
        indented_word.push_str(word.as_ref());
        indented_word
    }

    pub fn from_words(words: &[impl AsRef<str>], indentation_depth: usize) -> Self {
        Self {
            words: words
                .iter()
                .next()
                .map(|first| Self::indent_word(first, indentation_depth))
                .into_iter()
                .chain(words.iter().skip(1).map(|w| w.as_ref().to_string()))
                .collect(),
        }
    }

    pub fn from_word(word: impl AsRef<str>, indentation_depth: usize) -> Self {
        Self {
            words: vec![Self::indent_word(word, indentation_depth)],
        }
    }

    pub fn iter(&self) -> LineIterator {
        LineIterator {
            line: self,
            index: 0,
        }
    }
}

impl Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.words.join(" "))
    }
}

pub(crate) struct LineIterator<'l> {
    line: &'l Line,
    index: usize,
}

impl<'l> Iterator for LineIterator<'l> {
    type Item = &'l str;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.line.words.get(self.index);
        self.index += 1;
        value.map(String::as_str)
    }
}
