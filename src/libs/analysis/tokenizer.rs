use std::collections::VecDeque;

pub fn tokenize(input: String) -> VecDeque<String> {
    let mut out = VecDeque::new();
    let mut it = input.chars().peekable();

    while it.peek().is_some() {
        if it.next_if(|c| c.is_whitespace()).is_some() {
            continue;
        }

        if it.next_if_eq(&':').is_some() && it.next_if_eq(&':').is_some() {
            out.push_back("::".into());
            continue;
        }

        if it.next_if_eq(&'[').is_some() && it.next_if_eq(&']').is_some() {
            out.push_back("[]".into());
            continue;
        }

        if it.next_if_eq(&'(').is_some() {
            out.push_back("(".into());
            continue;
        }

        if it.next_if_eq(&')').is_some() {
            out.push_back(")".into());
            continue;
        }

        if it.next_if_eq(&'-').is_some() {
            if it.next_if_eq(&'>').is_some() {
                out.push_back("->".into());
                continue;
            }

            let mut integer_literal = String::from("-");
            while it.peek().is_some_and(|c| c.is_ascii_digit()) {
                integer_literal.push(it.next().unwrap());
            }
            out.push_back(integer_literal);
            continue;
        }

        if it.next_if_eq(&'+').is_some() {
            let mut integer_literal = String::from("+");
            while it.peek().is_some_and(|c| c.is_ascii_digit()) {
                integer_literal.push(it.next().unwrap());
            }
            out.push_back(integer_literal);
            continue;
        }

        if let Some(c) = it.next_if(|&c| "|=*<".contains(c)) {
            out.push_back(c.into());
            continue;
        }

        if it.peek().is_some_and(|c| c.is_ascii_digit()) {
            let mut integer_literal = String::new();
            while it.peek().is_some_and(|c| c.is_ascii_digit()) {
                integer_literal.push(it.next().unwrap());
            }
            out.push_back(integer_literal);
            continue;
        }

        if it.peek().is_some_and(|c| c.is_ascii_lowercase()) {
            let mut identifier = String::new();
            while it.peek().is_some_and(|&c| c.is_alphanumeric() || c == '_') {
                identifier.push(it.next().unwrap());
            }
            out.push_back(identifier);
            continue;
        }

        out.push_back(it.next().unwrap().into());
    }

    out
}
