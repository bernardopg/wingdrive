use std::io::{self, BufRead, Write};

use anyhow::Result;
use wing_core::infra::action::ConfirmationRequest;

/// Prompt the user for confirmation before executing a dangerous action.
///
/// Behavior:
/// - Returns Ok(()) if the user confirms with "y" or "yes" (case-insensitive)
/// - Respects an explicit `assume_yes` flag to skip prompting
/// - Also respects `WING_CLI_YES=1` environment variable to skip prompting
/// - Otherwise returns an error ("Aborted by user") to allow early exit
pub fn confirm_or_abort(prompt: &str, assume_yes: bool) -> Result<()> {
	if assume_yes
		|| std::env::var("WING_CLI_YES")
			.map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
			.unwrap_or(false)
	{
		return Ok(());
	}

	confirm_with(prompt, &mut io::stdin().lock(), &mut io::stderr().lock())
}

fn confirm_with(prompt: &str, input: &mut impl BufRead, output: &mut impl Write) -> Result<()> {
	writeln!(output, "{prompt} [y/N]: ")?;
	let response = read_response(input, output)?;
	if response.eq_ignore_ascii_case("y") || response.eq_ignore_ascii_case("yes") {
		Ok(())
	} else {
		anyhow::bail!("Aborted by user")
	}
}

fn read_response(input: &mut impl BufRead, output: &mut impl Write) -> Result<String> {
	output.flush()?;
	let mut line = String::new();
	if input.read_line(&mut line)? == 0 {
		anyhow::bail!("Aborted by user: end of input");
	}
	Ok(line.trim().to_owned())
}

/// Prompts for a numbered choice and returns its zero-based index.
pub fn prompt_for_choice(request: ConfirmationRequest) -> Result<usize> {
	prompt_for_list(&request.message, &request.choices)
}

/// Prompts for text and returns an empty optional response as None.
pub fn prompt_for_text(prompt: &str, optional: bool) -> Result<Option<String>> {
	text_with(
		prompt,
		optional,
		&mut io::stdin().lock(),
		&mut io::stdout().lock(),
	)
}

fn text_with(
	prompt: &str,
	optional: bool,
	input: &mut impl BufRead,
	output: &mut impl Write,
) -> Result<Option<String>> {
	let qualifier = if optional { " (optional)" } else { "" };
	loop {
		write!(output, "{prompt}{qualifier}: ")?;
		let response = read_response(input, output)?;
		if !response.is_empty() {
			return Ok(Some(response));
		}
		if optional {
			return Ok(None);
		}
		writeln!(output, "This field is required. Please enter a value.")?;
	}
}

/// Prompts for a numbered choice and returns its zero-based index.
pub fn prompt_for_list(message: &str, choices: &[String]) -> Result<usize> {
	list_with(
		message,
		choices,
		&mut io::stdin().lock(),
		&mut io::stdout().lock(),
	)
}

fn list_with(
	message: &str,
	choices: &[String],
	input: &mut impl BufRead,
	output: &mut impl Write,
) -> Result<usize> {
	anyhow::ensure!(!choices.is_empty(), "No choices available");
	writeln!(output, "{message}")?;
	for (index, choice) in choices.iter().enumerate() {
		writeln!(output, "  [{}]: {choice}", index + 1)?;
	}
	loop {
		write!(output, "Please select an option (1-{}): ", choices.len())?;
		if let Some(number) = read_response(input, output)?
			.parse::<usize>()
			.ok()
			.filter(|number| (1..=choices.len()).contains(number))
		{
			return Ok(number - 1);
		}
		writeln!(
			output,
			"Invalid input. Please enter a number between 1 and {}.",
			choices.len()
		)?;
	}
}

/// Modern arrow-key based selection with inquire.
/// Supports arrow keys (↑↓) and number shortcuts (1, 2, 3...).
pub fn select(message: &str, choices: &[String]) -> Result<usize> {
	use inquire::Select;

	let selection = Select::new(message, choices.to_vec())
		.with_page_size(10)
		.with_help_message("Use ↑↓ to navigate, Enter to select, or type a number")
		.prompt()?;

	// Find the index of the selected item
	let idx = choices
		.iter()
		.position(|c| c == &selection)
		.ok_or_else(|| anyhow::anyhow!("Selection not found"))?;

	Ok(idx)
}

/// Modern text input with inquire.
pub fn text(message: &str, optional: bool) -> Result<Option<String>> {
	use inquire::Text;

	let prompt_text = if optional {
		format!("{} (optional)", message)
	} else {
		message.to_string()
	};

	let mut prompt = Text::new(&prompt_text);

	if !optional {
		prompt = prompt.with_validator(|input: &str| {
			if input.trim().is_empty() {
				Ok(inquire::validator::Validation::Invalid(
					"This field is required".into(),
				))
			} else {
				Ok(inquire::validator::Validation::Valid)
			}
		});
	}

	let result = prompt.prompt()?;

	if result.trim().is_empty() && optional {
		Ok(None)
	} else {
		Ok(Some(result))
	}
}

/// Modern password input with inquire (hides input).
pub fn password(message: &str, optional: bool) -> Result<Option<String>> {
	use inquire::Password;

	let prompt_text = if optional {
		format!("{} (optional)", message)
	} else {
		message.to_string()
	};

	let mut prompt = Password::new(&prompt_text).without_confirmation();

	if !optional {
		prompt = prompt.with_validator(|input: &str| {
			if input.trim().is_empty() {
				Ok(inquire::validator::Validation::Invalid(
					"This field is required".into(),
				))
			} else {
				Ok(inquire::validator::Validation::Valid)
			}
		});
	}

	let result = prompt.prompt()?;

	if result.trim().is_empty() && optional {
		Ok(None)
	} else {
		Ok(Some(result))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::io::Cursor;

	#[test]
	fn accepts_affirmative_answers_and_rejects_decline() {
		for (answer, expected) in [
			(" YES \n", true),
			("y\n", true),
			("no\n", false),
			("\n", false),
		] {
			let mut output = Vec::new();
			let result = confirm_with("Continue?", &mut Cursor::new(answer), &mut output);
			assert_eq!(result.is_ok(), expected);
			assert_eq!(output, b"Continue? [y/N]: \n");
		}
		assert!(confirm_or_abort("Continue?", true).is_ok());
	}

	#[test]
	fn closed_input_aborts_choice_and_text_prompts() {
		let choices = ["First".into(), "Second".into()];
		let end_of_input = "Aborted by user: end of input";
		let mut output = Vec::new();
		assert_eq!(
			list_with("Pick", &choices, &mut Cursor::new("bad\n"), &mut output)
				.unwrap_err()
				.to_string(),
			end_of_input
		);
		for optional in [true, false] {
			assert_eq!(
				text_with("Name", optional, &mut Cursor::new(""), &mut output)
					.unwrap_err()
					.to_string(),
				end_of_input
			);
		}
		assert_eq!(
			confirm_with("Continue?", &mut Cursor::new(""), &mut output)
				.unwrap_err()
				.to_string(),
			end_of_input
		);
	}

	#[test]
	fn retries_invalid_choices_and_preserves_list_indices() {
		let choices = ["Same".into(), "Same".into()];
		let mut output = Vec::new();
		assert_eq!(
			list_with(
				"Pick",
				&choices,
				&mut Cursor::new("0\n3\nx\n 2 \n"),
				&mut output
			)
			.unwrap(),
			1
		);
		let output = String::from_utf8(output).unwrap();
		assert!(output.contains("[1]: Same"));
		assert!(output.contains("[2]: Same"));
		assert_eq!(output.matches("Invalid input").count(), 3);
		assert_eq!(
			list_with("Pick", &[], &mut Cursor::new("1"), &mut Vec::new())
				.unwrap_err()
				.to_string(),
			"No choices available"
		);
	}

	#[test]
	fn trims_text_and_distinguishes_blank_from_eof() {
		let mut output = Vec::new();
		assert_eq!(
			text_with("Name", false, &mut Cursor::new("\n  Ana \n"), &mut output).unwrap(),
			Some("Ana".into())
		);
		assert!(String::from_utf8(output)
			.unwrap()
			.contains("This field is required"));
		assert_eq!(
			text_with("Name", true, &mut Cursor::new(" \n"), &mut Vec::new()).unwrap(),
			None
		);
	}

	#[test]
	fn propagates_input_error() {
		struct FailedRead;
		impl io::Read for FailedRead {
			fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
				Err(io::Error::other("fixture read failure"))
			}
		}
		let mut input = io::BufReader::new(FailedRead);
		let error = read_response(&mut input, &mut Vec::new()).unwrap_err();
		assert_eq!(error.to_string(), "fixture read failure");
	}
}
