/// Estimate token count for a text string.
///
/// Uses a weighted approach:
/// - ASCII/Latin characters: ~4 chars per token
/// - CJK characters: ~1.5 chars per token
/// - Other Unicode: ~2 chars per token
/// - Overhead: 4 tokens per message (role, separators)
pub fn estimate_tokens(text: &str) -> u64 {
    if text.is_empty() {
        return 0;
    }

    let mut ascii_count = 0u64;
    let mut cjk_count = 0u64;
    let mut other_count = 0u64;

    for ch in text.chars() {
        if ch.is_ascii() {
            ascii_count += 1;
        } else if is_cjk(ch) {
            cjk_count += 1;
        } else {
            other_count += 1;
        }
    }

    let tokens = (ascii_count as f64 / 4.0)
        + (cjk_count as f64 / 1.5)
        + (other_count as f64 / 2.0);

    tokens.ceil().max(1.0) as u64
}

fn is_cjk(ch: char) -> bool {
    matches!(ch,
        '\u{4E00}'..='\u{9FFF}' |   // CJK Unified Ideographs
        '\u{3400}'..='\u{4DBF}' |   // CJK Unified Ideographs Extension A
        '\u{F900}'..='\u{FAFF}' |   // CJK Compatibility Ideographs
        '\u{2E80}'..='\u{2EFF}' |   // CJK Radicals Supplement
        '\u{3000}'..='\u{303F}' |   // CJK Symbols and Punctuation
        '\u{31C0}'..='\u{31EF}' |   // CJK Strokes
        '\u{FE30}'..='\u{FE4F}' |   // CJK Compatibility Forms
        '\u{20000}'..='\u{2A6DF}' | // CJK Unified Ideographs Extension B
        '\u{2A700}'..='\u{2B73F}' | // CJK Unified Ideographs Extension C
        '\u{2B740}'..='\u{2B81F}'   // CJK Unified Ideographs Extension D
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_string() {
        assert_eq!(estimate_tokens(""), 0);
    }

    #[test]
    fn test_ascii_text() {
        let tokens = estimate_tokens("Hello, world!");
        assert!(tokens >= 2 && tokens <= 4);
    }

    #[test]
    fn test_cjk_text() {
        let tokens = estimate_tokens("你好世界");
        assert!(tokens >= 2 && tokens <= 4);
    }

    #[test]
    fn test_mixed_text() {
        let tokens = estimate_tokens("Hello 你好");
        assert!(tokens >= 2 && tokens <= 4);
    }

    #[test]
    fn test_long_text() {
        let text = "a".repeat(1000);
        let tokens = estimate_tokens(&text);
        assert_eq!(tokens, 250); // 1000 / 4 = 250
    }

    #[test]
    fn test_code_text() {
        let code = "fn main() {\n    println!(\"Hello, world!\");\n}";
        let tokens = estimate_tokens(code);
        assert!(tokens >= 5 && tokens <= 15);
    }
}
