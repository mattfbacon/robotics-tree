use std::path::PathBuf;
use std::sync::Arc;

use comrak::nodes::NodeValue;
use comrak::{
	format_html, parse_document, Arena, ComrakExtensionOptions, ComrakOptions, ComrakParseOptions,
	ComrakRenderOptions, ListStyleType,
};
use once_cell::sync::Lazy;

use crate::ItemMeta;

fn copy_url(url_str: &str, category: &str) -> Option<String> {
	if let Some(relative_path) = url_str.strip_prefix("./") {
		let in_path = ["items", category, relative_path]
			.into_iter()
			.collect::<PathBuf>();
		let out_path = ["dist", "img", category, relative_path]
			.into_iter()
			.collect::<PathBuf>();

		std::fs::create_dir_all(["dist", "img", category].into_iter().collect::<PathBuf>())
			.expect("ensuring dist subdirectory for items");
		std::fs::copy(&in_path, &out_path)
			.unwrap_or_else(|_| panic!("copying image from {in_path:?} to {out_path:?}"));
		Some(format!("img/{category}/{relative_path}"))
	} else {
		None
	}
}

macro_rules! html_name {
	($name:expr) => {
		html5ever::QualName {
			prefix: None,
			ns: {
				use html5ever::namespace_url; // `ns!` macro is unhygienic
				html5ever::ns!()
			},
			local: html5ever::local_name!("src"),
		}
	};
}

const FRONT_MATTER_DELIMITER: &str = "---";
static OPTIONS: Lazy<ComrakOptions> = Lazy::new(|| ComrakOptions {
	extension: ComrakExtensionOptions {
		autolink: true,
		description_lists: true,
		footnotes: true,
		front_matter_delimiter: Some(FRONT_MATTER_DELIMITER.to_owned()),
		header_ids: None,
		strikethrough: true,
		superscript: true,
		table: true,
		tagfilter: true,
		tasklist: true,
	},
	parse: ComrakParseOptions {
		smart: true,
		default_info_string: None,
	},
	render: ComrakRenderOptions {
		hardbreaks: false,
		github_pre_lang: true,
		width: 0,
		unsafe_: true,
		escape: false,
		list_style: ListStyleType::Dash,
	},
});

pub fn render(
	file_path: &Arc<str>,
	input: &str,
	category: &str,
	failure_channel: &crate::validate::FailureChannel,
) -> (String, ItemMeta) {
	eprintln!("rendering markdown");

	let arena = Arena::new();
	let root = parse_document(&arena, input, &OPTIONS);
	let mut metadata: Option<ItemMeta> = None;

	for node in root.descendants() {
		let value = &mut node.data.borrow_mut().value;
		match value {
			NodeValue::FrontMatter(content) => {
				assert!(metadata.is_none());
				let content = std::str::from_utf8(content).expect("item metadata is not valid UTF-8");
				let content = content
					.trim()
					.strip_prefix(FRONT_MATTER_DELIMITER)
					.unwrap()
					.strip_suffix(FRONT_MATTER_DELIMITER)
					.unwrap()
					.trim();
				metadata = Some(toml::from_str(content).expect("item metadata is invalid"));
			}
			NodeValue::Image(image) => {
				let url_str = std::str::from_utf8(&image.url).expect("link URL is not valid UTF-8");
				if let Some(new_url) = copy_url(url_str, category) {
					image.url = new_url.into_bytes();
				}
			}
			NodeValue::HtmlBlock(block) => {
				let content_str =
					std::str::from_utf8(&block.literal).expect("HTML content is not valid UTF-8");
				crate::validate::spawn_validation(
					content_str.to_owned(),
					Arc::clone(file_path),
					find_line_in_input(input, content_str)
						.expect("could not find location of HTML block within input"),
					failure_channel.clone(),
				);
				let mut fragment = scraper::Html::parse_fragment(content_str);
				for value in fragment.tree.values_mut() {
					if let scraper::Node::Element(element) = value {
						if element.name().eq_ignore_ascii_case("img") {
							if let Some(src) = element.attrs.get_mut(&html_name!("src")) {
								if let Some(new_url) = copy_url(src.as_ref(), category) {
									*src = new_url.into();
								}
							}
						}
					}
				}
				block.literal = fragment.root_element().html().into_bytes();
			}
			NodeValue::CodeBlock(block) => {
				let html = hl::render_html(
					std::str::from_utf8(&block.literal)
						.expect("code block is not valid UTF-8")
						.trim()
						.chars()
						.collect(),
					std::str::from_utf8(&block.info).unwrap(),
				);

				// `block_type` is not an enum for bad reasons. 1 = code in <pre> tag
				*value = NodeValue::HtmlBlock(comrak::nodes::NodeHtmlBlock {
					block_type: 1,
					literal: html.into_bytes(),
				});
			}
			_ => (),
		}
	}

	let mut output = Vec::new();
	format_html(root, &OPTIONS, &mut output).expect("generating HTML from parsed markdown");
	let output = String::from_utf8(output).expect("generated HTML is not valid UTF-8");

	(output, metadata.expect("no metadata in markdown file"))
}

/*
const HIGHLIGHT_NAMES: &[&str] = &[
	"attribute",
	"label",
	"constant",
	"function-builtin",
	"function-macro",
	"function",
	"keyword",
	"operator",
	"property",
	"punctuation",
	"punctuation-bracket",
	"punctuation-delimiter",
	"string",
	"string-special",
	"tag",
	"escape",
	"type",
	"type-builtin",
	"constructor",
	"variable",
	"variable-builtin",
	"variable-parameter",
	"comment",
];
*/

/*
fn code_to_html(block_type: &str, code: &str) -> Option<String> {
	/*
	use std::fmt::Write as _;

	use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};

	match block_type {
		"" => return None,
		"cpp" | "c++" => (),
		other => panic!("unknown markdown codeblock language tag {other:?}. supported tags are `c++`/`cpp` or no tag at all."),
	}

	let mut highlighter = Highlighter::new();
	let config = HighlightConfiguration::new(
		tree_sitter_cpp::language(),
		tree_sitter_cpp::HIGHLIGHT_QUERY,
		"",
		"",
	)
	.unwrap();
	let events = highlighter
		.highlight(&config, code.as_bytes(), None, |_| None)
		.expect("creating highlighter event stream");

	let mut ret = "<pre><code>".to_owned();

	for event in events {
		let event = event.expect("reading highlighter event");
		match event {
			HighlightEvent::Source { start, end } => {
				write!(
					ret,
					"{}",
					askama_escape::escape(code.get(start..end).unwrap(), askama_escape::Html)
				)
				.unwrap();
			}
			HighlightEvent::HighlightStart(Highlight(index)) => {
				let class = HIGHLIGHT_NAMES[index];
				write!(ret, "<span class=\"{class}\">").unwrap();
			}
			HighlightEvent::HighlightEnd => {
				write!(ret, "</span>").unwrap();
			}
		}
	}

	write!(ret, "</pre></code>").unwrap();
	Some(ret)
	*/
}
*/

pub fn find_line_in_input(input: &str, text: &str) -> Option<u32> {
	let byte_loc = input.find(text)?;
	let newlines = input
		.bytes()
		.take(byte_loc)
		.filter(|&ch| ch == b'\n')
		.count();
	Some(u32::try_from(newlines).expect("yeah right you have a 4 billion line file") + 1)
}
