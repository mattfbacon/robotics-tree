use std::path::PathBuf;

use crate::ItemMeta;

pub fn render(input: &str, category: &str) -> (String, ItemMeta) {
	use comrak::nodes::NodeValue;
	use comrak::{
		format_html, parse_document, Arena, ComrakExtensionOptions, ComrakOptions, ComrakParseOptions,
		ComrakRenderOptions, ListStyleType,
	};
	use once_cell::sync::Lazy;

	const FRONT_MATTER_DELIMITER: &'static str = "---";
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
				let content = std::str::from_utf8(&content).expect("item metadata is not valid UTF-8");
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
				if let Some(relative_path) = url_str.strip_prefix("./") {
					std::fs::create_dir_all(["dist", "img", category].into_iter().collect::<PathBuf>())
						.expect("ensuring dist subdirectory for items");
					std::fs::copy(
						["items", category, relative_path]
							.into_iter()
							.collect::<PathBuf>(),
						["dist", "img", category, relative_path]
							.into_iter()
							.collect::<PathBuf>(),
					)
					.expect("copying image from items directory to dist directory");
					image.url = format!("img/{category}/{relative_path}").into_bytes();
				}
			}
			_ => (),
		}
	}

	let mut output = Vec::new();
	format_html(root, &OPTIONS, &mut output).expect("generating HTML from parsed markdown");
	let output = String::from_utf8(output).expect("generated HTML is not valid UTF-8");

	(output, metadata.expect("no metadata in markdown file"))
}
