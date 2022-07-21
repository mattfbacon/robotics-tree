use std::path::Path;

use askama::Template as _;

mod filters {
	pub fn md(s: &str) -> ::askama::Result<String> {
		Ok(comrak::markdown_to_html(
			s,
			&comrak::ComrakOptions {
				extension: comrak::ComrakExtensionOptions {
					autolink: true,
					description_lists: true,
					footnotes: true,
					front_matter_delimiter: None,
					header_ids: None,
					strikethrough: true,
					superscript: true,
					table: true,
					tagfilter: true,
					tasklist: true,
				},
				parse: comrak::ComrakParseOptions {
					smart: true,
					default_info_string: None,
				},
				render: comrak::ComrakRenderOptions {
					hardbreaks: false,
					github_pre_lang: true,
					width: 0,
					unsafe_: true,
					escape: false,
					list_style: comrak::ListStyleType::Dash,
				},
			},
		))
	}
}

#[derive(askama::Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
	main_items: &'a [ExternalItem],
	programming_items: &'a [ExternalItem],
	building_items: &'a [ExternalItem],
	sidebar_items: &'a [ExternalItem],
}

impl IndexTemplate<'_> {
	fn iterate_non_sidebar_tagged(&self) -> impl Iterator<Item = (&'static str, usize, &'_ Item)> {
		macro_rules! make_iter {
			($name:expr, $inner:expr) => {
				$inner
					.into_iter()
					.enumerate()
					.map(move |(index, ExternalItem { item, .. })| ($name, index, item))
			};
		}

		make_iter!("main", self.main_items)
			.chain(make_iter!("programming", self.programming_items))
			.chain(make_iter!("building", self.building_items))
	}
}

#[derive(askama::Template)]
#[template(path = "sidebar_page.html")]
struct SidebarPageTemplate<'a> {
	item: &'a ExternalItem,
}

struct Writer<W>(W);

impl<W: std::io::Write> std::fmt::Write for Writer<W> {
	fn write_str(&mut self, s: &str) -> std::fmt::Result {
		self.0.write_all(s.as_bytes()).expect("writing to writer");
		Ok(())
	}
	fn write_fmt(&mut self, f: std::fmt::Arguments<'_>) -> std::fmt::Result {
		self.0.write_fmt(f).expect("writing to writer");
		Ok(())
	}
}

fn open_output(name: impl AsRef<std::path::Path>) -> Writer<impl std::io::Write> {
	let writer = std::fs::File::options()
		.write(true)
		.truncate(true)
		.create(true)
		.open(Path::new("dist").join(name))
		.expect("Opening output file");
	Writer(writer)
}

#[derive(serde::Deserialize)]
pub struct ItemMeta {
	pub name: Box<str>,
	pub description: Box<str>,
}

pub struct Item {
	pub name: Box<str>,
	pub description: Box<str>,
	pub content: Box<str>,
}

struct ExternalItem {
	id: u32,
	slug: Box<str>,
	item: Item,
}

fn load_items(dir: &str) -> Vec<ExternalItem> {
	let mut items: Vec<_> = std::fs::read_dir(Path::new("items").join(dir))
		.expect("reading sidebar items directory")
		.into_iter()
		.map(|entry| entry.expect("reading sidebar item file"))
		.filter(|entry| entry.path().extension().map_or(false, |ext| ext == "md"))
		.map(|entry| {
			let (id, slug) = entry
				.file_name()
				.to_str()
				.expect("sidebar item file name is invalid UTF-8")
				.split_once('-')
				.and_then(|(id, slug)| slug.strip_suffix(".md").map(move |slug| (id, slug)))
				.and_then(|(id, slug)| {
					id.parse()
						.ok()
						.map(move |id| (id, slug.to_owned().into_boxed_str()))
				})
				.expect("sidebar item file name is not in format `<id>-<slug>.md`");

			let path = entry.path();
			let content = std::fs::read_to_string(&path).expect("reading sidebar item file");
			let (meta, content) = content
				.strip_prefix("<!--")
				.and_then(|content| content.split_once("-->"))
				.expect("sidebar item file does not start with an HTML comment");
			let (meta, content) = (meta.trim(), content.trim());

			let ItemMeta { name, description } =
				toml::from_str(meta).expect("sidebar item meta is not valid TOML");
			let content = content.to_owned().into_boxed_str();

			ExternalItem {
				id,
				slug,
				item: Item {
					name,
					description,
					content,
				},
			}
		})
		.collect();
	items.sort_unstable_by_key(|item| item.id);
	items
}

fn main() {
	for entry in std::fs::read_dir("dist").expect("reading dist directory") {
		let entry = entry.expect("reading dist directory entry").path();
		if entry.extension().map_or(false, |ext| ext == "html") {
			std::fs::remove_file(entry).expect("deleting HTML file in dist directory");
		}
	}

	std::fs::create_dir_all("dist").expect("creating dist directory");

	let mut index = open_output("index.html");

	let main_items = load_items("main");
	let programming_items = load_items("programming");
	let building_items = load_items("building");
	let sidebar_items = load_items("sidebar");

	IndexTemplate {
		main_items: &main_items,
		programming_items: &programming_items,
		building_items: &building_items,
		sidebar_items: &sidebar_items,
	}
	.render_into(&mut index)
	.expect("Rendering index template");

	for sidebar_item in &sidebar_items {
		let mut file = open_output(format!("{}.html", sidebar_item.slug));
		SidebarPageTemplate { item: sidebar_item }
			.render_into(&mut file)
			.expect("Rendering sidebar page template");
	}
}
