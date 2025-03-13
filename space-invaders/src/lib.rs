turbo::init! {
    struct GameState {
        player_x: f32,
        player_y: f32,
        invaders: Vec<struct Invader {
            x: f32,
            y: f32,
            moving_right: bool,
            sprites: [String; 2],
        }>,
        bullets: Vec<struct Bullet {
            x: f32,
            y: f32,
        }>,
        invader_direction_change: bool,
        score: u32,
        game_over: bool,
        tick: usize,
        move_rate: usize,
    } = {
        GameState::new()
    }
}

impl GameState {
    fn new() -> Self {
        Self {
            player_x: 128.0,
            player_y: 218.0,
            // 5 rows of 11 aliens
            invaders: (0..5)
                .flat_map(|row| {
                    (0..11).map(move |col| Invader {
                        x: 20.0 + (col as f32 * 16.0),
                        y: 20.0 + (row as f32 * 16.0),
                        moving_right: true,
                        sprites: match row {
                            1..=2 => ["invader_b_0".to_string(), "invader_b_1".to_string()],
                            3..=4 => ["invader_c_0".to_string(), "invader_c_1".to_string()],
                            _ => ["invader_a_0".to_string(), "invader_a_1".to_string()],
                        },
                    })
                })
                .collect(),
            bullets: vec![],
            invader_direction_change: false,
            score: 0,
            game_over: false,
            tick: 0,
            move_rate: 10,
        }
    }
}

// Implement the game loop
turbo::go!({
    let mut state = GameState::load();

    let won_game = state.invaders.is_empty();
    let lost_game = state.game_over;

    // Handle player input
    if !lost_game && !won_game {
        if gamepad(0).left.pressed() {
            state.player_x -= 2.0;
        }
        if gamepad(0).right.pressed() {
            state.player_x += 2.0;
        }
        if gamepad(0).a.just_pressed() || gamepad(0).start.just_pressed() {
            // Fire a bullet
            state.bullets.push(Bullet {
                x: state.player_x,
                y: state.player_y, // Starting from the player's position
            });
        }

        // Update bullet positions
        state.bullets.retain_mut(|bullet| {
            bullet.y -= 4.0; // Move the bullet upwards
            bullet.y > 0.0 // Keep the bullet if it's within the screen bounds
        });

        // Move invaders and check for direction change
        let mut hit_edge = false;
        if state.tick % state.move_rate == 0 {
            for invader in &mut state.invaders {
                let canvas_w = 224.0 as f32;
                invader.x += if invader.moving_right { 2.0 } else { -2.0 };
                if invader.x + 16.0 >= canvas_w || invader.x < 0. {
                    hit_edge = true;
                }
                if invader.y >= state.player_y {
                    state.game_over = true;
                    break;
                }
            }
        }

        // Increase move rate every 10s
        if state.tick % 600 == 0 {
            let increase = 1; // 6 frames (game runs at 60 frames per second).
            let minimum = 1; // This is the smallest frame delay between movement.
            state.move_rate = state.move_rate.saturating_sub(increase).max(minimum);
        }

        if hit_edge {
            for invader in &mut state.invaders {
                invader.y += 8.0; // Move down 8px when hitting the screen edge
                invader.moving_right = !invader.moving_right; // Change direction
            }
        }

        // Check for bullet collisions with invaders
        state.bullets.retain_mut(|bullet| {
            let mut bullet_hit = false;
            state.invaders.retain_mut(|invader| {
                let did_hit = bullet.x < invader.x + 16.0
                    && bullet.x + 2.0 > invader.x
                    && bullet.y < invader.y + 8.0
                    && bullet.y + 2.0 > invader.y;
                bullet_hit = bullet_hit || did_hit;
                if did_hit {
                    state.score += 1; // Increase score for hitting an invader
                }
                !did_hit
            });
            !bullet_hit // Keep the bullet if it didn't hit an invader
        });
    } else {
        if gamepad(0).a.just_pressed() || gamepad(0).start.just_pressed() {
            // Reset game
            state = GameState::new();
        }
    }

    // Draw the player
    sprite!("player", x = state.player_x - 8.0, y = state.player_y);

    // Draw the invaders
    for invader in &state.invaders {
        // Change sprite every .5s
        let sprite_index = if state.tick % 60 < 30 { 0 } else { 1 };
        let sprite_name = &invader.sprites[sprite_index];
        sprite!(sprite_name, x = invader.x, y = invader.y);
    }

    // Draw the bullets
    for bullet in &state.bullets {
        rect!(x = bullet.x, y = bullet.y, w = 2, h = 2, color = 0xffffffff);
    }

    // Draw the score
    text!("SCORE: {:0>5}", state.score; x = 10, y = 10, font = "large", color = 0xffffffff);

    if won_game {
        // TODO: draw game over text
        text!(
            "YOU WIN!",
            x = 80,
            y = 80,
            font = "large",
            color = 0xffffffff
        );
    }
    if lost_game {
        // TODO: draw game over text
        text!(
            "GAME OVER",
            x = 76,
            y = 80,
            font = "large",
            color = 0xffffffff
        );
    }

    // Save game state for the next frame
    state.tick += 1;
    state.save();
});
