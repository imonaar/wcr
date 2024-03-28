use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use clap::Parser;

type MyResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Parser, Debug)]
#[command(
    about = "The wc utility displays the number of lines, words, and bytes contained
     in each input file, or standard input (if no file is specified) to the
     standard output.",
     long_about = None
)]
#[command(version = "0.1.0")]
#[command(author = "Kevin")]
pub struct Config {
    #[arg(help = "file(s) to process", num_args=0.., value_name = "FILES",  default_value = "-")]
    files: Vec<String>,

    #[arg(
        short = 'l',
        long = "lines",
        help = "The number of lines in each input file is written to the standard output",
        value_name = "LINES"
    )]
    lines: bool,

    #[arg(
        short = 'w',
        long = "words",
        help = "The number of words in each input file is written to the standard output.",
        value_name = "WORDS"
    )]
    words: bool,

    #[arg(
        short = 'c',
        long = "bytes",
        help = "The number of bytes in each input file is written to the standard output.",
        value_name = "BYTES",
        conflicts_with = "chars"
    )]
    bytes: bool,

    #[arg(
        short = 'm',
        long = "chars",
        help = "The number of characters in each input file is written to the standard output.",
        value_name = "CHARS"
    )]
    chars: bool,
}

#[derive(Debug, PartialEq)]
struct FileInfo {
    num_lines: usize,
    num_words: usize,
    num_bytes: usize,
    num_chars: usize,
}

pub fn get_args() -> MyResult<Config> {
    let config = Config::parse();
    Ok(config)
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(std::io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn count(mut file: impl BufRead) -> MyResult<FileInfo> {
    let mut num_lines = 0;
    let mut num_words = 0;
    let mut num_bytes = 0;
    let mut num_chars = 0;

    let mut line = String::new();

    loop {
        let line_bytes = file.read_line(&mut line)?;
        if line_bytes == 0 {
            break;
        }

        num_lines += 1;
        num_bytes += line_bytes;
        num_words += line.split_whitespace().count();
        num_chars += line.chars().count();
        line.clear();
    }

    Ok(FileInfo {
        num_bytes,
        num_lines,
        num_chars,
        num_words,
    })
}

fn format_field(value: usize, show: bool) -> String {
    if show {
        format!("{:>8}", value)
    } else {
        "".to_string()
    }
}

pub fn run(mut config: Config) -> MyResult<()> {
    if [config.lines, config.words, config.bytes, config.chars]
        .iter()
        .all(|v| v == &false)
    {
        config.lines = true;
        config.words = true;
        config.bytes = true;
    }

    let mut total_lines = 0;
    let mut total_words = 0;
    let mut total_bytes = 0;
    let mut total_chars = 0;

    for filename in &config.files {
        match open(filename) {
            Err(e) => eprintln!("{}: {}", filename, e),
            Ok(file) => {
                if let Ok(res) = count(file) {
                    println!(
                        "{}{}{}{}{}",
                        format_field(res.num_lines, config.lines),
                        format_field(res.num_words, config.words),
                        format_field(res.num_bytes, config.bytes),
                        format_field(res.num_chars, config.chars),
                        if filename == "-" {
                            "".to_string()
                        } else {
                            format!(" {}", filename)
                        }
                    );

                    total_lines += res.num_lines;
                    total_words += res.num_words;
                    total_bytes += res.num_bytes;
                    total_chars += res.num_chars;
                }
            }
        }
    }
    if config.files.len() > 1 {
        println!(
            "{}{}{}{} total",
            format_field(total_lines, config.lines),
            format_field(total_words, config.words),
            format_field(total_bytes, config.bytes),
            format_field(total_chars, config.chars)
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_count() {
        let text = "I don't want the world. I just want your half.\r\n";
        let info = count(Cursor::new(text));
        assert!(info.is_ok());
        let expected = FileInfo {
            num_lines: 1,
            num_words: 10,
            num_chars: 48,
            num_bytes: 48,
        };

        assert_eq!(info.unwrap(), expected);
    }
}
