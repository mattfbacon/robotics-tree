// ---- DON'T EDIT! THIS IS AUTO GENERATED CODE ---- //
use std::fmt::Write as _;

use crate::lexers::Token;

pub struct Lexer {
	input: Vec<char>,
	pub position: usize,
	pub read_position: usize,
	pub ch: char,
}

fn is_letter(ch: char) -> bool {
	ch.is_alphabetic() || ch == '_'
}

impl Lexer {
	pub fn new(input: Vec<char>) -> Self {
		Self {
			input,
			position: 0,
			read_position: 0,
			ch: '\0',
		}
	}

	pub fn read_char(&mut self) {
		if self.read_position >= self.input.len() {
			self.ch = '\0';
		} else {
			self.ch = self.input[self.read_position];
		}
		self.position = self.read_position;
		self.read_position += 1;
	}

	pub fn next_token(&mut self) -> Token {
		let read_identifier = |l: &mut Lexer| -> Vec<char> {
			let position = l.position;
			while l.position < l.input.len() && is_letter(l.ch) {
				l.read_char();
			}
			l.input[position..l.position].to_vec()
		};

		let read_string = |l: &mut Lexer, ch: char| -> Vec<char> {
			let position = l.position;
			l.read_char();
			while l.position < l.input.len() && l.ch != ch {
				if l.ch == '\\' {
					l.read_char();
				}
				l.read_char();
			}
			l.read_char();
			if l.position > l.input.len() {
				l.position -= 1;
				l.read_position -= 1;
			}
			l.input[position..l.position].to_vec()
		};

		let read_number = |l: &mut Lexer| -> Vec<char> {
			let position = l.position;
			while l.position < l.input.len() && l.ch.is_numeric() {
				l.read_char();
			}
			l.input[position..l.position].to_vec()
		};

		let tok: Token;
		if self.ch == '/' {
			let next_id = String::from("/*").chars().collect::<Vec<_>>();
			let next_position = self.position + next_id.len();
			let end_id = String::from("*/").chars().collect::<Vec<_>>();
			if self.position + next_id.len() < self.input.len()
				&& self.input[self.position..next_position] == next_id
			{
				let mut identifier = next_id.clone();
				next_id.iter().for_each(|_| self.read_char());
				let start_position = self.position;
				while self.position < self.input.len() {
					if self.ch == '*' {
						let end_position = self.position + end_id.len();
						if end_position <= self.input.len() && self.input[self.position..end_position] == end_id
						{
							end_id.iter().for_each(|_| self.read_char());
							break;
						}
					}
					self.read_char();
				}
				identifier.append(&mut self.input[start_position..self.position].to_vec());
				return Token::COMMENT(identifier);
			}
		}
		if self.read_position < self.input.len()
			&& self.ch == '/'
			&& self.input[self.read_position] == '/'
		{
			return Token::COMMENT(read_string(self, '\n'));
		}

		match self.ch {
			'\n' => {
				tok = Token::ENDL(self.ch);
			}
			'\0' => {
				tok = Token::EOF;
			}
			'0' => {
				return if self.input[self.read_position] == 'x' {
					let start_position = self.position;
					self.read_char();
					self.read_char();
					while self.position < self.input.len() && (self.ch.is_numeric() || is_letter(self.ch)) {
						self.read_char();
					}
					let hexadecimal = &self.input[start_position..self.position];
					Token::INT(hexadecimal.to_vec())
				} else {
					let number = read_number(self);
					Token::INT(number)
				}
			}
			'<' => {
				tok = Token::STRING(vec![self.ch]);
			}
			'>' => {
				tok = Token::STRING(vec![self.ch]);
			}
			_ => {
				return if is_letter(self.ch) {
					#[allow(unused_variables)]
					let start_position = self.position;
					#[allow(unused_mut)]
					let mut identifier: Vec<char> = read_identifier(self);
					if self.ch.is_numeric() {
						let position = self.position;
						while self.position < self.input.len() {
							if !self.ch.is_numeric() && !is_letter(self.ch) {
								break;
							}
							self.read_char();
						}
						identifier.append(&mut self.input[position..self.position].to_vec());
					}
					match get_keyword_token(&identifier) {
						Ok(keyword_token) => keyword_token,
						Err(_) => {
							if self.ch == '(' {
								return Token::ENTITY(identifier);
							} else if self.ch.is_whitespace() {
								let mut position = self.position;
								let mut ch = self.input[position];
								while position < self.input.len() && ch.is_whitespace() {
									position += 1;
									if position < self.input.len() {
										ch = self.input[position];
									}
								}
								if ch == '(' {
									return Token::ENTITY(identifier);
								}
							}
							Token::IDENT(identifier)
						}
					}
				} else if self.ch.is_numeric() {
					let identifier: Vec<char> = read_number(self);
					Token::INT(identifier)
				} else if self.ch == '\'' {
					let str_value: Vec<char> = read_string(self, '\'');
					Token::STRING(str_value)
				} else if self.ch == '"' {
					let str_value: Vec<char> = read_string(self, '"');
					Token::STRING(str_value)
				} else {
					Token::ILLEGAL
				}
			}
		}
		self.read_char();
		tok
	}
}

pub fn get_keyword_token(identifier: &[char]) -> Result<Token, String> {
	let id: String = identifier.iter().collect();
	match &id[..] {
		"true" | "false" | "this" | "nullptr" | "NULL" | "size_t" | "int64_t" | "uint32_t" => {
			Ok(Token::CONSTANT(identifier.to_owned()))
		}
		"asm" | "auto" | "bool" | "break" | "const" | "class" | "char" | "catch" | "constexpr"
		| "continue" | "default" | "define" | "delete" | "do" | "double" | "else" | "enum"
		| "extern" | "explicit" | "float" | "final" | "friend" | "for" | "if" | "inline" | "int"
		| "long" | "namespace" | "new" | "noexcept" | "return" | "override" | "operator"
		| "include" | "endif" | "public" | "private" | "protected" | "pragma" | "short" | "signed"
		| "sizeof" | "static" | "static_cast" | "struct" | "switch" | "template" | "typedef"
		| "typename" | "try" | "throw" | "using" | "union" | "unsigned" | "void" | "virtual"
		| "volatile" | "while" => Ok(Token::KEYWORD(identifier.to_owned())),
		_ => Err(String::from("Not a keyword")),
	}
}

pub fn render_html(input: Vec<char>) -> String {
	let mut l = Lexer::new(input);
	l.read_char();
	let mut html = String::new();
	let mut line = 1;
	html.push_str("<table class=\"highlight-table\">");
	html.push_str("<tbody>");
	html.push_str("<tr>");
	write!(
		html,
		"<td class=\"hl-line\" data-line=\"{line}\">{line}</td><td>",
	)
	.unwrap();

	loop {
		let token = l.next_token();
		if token == Token::EOF {
			html.push_str("</td></tr>");
			break;
		}

		match token {
			Token::INT(value) => {
				write!(
					html,
					"<span class=\"hl-c\">{}</span>",
					value.iter().collect::<String>()
				)
				.unwrap();
			}
			Token::IDENT(value) => {
				html.push_str(&value.iter().collect::<String>());
			}
			Token::STRING(value) => {
				let mut s = String::new();
				for ch in value {
					if ch == '<' {
						s.push_str("&lt;");
					} else if ch == '>' {
						s.push_str("&gt;");
					} else {
						s.push(ch);
					}
				}
				write!(html, "<span class=\"hl-s\">{s}</span>").unwrap();
			}
			Token::ENTITY(value) => {
				write!(
					html,
					"<span class=\"hl-en\">{}</span>",
					value.iter().collect::<String>()
				)
				.unwrap();
			}
			Token::CONSTANT(value) => {
				write!(
					html,
					"<span class=\"hl-c\">{}</span>",
					value.iter().collect::<String>()
				)
				.unwrap();
			}
			Token::KEYWORD(value) => {
				write!(
					html,
					"<span class=\"hl-k\">{}</span>",
					value.iter().collect::<String>()
				)
				.unwrap();
			}
			Token::COMMENT(value) => {
				let mut lines = String::new();
				for ch in value {
					if ch == '<' {
						lines.push_str("&lt;");
					} else if ch == '>' {
						lines.push_str("&gt;");
					} else {
						lines.push(ch);
					}
				}
				let split = lines.split('\n');
				let split_len = split.clone().count();
				let mut index = 0;
				for val in split {
					if val.len() > 1 {
						write!(html, "<span class=\"hl-cmt\">{val}</span>").unwrap();
					}
					index += 1;
					if index != split_len {
						line += 1;
						html.push_str("</td></tr>");
						write!(
							html,
							"<tr><td class=\"hl-line\" data-line=\"{line}\">{line}</td><td>",
						)
						.unwrap();
					}
				}
			}
			Token::ENDL(_) => {
				line += 1;
				html.push_str("</td></tr>");
				write!(
					html,
					"<tr><td class=\"hl-line\" data-line=\"{line}\">{line}</td><td>",
				)
				.unwrap();
			}
			_ => {
				html.push(l.ch);
				l.read_char();
			}
		}
	}

	html.push_str("</tbody>");
	html.push_str("</table>");
	html
}
