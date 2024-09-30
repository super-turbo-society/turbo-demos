use crate::GameState;

pub fn assess_current_line(state:&mut GameState) {
        match &state.lines[state.current_line] {
            line if line.starts_with("<<") || line.starts_with("#") || line == "" => {
                // is a passage, send, comment, or blank line, so increment on to next line
                state.current_line += 1;
            },
            line if line.starts_with(">>") => {
                // get divert text value
                let mut divert_text = state.lines[state.current_line].chars();
                divert_text.next();
                divert_text.next();
                // move to divert area!
                let new_knot_index: usize = state.lines
                    .iter()
                    .position(|line| *line == format!("{}{}", "<< ", divert_text.as_str().trim()))
                    .unwrap();
                state.current_line = new_knot_index;
            },
            line if line.starts_with("]>") => {
                // choice logic
                super::evaluate_choice(state);
            },
            line if line.starts_with("!") => {
                // command block
                super::evaluate_command(state);
            },
            line if line.starts_with("-- end") => {
                // set character to none
                state.speaking_char = 0;
                // reset tweens to zero
                state.tween_done_once = false;
                // set state to end state
                state.scene = 2;
            }
            _ => {
                // regular line
                super::print_current_line(state);
            },
        }
    }