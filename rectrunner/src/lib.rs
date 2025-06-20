use turbo::{
    canvas::{circ, clear, rect, sprite, text},
    input::gamepad,
    sys::rand,
};

#[turbo::game]
struct GameState {
    frame: u32,
    player_x: f32,
    player_y: f32,
    velocity_y: f32,
    obstacles: Vec<Obstacle>,
    coins: Vec<Coin>,
    score: u32,
    lives: u32,
    is_game_over: bool,
    is_started: bool,
    speed: f32,
    collision_cooldown: u32,
    acceleration: f32,
    max_speed: f32,
    bg_x: f32,
    fg_x: f32,
}

impl GameState {
    fn new() -> Self {
        Self {
            frame: 0,
            player_x: 30.0,
            player_y: 10.0,
            velocity_y: 0.0,
            obstacles: vec![],
            coins: vec![],
            score: 0,
            lives: 1,
            is_game_over: false,
            is_started: false,
            speed: 2.0,
            collision_cooldown: 0,
            acceleration: 0.001,
            max_speed: 5.0,
            bg_x: 0.0,
            fg_x: 0.0,
        }
    }
    fn update(&mut self) {
        let input = gamepad(0);

        if !self.is_started {
            // Check for start input
            if input.start.pressed() {
                self.is_started = true;
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
        } else if self.is_game_over {
            // Check for restart input
            if input.start.pressed() {
                // Reset the game state
                *self = GameState {
                    frame: 0,
                    player_x: 30.0,
                    player_y: 10.0,
                    velocity_y: 0.0,
                    obstacles: vec![],
                    coins: vec![],
                    score: 0,
                    lives: 1, // Reset lives to three
                    is_game_over: false,
                    is_started: true,
                    speed: 2.0,
                    collision_cooldown: 0,
                    acceleration: 0.001, // Reset acceleration
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
                self.velocity_y = -3.0; // Move up
            }

            // Apply gravity and update player position
            self.velocity_y += 0.2; // Gravity
            self.player_y += self.velocity_y;

            // Ensure the player stays within the screen bounds
            if self.player_y < 0.0 {
                self.player_y = 0.0;
                self.velocity_y = 0.0;
            } else if self.player_y > 144.0 - 10.0 {
                self.player_y = 144.0 - 10.0;
                self.velocity_y = 0.0;
            }

            // Update obstacles and coins
            for obstacle in &mut self.obstacles {
                obstacle.x -= self.speed;
            }
            for coin in &mut self.coins {
                coin.x -= self.speed;
            }

            // Remove off-screen obstacles and coins
            self.obstacles.retain(|o| o.x + o.width > 0.0);
            self.coins.retain(|c| c.x > -5.0);
            // Generate new obstacles with dynamic gap size and more randomness
            if self.frame % 60 == 0 {
                let height = (rand() % 50 + 20) as f32; // Random height between 20 and 70

                // Add more variability to the gap size
                let base_gap = 50.0;
                let gap_variability = (rand() % 40 - 20) as f32; // Random variability between -20 and 20
                let gap = base_gap + (self.score / 100) as f32 + gap_variability;

                // Add the top obstacle
                self.obstacles.push(Obstacle {
                    x: 256.0,
                    y: 144.0 - height,
                    width: 10.0,
                    height,
                });

                // Add the bottom obstacle
                self.obstacles.push(Obstacle {
                    x: 256.0,
                    y: 0.0,
                    width: 10.0,
                    height: 144.0 - height - gap,
                });

                // Randomly generate an additional obstacle for more unpredictability
                if rand() % 10 < 3 {
                    // 30% chance to add an additional obstacle
                    let extra_height = (rand() % 50 + 10) as f32; // Random height between 10 and 60
                    let extra_gap = base_gap + (rand() % 30 - 15) as f32; // Random variability between -15 and 15
                    self.obstacles.push(Obstacle {
                        x: 256.0 + (rand() % 30 + 20) as f32, // Random x position between 20 and 50
                        y: 144.0 - extra_height,
                        width: 10.0,
                        height: extra_height,
                    });
                    self.obstacles.push(Obstacle {
                        x: 256.0 + (rand() % 30 + 20) as f32,
                        y: 0.0,
                        width: 10.0,
                        height: 144.0 - extra_height - extra_gap,
                    });
                }
            }

            // Generate new coins less frequently
            if self.frame % 300 == 0 {
                // Adjust the frequency here
                self.coins.push(Coin {
                    x: 256.0,
                    y: (rand() % 120) as f32,
                });
            }

            // Update background and foreground positions
            self.bg_x -= self.speed * 0.5;
            self.fg_x -= self.speed;

            // Reset positions for continuous scrolling
            if self.bg_x <= -256.0 {
                self.bg_x = 0.0;
            }
            if self.fg_x <= -256.0 {
                self.fg_x = 0.0;
            }

            // Check for collisions with obstacles
            let player_width = 10.0;
            let player_height = 10.0;
            let mut has_collision = false;

            for obstacle in &self.obstacles {
                if self.player_x < obstacle.x + obstacle.width
                    && self.player_x + player_width > obstacle.x
                    && self.player_y < obstacle.y + obstacle.height
                    && self.player_y + player_height > obstacle.y
                {
                    has_collision = true;
                    break;
                }
            }

            if has_collision && self.collision_cooldown == 0 {
                if self.lives > 0 {
                    self.lives -= 1;
                    self.player_x = 30.0;
                    self.player_y = 10.0;
                    self.velocity_y = 0.0;
                    self.collision_cooldown = 30; // Set cooldown to prevent multiple decrements
                } else {
                    self.is_game_over = true;
                }
            }

            // Handle collision cooldown
            if self.collision_cooldown > 0 {
                self.collision_cooldown -= 1;
            }

            // Check for coin collection
            let mut coins_to_remove = Vec::new();
            for (i, coin) in self.coins.iter().enumerate() {
                if self.player_x < coin.x + 5.0
                    && self.player_x + 10.0 > coin.x
                    && self.player_y < coin.y + 5.0
                    && self.player_y + 10.0 > coin.y
                {
                    coins_to_remove.push(i); // Collect the index for removal
                    self.lives += 1; // Increase a life
                }
            }

            // Remove collected coins by index
            for &i in coins_to_remove.iter().rev() {
                self.coins.remove(i);
            }

            // Increase score
            self.score += 1;

            // Gradually increase speed over time, but cap it at maximum speed
            if self.speed < self.max_speed {
                self.speed += self.acceleration;
                if self.speed > self.max_speed {
                    self.speed = self.max_speed;
                }
            }

            // Clear the screen
            clear(0x00ffffff);

            // Draw the background
            sprite!("bg_mountains", x = self.bg_x, y = 70);
            sprite!("bg_mountains", x = self.bg_x + 256.0, y = 70,);

            // Draw the foreground
            sprite!("fg_path", x = self.fg_x as i32, y = 120,);
            sprite!("fg_path", x = self.fg_x + 256.0, y = 120,);

            // Draw the player
            sprite!(
                "npc_spex",
                x = self.player_x - 5.0,
                y = self.player_y - 25.0,
            );

            // Draw obstacles
            for obstacle in &self.obstacles {
                rect!(
                    x = obstacle.x,
                    y = obstacle.y,
                    w = obstacle.width,
                    h = obstacle.height,
                    color = 0x555555ff
                );
            }

            // Draw coins
            for coin in &self.coins {
                circ!(x = coin.x, y = coin.y, d = 5, color = 0xffd700ff);
            }

            // Draw the dark background rectangle for the score and lives area
            rect!(x = 0, y = 0, w = 256, h = 20, color = 0x000000ff);

            // Draw the score and lives text
            text!("Score: {}", self.score; x = 10, y = 6, font = "medium", color = 0x00ffffff);
            text!("Lives: {}", self.lives; x = 175, y = 6, font = "medium", color = 0x00ffffff);

            self.frame += 1;
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Obstacle {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct Coin {
    x: f32,
    y: f32,
}
