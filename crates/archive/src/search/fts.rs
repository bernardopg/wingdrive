//! FTS5 query utilities.

/// Turns text a person typed into an FTS5 match expression, or `None` when it has no words.
///
/// Every word becomes a quoted prefix term, so `tomat` finds `tomatoes`, and characters that mean
/// something to FTS5 (`"`, `-`, `*`, `:`, `NOT`) are searched as text instead of failing the
/// query. Words split where the `unicode61` tokenizer splits them, and all must match. Source
/// indexes use the `porter` stemmer, which stems the prefix too: `harn` does not reach `harness`,
/// whose stem is `har`.
pub fn match_expression(text: &str) -> Option<String> {
	let terms: Vec<String> = text
		.split(|c: char| !c.is_alphanumeric())
		.filter(|word| !word.is_empty())
		.map(|word| format!("\"{word}\"*"))
		.collect();
	if terms.is_empty() {
		None
	} else {
		Some(terms.join(" "))
	}
}

#[cfg(test)]
mod tests {
	use super::match_expression;

	#[test]
	fn words_become_quoted_prefix_terms() {
		assert_eq!(match_expression("harn").as_deref(), Some("\"harn\"*"));
		assert_eq!(
			match_expression("  Weekly   review ").as_deref(),
			Some("\"Weekly\"* \"review\"*")
		);
	}

	#[test]
	fn fts_syntax_is_searched_as_text() {
		assert_eq!(
			match_expression("\"rust-lang\" OR title:*").as_deref(),
			Some("\"rust\"* \"lang\"* \"OR\"* \"title\"*")
		);
		assert_eq!(match_expression("ação").as_deref(), Some("\"ação\"*"));
	}

	#[test]
	fn text_without_words_matches_nothing() {
		assert_eq!(match_expression(""), None);
		assert_eq!(match_expression(" -*\"() "), None);
	}
}
