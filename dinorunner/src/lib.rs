turbo::cfg! {r#"
    name = "DinoRunner"
    version = "1.0.0"
    author = "Turbo"
    description = "DinoRunner!"
    [settings]
    resolution = [256, 144]
"#}

turbo::init! {
struct GameState {
    frame: u32,
    player_x: f32,
    player_y: f32,
    velocity_y: f32,
    tree_positions: Vec<i32>,
    is_started: bool,
    is_game_over: bool,
    score: u32,
    high_score: u32,
} = Self {
    frame: 0,
    player_x: 0.0,
    player_y: 90.0,
    velocity_y: 0.0,
    tree_positions: vec![300, 450, 600],
    is_started: false,
    is_game_over: false,
    score: 0,
    high_score: 0,
}
}

turbo::go! {
    let mut state = GameState::load();
    let input = gamepad(0);


    if !state.is_started {

        text!(
            "Press Start to Play",
            x = 55,
            y = 70,
            font = Font::L,
            color = 0xffd700ff
        );


        if input.start.pressed() {
            state.is_started = true;
        }

        state.save();
        return;
    }


    if state.is_game_over {
        // Display Game Over message
        text!("Game Over!", x = 85, y = 75, font = Font::L, color = 0xff0000ff);

        // Update high score if the current score is greater
        if state.score > state.high_score {
            state.high_score = state.score;
        }

        // Restart the game if the player presses start
        if input.start.pressed() {
            state = GameState {
                frame: 0,
                player_x: 0.0,
                player_y: 90.0,
                velocity_y: 0.0,
                tree_positions: vec![300, 450, 600],
                is_started: true,
                is_game_over: false,
                score: 0,
                high_score: state.high_score,  // Keep the high score between games
            };
        }

        state.save();
        return;
    }

    // Check if the player is on the ground
    let on_ground = state.player_y >= 86.0;

    // Handle user input for jumping
    if input.up.pressed() && on_ground {
        state.velocity_y = -7.2; // Stronger initial jump
    }

    // Apply gravity and update player position
    state.velocity_y += 0.4;
    state.player_y += state.velocity_y;

    // Ensure the player stays within the screen bounds
    if state.player_y < 0.0 {
        state.player_y = 0.0;
        state.velocity_y = 0.0;
    } else if state.player_y > 90.0 {
        state.player_y = 90.0;
        state.velocity_y = 0.0;
    }


    // Background movement (slowest)
    let bg_x = -100 - (state.frame / 10) as i32;
    let bg_width = 256;
    sprite!("bglonger", x = bg_x % bg_width, y = -250, fps = fps::FAST);
    sprite!("bglonger", x = (bg_x % bg_width) + bg_width, y = -250, fps = fps::FAST);

    // Floor movement (faster)
    let floor_width = 45;
    let num_floors = 10;
    let speed_factor = 5;
    let floor_x_offset = (state.frame * speed_factor) as i32;
    for i in 0..num_floors {
        let x_pos = (i * floor_width - floor_x_offset) % (num_floors * floor_width);
        sprite!("floor", x = x_pos, y = 114);
        sprite!("floor", x = x_pos + num_floors * floor_width, y = 114);
    }
    // Day-Night Cycle: Show Sun, Moon, and Stars
    if state.frame % 1000 < 500 {
        // Daytime: Show sun
        sprite!("sun64", x = 90, y = -10);
    } else {
        // Nighttime: Show moon and stars
        sprite!("moon", x = 90, y = -1);
        sprite!("star", x = 40, y = 5);
        sprite!("star", x = 190, y = 5);
    }
    sprite!("dinorun-Sheet", x = state.player_x, y = state.player_y, fps = fps::FAST);

    // Move the trees along with the floor and reposition them
    let tree_speed_factor = 5;

    for i in 0..state.tree_positions.len() {
        // Move the tree to the left by the speed factor
        state.tree_positions[i] -= tree_speed_factor;

        // If the tree goes off the left side of the screen, reset its position to the right
        if state.tree_positions[i] < -32 {  // Assuming the tree sprite is 32px wide
            state.tree_positions[i] = 256 + (rand() % 70 + 50) as i32; // Increased random gap between 50 and 120
            state.score += 1; // Increment the score when a tree resets
        }

        // Check for collision with the dino
        if state.player_x + 16.0 > state.tree_positions[i] as f32  // Dino's width is assumed to be 16px
            && state.player_x < (state.tree_positions[i] + 32) as f32  // Tree's width is 32px
            && state.player_y + 16.0 > 105.0 // Tree's top is at y=105
        {
            state.is_game_over = true;
        }

        // Draw the tree at its current position, ensuring it's on the ground
        sprite!("tree32px", x = state.tree_positions[i], y = 105, fps = fps::FAST);
    }

    // Cloud movement (slowest, creating parallax effect)
    let cloud_width = 32;
    let gap = 24;
    let total_width = 9 * (cloud_width + gap);
    let cloud_x_offset = (state.frame / 7) as i32;
    for col in 0..9 {
        let x = (col * (cloud_width + gap) - cloud_x_offset) % total_width;
        let y = 8;
        sprite!("clouds21", x = x, y = y);
        sprite!("clouds21", x = x + total_width, y = y);
    }

    // Display the score and high score at the top of the screen
    text!("Score: {}", state.score; x = 170, y = 30, font = Font::L, color = 0x0000ffff);
    text!("High Score: {}", state.high_score; x = 10, y = 30, font = Font::M, color = 0x0000ffff);

    // Increment the frame counter
    state.frame += 1;
    state.save();
}
