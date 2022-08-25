use super::Token;

pub fn lex(input: &[u8]) -> impl Iterator<Item = Token<'_>> {
	std::iter::once(Token {
		ty: None,
		text: input,
	})
}
