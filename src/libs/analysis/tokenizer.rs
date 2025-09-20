use std::collections::VecDeque;

pub fn tokenize(input: String) -> VecDeque<String> {
    let mut out = VecDeque::new();
    let mut it = input.chars().peekable();

    while let Some(&c) = it.peek() {
        if c.is_whitespace() {
            it.next();
            continue;
        }

        if c == ':' {
            it.next();
            if matches!(it.peek(), Some(':')) {
                it.next();
                out.push_back("::".into());
                continue;
            }
            out.push_back(":".into());
            continue;
        }

        if c == '-' {
            it.next();
            if matches!(it.peek(), Some('>')) {
                it.next();
                out.push_back("->".into());
                continue;
            }

            if matches!(it.peek(), Some(d) if d.is_ascii_digit()) {
                let mut s = String::from("-");
                while let Some(&d) = it.peek() {
                    if d.is_ascii_digit() {
                        s.push(d);
                        it.next();
                    } else {
                        break;
                    }
                }
                out.push_back(s);
                continue;
            }
            out.push_back("-".into());
            continue;
        }

        if c == '(' {
            it.next();
            out.push_back("(".into());
            continue;
        }

        if c == ')' {
            it.next();
            out.push_back(")".into());
            continue;
        }

        if c == '[' {
            it.next();
            if matches!(it.peek(), Some(']')) {
                it.next();
                out.push_back("[]".into());
            } else {
                out.push_back("[".into());
            }
            continue;
        }

        if c == ']' {
            it.next();
            out.push_back("]".into());
            continue;
        }

        if "+*<|=".contains(c) {
            out.push_back(c.to_string());
            it.next();
            continue;
        }

        if c.is_ascii_digit() {
            let mut s = String::new();
            while let Some(&d) = it.peek() {
                if d.is_ascii_digit() {
                    s.push(d);
                    it.next();
                } else {
                    break;
                }
            }
            out.push_back(s);
            continue;
        }

        if c.is_ascii_lowercase() {
            let mut s = String::new();
            while let Some(&ch) = it.peek() {
                if ch.is_ascii_alphanumeric() || ch == '_' {
                    s.push(ch);
                    it.next();
                } else {
                    break;
                }
            }
            out.push_back(s);
            continue;
        }

        out.push_back(c.to_string());
        it.next();
    }

    out
}
