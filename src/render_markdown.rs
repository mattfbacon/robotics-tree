use std::path::PathBuf;
use std::sync::Arc;

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

pub fn render(
	file_path: &Arc<str>,
	input: &str,
	category: &str,
	failure_channel: &crate::validate::FailureChannel,
) -> (String, ItemMeta) {
	use comrak::nodes::NodeValue;
	use comrak::{
		format_html, parse_document, Arena, ComrakExtensionOptions, ComrakOptions, ComrakParseOptions,
		ComrakRenderOptions, ListStyleType,
	};
	use once_cell::sync::Lazy;

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

	eprintln!("rendering markdown");

	let arena = Arena::new();
	let root = parse_document(&arena, input, &OPTIONS);
	let mut metadata: Option<ItemMeta> = None;

	for node in root.descendants() {
		match &mut node.data.borrow_mut().value {
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
			_ => (),
		}
	}

	let mut output = Vec::new();
	format_html(root, &OPTIONS, &mut output).expect("generating HTML from parsed markdown");
	let output = String::from_utf8(output).expect("generated HTML is not valid UTF-8");

	(output, metadata.expect("no metadata in markdown file"))
}

pub fn find_line_in_input(input: &str, text: &str) -> Option<u32> {
	let byte_loc = input.find(text)?;
	let newlines = input
		.bytes()
		.take(byte_loc)
		.filter(|&ch| ch == b'\n')
		.count();
	Some(u32::try_from(newlines).expect("yeah right you have a 4 billion line file") + 1)
}
