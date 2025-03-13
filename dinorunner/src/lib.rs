turbo::init! {
struct GameState {
    frame: u32,
    player_x: f32,
    player_y: f32,
    velocity_y: f32,
    tree_positions: Vec<i32>,
    tree16_positions: Vec<i32>,
    is_started: bool,
    is_game_over: bool,
    score: u32,
    high_score: u32,
    bird_x: f32,
    bird_y: f32,
    bird_velocity: f32,
    bird_active: bool,
    bird_animation_frame: u32,
} = Self {
    frame: 0,
    player_x: 0.0,
    player_y: 90.0,
    velocity_y: 0.0,
    tree_positions: vec![400],
    tree16_positions: vec![900],  // Initial positions for 16x16 trees
    is_started: false,
    is_game_over: false,
    score: 0,
    high_score: 0,
    bird_x: (rand() % 256) as f32,
    bird_y: -32.0, // Bird starts off-screen
    bird_velocity: 1.0 + (rand() % 3) as f32, // Random speed
    bird_active: false, // Bird is not initially active
    bird_animation_frame: 0,
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
            font = "large",
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
        text!("Game Over!", x = 85, y = 75, font = "large", color = 0xff0000ff);

        if state.score > state.high_score {
            state.high_score = state.score;
        }

        if input.start.pressed() {
            state = GameState {
                frame: 0,
                player_x: 0.0,
                player_y: 90.0,
                velocity_y: 0.0,
                tree_positions: vec![400],
                tree16_positions: vec![900], // Reset 16x16 tree positions
                is_started: true,
                is_game_over: false,
                score: 0,
                high_score: state.high_score,
                bird_x: (rand() % 256) as f32,
                bird_y: -32.0,
                bird_velocity: 1.0 + (rand() % 3) as f32,
                bird_active: false,
                bird_animation_frame: 0,
            };
        }

        state.save();
        return;
    }

    let on_ground = state.player_y >= 86.0;

    if input.up.pressed() && on_ground {
        state.velocity_y = -7.2;
    }

    state.velocity_y += 0.4;
    state.player_y += state.velocity_y;

    if state.player_y < 0.0 {
        state.player_y = 0.0;
        state.velocity_y = 0.0;
    } else if state.player_y > 90.0 {
        state.player_y = 90.0;
        state.velocity_y = 0.0;
    }

    let bg_x = -100 - (state.frame / 10) as i32;
    let bg_width = 256;
    // Interval for toggling between day and night mode
    let cycle_interval = 12;
    let cycle_phase = (state.score / cycle_interval) % 2; // Alternates between 0 (day) and 1 (night)

    if cycle_phase == 1 {
        // Night mode: Use "backgroundnight" sprite for the background
        sprite!("backgroundnight", x = bg_x % bg_width, y = -250);
        sprite!("backgroundnight", x = (bg_x % bg_width) + bg_width, y = -250);

        // Show stars
        for col in 0..9 {
            let cloud_width = 32;
            let gap = 24;
            let total_width = 9 * (cloud_width + gap);
            let x_offset = (state.frame / 7) as i32;
            let x = (col * (cloud_width + gap) - x_offset) % total_width;
            let y = 8;

            sprite!("star", x = x, y = y);
            sprite!("star", x = x + total_width, y = y);
        }

        // Draw the moon
        sprite!("moon", x = 90, y = -1);
    } else {
        // Day mode: Use "bglonger" sprite for the background
        sprite!("bglonger", x = bg_x % bg_width, y = -250);
        sprite!("bglonger", x = (bg_x % bg_width) + bg_width, y = -250);

        // Show clouds
        for col in 0..9 {
            let cloud_width = 32;
            let gap = 24;
            let total_width = 9 * (cloud_width + gap);
            let x_offset = (state.frame / 7) as i32;
            let x = (col * (cloud_width + gap) - x_offset) % total_width;
            let y = 8;

            sprite!("clouds21", x = x, y = y);
            sprite!("clouds21", x = x + total_width, y = y);
        }

        // Sun is shown in day mode
        sprite!("sun64", x = 90, y = -10);
    }

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

    sprite!("dinorun-Sheet", x = state.player_x, y = state.player_y);

    let tree_speed_factor = 5;

     for i in 0..state.tree_positions.len() {
         state.tree_positions[i] -= tree_speed_factor;

         if state.tree_positions[i] < -32 {
             state.tree_positions[i] = 256 + (rand() % 200 + 200) as i32; // More spacing between trees
             state.score += 1;
         }

         // Collision detection for 32x32 trees
         if state.player_x + 16.0 > state.tree_positions[i] as f32
             && state.player_x < (state.tree_positions[i] + 32) as f32
             && state.player_y + 16.0 > 105.0
         {
             state.is_game_over = true;
         }

         sprite!("tree32px", x = state.tree_positions[i], y = 105);
     }

     // Move and reposition tree16x16
 for i in 0..state.tree16_positions.len() {
     state.tree16_positions[i] -= tree_speed_factor;

     if state.tree16_positions[i] < -16 {
         state.tree16_positions[i] = 256 + (rand() % 200 + 200) as i32; // More spacing for 16x16 trees
         state.score += 1;
     }

     // Corrected collision detection for 16x16 trees using integers
       let player_right = state.player_x + 16.0;  // Right edge of the player
         let player_left = state.player_x;          // Left edge of the player
         let player_bottom = state.player_y + 16.0; // Bottom edge of the player
         let player_top = state.player_y;           // Top edge of the player
         // Tree 16x16 bounds
         let tree16_x = state.tree16_positions[i] as f32; // X position of the 16x16 tree
         let tree16_right = tree16_x + 16.0; // Right edge of the 16x16 tree
         let tree16_y = 102.0; // Top of the 16x16 tree
         let tree16_bottom = tree16_y + 16.0; // Bottom edge of the 16x16 tree

         // Check for X and Y axis overlaps (collision box)
         if player_right > tree16_x && player_left < tree16_right &&
             player_bottom > tree16_y && player_top < tree16_bottom {
             state.is_game_over = true;
         }

     sprite!("tree16x16", x = tree16_x, y = 118); // Y-position of 16x16 trees
 }


    text!("Score: {}", state.score; x = 170, y = 30, font = "large", color = 0x0000ffff);
      text!("High Score: {}", state.high_score; x = 10, y = 30, font = "medium", color = 0x0000ffff);


    state.frame += 1;
    state.save();
}
