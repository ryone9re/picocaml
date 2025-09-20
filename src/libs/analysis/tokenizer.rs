use std::collections::VecDeque;

pub fn tokenize(token: String) -> VecDeque<String> {
    token.split_whitespace().map(str::to_string).collect()
}
