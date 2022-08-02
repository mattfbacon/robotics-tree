#![deny(
	absolute_paths_not_starting_with_crate,
	future_incompatible,
	keyword_idents,
	macro_use_extern_crate,
	meta_variable_misuse,
	missing_abi,
	missing_copy_implementations,
	non_ascii_idents,
	nonstandard_style,
	noop_method_call,
	pointer_structural_match,
	private_in_public,
	rust_2018_idioms
)]
#![warn(unused_qualifications)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

use std::path::Path;

use askama::Template as _;

mod render_markdown;

#[derive(askama::Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
	doc_root: &'a str,
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
	doc_root: &'a str,
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

fn open_output(name: impl AsRef<Path>) -> Writer<impl std::io::Write> {
	eprintln!("open_output on {:?}", name.as_ref());
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
	/// already rendered to HTML
	pub content: Box<str>,
}

struct ExternalItem {
	id: u32,
	slug: Box<str>,
	item: Item,
}

fn load_items(dir: &str) -> Vec<ExternalItem> {
	let mut items: Vec<_> = std::fs::read_dir(Path::new("items").join(dir))
		.expect("reading items directory")
		.into_iter()
		.map(|entry| entry.expect("reading item file"))
		.filter(|entry| entry.path().extension().map_or(false, |ext| ext == "md"))
		.map(|entry| {
			let path = entry.path();
			eprintln!("loading item {path:?}");

			let (id, slug) = entry
				.file_name()
				.to_str()
				.expect("item file name is invalid UTF-8")
				.split_once('-')
				.and_then(|(id, slug)| slug.strip_suffix(".md").map(move |slug| (id, slug)))
				.and_then(|(id, slug)| {
					id.parse()
						.ok()
						.map(move |id| (id, slug.to_owned().into_boxed_str()))
				})
				.expect("item file name is not in format `<id>-<slug>.md`");

			let input = std::fs::read_to_string(&path).expect("reading item file");
			let (content, ItemMeta { name, description }) = render_markdown::render(&input, dir);
			let content = content.into_boxed_str();

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
	let doc_root = std::env::var("DOC_ROOT").ok().unwrap_or("".to_owned());

	if Path::new("dist").exists() {
		std::fs::remove_dir_all("dist").expect("clearing dist directory");
	}
	dircpy::copy_dir("static", "dist").expect("copying static files to dist directory");

	let mut index = open_output("index.html");

	let main_items = load_items("main");
	let programming_items = load_items("programming");
	let building_items = load_items("building");
	let sidebar_items = load_items("sidebar");

	IndexTemplate {
		doc_root: &doc_root,
		main_items: &main_items,
		programming_items: &programming_items,
		building_items: &building_items,
		sidebar_items: &sidebar_items,
	}
	.render_into(&mut index)
	.expect("Rendering index template");

	for sidebar_item in &sidebar_items {
		let mut file = open_output(format!("{}.html", sidebar_item.slug));
		SidebarPageTemplate {
			doc_root: &doc_root,
			item: sidebar_item,
		}
		.render_into(&mut file)
		.expect("Rendering sidebar page template");
	}
}
