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

	let copy_url = |url_str: &str| {
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
			Some(format!("img/{category}/{relative_path}"))
		} else {
			None
		}
	};

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
				if let Some(new_url) = copy_url(url_str) {
					image.url = new_url.into_bytes();
				}
			}
			NodeValue::HtmlBlock(block) => {
				let content_str =
					std::str::from_utf8(&block.literal).expect("HTML content is not valid UTF-8");
				let mut fragment = scraper::Html::parse_fragment(content_str);
				assert_eq!(fragment.errors, &[] as &[std::borrow::Cow<'_, str>]);
				for value in fragment.tree.values_mut() {
					if let scraper::Node::Element(element) = value {
						if element.name().eq_ignore_ascii_case("img") {
							if let Some(src) = element.attrs.get_mut(&html5ever::QualName {
								prefix: None,
								ns: {
									use html5ever::namespace_url; // `ns!` macro is unhygienic
									html5ever::ns!()
								},
								local: html5ever::local_name!("src"),
							}) {
								if let Some(new_url) = copy_url(src.as_ref()) {
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
