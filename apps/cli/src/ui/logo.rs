/// Palette for [`LOGO`], indexed by its digits.
const PALETTE: [(u8, u8, u8); 6] = [
	(20, 14, 41),
	(15, 10, 31),
	(46, 33, 100),
	(79, 56, 166),
	(77, 55, 168),
	(103, 74, 222),
];

/// `.github/logo.svg` rendered at 32x32 px; `.` is transparent.
const LOGO: [&str; 32] = [
	".....0111111111111111111110.....",
	"...11111111111111111111111111...",
	"..1111111111111111111111111111..",
	".111111111111111111111111111111.",
	".111111111111111111111111111111.",
	"01111111111111111111111111111110",
	"11111111111111111111102334111111",
	"11111111111111111123555552111111",
	"11111111111111112355555541111111",
	"11111111111111235555555521111111",
	"11111111111113555555555211111111",
	"11111111111035555555554111111111",
	"11111111110555555555542421111111",
	"11111111105555555555534401111111",
	"11111111055555555554444211111111",
	"11111111355555555544442111111111",
	"11111114555555553444421111111111",
	"11111125555553344444210111111111",
	"11111145424334444442222011111111",
	"11111021123344444422220111111111",
	"11111111234444442222201111111111",
	"11111112444222222222011111111111",
	"11111102200222222220111111111111",
	"11111111102222222011111111111111",
	"11111111022222011111111111111111",
	"11111110220111111111111111111111",
	"01111111111111111111111111111110",
	".111111111111111111111111111111.",
	".111111111111111111111111111111.",
	"..1111111111111111111111111111..",
	"...11111111111111111111111111...",
	".....0111111111111111111110.....",
];

/// RGB colour of one logo pixel, or `None` where the logo is transparent.
fn pixel(x: usize, y: usize) -> Option<(u8, u8, u8)> {
	let digit = LOGO[y].as_bytes()[x].checked_sub(b'0')?;
	PALETTE.get(digit as usize).copied()
}

/// Print the WingDrive logo using ANSI colors and Unicode half-blocks
pub fn print_logo_colored() {
	println!();

	// Each character cell holds two pixel rows: the top one as foreground, the bottom as background
	for row in (0..LOGO.len()).step_by(2) {
		print!("                    ");
		for col in 0..LOGO[row].len() {
			match (pixel(col, row), pixel(col, row + 1)) {
				(Some((r, g, b)), Some((br, bg, bb))) => {
					print!("\x1b[38;2;{r};{g};{b}m\x1b[48;2;{br};{bg};{bb}m▀\x1b[0m");
				}
				(Some((r, g, b)), None) => print!("\x1b[38;2;{r};{g};{b}m▀\x1b[0m"),
				(None, Some((r, g, b))) => print!("\x1b[38;2;{r};{g};{b}m▄\x1b[0m"),
				(None, None) => print!(" "),
			}
		}
		println!();
	}

	println!();
	println!("                                WINGDRIVE");
	println!();
}

/// Display a compact version of the logo
pub fn print_compact_logo() {
	println!("WingDrive CLI v2");
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn logo_rows_are_square_and_use_known_colors() {
		for row in LOGO {
			assert_eq!(row.len(), LOGO.len());
			assert!(row
				.bytes()
				.all(|b| b == b'.' || ((b - b'0') as usize) < PALETTE.len()));
		}
	}
}
