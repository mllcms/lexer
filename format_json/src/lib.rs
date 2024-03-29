use std::{borrow::Borrow, fs, io, mem, path::Path};

use color_string::{wcs, Font};

#[derive(Debug, Clone)]
pub struct Formatter {
    pub raw: bool,
    pub buf: String,
    /// 缩进符号
    pub indent: String,
    /// 最大长度
    pub fix_len: usize,
    pub key_color: Font,
    pub string_color: Font,
    pub key_word_color: Font,
}

impl Default for Formatter {
    fn default() -> Self {
        Self {
            raw: false,
            fix_len: 60,
            buf: String::new(),
            indent: String::from("  "),
            key_color: Font::Color(224, 108, 117),
            string_color: Font::Color(152, 195, 121),
            key_word_color: Font::Color(209, 154, 102),
        }
    }
}

impl Formatter {
    pub fn save(&mut self, path: impl AsRef<Path>, json: &str) -> io::Result<()> {
        fs::write(path, self.format_raw(json))
    }

    pub fn save_compress(&mut self, path: impl AsRef<Path>, json: &str) -> io::Result<()> {
        fs::write(path, self.compress(json))
    }

    pub fn format(&mut self, json: &str) -> String {
        self.raw = false;
        self._format(json)
    }

    pub fn format_raw(&mut self, json: &str) -> String {
        self.raw = true;
        self._format(json)
    }

    pub fn compress(&self, json: &str) -> String {
        let mut buf = String::new();
        let mut is_string = false;
        let mut prev = ' ';
        for char in json.chars() {
            if !is_string && char.is_ascii_whitespace() {
                continue;
            }
            if prev != '\\' && char == '"' {
                is_string = !is_string;
            }
            prev = char;
            buf.push(char);
        }
        buf
    }

    fn _format(&mut self, json: &str) -> String {
        let mut prev = ' '; // 上一个字符
        let mut is_string = false; // 是否为 String
        let mut sign = Vec::new(); // 符号栈主要是 '{' 和 '[' 用来控制缩进
        let mut buf = String::new(); // 缓冲区

        for c in json.chars() {
            match c {
                '"' if prev != '\\' => is_string = !is_string, // 匹配字符串
                _ if is_string => buf.push(c),                 // 字符串模式添加任意数据
                _ if c.is_ascii_whitespace() => continue,      // 过滤空格和换行符,防止下游边界判断失误
                ':' => self.add_key(&mut buf, sign.len()),
                '[' | '{' => {
                    let mut start = self.buf.len(); // 记录边界起始位置用于后期优化
                    match sign.last() {
                        Some(('[', _)) => {
                            self.push(format!("{}{c}\n", self.padding(sign.len())));
                            start += self.indent.len() * sign.len(); // 数据没有 key 单独缩进重新计算起始位置
                        }
                        _ => self.push_line(c),
                    }
                    sign.push((c, start))
                }
                ',' | ']' | '}' => {
                    let (s, start) = sign.last().cloned().unwrap_or_default();
                    match (prev, s) {
                        _ if buf.is_empty() => {}                             // 缓冲区没有数据不进行任何处理
                        ('"', '{') => self.add_string(&mut buf, 0),           // { "key": "data" }
                        (_, '{') => self.add_key_world(&mut buf, 0),          // { "key": null }
                        ('"', '[') => self.add_string(&mut buf, sign.len()),  // ["data"]
                        (_, '[') => self.add_key_world(&mut buf, sign.len()), // [1,2,3]
                        ('"', _) => self.add_string(&mut buf, sign.len()),    // "data"
                        _ => self.add_key_world(&mut buf, sign.len()),        // null
                    }
                    if c == ',' {
                        self.push_line(c)
                    } else {
                        sign.pop();
                        self.push(format!("\n{}{}", self.padding(sign.len()), c));
                        self.fix(start);
                    }
                }
                _ => buf.push(c),
            }
            prev = c;
        }
        let mut res = String::new();
        mem::swap(&mut res, &mut self.buf);
        res
    }

    fn push(&mut self, buf: impl Borrow<str>) {
        self.buf += buf.borrow()
    }

    fn push_line(&mut self, c: char) {
        self.buf.push(c);
        self.buf.push('\n')
    }

    /// 添加缩进
    fn padding(&self, n: usize) -> String {
        self.indent.repeat(n)
    }

    fn fix(&mut self, start: usize) {
        let mut size = 0;
        let mut flag = false;
        let indent = self.indent.chars().next().unwrap_or(' ');
        // 处理颜色字符宽度
        for c in self.buf[start..].chars() {
            match c {
                '\x1b' => flag = true,
                'm' if flag => flag = false,
                _ if flag => continue,
                ' ' | '\n' => continue,
                _ if c == indent => continue,
                _ => size += 1,
            }
            if size > self.fix_len {
                return;
            }
        }

        let s = self.buf[start..]
            .replace(&self.indent, "")
            .replace('\n', "")
            .replace(',', ", ");
        self.buf.truncate(start);
        self.push(s)
    }

    fn add_key(&mut self, buf: &mut String, n: usize) {
        self.push(self.padding(n));
        if self.raw {
            self.push(format!("{:#?}", buf))
        } else {
            wcs!(&mut self.buf, self.key_color; "{:#?}",buf);
        }
        self.buf.push_str(": ");
        buf.truncate(0)
    }

    fn add_key_world(&mut self, buf: &mut String, n: usize) {
        self.push(self.padding(n));
        if self.raw {
            self.push(buf.as_str())
        } else {
            wcs!(&mut self.buf, self.key_word_color; "{}",buf);
        }
        buf.truncate(0)
    }

    fn add_string(&mut self, buf: &mut String, n: usize) {
        self.push(self.padding(n));
        if self.raw {
            self.push(format!("{:#?}", buf))
        } else {
            wcs!(&mut self.buf, self.string_color; "{:#?}",buf);
        }
        buf.truncate(0)
    }
}
