use std::fmt::{self, Display, Formatter};
use std::io::{Cursor, Read};
use std::sync::Arc;

pub type FailureChannel = std::sync::mpsc::SyncSender<FailedValidation>;

pub fn spawn_validation(
	content: String,
	in_file: Arc<str>,
	at_line: u32, // one indexed
	failure_channel: FailureChannel,
) {
	std::thread::spawn(move || validate(&content, in_file, at_line, &failure_channel));
}

pub struct FailedValidation {
	pub reason: String,
	pub in_file: Arc<str>,
	/// one indexed
	pub at_line: u32,
}

impl Display for FailedValidation {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
		let Self {
			reason,
			in_file,
			at_line,
		} = self;
		let reason = reason.trim();
		write!(
			formatter,
			"‼ validation failed for HTML fragment at line {at_line} of file {in_file:?}:\n‼ (line numbers within the error will be offset by 1)\n{reason}"
		)?;
		Ok(())
	}
}

fn validate(content: &str, in_file: Arc<str>, at_line: u32, failure_channel: &FailureChannel) {
	let reader = ureq::post("https://validator.w3.org/nu/?out=text")
		.set("Content-Type", "text/html; charset=utf-8")
		.send(doctor_content(content))
		.expect("sending validator request")
		.into_reader();
	let mut output = String::new();
	<Box<_> as Read>::take(reader, 1024 * 1024)
		.read_to_string(&mut output)
		.unwrap();
	if output.trim() != "The document validates according to the specified schema(s)." {
		failure_channel
			.send(FailedValidation {
				reason: output,
				in_file,
				at_line,
			})
			.unwrap();
	}
}

fn doctor_content(content: &str) -> impl Read + '_ {
	Cursor::new("<!DOCTYPE html><html lang=\"en\"><head><title>Fake Title</title></head><body>\n")
		.chain(Cursor::new(content))
		.chain(Cursor::new("\n</body></html>"))
}
