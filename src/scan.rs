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

    #[test]
    fn a_new_scanner_has_expected_source_and_an_offset_of_0() {
        let scanner = Scanner::new("source");
        assert_eq!(scanner.source, "source");
        assert_eq!(scanner.offset, 0);
    }

    #[test]
    fn a_new_scanner_has_only_one_spanned_in_it() {
        let mut scanner = Scanner::new("source");
        assert_eq!(
            scanner.next(),
            Some(Spanned {
                token: "source",
                span: Span::new(0, 6)
            })
        );
        assert_eq!(scanner.next(), None);
    }

    #[test]
    fn scanner_returns_the_expected_spans() {
        let source = "1 2 3 mul pop #this is a comment\n# another comment\n4 foo bar";
        let scanner = Scanner::new(source);
        let expected = vec![
            Spanned {
                token: "1",
                span: Span::new(0, 1),
            },
            Spanned {
                token: "2",
                span: Span::new(2, 3),
            },
            Spanned {
                token: "3",
                span: Span::new(4, 5),
            },
            Spanned {
                token: "mul",
                span: Span::new(6, 9),
            },
            Spanned {
                token: "pop",
                span: Span::new(10, 13),
            },
            Spanned {
                token: "4",
                span: Span::new(51, 52),
            },
            Spanned {
                token: "foo",
                span: Span::new(53, 56),
            },
            Spanned {
                token: "bar",
                span: Span::new(57, 60),
            },
        ];
        for (result, expected) in scanner.zip(expected) {
            assert_eq!(result, expected);
        }
    }
}
