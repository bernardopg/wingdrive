//! Minimal Mustache-compatible template rendering.
//!
//! The workspace only needs the subset the `.cargo/config.toml.mustache` template uses:
//! `{{name}}` for escaped values, `{{{name}}}` for raw values, and `{{#name}}...{{/name}}`
//! sections (including the `{{^name}}` inverted form).
//!
//! The `mustache` crate pins `log` to 0.3.x, which is semver-incompatible with the `log` 0.4
//! that `pdfium-render` needs. Since `pdfium-render` asks for a bare `log = "0"`, cargo may
//! hand it the 0.3.x, and the crate stops compiling because log 0.3 does not re-export the
//! `log::error!` macro path. Rendering the handful of tags we use keeps `log 0.3.x` out of
//! the graph entirely.

use anyhow::{anyhow, bail, Result};

/// A value a template can receive.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value<'a> {
	/// A string. Escaped for `{{name}}`, emitted as-is for `{{{name}}}`.
	Str(&'a str),
	/// A number, rendered as its decimal digits.
	Num(u64),
}

impl Value<'_> {
	fn is_falsy(&self) -> bool {
		match self {
			Value::Str(s) => s.is_empty(),
			Value::Num(n) => *n == 0,
		}
	}
}

/// Escapes the five HTML characters, matching Mustache's default `{{name}}` handling.
fn escape(input: &str) -> String {
	let mut out = String::with_capacity(input.len());
	for c in input.chars() {
		match c {
			'&' => out.push_str("&amp;"),
			'<' => out.push_str("&lt;"),
			'>' => out.push_str("&gt;"),
			'"' => out.push_str("&quot;"),
			'\'' => out.push_str("&#39;"),
			_ => out.push(c),
		}
	}
	out
}

/// One parsed template node.
enum Node<'a> {
	Literal(&'a str),
	Variable {
		name: &'a str,
		raw: bool,
	},
	Section {
		name: &'a str,
		inverted: bool,
		children: Vec<Node<'a>>,
	},
}

/// A tag found in the source, with the byte range it occupies.
///
/// `inverted` is only a marker for the parser, which reads the leading `^` directly;
/// keeping it out of this struct avoids a field the rest of the module never uses.
struct Tag<'a> {
	name: &'a str,
	/// True for `{{{name}}}`.
	raw: bool,
	/// Byte index just past the closing braces.
	end: usize,
}

/// Finds the next tag at or after `from`, or None if the rest is literal text.
fn next_tag(template: &str, from: usize) -> Option<(usize, Tag<'_>)> {
	let rest = &template[from..];
	let open = rest.find("{{")?;
	let start = from + open;
	let body_start = start + 2;
	let body_end = template[body_start..].find("}}")? + body_start;
	// `{{{name}}}` has no whitespace between the braces, so a `{` immediately after the
	// opening `{{` marks the raw form. Its closing `}}}` carries one extra brace.
	let raw = template[body_start..].starts_with('{');
	let body = template[body_start..body_end].trim();
	let name = body
		.strip_prefix('{')
		.map_or(body, |inner| inner.trim_end_matches('}'));
	let name = name.trim();
	let end = if raw { body_end + 3 } else { body_end + 2 };
	Some((start, Tag { name, raw, end }))
}

/// Parses the template into a node tree.
///
/// Unclosed or mismatched sections are errors rather than silently dropped content: the
/// generated file feeds Cargo's build configuration, so a typo in a tag must not quietly
/// drop the linker flags or the FFmpeg paths.
fn parse(template: &str) -> Result<Vec<Node<'_>>> {
	let mut nodes: Vec<Node> = Vec::new();
	// Each open section pushes its accumulator; the tag name closes it.
	let mut stack: Vec<(&str, bool, Vec<Node>)> = Vec::new();
	let mut cursor = 0;

	while let Some((start, tag)) = next_tag(template, cursor) {
		if start > cursor {
			let text = &template[cursor..start];
			append(stack.last_mut(), &mut nodes, Node::Literal(text));
		}

		if tag.name.is_empty() {
			bail!("empty tag at byte {start}");
		}

		match tag.name.as_bytes()[0] {
			b'#' => {
				let name = &tag.name[1..];
				stack.push((name, false, Vec::new()));
			}
			b'^' => {
				let name = &tag.name[1..];
				stack.push((name, true, Vec::new()));
			}
			b'/' => {
				let closed = &tag.name[1..];
				let (name, inverted, children) = stack
					.pop()
					.ok_or_else(|| anyhow!("closing tag /{} without an open section", closed))?;
				if name != closed {
					bail!("section #{} closed by /{}", name, closed);
				}
				append(
					stack.last_mut(),
					&mut nodes,
					Node::Section {
						name,
						inverted,
						children,
					},
				);
			}
			_ => append(
				stack.last_mut(),
				&mut nodes,
				Node::Variable {
					name: tag.name,
					raw: tag.raw,
				},
			),
		}

		cursor = tag.end;
	}

	if let Some((name, _, _)) = stack.last() {
		bail!("section #{} is never closed", name);
	}

	if cursor < template.len() {
		append(
			stack.last_mut(),
			&mut nodes,
			Node::Literal(&template[cursor..]),
		);
	}

	Ok(nodes)
}

/// Pushes `node` onto the innermost open section, or the root when none is open.
fn append<'a>(
	open: Option<&mut (&'a str, bool, Vec<Node<'a>>)>,
	root: &mut Vec<Node<'a>>,
	node: Node<'a>,
) {
	match open {
		Some((_, _, children)) => children.push(node),
		None => root.push(node),
	}
}

fn render_nodes<'a>(
	nodes: &[Node<'_>],
	lookup: &dyn Fn(&str) -> Option<Value<'a>>,
	out: &mut String,
) {
	for node in nodes {
		match node {
			Node::Literal(text) => out.push_str(text),
			Node::Variable { name, raw } => {
				if let Some(value) = lookup(name) {
					if *raw {
						out.push_str(match value {
							Value::Str(s) => s,
							Value::Num(n) => {
								out.push_str(&n.to_string());
								continue;
							}
						});
					} else {
						match value {
							Value::Str(s) => out.push_str(&escape(s)),
							Value::Num(n) => out.push_str(&n.to_string()),
						}
					}
				}
			}
			Node::Section {
				name,
				inverted,
				children,
			} => {
				let truthy = matches!(lookup(name), Some(v) if !v.is_falsy());
				if truthy != *inverted {
					render_nodes(children, lookup, out);
				}
			}
		}
	}
}

/// Renders `template`, resolving every tag through `lookup`.
///
/// A missing name renders as an empty string and counts as falsy for sections, matching
/// Mustache. Numbers render as their decimal value, which is what `jobs = {{buildJobs}}`
/// relies on.
pub fn render<'a>(template: &str, lookup: &dyn Fn(&str) -> Option<Value<'a>>) -> Result<String> {
	let nodes = parse(template)?;
	let mut out = String::with_capacity(template.len());
	render_nodes(&nodes, &lookup, &mut out);
	Ok(out)
}

#[cfg(test)]
mod tests {
	use super::*;

	fn lookup<'a>(map: &'a [(&'static str, Value<'a>)]) -> impl Fn(&str) -> Option<Value<'a>> + 'a {
		move |name| map.iter().find(|(k, _)| *k == name).map(|(_, v)| *v)
	}

	#[test]
	fn escapes_plain_and_passes_through_raw() {
		let tpl = "plain={{a}} raw={{{a}}} html=<{{h}}>";
		let map = [("a", Value::Str("x&y")), ("h", Value::Str("<b>"))];
		let rendered = render(tpl, &lookup(&map)).unwrap();
		assert_eq!(rendered, "plain=x&amp;y raw=x&y html=<&lt;b&gt;>");
	}

	#[test]
	fn sections_and_inverted_sections_are_exclusive() {
		let tpl = "{{#flag}}on{{/flag}}{{^flag}}off{{/flag}}";
		assert_eq!(
			render(tpl, &lookup(&[("flag", Value::Num(1))])).unwrap(),
			"on"
		);
		assert_eq!(
			render(tpl, &lookup(&[("flag", Value::Num(0))])).unwrap(),
			"off"
		);
		assert_eq!(render(tpl, &lookup(&[])).unwrap(), "off");
	}

	#[test]
	fn empty_string_is_falsy() {
		let tpl = "[{{#v}}V{{/v}}]";
		assert_eq!(
			render(tpl, &lookup(&[("v", Value::Str(""))])).unwrap(),
			"[]"
		);
		assert_eq!(
			render(tpl, &lookup(&[("v", Value::Str("x"))])).unwrap(),
			"[V]"
		);
	}

	#[test]
	fn nested_sections() {
		let tpl = "{{#a}}A{{#b}}B{{/b}}{{/a}}";
		let map = [("a", Value::Num(1)), ("b", Value::Num(1))];
		assert_eq!(render(tpl, &lookup(&map)).unwrap(), "AB");
	}

	#[test]
	fn numbers_render_as_decimals() {
		let tpl = "jobs = {{buildJobs}}";
		let map = [("buildJobs", Value::Num(3))];
		assert_eq!(render(tpl, &lookup(&map)).unwrap(), "jobs = 3");
	}

	#[test]
	fn raw_tags_preserve_windows_paths_unescaped() {
		let tpl = "value = \"{{{nativeDeps}}}\"";
		let map = [("nativeDeps", Value::Str(r"C:\wingdrive"))];
		assert_eq!(
			render(tpl, &lookup(&map)).unwrap(),
			r#"value = "C:\wingdrive""#
		);
	}

	#[test]
	fn missing_names_render_empty() {
		assert_eq!(render("[{{missing}}]", &lookup(&[])).unwrap(), "[]");
	}

	#[test]
	fn rejects_unclosed_section() {
		assert!(render("{{#a}}oops", &lookup(&[])).is_err());
	}

	#[test]
	fn rejects_mismatched_close() {
		assert!(render("{{#a}}{{/b}}", &lookup(&[])).is_err());
	}

	#[test]
	fn rejects_empty_tag() {
		assert!(render("{{ }}", &lookup(&[])).is_err());
	}

	#[test]
	fn renders_the_cargo_config_template_shape() {
		let tpl = "jobs = {{buildJobs}}\n{{#isMacOS}}[target.aarch64-apple-darwin]\n{{/isMacOS}}{{#hasiOS}}\n[target.aarch64-apple-ios]\n{{/hasiOS}}";
		let linux = [
			("buildJobs", Value::Num(4)),
			("isMacOS", Value::Num(0)),
			("hasiOS", Value::Num(0)),
		];
		assert_eq!(render(tpl, &lookup(&linux)).unwrap(), "jobs = 4\n");
		let macos = [
			("buildJobs", Value::Num(4)),
			("isMacOS", Value::Num(1)),
			("hasiOS", Value::Num(0)),
		];
		assert!(render(tpl, &lookup(&macos))
			.unwrap()
			.contains("aarch64-apple-darwin"));
	}

	/// The real template is the contract; a regression here breaks every `xtask` build.
	#[test]
	fn renders_the_real_cargo_config_template() {
		let tpl = include_str!("../../.cargo/config.toml.mustache");
		let map = [
			("nativeDeps", Value::Str("deps")),
			("protoc", Value::Str("deps/bin/protoc")),
			("mobileNativeDeps", Value::Str("mobile")),
			("androidNdkHome", Value::Str("ndk")),
			("hostTag", Value::Str("linux-x86_64")),
			("isWin", Value::Num(0)),
			("isMacOS", Value::Num(0)),
			("isLinux", Value::Num(1)),
			("hasiOS", Value::Num(0)),
			("hasAndroid", Value::Num(1)),
			("hasLLD", Value::Num(1)),
			("sdkPath", Value::Str("")),
			("useSystemFfmpeg", Value::Num(0)),
			("buildJobs", Value::Num(4)),
		];
		let rendered = render(tpl, &lookup(&map)).unwrap();
		let parsed: toml::Value = toml::from_str(&rendered).unwrap();
		assert_eq!(parsed["build"]["jobs"].as_integer(), Some(4));
		assert_eq!(
			parsed["env"]["PROTOC"]["value"].as_str(),
			Some("deps/bin/protoc")
		);
		// Linux emits the rpath flag that keeps the bundled libs next to the daemon binary.
		assert!(rendered.contains("link-arg=-Wl,-rpath=${ORIGIN}/../lib/wingdrive"));
		// Android section only appears when the NDK variables are populated.
		assert!(rendered.contains("aarch64-linux-android35-clang"));
	}
}
