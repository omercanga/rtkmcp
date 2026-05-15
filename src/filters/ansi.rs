use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref ANSI_RE: Regex = Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]").unwrap();
}

pub fn strip(text: &str) -> String {
    ANSI_RE.replace_all(text, "").into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_color_codes() {
        assert_eq!(strip("\x1b[31mError\x1b[0m"), "Error");
    }

    #[test]
    fn plain_text_unchanged() {
        assert_eq!(strip("hello world"), "hello world");
    }
}
