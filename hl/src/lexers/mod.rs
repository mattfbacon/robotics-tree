pub mod cpp;
pub mod raw;

/*
#[derive(PartialEq, Eq, Debug)]
pub enum Token {
	ILLEGAL,
	EOF,
	ENDL(u8),
	CH(u8),
	HEAD(String),
	IDENT(String),
	CONSTANT(String),
	INT(String),
	ENTITYTAG(String),
	COMMENT(String),
	ENTITY(String),
	STRING(String),
	KEYWORD(String),
	VAR(String),
}
*/

#[derive(Debug, Clone, Copy)]
pub enum TokenType {
	Field,
	MethodCall,
	Operator,
	Keyword,
	Comment,
	StringOrCharLiteral,
	Constant,
	NumberLiteral,
}

impl TokenType {
	pub fn class(self) -> &'static str {
		match self {
			Self::Field => "field",
			Self::MethodCall => "method-call",
			Self::Operator => "operator",
			Self::Keyword => "keyword",
			Self::Comment => "comment",
			Self::StringOrCharLiteral => "string-or-char-literal",
			Self::Constant => "constant",
			Self::NumberLiteral => "number-literal",
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub struct Token<'a> {
	pub ty: Option<TokenType>,
	pub text: &'a [u8],
}
