use turbo::*;
use turbo::prelude::Font;

// consts for textboxes related
//const TB_WIDTH: u16 = 384;
//const TB_HEIGHT: u16 = 174;
const TB_X: u16 = 0;
const TB_Y: u16 = 174;
const TB_PADDING: u16 = 8;
const CHOICE_OPTIONS: (&str, &str, &str, &str,) = ("<", ">", "^", "v");

pub fn render_textbox(dialogue: &Vec<String>) -> bool {
	//rect!(w = TB_WIDTH, h = TB_HEIGHT, x = TB_X, y = TB_Y, color = 0x000000ff);
	
	text!(
		dialogue[1].as_str(),
		x = TB_X + 2 * TB_PADDING,
		y = TB_Y + TB_PADDING
	);
	
	// its rendering it
	true
}

pub fn render_choice_textbox(choices: &Vec<String>) -> bool {
	choices.iter().enumerate().for_each(|(index, choice)| {
		
		let choice_sanitized: &str;
		if choice.starts_with("~") {
			// remove the ~ for display
			choice_sanitized = &choice[1..choice.len()];
		}
		else {
			choice_sanitized = choice;
		}
		
		match index {
			0 => {
				text!(
					format!("{} {}", CHOICE_OPTIONS.0, choice_sanitized).as_str(),
					x = TB_X + 4 + TB_PADDING,
					y = TB_Y + TB_PADDING
				);
				
				if choice.starts_with("~") {
					path!(
						start = (TB_X + TB_PADDING, TB_Y + TB_PADDING + 3), 
						end = (384 / 2 - TB_PADDING, TB_Y + TB_PADDING + 3),
						width = 1,
						color = 0xffffff99,
					);
				}
				
			},
			1 => {
				text!(
					format!("{} {}", CHOICE_OPTIONS.1, choice_sanitized).as_str(),
					x = TB_X + 4 + TB_PADDING,
					y = TB_Y + 16 + TB_PADDING
				);
				
				if choice.starts_with("~") {
					path!(
						start = (TB_X + TB_PADDING, TB_Y + 16 + TB_PADDING + 3), 
						end = (384 / 2 - TB_PADDING, TB_Y + 16 + TB_PADDING + 3),
						width = 1,
						color = 0xffffff99,
					);
				}
				
			},
			2 => {
				text!(
					format!("{} {}", CHOICE_OPTIONS.2, choice_sanitized).as_str(),
					x = 384 / 2 + TB_PADDING, 
					y = TB_Y + TB_PADDING
				);
				
				if choice.starts_with("~") {
					path!(
						start = (384 / 2 + TB_PADDING, TB_Y + TB_PADDING + 3), 
						end = (384 - TB_PADDING, TB_Y + TB_PADDING + 3),
						width = 1,
						color = 0xffffff99,
					);
				}
			},
			3 => {
				text!(
					format!("{} {}", CHOICE_OPTIONS.3, choice_sanitized).as_str(),
					x = 384 / 2 + TB_PADDING, 
					y = TB_Y + 16 + TB_PADDING
				);
				
				if choice.starts_with("~") {
					path!(
						start = (384 / 2 + TB_PADDING, TB_Y + 16 + TB_PADDING + 3), 
						end = (384 - TB_PADDING, TB_Y + 16 + TB_PADDING + 3),
						width = 1,
						color = 0xffffff99,
					);
				}
			},
			_ => {panic!("CRITICAL: Number of choices exceeds allowed 4.")},
		}
	});
	
	true
}