// Define the game configuration
// Define the game state
turbo::init! {
    struct GameState {
        frame: u32,
        player_x: f32,
        player_y: f32,
        velocity_y: f32,
        obstacles: Vec<Obstacle>,
        coins: Vec<Coin>,
        score: f32,
        lives: u32,
        is_game_over: bool,
        is_started: bool,
        speed: f32,
        collision_cooldown: u32,
        acceleration: f32,
        score_acceleration: f32,
        max_speed: f32,
        bg_x: f32,
        fg_x: f32,
    } = {
        Self {
            frame: 0,
            player_x: 30.0,
            player_y: 10.0,
            velocity_y: 0.0,
            obstacles: vec![],
            coins: vec![],
            score: 0.0,
            lives: 1,
            is_game_over: false,
            is_started: false,
            speed: 2.0,
            collision_cooldown: 0,
            acceleration: 0.001,
            score_acceleration: 0.001,
            max_speed: 2.0,
            bg_x: 0.0,
            fg_x: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, BorshDeserialize, BorshSerialize)]
struct Obstacle {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Debug, Clone, PartialEq, BorshDeserialize, BorshSerialize)]
struct Coin {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

turbo::go!({
    let mut state = GameState::load();

    let input = gamepad(0);

    if !state.is_started {
        // Check for start input
        if input.start.pressed() {
            state.is_started = true;
        }

        // Clear the screen
        clear(0x00ffffff);

        // Draw the background rectangle for the "Press Start to Play" text
        rect!(x = -75, y = 0, w = 350, h = 200, color = 0x000000ff); // Combined height and position

        // Display Start message
        text!(
            "Press Start to Play",
            x = 55,
            y = 70,
            font = "large",
            color = 0xffd700ff
        );
    } else if state.is_game_over {
        // Check for restart input
        if input.start.pressed() {
            // Reset the game state
            state = GameState {
                frame: 0,
                player_x: 30.0,
                player_y: 10.0,
                velocity_y: 0.0,
                obstacles: vec![],
                coins: vec![],
                score: 0.0,
                lives: 1, // Reset lives to three
                is_game_over: false,
                is_started: true,
                speed: 2.0,
                collision_cooldown: 0,
                acceleration: 0.001, // Reset acceleration
                score_acceleration: 0.001,
                max_speed: 5.0,      // Reset maximum speed
                bg_x: 0.0,           // Reset background position
                fg_x: 0.0,           // Reset foreground position
            };
        }

        // Clear the screen
        clear(0x00ffffff);

        // Draw the background rectangle for both "Game Over" and "Press Start to Restart" texts
        rect!(x = -75, y = 0, w = 350, h = 200, color = 0x000000ff); // Combined height and position

        // Display Game Over message
        text!(
            "Game Over",
            x = 98,
            y = 70,
            font = "large",
            color = 0xff0000ff
        );

        // Display Restart message
        text!(
            "Press Start to Restart",
            x = 80,
            y = 90,
            font = "medium",
            color = 0x00ffffff
        );
    } else {
        // Handle user input for jumping
        if input.up.pressed() {
            state.velocity_y = -3.0; // Move up
        }

        // Apply gravity and update player position
        state.velocity_y += 0.2; // Gravity
        state.player_y += state.velocity_y;

        // Ensure the player stays within the screen bounds
        if state.player_y < 30.0 {
            state.player_y = 30.0;
            state.velocity_y = 0.0;
        } else if state.player_y > 144.0 - 10.0 {
            state.player_y = 144.0 - 10.0;
            state.velocity_y = 0.0;
        }

        // Update obstacles and coins
        for obstacle in &mut state.obstacles {
            obstacle.x -= state.speed;
        }
        for coin in &mut state.coins {
            coin.x -= state.speed;
        }

        // Remove off-screen obstacles and coins
        state.obstacles.retain(|o| o.x + o.width > 0.0);
        state.coins.retain(|c| c.x > -5.0);

        if state.frame % 60 == 0 {
            // Favor gap size of 50 with small variation
            let base_gap = 50.0f32;
            let gap_variation = (rand() % 11) as f32 - 5.0; // -5.0 to +5.0
            let gap_size = (base_gap + gap_variation).clamp(40.0, 60.0);
        
            let min_gap_y = 40.0f32;
            let max_gap_y = 144.0 - gap_size - 45.0;
        
            let gap_y_range = (max_gap_y - min_gap_y).max(1.0) as u32;
            let gap_y = (rand() % gap_y_range + min_gap_y as u32) as f32;
        
            // Bottom obstacle
            state.obstacles.push(Obstacle {
                x: 256.0,
                y: 0.0,
                width: 10.0,
                height: gap_y,
            });
        
            // Top obstacle
            state.obstacles.push(Obstacle {
                x: 256.0,
                y: gap_y + gap_size,
                width: 10.0,
                height: 144.0 - (gap_y + gap_size),
            });
        
            if state.frame % 320 == 0 {
                state.coins.push(Coin {
                    x: 256.0,
                    y: gap_y + (gap_size / 2.0) - 5.0,
                    width: 10.0,
                    height: 10.0,
                });
            }
        }
        
                     
        // Update background and foreground positions
        state.bg_x -= state.speed * 0.5;
        state.fg_x -= state.speed;

        // Reset positions for continuous scrolling
        if state.bg_x <= -256.0 {
            state.bg_x = 0.0;
        }
        if state.fg_x <= -256.0 {
            state.fg_x = 0.0;
        }

        // Check for collisions with obstacles
        let player_width = 12.0;
        let player_height = 16.0; 
        let mut has_collision = false;

        for obstacle in &state.obstacles {
            if state.player_x + 5.0 < obstacle.x + obstacle.width
                && state.player_x + 5.0 + player_width > obstacle.x
                && state.player_y - 15.0 < obstacle.y + obstacle.height
                && state.player_y - 15.0 + player_height > obstacle.y
            {
                has_collision = true;
                break;
            }
        }
        

        if has_collision && state.collision_cooldown == 0 {
            if state.lives > 0 {
                state.lives -= 1;
                state.player_x = 30.0;
                state.player_y = 10.0;
                state.velocity_y = 0.0;
                state.collision_cooldown = 30; // Set cooldown to prevent multiple decrements
            } else {
                state.is_game_over = true;
            }
        }

        // Handle collision cooldown
        if state.collision_cooldown > 0 {
            state.collision_cooldown -= 1;
        }

        // Check for coin collection
        let mut coins_to_remove = Vec::new();
        for (i, coin) in state.coins.iter().enumerate() {
            if state.player_x + 5.0 < coin.x + coin.width
                && state.player_x + 5.0 + player_width > coin.x
                && state.player_y - 15.0 < coin.y + coin.height
                && state.player_y - 15.0 + player_height > coin.y
            {
                coins_to_remove.push(i); // Collect the index for removal
                state.lives += 1; // Increase a life
            }
        }

        // Remove collected coins by index
        for &i in coins_to_remove.iter().rev() {
            state.coins.remove(i);
        }
        // Gradually increase speed over time, but cap it at maximum speed
        if state.speed < state.max_speed {
            state.speed += state.acceleration;
            if state.speed > state.max_speed {
                state.speed = state.max_speed;
            }
        }

        state.score += 1.0;

        // Clear the screen
        clear(0x00ffffff);

        // Draw the background
        sprite!("bg_mountains", x = state.bg_x, y = 70);
        sprite!("bg_mountains", x = state.bg_x + 256.0, y = 70,);

        // Draw the foreground
        sprite!("fg_path", x = state.fg_x as i32, y = 120,);
        sprite!("fg_path", x = state.fg_x + 256.0, y = 120,);

        let sprite_x = 5.0;
        let sprite_y = 25.0;
        // Draw the player

        sprite!(
            "shadow",
            x = state.player_x - sprite_x,
            y = 134.0 - sprite_y,
        );

        sprite!(
            "npc_spex_shadowless",
            x = state.player_x - sprite_x,
            y = state.player_y - sprite_y,
        );

        // Draw obstacles
        for obstacle in &state.obstacles {
            rect!(
                x = obstacle.x,
                y = obstacle.y,
                w = obstacle.width,
                h = obstacle.height,
                color = 0x555555ff
            );
        }

        // Draw coins

        for coin in &state.coins {
            ellipse!(x = coin.x, y = coin.y, w = coin.width, h = coin.height, color = 0xffd700ff);
        }

        // Draw player's collision box
        //rect!(x = state.player_x + 5.0, y = state.player_y - 15.0, w = player_width, h = player_height, color = 0xffffffff);
        
        // Debug: show max bottom platform (gap near top)
        // let max_gap_size = 40.0; // Smallest possible gap
        // let max_bottom = 144.0 - max_gap_size - 45.0;
        // rect!(x = 150, y = 0.0, w = 10.0, h = max_bottom, color = 0xff000088);

        // // Debug: show max top platform (gap near bottom)
        // let gap_y_low = 60.0;
        // let max_top = 144.0 - (gap_y_low + max_gap_size);
        // rect!(x = 170, y = gap_y_low + max_gap_size, w = 10.0, h = max_top, color = 0x0000ff88);

        // Draw the dark background rectangle for the score and lives area
        rect!(x = 0, y = 0, w = 256, h = 20, color = 0x000000ff);

        // Draw the score and lives text
        text!("Score: {:.0}", state.score; x = 10, y = 6, font = "medium", color = 0x00ffffff);
        text!("Lives: {}", state.lives; x = 175, y = 6, font = "medium", color = 0x00ffffff);

        state.frame += 1;
    }

    // Save the updated state
    state.save();
});