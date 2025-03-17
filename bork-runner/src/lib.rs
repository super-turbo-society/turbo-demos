mod state;
use state::*;

// Define the game state initialization
turbo::init! {
    struct GameState {
        is_ready: bool,
        dog_x: f32,
        dog_y: f32,
        tick: u32,
        last_bork: u32,
        bork_rate: u32,
        bork_range: f32,
        last_enemy_spawn: u32,
        enemy_spawn_rate: u32,
        is_jumping: bool,
        energy: u32,
        max_energy: u32,
        recharge_rate: u32,
        vel_y: f32,
        borks: Vec<Bork>,
        enemies: Vec<Enemy>,
        powerups: Vec<Powerup>,
        score: u32,
        health: u32,
        has_bat: bool,
        last_bat_swing: u32,
        can_fire_multiple_borks: bool,
        last_game_over: u32,
    } = {
        Self::new()
    }
}

impl GameState {
    pub fn new() -> Self {
        Self {
            is_ready: false,
            dog_x: 20.0,
            dog_y: 100.0,
            tick: 0,
            last_bork: 0,
            bork_rate: 10,
            bork_range: 96.0,
            last_enemy_spawn: 0,
            enemy_spawn_rate: 100,
            is_jumping: false,
            energy: 10,
            max_energy: 10,
            recharge_rate: 25,
            vel_y: 0.0,
            borks: vec![],
            enemies: vec![],
            powerups: vec![],
            score: 0,
            health: 3,
            has_bat: true,
            last_bat_swing: 0,
            can_fire_multiple_borks: false,
            last_game_over: 0,
        }
    }
}

// Implement the game loop
turbo::go!({
    let mut state = GameState::load();

    let gp = gamepad(0);

    if !state.is_ready && state.tick >= state.enemy_spawn_rate {
        state.is_ready = true;
        state.is_jumping = true;
        state.vel_y = -3.;
    }

    if state.last_game_over == 0 && state.is_ready {
        // Bork!!!
        if gp.start.just_released() || mouse(0).left.just_pressed() {
            if state.tick - state.last_bork >= state.bork_rate && state.energy > 0 {
                state.borks.push(Bork::new(state.dog_x, state.dog_y));
                state.last_bork = state.tick;
                state.energy -= 1;
            }
        }

        // Swing bat
        if gp.right.just_pressed() && state.has_bat {
            // Bat melee attack logic
            for enemy in state.enemies.iter_mut() {
                if state.dog_x < enemy.x + ENEMY_WIDTH
                    && state.dog_x + BAT_RANGE > enemy.x
                    && state.dog_y < enemy.y + ENEMY_HEIGHT
                    && state.dog_y + BAT_RANGE > enemy.y
                {
                    enemy.hits = enemy.max_hits; // Mark the enemy as hit by the bat
                                                 // state.score += 15; // Increase score for hitting with the bat
                }
            }
            state.last_bat_swing = state.tick;
        }

        // Physics and jump logic
        if gp.up.just_pressed() && state.energy > 0 {
            state.is_jumping = true;
            if state.vel_y > -3.0 {
                state.vel_y = (state.vel_y + -2.5).max(-3.0);
            }
            state.energy -= 1;
        } else if gp.down.just_pressed() {
            state.is_jumping = true;
            state.vel_y = (state.vel_y + 1.).min(6.);
        }
    }

    // Apply gravity
    if state.is_jumping {
        state.dog_y += state.vel_y;
        let floor = match state.health {
            0 | 1 => 1.1,
            2 => 0.85,
            _ => 0.25,
        };
        if state.vel_y < floor {
            state.vel_y += match state.health {
                0 | 1 => 0.12,
                2 => 0.11,
                _ => 0.1,
            };
        }
        if state.last_game_over == 0 {
            if state.dog_y > CANVAS_HEIGHT as f32 || state.dog_y < -DOGE_HEIGHT {
                state.health = 0;
                state.last_game_over = state.tick;
            }
        }
    }

    // Increase energy
    if state.last_game_over == 0
        && state.tick % state.recharge_rate == 0
        && state.energy < state.max_energy
    {
        state.energy += 1;
    }

    // Update borks
    state.borks.retain_mut(|bork| {
        bork.update();
        let mut collided = false;
        for enemy in state.enemies.iter_mut() {
            if bork.x < enemy.x + ENEMY_WIDTH
                && bork.x + BORK_WIDTH > enemy.x
                && bork.y < enemy.y + ENEMY_HEIGHT
                && bork.y + BORK_HEIGHT > enemy.y
            {
                enemy.hits += 1; // Mark the enemy as hit by the bork
                collided = true;
            }
        }
        !collided && bork.x < state.dog_x + state.bork_range
    });

    // Spawn and update enemies
    if state.tick - state.last_enemy_spawn >= state.enemy_spawn_rate {
        // if rand() % 100 < 2 {
        let vel_x = -1.0 + ((state.tick / 10) as f32 * -0.01).max(-1.);
        let modifier = (rand() % 200) as f32 / 100.;
        let vel_x = vel_x * modifier;
        // let vel_x = vel_x + ((state.enemy_spawn_rate as f32)  + ((rand() % 100) as f32)) / -100.;
        state.enemies.push(Enemy::new(vel_x));
        state.last_enemy_spawn = state.tick;
        if state.tick > 60 * 1 && state.enemy_spawn_rate > 30 {
            state.enemy_spawn_rate -= 2;
        }
        // }
    }
    state.enemies.retain_mut(|enemy| {
        enemy.update();
        if state.dog_x < enemy.x + ENEMY_WIDTH
            && state.dog_x + DOGE_WIDTH > enemy.x
            && state.dog_y < enemy.y + ENEMY_HEIGHT
            && state.dog_y + DOGE_HEIGHT > enemy.y
        {
            if state.health > 0 {
                state.health -= 1;
            }
            if state.health == 0 {
                state.last_game_over = state.tick;
            }
            enemy.hits += 1; // Mark the enemy as hit
        }
        if enemy.hits >= enemy.max_hits {
            state.score += 10;
            return false;
        }
        enemy.x > -ENEMY_WIDTH
    });

    // Spawning powerups
    if state.is_ready {
        // if rand() % 100 < 10 { // Example probability for SpeedBoost
        //     state.powerups.push(Powerup::new(CANVAS_WIDTH as f32, (rand() % CANVAS_HEIGHT) as f32, 0.0, 0.0, PowerupType::SpeedBoost));
        // }
        // if rand() % 100 < 5 { // Example probability for MultiBork
        //     state.powerups.push(Powerup::new(CANVAS_WIDTH as f32, (rand() % CANVAS_HEIGHT) as f32, 0.0, 0.0, PowerupType::MultiBork));
        // }
        // if rand() % 100 < 3 { // Example probability for DoubleJump
        //     let initial_y = (rand() % CANVAS_HEIGHT) as f32;
        //     state.powerups.push(Powerup::new(CANVAS_WIDTH as f32, initial_y, 0.0, 2.0, PowerupType::DoubleJump));
        // }
        if rand() % 100 < 2 {
            // Example probability for Bat
            state.powerups.push(Powerup::new(
                CANVAS_WIDTH as f32,
                (rand() % CANVAS_HEIGHT) as f32,
                0.0,
                0.0,
                PowerupType::Bat,
            ));
        }
    }

    // Powerup collection
    state.powerups.retain_mut(|powerup| {
        match powerup.powerup_type {
            PowerupType::DoubleJump => {
                // Sinusoidal movement
                powerup.y += f32::sin(powerup.angle) * 2.0; // Adjust amplitude as needed
                powerup.angle += 0.1; // Adjust frequency as needed
                                      // Grant an extra jump
                state.energy = 2;
            }
            PowerupType::SpeedBoost => {
                // Simple leftward movement
                powerup.x -= 2.0;
                // Increase bork speed logic
                for bork in state.borks.iter_mut() {
                    bork.vel_x *= 1.5; // Example: Increase speed by 50%
                }
            }
            PowerupType::MultiBork => {
                // Diagonal movement
                powerup.x -= 2.0;
                powerup.y += powerup.vel_y;
                // Change direction when hitting the screen bounds
                // if powerup.y <= 0.0 || powerup.y >= CANVAS_HEIGHT.0 {
                //     powerup.velocity_y = -powerup.velocity_y;
                // }
                // Enable multi-bork logic
                state.can_fire_multiple_borks = true;
            }
            PowerupType::Bat => {
                // Sinusoidal movement
                powerup.x -= 2.0;
                powerup.y += f32::sin(powerup.angle) * 5.0;
                powerup.angle += 0.1;
                // Enable bat melee attack
                state.has_bat = true;
            }
        }
        if state.dog_x < powerup.x + POWERUP_WIDTH
            && state.dog_x + DOGE_WIDTH > powerup.x
            && state.dog_y < powerup.y + POWERUP_HEIGHT
            && state.dog_y + DOGE_HEIGHT > powerup.y
        {
            if powerup.powerup_type == PowerupType::Bat && state.last_game_over == 0 {
                state.score += 1;
            }
            false // Remove the powerup after applying its effect
        } else {
            true // Keep the powerup if it hasn't been collected
        }
    });

    // Draw game elements
    clear(0x00ffffff);
    // Draw speed lines
    let line_count = 15; // Number of speed lines
    let max_speed = 25; // Maximum speed of the lines
    let line_width = 128; // Screen width

    for i in 0..line_count {
        let speed = (i + 1) as u32 * max_speed / line_count; // Varying speeds for each line
                                                             // let height = 1 + i; // Varying heights for each line
        let height = 1;
        let y_position = ((i * 28) % 144) as i32; // Vertical position of each line
        let x_position = (state.tick * speed) as i32 % (512) as i32 - 20; // Moving from right to left
        canvas::rect!(
            w = line_width,
            h = height,
            x = 256 + -x_position,
            y = y_position,
            color = 0xffffff88
        ); // Draw the line
    }
    if state.last_game_over == 0 {
        let (balloons, doge) = match state.health {
            0 | 1 => ("one_balloon", "doge_worried"),
            2 => ("two_balloons", "doge_worried"),
            _ => ("three_balloons", "doge_worried"),
        };
        let speed = if state.vel_y > 0. { 1.0 } else { 0.5 };
        canvas::sprite!(
            balloons,
            x = state.dog_x - DOGE_WIDTH,
            y = state.dog_y - 16.,
        );
        canvas::sprite!(
            doge,
            x = state.dog_x - DOGE_WIDTH,
            y = state.dog_y,
            animation_speed = speed,
        );
    } else {
        canvas::sprite!(
            "sad_doge",
            x = state.dog_x - DOGE_WIDTH,
            y = state.dog_y,
            animation_speed = 2.0,
        );
    }
    for bork in state.borks.iter() {
        bork.draw();
    }
    for enemy in state.enemies.iter() {
        enemy.draw();
    }
    for powerup in state.powerups.iter() {
        powerup.draw();
    }

    // Display health and score
    canvas::rect!(w = 256, h = 24, color = 0xffffffaa);
    let seconds = if state.last_game_over > 0 {
        state.last_game_over
    } else {
        state.tick
    } / 60;
    let minutes = seconds / 60;
    let seconds = seconds % 60;
    let mmss = &format!("{:02}:{:02}", minutes, seconds);
    canvas::text!("time", x = 118, y = 3, color = 0x000000ff, font = "small");
    canvas::text!(mmss, x = 108, y = 9, font = "large", color = 0x000000aa);
    canvas::text!(mmss, x = 108, y = 8, font = "large", color = 0x000000ff);

    canvas::text!(
        "BORK points",
        x = 190,
        y = 3,
        color = 0x000000ff,
        font = "small"
    );
    canvas::text!("${:06}", state.score; x = 190, y = 9, font = "large", color = 0x000000aa);
    canvas::text!("${:06}", state.score; x = 190, y = 8, font = "large", color = 0x000000ff);
    // canvas::text!(&format!("Health: {}", state.health), x = 10, y = 20, font = "medium", color = 0xffffffff);
    // canvas::text!(&format!("Energy: {}", state.energy), x = 10, y = 30, font = "medium", color = 0xffffffff);

    canvas::sprite!("energy", x = 4, y = 5);
    let energy_color = match state.energy as f32 / state.max_energy as f32 {
        n if n <= 0.25 => 0xff0000ff,
        n if n <= 0.75 => 0xec8915ff,
        _ => 0x00a0ffff,
    };
    canvas::text!("energy", x = 20, y = 3, color = 0x000000ff, font = "small");
    canvas::rect!(
        w = 4 * state.energy,
        h = 6,
        color = energy_color,
        x = 18,
        y = 10
    );

    if state.tick < (60 / 2) {
        canvas::text!("3", x = 124, y = 64, font = "large", color = 0x000000ff);
    } else if state.tick < (120 / 2) {
        canvas::text!("2", x = 124, y = 64, font = "large", color = 0x000000ff);
    } else if state.tick < (180 / 2) {
        canvas::text!("1", x = 124, y = 64, font = "large", color = 0x000000ff);
    } else if state.tick < (240 / 2) {
        canvas::text!("GO!", x = 118, y = 64, font = "large", color = 0x000000ff);
    }

    // Game over logic
    if state.last_game_over > 0 {
        canvas::text!(
            "GAME OVER",
            x = 90,
            y = 73,
            font = "large",
            color = 0x000000aa
        );
        canvas::text!(
            "GAME OVER",
            x = 90,
            y = 72,
            font = "large",
            color = 0xff0000ff
        );
        // Add logic to restart or exit the game
        if state.tick - state.last_game_over > 60 {
            if state.tick / 2 % 32 < 16 {
                canvas::text!(
                    "- press start -",
                    x = 88,
                    y = 84,
                    font = "medium",
                    color = 0x000000aa
                );
                canvas::text!(
                    "- press start -",
                    x = 88,
                    y = 83,
                    font = "medium",
                    color = 0x000000ff
                );
            }
            if gp.start.just_pressed() || mouse(0).left.just_pressed() {
                state = GameState::new()
            }
        }
    }

    state.tick += 1;
    state.save();
});
