extern crate editorconfig;
use self::editorconfig::get_config;

use std::path::Path;

pub enum IndentStyle {
    Tab,
    Space
}

pub enum IndentSize {
    Size(u8),
    Tab
}

pub enum EndOfLine {
    Lf,
    CrLf,
    Cr
}

pub enum Charset {
    Latin1,
    UTF8,
    UTF16BE,
    UTF16LE
}

pub struct Config {
    indent_style: IndentStyle,
    indent_size: IndentSize,
    tab_width: u8,
    end_of_line: EndOfLine,
    charset: Charset,
    pub trim_trailing_whitespace: bool,
    pub insert_final_newline: bool,
}

impl Config {
    pub fn config_for(path: Option<&str>) -> Config {
        let mut result = Config {
            indent_style: IndentStyle::Space,
            indent_size: IndentSize::Size(4),
            tab_width: 4,
            end_of_line: EndOfLine::Lf,
            charset: Charset::UTF8,
            trim_trailing_whitespace: true,
            insert_final_newline: true,
        };
        if let Some(path) = path {
            let path = Path::new(path);
            let conf = get_config(path);
            if let Ok(conf) = conf {
                if let Some(style) = conf.get("indent_style") {
                    if style == "tab" {
                        result.indent_style = IndentStyle::Tab;
                    } else if style == "space" {
                        result.indent_style = IndentStyle::Space;
                    }
                }

                if let Some(size) = conf.get("indent_size") {
                    if size == "tab" {
                        result.indent_size = IndentSize::Tab;
                    } else {
                        if let Ok(size) = size.parse() {
                            result.indent_size = IndentSize::Size(size);
                        }
                    }
                }

                if let Some(width) = conf.get("tab_width") {
                    if let Ok(width) = width.parse() {
                        result.tab_width = width;
                    }
                }

                if let Some(eol) = conf.get("end_of_line") {
                    if eol == "cr" {
                        result.end_of_line = EndOfLine::Cr;
                    } else if eol == "crlf" {
                        result.end_of_line = EndOfLine::CrLf;
                    } else if eol == "lf" {
                        result.end_of_line = EndOfLine::Lf;
                    }
                }

                if let Some(charset) = conf.get("charset") {
                    if charset == "latin1" {
                        result.charset = Charset::Latin1;
                    } else if charset == "utf-8" {
                        result.charset = Charset::UTF8;
                    } else if charset == "utf-16be" {
                        result.charset = Charset::UTF16BE;
                    } else if charset == "utf-16le" {
                        result.charset = Charset::UTF16LE;
                    }
                }

                if let Some(ttw) = conf.get("trim_trailing_whitespace") {
                    if ttw == "true" {
                        result.trim_trailing_whitespace = true;
                    } else if ttw == "false" {
                        result.trim_trailing_whitespace = false;
                    }
                }

                if let Some(ifn) = conf.get("insert_final_newline") {
                    if ifn == "true" {
                        result.insert_final_newline = true;
                    } else if ifn == "false" {
                        result.insert_final_newline = false;
                    }
                }
            }
        }
        result
    }

    pub fn indent(&self) -> String {
        match self.indent_style {
            IndentStyle::Tab => "\t".to_string(),
            IndentStyle::Space => {
                let spaces = match self.indent_size {
                    IndentSize::Tab => self.tab_width,
                    IndentSize::Size(n) => n
                } as usize;
                " ".repeat(spaces)
            }
        }
    }
    
    pub fn line_sep(&self) -> &str {
        match self.end_of_line {
            EndOfLine::Lf => "\n",
            EndOfLine::CrLf => "\r\n",
            EndOfLine::Cr => "\r"
        }
    }
}
