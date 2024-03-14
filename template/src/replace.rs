use std::collections::HashMap;

use derive_more::Deref;

#[derive(Debug, Default, Deref)]
pub struct Replace(pub HashMap<String, String>);

impl Replace {
    /// `{{ from }} -> to`
    pub fn replace(&self, source: impl AsRef<str>) -> String {
        let mut keyword = String::new();
        let mut output = String::new();
        let mut prev = ' ';

        for char in source.as_ref().chars() {
            match char {
                // end match keyword
                '}' if !keyword.is_empty() && prev == '}' => {
                    let key = keyword[1..keyword.len() - 1].trim();
                    if let Some(to) = self.0.get(key) {
                        output.pop(); // 去掉 ‘{’
                        Self::push(&mut output, to);
                    } else {
                        output.push_str(&keyword);
                        output.push(char);
                    }
                    keyword.truncate(0);
                }
                // add keyword
                c if !keyword.is_empty() => keyword.push(c),
                // begin match keyword
                '{' if prev == '{' => keyword.push(char),
                c => output.push(c),
            }
            prev = char
        }
        output
    }

    fn push(source: &mut String, to: &str) {
        let is_ascii_uppercase = source
            .as_bytes()
            .iter()
            .rev()
            .take_while(|t| !t.is_ascii_whitespace() && !t.is_ascii_punctuation())
            .any(u8::is_ascii_uppercase);

        let mut chars = to.chars();
        if is_ascii_uppercase {
            if let Some(c) = chars.next() {
                source.push(c.to_ascii_uppercase())
            }
        }
        source.extend(chars)
    }
}

#[macro_export]
macro_rules! replace {
    ($($key:expr => $value:expr $(,)?)*) => {{
        let mut reps = Replace::default();
        $(
            reps.0.insert($key.into(), $value.into());
        )*
        reps
    }};
}

#[test]
fn aaa() {
    let reps = replace! {
        "name" => "user",
        "author" => "Mll"
    };

    let s1 = String::from("// {{ author }} \nfn Get{{ name }}() {}");
    let s2 = String::from("fn put_{{ name }}() {}");
    println!("{}", reps.replace(s1));
    println!("{}", reps.replace(s2));
}
