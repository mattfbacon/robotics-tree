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
#![warn(clippy::pedantic, missing_docs)]
#![allow(clippy::let_underscore_drop, clippy::missing_errors_doc)]
#![forbid(unsafe_code)]

use std::fmt::Write;

mod lexers;

use lexers::{cpp, raw, Token};

#[derive(Debug, Clone, Copy)]
pub enum Language {
	CPlusPlus,
	None,
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("invalid language tag: valid tags are `cpp`/`c++` or an empty tag for no highlighting.")]
pub struct LanguageFromStrError;

impl std::str::FromStr for Language {
	type Err = LanguageFromStrError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"cpp" | "c++" => Ok(Self::CPlusPlus),
			"" => Ok(Self::None),
			_ => Err(LanguageFromStrError),
		}
	}
}

pub fn render_html(buf: &mut dyn std::fmt::Write, input: &str, lang: Language) -> std::fmt::Result {
	match lang {
		Language::CPlusPlus => render_html_inner(buf, cpp::lex(input.as_bytes())),
		Language::None => render_html_inner(buf, raw::lex(input.as_bytes())),
	}
}

fn escape(inner: &str) -> askama_escape::Escaped<'_, askama_escape::Html> {
	askama_escape::escape(inner, askama_escape::Html)
}

fn render_html_inner<'a>(
	buf: &mut dyn Write,
	tokens: impl Iterator<Item = Token<'a>>,
) -> std::fmt::Result {
	let mut line_number = 1;

	macro_rules! start_line {
		() => {
			write!(buf, "<tr><td class=\"hl-line\">{line_number}</td><td>")?;
		};
	}
	macro_rules! end_line {
		() => {
			write!(buf, "</td></tr>")?;
			line_number += 1;
		};
	}
	macro_rules! start_wrap {
		($token:ident) => {
			if let Some(ty) = $token.ty {
				write!(buf, "<span class=\"hl-{}\">", ty.class())?;
			}
		};
	}
	macro_rules! end_wrap {
		($token:ident) => {
			if $token.ty.is_some() {
				write!(buf, "</span>")?;
			}
		};
	}

	write!(
		buf,
		"<div class=\"highlight-container\"><table class=\"highlight-table\"><tbody>"
	)?;
	start_line!();
	for token in tokens {
		let text = std::str::from_utf8(token.text).unwrap();
		let mut lines = text.split('\n');
		start_wrap!(token);
		write!(buf, "{}", escape(lines.next().unwrap()))?;
		for line in lines {
			end_wrap!(token);
			end_line!();
			start_line!();
			start_wrap!(token);
			write!(buf, "{}", escape(line))?;
		}
		end_wrap!(token);
	}
	end_line!();
	write!(buf, "</tbody></table></div>")?;
	Ok(())
}

/*
pub fn render_html(buf: &mut dyn Write, input: &str) -> Result {
	write!(buf, "<table class=\"highlight-table\"><tbody>")?;

	for (line_number, line) in input
		.split('\n')
		.enumerate()
		.map(|(line_number, line)| (line_number + 1, line))
	{
		write!(
			buf,
			"<tr><td class=\"hl-line\" data-line=\"{line_number}\">{line_number}</td><td>{}</td></tr>",
			escape(line),
		)?;
	}

	write!(buf, "</tbody></table>")?;

	Ok(())
}
*/
