use askama::Template as _;
use items::Item;

mod items;

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

	pub fn slug(s: &str) -> ::askama::Result<String> {
		Ok(s.to_lowercase().replace(' ', "-"))
	}
}

#[derive(askama::Template)]
#[template(path = "index.html")]
struct IndexTemplate {
	main_items: &'static [Item],
	programming_items: &'static [Item],
	building_items: &'static [Item],
	sidebar_items: &'static [Item],
}

impl IndexTemplate {
	fn iterate_all_tagged(&self) -> impl Iterator<Item = (&'static str, usize, &'static Item)> {
		macro_rules! make_iter {
			($name:expr, $inner:expr) => {
				$inner
					.into_iter()
					.enumerate()
					.map(move |(index, item)| ($name, index, item))
			};
		}

		make_iter!("main", self.main_items)
			.chain(make_iter!("programming", self.programming_items))
			.chain(make_iter!("building", self.building_items))
	}
}

#[derive(askama::Template)]
#[template(path = "sidebar_page.html")]
struct SidebarPageTemplate {
	item: &'static Item,
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
		.open(<_ as AsRef<std::path::Path>>::as_ref("dist").join(name.as_ref()))
		.expect("Opening output file");
	Writer(writer)
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

	IndexTemplate {
		main_items: items::MAIN_ITEMS,
		programming_items: items::PROGRAMMING_ITEMS,
		building_items: items::BUILDING_ITEMS,
		sidebar_items: items::SIDEBAR_ITEMS,
	}
	.render_into(&mut index)
	.expect("Rendering index template");

	for sidebar_item in items::SIDEBAR_ITEMS {
		let mut file = open_output(format!(
			"{}.html",
			filters::slug(sidebar_item.name).unwrap()
		));
		SidebarPageTemplate { item: sidebar_item }
			.render_into(&mut file)
			.expect("Rendering sidebar page template");
	}
}
