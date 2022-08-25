pub mod lexers;
pub use crate::lexers::{cpp, raw};

pub fn render_html(input: Vec<char>, lang: &str) -> String {
	match lang {
		"cpp" | "c++" => cpp::render_html(input),
		"" => raw::render_html(input),
		other => panic!(
			"unknown markdown language tag {other:?}. valid are c++/cpp, or empty for no highlighting"
		),
	}
}
