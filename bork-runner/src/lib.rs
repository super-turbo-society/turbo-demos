//lib.rs

use turbo::*;

mod state;
use state::*;

#[turbo::game]
struct GameState {
        is_ready: bool,
        dog_x: f32,
        dog_y: f32,
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
}

impl GameState {
    fn new() -> Self {
        Self {
            is_ready: false,
            dog_x: 20.0,
            dog_y: 100.0,
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
    fn update(&mut self) {
        let t = time::tick() as u32;

        let gp = gamepad::get(0);

        if !self.is_ready && t >= self.enemy_spawn_rate {
            self.is_ready = true;
            self.is_jumping = true;
            self.vel_y = -3.;
        }

        if self.last_game_over == 0 && self.is_ready {
            // Bork!!!
            if gp.start.just_released() || pointer::screen().just_pressed() {
                if t - self.last_bork >= self.bork_rate && self.energy > 0 {
                    self.borks.push(Bork::new(self.dog_x, self.dog_y));
                    self.last_bork = t;
                    self.energy -= 1;
                }
            }

            // Swing bat
            if gp.right.just_pressed() && self.has_bat {
                // Bat melee attack logic
                for enemy in self.enemies.iter_mut() {
                    if self.dog_x < enemy.x + ENEMY_WIDTH
                        && self.dog_x + BAT_RANGE > enemy.x
                        && self.dog_y < enemy.y + ENEMY_HEIGHT
                        && self.dog_y + BAT_RANGE > enemy.y
                    {
                        enemy.hits = enemy.max_hits; // Mark the enemy as hit by the bat
                                                    // self.score += 15; // Increase score for hitting with the bat
                    }
                }
                self.last_bat_swing = t;
            }

            // Physics and jump logic
            if gp.up.just_pressed() && self.energy > 0 {
                self.is_jumping = true;
                if self.vel_y > -3.0 {
                    self.vel_y = (self.vel_y + -2.5).max(-3.0);
                }
                self.energy -= 1;
            } else if gp.down.just_pressed() {
                self.is_jumping = true;
                self.vel_y = (self.vel_y + 1.).min(6.);
            }
        }

        // Apply gravity
        if self.is_jumping {
            self.dog_y += self.vel_y;
            let floor = match self.health {
                0 | 1 => 1.1,
                2 => 0.85,
                _ => 0.25,
            };
            if self.vel_y < floor {
                self.vel_y += match self.health {
                    0 | 1 => 0.12,
                    2 => 0.11,
                    _ => 0.1,
                };
            }
            if self.last_game_over == 0 {
                if self.dog_y > CANVAS_HEIGHT as f32 || self.dog_y < -DOGE_HEIGHT {
                    self.health = 0;
                    self.last_game_over = t;
                }
            }
        }

        // Increase energy
        if self.last_game_over == 0 && t % self.recharge_rate == 0 && self.energy < self.max_energy
        {
            self.energy += 1;
        }

        // Update borks
        self.borks.retain_mut(|bork| {
            bork.update();
            let mut collided = false;
            for enemy in self.enemies.iter_mut() {
                if bork.x < enemy.x + ENEMY_WIDTH
                    && bork.x + BORK_WIDTH > enemy.x
                    && bork.y < enemy.y + ENEMY_HEIGHT
                    && bork.y + BORK_HEIGHT > enemy.y
                {
                    enemy.hits += 1; // Mark the enemy as hit by the bork
                    collided = true;
                }
            }
            !collided && bork.x < self.dog_x + self.bork_range
        });

        // Spawn and update enemies
        if t - self.last_enemy_spawn >= self.enemy_spawn_rate {
            let vel_x = -1.0 + ((t / 10) as f32 * -0.01).max(-1.);
            let modifier = (random::rand() % 200) as f32 / 100.;
            let vel_x = vel_x * modifier;
            self.enemies.push(Enemy::new(vel_x));
            self.last_enemy_spawn = t;
            if t > 60 * 1 && self.enemy_spawn_rate > 30 {
                self.enemy_spawn_rate -= 2;
            }
        }
        self.enemies.retain_mut(|enemy| {
            enemy.update();
            if self.dog_x < enemy.x + ENEMY_WIDTH
                && self.dog_x + DOGE_WIDTH > enemy.x
                && self.dog_y < enemy.y + ENEMY_HEIGHT
                && self.dog_y + DOGE_HEIGHT > enemy.y
            {
                if self.health > 0 {
                    self.health -= 1;
                }
                if self.health == 0 {
                    self.last_game_over = t;
                }
                enemy.hits += 1; // Mark the enemy as hit
            }
            if enemy.hits >= enemy.max_hits {
                self.score += 10;
                return false;
            }
            enemy.x > -ENEMY_WIDTH
        });

        // Spawning powerups
        if self.is_ready {
            // if rand() % 100 < 10 { // Example probability for SpeedBoost
            //     self.powerups.push(Powerup::new(CANVAS_WIDTH as f32, (rand() % CANVAS_HEIGHT) as f32, 0.0, 0.0, PowerupType::SpeedBoost));
            // }
            // if rand() % 100 < 5 { // Example probability for MultiBork
            //     self.powerups.push(Powerup::new(CANVAS_WIDTH as f32, (rand() % CANVAS_HEIGHT) as f32, 0.0, 0.0, PowerupType::MultiBork));
            // }
            // if rand() % 100 < 3 { // Example probability for DoubleJump
            //     let initial_y = (rand() % CANVAS_HEIGHT) as f32;
            //     self.powerups.push(Powerup::new(CANVAS_WIDTH as f32, initial_y, 0.0, 2.0, PowerupType::DoubleJump));
            // }
            if random::rand() % 100 < 2 {
                // Example probability for Bat
                self.powerups.push(Powerup::new(
                    CANVAS_WIDTH as f32,
                    (random::rand() % CANVAS_HEIGHT) as f32,
                    0.0,
                    0.0,
                    PowerupType::Bat,
                ));
            }
        }

        // Powerup collection
        self.powerups.retain_mut(|powerup| {
            match powerup.powerup_type {
                PowerupType::DoubleJump => {
                    // Sinusoidal movement
                    powerup.y += f32::sin(powerup.angle) * 2.0; // Adjust amplitude as needed
                    powerup.angle += 0.1; // Adjust frequency as needed
                                        // Grant an extra jump
                    self.energy = 2;
                }
                PowerupType::SpeedBoost => {
                    // Simple leftward movement
                    powerup.x -= 2.0;
                    // Increase bork speed logic
                    for bork in self.borks.iter_mut() {
                        bork.vel_x *= 1.5; // Example: Increase speed by 50%
                    }
                }
                PowerupType::MultiBork => {
                    // Diagonal movement
                    powerup.x -= 2.0;
                    powerup.y += powerup.vel_y;
                    // Enable multi-bork logic
                    self.can_fire_multiple_borks = true;
                }
                PowerupType::Bat => {
                    // Sinusoidal movement
                    powerup.x -= 2.0;
                    powerup.y += f32::sin(powerup.angle) * 5.0;
                    powerup.angle += 0.1;
                    // Enable bat melee attack
                    self.has_bat = true;
                }
            }
            if self.dog_x < powerup.x + POWERUP_WIDTH
                && self.dog_x + DOGE_WIDTH > powerup.x
                && self.dog_y < powerup.y + POWERUP_HEIGHT
                && self.dog_y + DOGE_HEIGHT > powerup.y
            {
                if powerup.powerup_type == PowerupType::Bat && self.last_game_over == 0 {
                    self.score += 1;
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
            let height = 1;
            let y_position = ((i * 28) % 144) as i32; // Vertical position of each line
            let x_position = (t * speed) as i32 % (512) as i32 - 20; // Moving from right to left
            rect!(
                w = line_width,
                h = height,
                x = 256 + -x_position,
                y = y_position,
                color = 0xffffff88
            ); // Draw the line
        }
        if self.last_game_over == 0 {
            let (balloons, doge) = match self.health {
                0 | 1 => ("one_balloon", "doge_worried"),
                2 => ("two_balloons", "doge_worried"),
                _ => ("three_balloons", "doge_worried"),
            };
            let speed = if self.vel_y > 0. { 1.0 } else { 0.5 };
            sprite!(
                balloons,
                x = self.dog_x - DOGE_WIDTH,
                y = self.dog_y - 16.,
            );
            sprite!(
                doge,
                x = self.dog_x - DOGE_WIDTH,
                y = self.dog_y,
                animation_speed = speed,
            );
        } else {
            sprite!(
                "sad_doge",
                x = self.dog_x - DOGE_WIDTH,
                y = self.dog_y,
                animation_speed = 2.0,
            );
        }
        for bork in self.borks.iter() {
            bork.draw();
        }
        for enemy in self.enemies.iter() {
            enemy.draw();
        }
        for powerup in self.powerups.iter() {
            powerup.draw();
        }

        // Display health and score
        rect!(w = 256, h = 24, color = 0xffffffaa);
        let seconds = if self.last_game_over > 0 {
            self.last_game_over
        } else {
            t
        } / 60;
        let minutes = seconds / 60;
        let seconds = seconds % 60;
        let mmss = &format!("{:02}:{:02}", minutes, seconds);
        text!("time", x = 118, y = 3, color = 0xff0000ff, font = "small");
        text!(mmss, x = 108, y = 9, font = "large", color = 0x000000aa);
        text!(mmss, x = 108, y = 8, font = "large", color = 0x000000ff);

        text!(
            "BORK points",
            x = 190,
            y = 3,
            color = 0x000000ff,
            font = "small"
        );
        text!("${:06}", self.score; x = 190, y = 9, font = "large", color = 0x000000aa);
        text!("${:06}", self.score; x = 190, y = 8, font = "large", color = 0x000000ff);

        sprite!("energy", x = 4, y = 5);
        let energy_color = match self.energy as f32 / self.max_energy as f32 {
            n if n <= 0.25 => 0xff0000ff,
            n if n <= 0.75 => 0xec8915ff,
            _ => 0x00a0ffff,
        };
        text!("energy", x = 20, y = 3, color = 0x000000ff, font = "small");
        rect!(
            w = 4 * self.energy,
            h = 6,
            color = energy_color,
            x = 18,
            y = 10
        );

        if t < (60 / 2) {
            text!("3", x = 124, y = 64, font = "large", color = 0x000000ff);
        } else if t < (120 / 2) {
            text!("2", x = 124, y = 64, font = "large", color = 0x000000ff);
        } else if t < (180 / 2) {
            text!("1", x = 124, y = 64, font = "large", color = 0x000000ff);
        } else if t < (240 / 2) {
            text!("GO!", x = 118, y = 64, font = "large", color = 0x000000ff);
        }

        // Game over logic
        if self.last_game_over > 0 {
            text!(
                "GAME OVER",
                x = 90,
                y = 73,
                font = "large",
                color = 0x000000aa
            );
            text!(
                "GAME OVER",
                x = 90,
                y = 72,
                font = "large",
                color = 0xff0000ff
            );
            // Add logic to restart or exit the game
            if t - self.last_game_over > 60 {
                if t / 2 % 32 < 16 {
                    text!(
                        "- press start -",
                        x = 88,
                        y = 84,
                        font = "medium",
                        color = 0x000000aa
                    );
                    text!(
                        "- press start -",
                        x = 88,
                        y = 83,
                        font = "medium",
                        color = 0x000000ff
                    );
                }
                if gp.start.just_pressed() || pointer::screen().just_pressed() {
                    *self = Self::new();
                }
            }
        }
    }
}


