use crate::GameState;

pub fn evaluate_choice(state:&mut GameState) {
    // split the current line at the ]>
    let choices: Vec<String> = state.lines[state.current_line]
        .split("]>")
        .filter(|&element| element != "")
        .map(|choice| choice.trim().to_string())
        .collect();
    
    // set speaking character to none for choices
    state.speaking_char = 0;
    
    super::textbox::render_choice_textbox(&choices);
    
    // look forward to next line, split at >> to get diverts
    let diverts: Vec<String> = state.lines[state.current_line + 1]
        .split(">>")
        .filter(|&element| element != "")
        .map(|divert| divert.trim().to_string())
        .collect();
    
    // NUMBER OF CHOICES DETERMINES NUMBER OF IF STATEMENTS?

    // do input check for left or right
    if turbo::prelude::gamepad(0).left.just_pressed() && choices.len() >= 1 {
        if diverts[0] == "NULL" {
            // pass entirely
            return;
        }
        // search the full script to see where << that is, get that index, set current line to that
        let new_knot_index: usize = state.lines
            .iter()
            .position(|line| *line ==  format!("{}{}", "<< ", diverts[0]))
            .unwrap();
        
        state.current_line = new_knot_index;

        state.tween_done_once = false;
    }
    else if turbo::prelude::gamepad(0).right.just_pressed() && choices.len() >= 2 {
        if diverts[1] == "NULL" {
            // pass entirely
            return;
        }
        // search the full script to see where << that is, get that index, set current line to that
        let new_knot_index: usize = state.lines
            .iter()
            .position(|line| *line ==  format!("{}{}", "<< ", diverts[1]))
            .unwrap();
        
        state.current_line = new_knot_index;

        state.tween_done_once = false;
    }
    else if turbo::prelude::gamepad(0).up.just_pressed() && choices.len() >= 3 {
        if diverts[2] == "NULL" {
            // pass entirely
            return;
        }
        // search the full script to see where << that is, get that index, set current line to that
        let new_knot_index: usize = state.lines
            .iter()
            .position(|line| *line ==  format!("{}{}", "<< ", diverts[2]))
            .unwrap();
        
        state.current_line = new_knot_index;

        state.tween_done_once = false;
    }
    else if turbo::prelude::gamepad(0).down.just_pressed() && choices.len() >= 4 {
        if diverts[3] == "NULL" {
            // pass entirely
            return;
        }
        // search the full script to see where << that is, get that index, set current line to that
        let new_knot_index: usize = state.lines
            .iter()
            .position(|line| *line ==  format!("{}{}", "<< ", diverts[3]))
            .unwrap();
        
        state.current_line = new_knot_index;

        state.tween_done_once = false;
    }
}