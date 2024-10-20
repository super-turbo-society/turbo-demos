use crate::GameState;

pub fn evaluate_command(state:&mut GameState) {
        // no one talking!
        state.speaking_char = 0;
        // increment wait time, like a local tick basically
        state.wait_timer += 1;

        // split at / for time
        let command_with_arg: Vec<String> = state.lines[state.current_line]
            .split("/")
            .filter(|&element| element != "")
            .map(|element| element.trim().to_string())
            .collect();
        let (command, arg) = (&command_with_arg[0], &command_with_arg[1]);

        match command.as_str() {
            "! WAIT" => {
                // dont increment the line for a period of time
                if state.wait_timer == arg.parse::<u16>().unwrap() * 60 {
                    state.current_line += 1;
                    // reset wait_timer
                    state.wait_timer = 0;

                    state.tween_done_once = false;
                }
            },
            _ => {}
        }
    }