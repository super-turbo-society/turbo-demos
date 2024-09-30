use crate::GameState;

pub fn print_current_line(state:&mut GameState) {
    // split at the : to get character and line
    let statement: Vec<String> = state.lines[state.current_line]
        .split(":")
        .filter(|&element| element != "")
        .map(|element| element.trim().to_string())
        .collect();
    // draw char portrait
    match statement[0].as_str() {
        "NOAH" => state.speaking_char = 1,
        "MYLAN" => state.speaking_char = 2,
        _ => {},
    }
    
    // draw textbox
    super::textbox::render_textbox(&statement);
    
    // move this maybe into a bespoke input checker?
    if turbo::prelude::gamepad(0).start.just_pressed() {
        state.current_line += 1;
        state.tween_done_once = false;
    }
}