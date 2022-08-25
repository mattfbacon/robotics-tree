use super::{Token, TokenType};

#[derive(Copy, Clone)]
enum State<'a> {
	Normal,
	FindKeywordsUpToThen {
		first_yield: Option<Token<'a>>,
		up_to: usize,
		then: Option<SpecialStart>,
	},
	StringLiteral {
		delimiter: u8,
	},
	BlockComment,
	LineComment,
}

struct Lexer<'a> {
	input: &'a [u8],
	cursor: usize,
	state: State<'a>,
}

#[derive(Copy, Clone)]
enum SpecialStart {
	StringLiteral { delimiter: u8 },
	BlockComment,
	LineComment,
}

struct FindKeywordResult {
	start: usize,
	len: usize,
	ty: TokenType,
}

impl<'a> Lexer<'a> {
	fn remaining(&self) -> &'a [u8] {
		&self.input[self.cursor..]
	}

	fn next_byte(&mut self) -> Option<u8> {
		let ret = self.input.get(self.cursor).copied();
		if ret.is_some() {
			self.cursor += 1;
		}
		ret
	}

	fn is_first_identifier_char(ch: u8) -> bool {
		ch.is_ascii_alphabetic() || ch == b'_'
	}

	fn is_identifier_char(ch: u8) -> bool {
		ch.is_ascii_alphanumeric() || ch == b'_'
	}

	#[allow(clippy::range_plus_one)] // would imply the wrong thing
	fn read_number(haystack: &[u8]) -> &[u8] {
		fn inner(haystack: &[u8]) -> &[u8] {
			let mut iter = haystack.iter().copied();
			match iter.next() {
				Some(b'0') => match iter.next() {
					Some(b'x' | b'X') => {
						let num_hex_digits = iter
							.take_while(|&ch| ch.is_ascii_hexdigit() || ch == b'\'')
							.count();
						&haystack[..num_hex_digits + 2]
					}
					Some(other) if b"01234567\'".contains(&other) => {
						let num_rest_octal_digits = iter.take_while(|ch| b"01234567\'".contains(ch)).count();
						&haystack[..num_rest_octal_digits + 2]
					}
					_ => &haystack[..1],
				},
				Some(b'1'..=b'9' | b'.') => {
					let num_rest_digits = iter
						.take_while(|&ch| matches!(ch, b'0'..=b'9' | b'\'' | b'.' | b'e'))
						.count();
					&haystack[..1 + num_rest_digits]
				}
				_ => &[],
			}
		}
		let matched_len = inner(haystack).len();
		if matched_len == 0 {
			&[]
		} else {
			let mut iter = haystack[matched_len..].iter().copied();
			match iter.next() {
				Some(b'u' | b'U') => match iter.next() {
					Some(b'l' | b'L') => match iter.next() {
						Some(b'l' | b'L') => &haystack[..matched_len + 3],
						_ => &haystack[..matched_len + 2],
					},
					_ => &haystack[..matched_len + 1],
				},
				Some(b'l' | b'L') => match iter.next() {
					Some(b'l' | b'L') => &haystack[..matched_len + 2],
					_ => &haystack[..matched_len + 1],
				},
				_ => &haystack[..matched_len],
			}
		}
	}

	fn read_identifier(haystack: &[u8]) -> &[u8] {
		if let Some(&first) = haystack.first() {
			if !Self::is_first_identifier_char(first) {
				return &[];
			}

			let num_continue = haystack
				.iter()
				.skip(1)
				.take_while(|&&ch| Self::is_identifier_char(ch))
				.count();
			#[allow(clippy::range_plus_one)] // would imply the wrong thing
			&haystack[..1 + num_continue]
		} else {
			&[]
		}
	}

	const KEYWORDS: &'static [(TokenType, &'static [&'static [u8]])] = &[
		(
			TokenType::Constant,
			&[b"true", b"false", b"this", b"nullptr", b"NULL"],
		),
		(
			TokenType::Keyword,
			&[
				b"asm",
				b"auto",
				b"bool",
				b"break",
				b"const",
				b"class",
				b"char",
				b"catch",
				b"constexpr",
				b"continue",
				b"default",
				b"define",
				b"delete",
				b"do",
				b"double",
				b"else",
				b"enum",
				b"extern",
				b"explicit",
				b"float",
				b"final",
				b"friend",
				b"for",
				b"if",
				b"inline",
				b"int",
				b"long",
				b"namespace",
				b"new",
				b"noexcept",
				b"return",
				b"override",
				b"operator",
				b"include",
				b"endif",
				b"public",
				b"private",
				b"protected",
				b"pragma",
				b"short",
				b"signed",
				b"sizeof",
				b"static",
				b"static_cast",
				b"struct",
				b"switch",
				b"template",
				b"typedef",
				b"typename",
				b"try",
				b"throw",
				b"using",
				b"union",
				b"unsigned",
				b"void",
				b"virtual",
				b"volatile",
				b"while",
			],
		),
	];

	fn find_keyword(haystack: &[u8]) -> Option<FindKeywordResult> {
		let mut idx = 0;

		while idx < haystack.len() {
			let slice = &haystack[idx..];

			if let Some(b'#') = slice.first().copied() {
				let identifier = Self::read_identifier(&slice[1..]);
				if !identifier.is_empty() {
					return Some(FindKeywordResult {
						start: idx,
						len: 1 + identifier.len(),
						ty: TokenType::Keyword,
					});
				}
			}

			// these operators can amalgamate together, so not all are listed. for example `*` + `=` = `*=`.
			if let Some(
				b'~' | b'!' | b'-' | b'+' | b'&' | b'*' | b'/' | b'%' | b'^' | b'|' | b':' | b'=',
			) = slice.first().copied()
			{
				return Some(FindKeywordResult {
					start: idx,
					len: 1,
					ty: TokenType::Operator,
				});
			}

			if let Some(access) = [b".*" as &[u8], b"->*", b".", b"->"]
				.into_iter()
				.find(|operator| slice.starts_with(operator))
			{
				let next = access.len();
				let identifier = Self::read_identifier(&slice[next..]);
				if !identifier.is_empty() {
					let ty = if slice.get(next + identifier.len()).copied() == Some(b'(') {
						TokenType::MethodCall
					} else {
						TokenType::Field
					};

					return Some(FindKeywordResult {
						start: idx + next,
						len: identifier.len(),
						ty,
					});
				}
			}

			let identifier = Self::read_identifier(slice);
			if !identifier.is_empty() {
				for &(category, members) in Self::KEYWORDS.iter() {
					if let Some(matched) = members.iter().find(|&&member| identifier == member) {
						return Some(FindKeywordResult {
							start: idx,
							len: matched.len(),
							ty: category,
						});
					}
				}
				idx += identifier.len();
				continue;
			}

			let number = Self::read_number(slice);
			if !number.is_empty() {
				return Some(FindKeywordResult {
					start: idx,
					len: number.len(),
					ty: TokenType::NumberLiteral,
				});
			}

			idx += 1;
		}

		None
	}

	fn find_special_start(haystack: &[u8]) -> Option<(usize, SpecialStart)> {
		let mut iter = haystack.iter().copied().enumerate();

		while let Some((idx, byte)) = iter.next() {
			match byte {
				// treat character literals as string literals for simplicity
				b'"' | b'\'' => return Some((idx, SpecialStart::StringLiteral { delimiter: byte })),
				b'/' => match iter.next()?.1 {
					b'/' => return Some((idx, SpecialStart::LineComment)),
					b'*' => return Some((idx, SpecialStart::BlockComment)),
					_ => continue,
				},
				_ => continue,
			}
		}

		None
	}

	fn next_normal(&mut self) -> Token<'a> {
		let remaining = self.remaining();
		match Self::find_special_start(remaining) {
			Some((inner_start, special_type)) => {
				self.next_find_keywords(None, self.cursor + inner_start, Some(special_type))
			}
			None => self.next_find_keywords(None, self.input.len(), None),
		}
	}

	fn next_string_literal(&mut self, delimiter: u8) -> Token<'a> {
		self.state = State::Normal;

		// account for starting quotation mark
		let start = self.cursor;
		self.cursor += 1;

		while let Some(byte) = self.next_byte() {
			match byte {
				b'\\' => {
					self.next_byte();
				}
				end if end == delimiter => break,
				_ => continue,
			}
		}

		Token {
			ty: Some(TokenType::StringOrCharLiteral),
			text: &self.input[start..self.cursor],
		}
	}

	fn next_line_comment(&mut self) -> Token<'a> {
		self.state = State::Normal;

		// account for starting `//`
		let start = self.cursor;
		self.cursor += 2;

		while let Some(byte) = self.next_byte() {
			match byte {
				b'\n' => break,
				_ => continue,
			}
		}

		Token {
			ty: Some(TokenType::Comment),
			text: &self.input[start..self.cursor],
		}
	}

	fn next_block_comment(&mut self) -> Token<'a> {
		self.state = State::Normal;

		// account for starting `/*`
		let start = self.cursor;
		self.cursor += 2;

		while let Some(byte) = self.next_byte() {
			match byte {
				b'*' => match self.next_byte() {
					Some(b'/') => break,
					_ => continue,
				},
				_ => continue,
			}
		}

		Token {
			ty: Some(TokenType::Comment),
			text: &self.input[start..self.cursor],
		}
	}

	fn next_find_keywords(
		&mut self,
		first_yield: Option<Token<'a>>,
		up_to: usize,
		then: Option<SpecialStart>,
	) -> Token<'a> {
		if let Some(first_yield) = first_yield {
			self.state = State::FindKeywordsUpToThen {
				first_yield: None,
				up_to,
				then,
			};
			return first_yield;
		}

		macro_rules! done {
			() => {
				self.cursor = up_to;
				self.state = match then {
					Some(SpecialStart::StringLiteral { delimiter }) => State::StringLiteral { delimiter },
					Some(SpecialStart::BlockComment) => State::BlockComment,
					Some(SpecialStart::LineComment) => State::LineComment,
					None => State::Normal,
				};
			};
		}

		let remaining = &self.input[self.cursor..up_to];
		if let Some(FindKeywordResult { start, len, ty }) = Self::find_keyword(remaining) {
			self.cursor += start + len;
			let up_to_keyword_token = Token {
				ty: None,
				text: &remaining[..start],
			};
			let keyword_token = Token {
				ty: Some(ty),
				text: &remaining[start..start + len],
			};
			self.state = State::FindKeywordsUpToThen {
				first_yield: Some(keyword_token),
				up_to,
				then,
			};
			up_to_keyword_token
		} else {
			done!();
			Token {
				ty: None,
				text: remaining,
			}
		}
	}
}

impl<'a> Iterator for Lexer<'a> {
	type Item = Token<'a>;

	fn next(&mut self) -> Option<Token<'a>> {
		if self.remaining().is_empty() {
			None
		} else {
			Some(match self.state {
				State::Normal => self.next_normal(),
				State::FindKeywordsUpToThen {
					first_yield,
					up_to,
					then,
				} => self.next_find_keywords(first_yield, up_to, then),
				State::StringLiteral { delimiter } => self.next_string_literal(delimiter),
				State::LineComment => self.next_line_comment(),
				State::BlockComment => self.next_block_comment(),
			})
		}
	}
}

pub fn lex(input: &[u8]) -> impl Iterator<Item = Token<'_>> {
	Lexer {
		input,
		cursor: 0,
		state: State::Normal,
	}
}
