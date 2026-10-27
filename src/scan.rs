use crate::token::{Op, Span, Spanned, Token};

#[derive(Debug)]
struct Scanner<'src> {
    source: &'src str,
    offset: usize,
}

impl<'src> Scanner<'src> {
    pub fn new(source: &'src str) -> Self {
        Self { source, offset: 0 }
    }
}

impl<'src> Iterator for Scanner<'src> {
    type Item = Spanned<&'src str>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let base = self.offset;
            let mut chars = self.source[base..].char_indices();

            // Skip any leading whitespace
            let Some((relative_start, candidate)) = chars.find(|(_, ch)| !ch.is_whitespace())
            else {
                self.offset = self.source.len();
                return None;
            };

            if candidate == '#' {
                // Skip to the next line
                let Some((offset, _)) = chars.find(|&(_, ch)| ch == '\n' || ch == '\r') else {
                    self.offset = self.source.len();
                    return None;
                };
                // Span.end is exclusive
                self.offset = base + offset + 1;
                continue;
            }

            let start = base + relative_start;

            // Find the next whitespace, comment marker or end of source
            let end = chars
                .find(|&(_, ch)| ch.is_whitespace() || ch == '#')
                .map_or(self.source.len(), |(relative_end, _)| base + relative_end);
            self.offset = end;

            return Some(Spanned {
                token: &self.source[start..end],
                span: Span::new(start, end),
            });
        }
    }
}

pub fn scan(source: &str) -> impl Iterator<Item = Spanned<&str>> {
    Scanner::new(source)
}

// Classify each Lexeme into a Token, keeping the span for
// error messages.
pub fn classify<'src>(
    lex_iter: impl Iterator<Item = Spanned<&'src str>>,
) -> impl Iterator<Item = Spanned<Token<'src>>> {
    lex_iter.map(|s| s.map(Token::new))
}

#[cfg(test)]
mod tests {
    use super::*;
}
