const EXPRESSION_ERROR: &str = "表达式有误";

#[derive(Debug, Clone, PartialEq)]
pub enum Symbol {
    Add,
    Sub,
    Mul,
    Div,
    Per,
    ParL,
    ParR,
}

impl Symbol {
    fn parse(symbol: char) -> Result<Self, String> {
        match symbol {
            '+' => Ok(Self::Add),
            '-' => Ok(Self::Sub),
            '*' => Ok(Self::Mul),
            '/' => Ok(Self::Div),
            '%' => Ok(Self::Per),
            '(' => Ok(Self::ParL),
            ')' => Ok(Self::ParR),
            _ => Err(format!("不支持运算 {symbol} 符号")),
        }
    }

    fn compute(&self, number: &mut Vec<f64>) -> Result<(), &str> {
        let b = number.pop().ok_or(EXPRESSION_ERROR)?;
        if let Self::Per = self {
            number.push(b / 100.0);
            return Ok(());
        }

        let a = number.pop().ok_or(EXPRESSION_ERROR)?;
        match self {
            Symbol::Add => number.push(a + b),
            Symbol::Sub => number.push(a - b),
            Symbol::Mul => number.push(a * b),
            Symbol::Div if b != 0.0 => number.push(a / b),
            Symbol::Div => return Err("除数不能为 0"),
            _ => return Err(EXPRESSION_ERROR),
        }
        Ok(())
    }
}

/// 计算算式表达式
pub fn arithmetic(expr: &str) -> Result<f64, String> {
    let mut symbol = Vec::new();
    let mut number: Vec<f64> = Vec::new();
    let mut prev = char::default();
    let mut buf = String::new();

    for c in expr.chars() {
        // println!("{:?}", symbol);
        // println!("{:?}", number);
        // println!("{:?}\n", buf);

        match c {
            '0'..='9' | '.' => buf.push(c),
            _ if c.is_ascii_whitespace() => continue,
            _ => {
                let s = Symbol::parse(c)?;

                // 遇到 +- 如果上一个字符不是 数字, ’)‘, '%' 就认定为正负数
                if matches!(s, Symbol::Add | Symbol::Sub if !matches!(prev, '0'..='9' | ')' | '%')) {
                    buf.push(c);
                    continue;
                }

                // 解析为浮点数并重置缓冲区
                if !buf.is_empty() {
                    number.push(parse_f64(&buf)?);
                    buf.truncate(0);
                }

                match s {
                    // 括号前面为数字省略 *
                    Symbol::ParL if prev.is_ascii_digit() => {
                        symbol.push(Symbol::Mul);
                        symbol.push(Symbol::ParL);
                    }
                    Symbol::ParL => symbol.push(s),
                    // 遇到 ’)‘ 开始回栈直到遇见 ‘(’
                    Symbol::ParR => back_stack(&mut symbol, &mut number, Some(Symbol::ParL))?,
                    // 计算百分号
                    Symbol::Per => s.compute(&mut number)?,
                    _ => match symbol.last_mut() {
                        // 先乘除
                        Some(last @ (Symbol::Mul | Symbol::Div)) => {
                            last.compute(&mut number)?;
                            *last = s
                        }
                        _ => symbol.push(s),
                    },
                }
            }
        }

        // 记录上一个字符用于后续匹配
        prev = c;
    }

    // println!("{:?}", symbol);
    // println!("{:?}", number);
    // println!("{:?}", buf);

    if !buf.is_empty() {
        number.push(parse_f64(&buf)?)
    }
    back_stack(&mut symbol, &mut number, None)?;

    if number.len() == 1 {
        Ok(number.remove(0))
    } else {
        Err(EXPRESSION_ERROR.into())
    }
}

fn parse_f64(s: &str) -> Result<f64, &str> {
    s.parse::<f64>().map_err(|_| EXPRESSION_ERROR)
}

/// 回栈 消耗符号和数字
fn back_stack(symbol: &mut Vec<Symbol>, number: &mut Vec<f64>, meet: Option<Symbol>) -> Result<(), String> {
    loop {
        match symbol.pop() {
            s if s == meet => return Ok(()),
            Some(s) => s.compute(number)?,
            None => return Err(EXPRESSION_ERROR.into()),
        }
    }
}

#[test]
fn individually_tested() {}

#[test]
fn test_simple_arithmetic() {
    assert_eq!(Ok(0.5), arithmetic("50%"));
    assert_eq!(Ok(-0.5), arithmetic("-50%"));
    assert_eq!(Ok(50.0), arithmetic("+50"));
    assert_eq!(Ok(-50.0), arithmetic("-50"));

    assert_eq!(Ok(50.0), arithmetic("25 + 25"));
    assert_eq!(Ok(50.0), arithmetic("100 - 50"));
    assert_eq!(Ok(-50.0), arithmetic("50 - 100"));
    assert_eq!(Ok(-50.0), arithmetic("-100 + 50"));

    assert_eq!(Ok(50.0), arithmetic("25 * 2"));
    assert_eq!(Ok(-50.0), arithmetic("-25 * 2"));
    assert_eq!(Ok(-50.0), arithmetic("25 * -2"));
    assert_eq!(Ok(50.0), arithmetic("100 / 2"));
    assert_eq!(Ok(-50.0), arithmetic("100 / -2"));
    assert_eq!(Ok(-50.0), arithmetic("-100 / 2"));

    assert_eq!(Ok(10.5), arithmetic("10 + 50%"));
    assert_eq!(Ok(9.5), arithmetic("10 - 50%"));
    assert_eq!(Ok(50.0), arithmetic("100 * 50%"));
    assert_eq!(Ok(50.0), arithmetic("25 / 50%"));

    assert_eq!(Ok(0.3333333333333333), arithmetic("1 / 3"));
    assert_eq!(Ok(3.142857142857143), arithmetic("22 / 7"));
}

#[test]
fn test_mixed_arithmetic() {
    assert_eq!(Ok(20.0), arithmetic("10 - -10"));
    assert_eq!(Ok(30.0), arithmetic("10 - 2(-10)"));
    assert_eq!(Ok(100.0), arithmetic("(3+7) * (6+4)"));
    assert_eq!(Ok(-10.0), arithmetic("-5 * (4 - 1) + 20 / 4"));
    assert_eq!(Ok(224.12121212121212), arithmetic("3 * 4 + 10 * 7 / 33%"));
    assert_eq!(Ok(-1.794871794871795), arithmetic("(-2 * 5 + 3) / (4 - 10%)"));
}

#[test]
fn test_invalid_expressions() {
    assert_eq!(Err("除数不能为 0".into()), arithmetic("10 / 0"));
    assert_eq!(Err("除数不能为 0".into()), arithmetic("100 / (50 - 50)"));
    assert_eq!(Err("不支持运算 $ 符号".into()), arithmetic("10$ * 100"));
    assert_eq!(Err(EXPRESSION_ERROR.into()), arithmetic("10 * (20 - )"));
    assert_eq!(Err(EXPRESSION_ERROR.into()), arithmetic("(10 + 20"));
    assert_eq!(Err(EXPRESSION_ERROR.into()), arithmetic("10 + 20)"));
    assert_eq!(Err(EXPRESSION_ERROR.into()), arithmetic("(20 + 30)10"));
}
