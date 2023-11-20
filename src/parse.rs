//! https://www.gnu.org/software/bash/manual/bash.html

use std::{borrow::Cow, iter, sync::Arc};

use anyhow::{bail, Result};

pub fn parse(filename: Arc<str>, src: &str) -> Result<()> {
    let tokens = lex(filename, src)?;
    dbg!(tokens);
    Ok(())
}

#[derive(Debug)]
pub struct Span {
    pub filename: Arc<str>,
    pub start: u32,
    pub end: u32,
}

#[derive(Debug)]
pub struct Token<'a> {
    pub value: Cow<'a, str>,
    pub restriction: Restriction,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Restriction {
    No,
    DoubleQuot,
    SingleQuot,
}

struct Lexer<'a> {
    filename: Arc<str>,
    src: &'a str,
    bytes: iter::Enumerate<std::str::Bytes<'a>>,
    tokens: Vec<Token<'a>>,
    cloned_escaped: Option<String>,
    last_start: u32,
}

impl<'a> Lexer<'a> {
    fn span_start_end(&self, start: u32, end: u32) -> Span {
        Span {
            filename: self.filename.clone(),
            start,
            end,
        }
    }

    fn handle_char(&mut self, i: u32, b: u8) -> Result<()> {
        match b {
            _ if is_metacharacter(b) => {
                self.commit(i);
                if b != b' ' && b != b'\t' && b != b'\n' {
                    self.tokens.push(Token {
                        value: self.src[(i as usize)..(i as usize + 1)].into(),
                        restriction: Restriction::No,
                        span: self.span_start_end(i, i + 1),
                    });
                }
            }
            b'\\' => {
                let Some((_, next)) = self.bytes.next() else {
                    bail!("Trailing \\ in the file {}", self.filename);
                };
                match &mut self.cloned_escaped {
                    Some(clone_escaped) if next != b'\n' => clone_escaped.push(next as char),
                    Some(_) => {}
                    cloned_escaped @ None => {
                        let mut value =
                            self.src[(self.last_start as usize)..(i as usize)].to_owned();
                        if next != b'\n' {
                            value.push(next as char);
                        }
                        *cloned_escaped = Some(value);
                    }
                }
            }
            _ => {
                if let Some(cloned_escaped) = &mut self.cloned_escaped {
                    cloned_escaped.push(b as char);
                }
            }
        }
        Ok(())
    }

    fn commit(&mut self, i: u32) {
        let span = self.span_start_end(self.last_start, i);
        let token = match self.cloned_escaped.take() {
            None => Token {
                value: self.src[(self.last_start as usize)..(i as usize)].into(),
                restriction: Restriction::No,
                span,
            },
            Some(cloned) => Token {
                value: cloned.clone().into(),
                restriction: Restriction::No,
                span,
            },
        };
        self.finish_word(i, token);
    }

    fn finish_word(&mut self, i: u32, token: Token<'a>) {
        self.cloned_escaped = None;
        self.last_start = i + 1;
        if token.value.starts_with('#') {
            while let Some((i, b)) = self.bytes.next() {
                if b == b'\n' {
                    self.last_start = i as u32 + 1;
                    return;
                }
            }
            // EOF
            self.last_start = self.src.len() as u32;
        } else {
            self.tokens.push(token);
        }
    }
}

fn lex(filename: Arc<str>, src: &str) -> Result<Vec<Token<'_>>> {
    let mut lexer = Lexer {
        filename,
        src,
        bytes: src.bytes().enumerate(),
        tokens: Vec::new(),
        cloned_escaped: None,
        last_start: 0,
    };

    while let Some((i, b)) = lexer.bytes.next() {
        let Ok(i) = i.try_into() else {
            bail!("file {} larger than 4GB", lexer.filename);
        };
        lexer.handle_char(i, b)?;
    }

    if lexer.last_start != (src.len() as u32) {
        lexer.commit(src.len() as u32);
    }

    Ok(lexer.tokens)
}

fn is_metacharacter(c: u8) -> bool {
    [b' ', b'\t', b'\n', b'|', b'&', b';', b'(', b')', b'<', b'>'].contains(&c)
}

#[cfg(test)]
mod tests {
    mod lex {
        use crate::parse::Restriction::{self, *};

        fn test_eq(src: &str, tokens: impl AsRef<[(&'static str, Restriction)]>) {
            let actual = super::super::lex("whatever".into(), src).unwrap();
            let to_compare: Vec<_> = actual
                .iter()
                .map(|tok| (tok.value.as_ref(), tok.restriction))
                .collect();

            assert_eq!(tokens.as_ref(), to_compare);
        }

        #[test]
        fn hello_world() {
            test_eq("Hello, world!", [("Hello,", No), ("world!", No)]);
        }

        #[test]
        fn newline_var() {
            test_eq("echo $a\nb", [("echo", No), ("$a", No), ("b", No)])
        }

        #[test]
        fn newline_var_escape() {
            test_eq("echo $a\\\nb", [("echo", No), ("$ab", No)])
        }

        #[test]
        fn metachars() {
            test_eq(
                "hello;world)yes",
                [
                    ("hello", No),
                    (";", No),
                    ("world", No),
                    (")", No),
                    ("yes", No),
                ],
            )
        }

        #[test]
        fn comment() {
            test_eq("hi # no", [("hi", No)]);
        }

        #[test]
        #[ignore = "TODO: this is buggy"]
        fn comment_escaped_newline() {
            test_eq("#test\\\nno", [("no", No)]);
        }

        #[test]
        fn strange_comment() {
            test_eq(
                "no#true hello;#yes",
                [("no#true", No), ("hello", No), (";", No)],
            );
        }
    }
}
