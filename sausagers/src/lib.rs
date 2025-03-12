turbo::init! {
    struct GameState {
        screen: enum Screen {
            Title,
            Game,
        },
        tick: u32,
        notification_timer: u32,
        hit_timer: u32,

        // Game elements
        score: u32,
        tutorial_active: bool,
        help_messages: Vec<String>,
        notifications: Vec<String>,

        // Entities
        players: Vec<Player>,
        projectiles: Vec<Projectile>,
        enemies: Vec<Enemy>,
        powerups: Vec<Powerup>,
    } = {
        Self::new()
    }
}

impl GameState {
    pub fn new() -> Self {
        let [screen_w, screen_h] = resolution();
        Self {
            // Initialize all fields with default values
            screen: Screen::Title,
            tick: 0,
            notification_timer: 0,
            hit_timer: 0,
            score: 0,
            tutorial_active: true,
            help_messages: vec![
                String::from("Use arrow keys to move"),
                String::from("Press A to shoot projectiles"),
            ],
            notifications: vec![
                "Use arrow keys to move.".to_string(),
                "Press SPACE or A to shoot.".to_string(),
                "Defeat enemies. Get powerups.".to_string(),
                "DESTROY ALL BUNS!".to_string(),
            ],
            players: vec![Player {
                id: 0,
                x: ((screen_w / 2) - 8) as f32,
                y: screen_h as f32,
                width: 17,
                height: 22,
                health: 3,
                max_health: 3,
                speed: 2.0,
                color: PLAYER_COLORS[0],
                accessory: None,
                projectile_type: ProjectileType::Splatter,
                projectile_damage: 1,
                skill_points: 0,
                skills: Skills {
                    speed_boost: false,
                    double_damage: false,
                },
                powerups: vec![],
                metrics: PlayerMetrics::new(),
            }],
            projectiles: vec![],
            enemies: vec![],
            powerups: vec![],
        }
    }
}

const MAX_POWERUPS: usize = 3;
const INVULNERABLE_FRAMES: u32 = 30;
const DEAD_FRAMES: u32 = 120;
const POWERUP_FRAMES: u32 = 60 * 30;
const MAX_PLAYERS: usize = 4;
const PLAYER_COLORS: [u32; MAX_PLAYERS] = [
    0xffffffff, // p1
    0xff0000ff, // p2
    0x00ff00ff, // p3
    0x0000ffff, // p4
];

turbo::go!({
    let mut state = GameState::load();

    // Draw moving parallax stars in the background
    clear(0x000333ff);
    let [screen_w, screen_h] = [canvas::size().0, canvas::size().1];
    draw_stars(&state, screen_w, screen_h);

    match state.screen.clone() {
        Screen::Game => {
            draw_game_screen(&state);
            update_game_screen(&mut state);
        }
        Screen::Title => {
            draw_title_screen(&state);
            update_title_screen(&mut state);
        }
    }

    state.tick += 1;
    state.save();
});

fn draw_title_screen(state: &GameState) {
    let [screen_w, screen_h] = [canvas::size().0, canvas::size().1];
    let screen_w = screen_w as i32;
    let screen_h = screen_h as i32;
    let center = screen_w / 2;

    // Logo
    let x = center - 48;
    let progress = (state.tick * 2).min(screen_h as u32);
    let y = screen_h - (progress as i32).min(screen_h);
    let t = progress as f32 / 10.;
    let scale = 2.0 + (t.sin() / 10.);
    let sw = 96.0 * scale;
    let xoff = sw as i32 / 4;
    let yoff = 32;
    for col in 0..9 {
        for row in 0..13 {
            let t = state.tick as i32 / 2;
            let xoff = -32 + (t % 32);
            let yoff = -32 + (t % 32);
            sprite!("hotdog", x = (col * 32) + xoff, y = (row * 32) + yoff);
        }
    }

    // Show players who joined
    let num_players = state.players.len();
    let left = center - ((num_players as i32 * 52) / 2);
    for (i, player) in state.players.iter().enumerate() {
        rect!(
            h = 14,
            w = 50,
            x = left + (i as i32 * 52),
            y = screen_h - 16,
            color = if player.color == 0xffffffff {
                0x000333ff
            } else {
                player.color
            },
            border_radius = 2,
        );
        text!(
           "P{} joined", player.id + 1;
            font = "medium",
            x = left + 4 + (i as i32 * (52)),
            y = screen_h - 12
        );
    }

    // Logo
    sprite!(
        "logo",
        x = x - xoff,
        y = y + yoff,
        scale_x = scale,
        scale_y = scale
    );
    if progress as i32 >= screen_h {
        // sprite!("logo", x = x - xoff, y = y + yoff, scale_x = scale, scale_y = scale);
        let x = (screen_w / 2) - ((11 * 8) / 2);
        let y = screen_h / 2;
        rect!(w = screen_w, h = 32, x = 0, y = y - 12, color = 0x000333ff);
        if state.tick % 60 < 30 {
            text!("PRESS START", font = "large", x = x, y = y);
        }
        // Show players who joined
        let num_players = state.players.len();
        for i in 0..num_players {
            let player = &state.players[i];
            draw_player(&player, num_players > 1);
        }
    }
}

fn update_title_screen(state: &mut GameState) {
    if gamepad(0).start.just_pressed() || gamepad(0).a.just_pressed() {
        state.screen = Screen::Game;
        state.tick = 0;
    }
    for i in 1..MAX_PLAYERS {
        let i = i as u32;
        if gamepad(i).a.just_pressed() || gamepad(i).b.just_pressed() {
            if state.players.iter().position(|p| p.id == i).is_none() {
                let mut player = state.players[0].clone();
                player.id = i;
                player.color = PLAYER_COLORS[i as usize];
                state.players.push(player)
            }
        }
    }
}

fn update_game_screen(state: &mut GameState) {
    let [screen_w, screen_h] = resolution();

    let is_game_over = state.players.iter().all(|p| p.health == 0);
    if is_game_over {
        // Restart
        if state.hit_timer == 0 && (gamepad(0).start.just_pressed() || gamepad(0).a.just_pressed())
        {
            let mut next_state = GameState::new();
            next_state.players = state
                .players
                .iter()
                .cloned()
                .map(|p| Player {
                    id: p.id,
                    color: PLAYER_COLORS[p.id as usize],
                    ..next_state.players[0].clone()
                })
                .collect();
            *state = next_state;
        }
    } else {
        for i in 1..MAX_PLAYERS {
            let i = i as u32;
            if gamepad(i).a.just_pressed() || gamepad(i).b.just_pressed() {
                if state.players.iter().position(|p| p.id == i).is_none() {
                    let mut player = GameState::new().players[0].clone();
                    player.id = i;
                    player.y -= 64.0;
                    player.color = PLAYER_COLORS[i as usize];
                    state.players.push(player)
                }
            }
        }
        for player in &mut state.players {
            let i = player.id;
            // Move player into view
            if state.tick <= 64 && player.y > screen_h as f32 - 64.0 {
                player.y -= 1.0;
            }
            // Handle user input
            else if player.health > 0 {
                // Calculate player speed
                let mut speed_mul = 1.0;
                for (powerup_effect, _) in &player.powerups {
                    if *powerup_effect == PowerupEffect::SpeedBoost {
                        speed_mul += 0.25;
                    }
                }
                let player_speed = player.speed * speed_mul;
                // Get player gamepad
                let gp = gamepad(i as u32);
                // Player movement handling
                if gp.up.pressed() {
                    // Move up
                    player.y = (player.y - player_speed).max(0.0);
                }
                if gp.down.pressed() {
                    // Move down
                    player.y = (player.y + player_speed).min((screen_h - player.height) as f32);
                }
                if gp.left.pressed() {
                    // Move left
                    player.x = (player.x - player_speed).max(0.0);
                }
                if gp.right.pressed() {
                    // Move right
                    player.x = (player.x + player_speed).min((screen_w - player.width) as f32);
                }

                // Shooting projectiles
                if gp.start.just_pressed() || gp.a.just_pressed() || gp.b.just_pressed() {
                    let mut bonus_damage = 0;
                    for (powerup_effect, _) in &player.powerups {
                        if *powerup_effect == PowerupEffect::DamageBoost {
                            bonus_damage += 1;
                        }
                    }
                    state.projectiles.push(Projectile {
                        x: player.x + ((player.width / 2) as f32) - 2.0,
                        y: player.y,
                        width: 6,
                        height: 8,
                        velocity: 5.0,
                        angle: -90.0,
                        damage: player.projectile_damage + bonus_damage,
                        projectile_type: player.projectile_type,
                        projectile_owner: ProjectileOwner::Player,
                        ttl: None,
                    });
                }
            }
        }
    }

    if state.powerups.len() < MAX_POWERUPS {
        // Every 30s, spawn a heal at a random location
        if state.tick % (60 * 30) == 0 && state.players.iter().any(|p| p.health < p.max_health) {
            state.powerups.push(Powerup {
                x: (rand() % screen_w) as f32,
                y: 24.0 + (rand() % screen_h / 2) as f32,
                width: 8,
                height: 8,
                effect: PowerupEffect::Heal,
                movement: PowerupMovement::Drifting(0.75),
            });
        }
        // Spawn a heal every 60s when player's health is low
        if state.tick % (60 * 60) == 0
            && state
                .players
                .iter()
                .any(|p| p.health == 1 && p.max_health < 5)
        {
            state.powerups.push(Powerup {
                x: (rand() % screen_w) as f32,
                y: (rand() % screen_h) as f32,
                width: 8,
                height: 8,
                effect: PowerupEffect::MaxHealthUp,
                movement: PowerupMovement::Floating(0.5),
            });
        }
    }

    // Start spawning enemies after intro dialog
    if state.tick > (state.notifications.len() as u32 + 1) * 240 {
        // Enemy spawning logic based on time elapsed
        // Define spawn intervals (in ticks) for enemies
        let initial_spawn_rate: u32 = 100; // Initial interval for enemy spawn
        let minimum_spawn_rate = 25; // Minimum interval after speeding up
        let speed_up_rate = 60 * 2; // Interval after which spawn rate increases

        // Calculate current spawn interval based on time elapsed
        let spawn_rate = std::cmp::max(
            minimum_spawn_rate,
            initial_spawn_rate.saturating_sub(state.tick / speed_up_rate),
        );
        // if state.player.health > 0 {
        //     text!(&format!("spawn rate: {spawn_rate}"), x = 4, y = 22, font = "small");
        // }
        if state.tick % spawn_rate == 0 && state.enemies.len() < 24 {
            state.enemies.push(match rand() % 8 {
                0 => Enemy::tank(),
                1 => Enemy::tank(),
                2 => Enemy::shooter(),
                3 => Enemy::shooter(),
                4 => Enemy::meteor(),
                5 => Enemy::zipper(),
                6 => Enemy::turret(),
                7 => Enemy::turret(),
                _ => unreachable!(),
            });
        }
    }
    // Handle player picking up power-ups
    for i in 0..state.players.len() {
        let player = &mut state.players[i];
        state.powerups.retain(|powerup| {
            if check_collision(
                powerup.x,
                powerup.y,
                powerup.width,
                powerup.height,
                player.x,
                player.y,
                player.width,
                player.height,
            ) {
                match powerup.effect {
                    PowerupEffect::Heal => {
                        player.health = (player.health + 1).min(player.max_health);
                        player.skill_points += 1;
                        state.notifications.push("+1 Health".to_string());
                    }
                    PowerupEffect::MaxHealthUp => {
                        player.max_health = (player.max_health + 1).min(5);
                        player.health = player.max_health;
                        player.skill_points += 1;
                        state.notifications.push("Max Health +1".to_string());
                    }
                    PowerupEffect::SpeedBoost => {
                        player.skill_points += 1;
                        player.powerups.push((powerup.effect, POWERUP_FRAMES));
                        state.notifications.push("Speed Boost (30s)".to_string());
                    }
                    PowerupEffect::DamageBoost => {
                        state.notifications.push(format!("Damage Boost (30s)"));
                        player.skill_points += 1;
                        player.powerups.push((powerup.effect, POWERUP_FRAMES));
                    }
                }
                return false; // Remove the power-up after it's picked up
            }
            return true;
        });
    }

    // Update projectiles and check for collisions with enemies
    let mut splashes = vec![];
    for i in 0..state.players.len() {
        let player = &mut state.players[i];
        state.projectiles.retain(|projectile| {
            let mut projectile_active = true;
            if projectile.projectile_owner != ProjectileOwner::Player {
                return projectile_active;
            }
            state.enemies.retain_mut(|enemy| {
                let did_collide = check_collision(
                    projectile.x,
                    projectile.y,
                    projectile.width,
                    projectile.height,
                    enemy.x,
                    enemy.y,
                    enemy.width,
                    enemy.height,
                );
                if did_collide {
                    enemy.health = enemy.health.saturating_sub(projectile.damage);
                    projectile_active = false; // Remove projectile on collision
                    if enemy.health == 0 {
                        state.score += 1;
                        player.skill_points += enemy.points; // To ensure this triggers only once per threshold
                        if state.powerups.len() < MAX_POWERUPS {
                            if rand() % 10 == 0 {
                                if state.powerups.len() < MAX_POWERUPS {
                                    state.powerups.push(Powerup {
                                        x: enemy.x,
                                        y: enemy.y,
                                        width: 8,
                                        height: 8,
                                        effect: PowerupEffect::SpeedBoost, // TODO: maybe randomize
                                        movement: PowerupMovement::Floating(0.1),
                                    });
                                }
                            } else if rand() % 100 == 0 {
                                // Spawn additional power-up when player reaches skill point threshold
                                if player.skill_points > 500 {
                                    // Adjust the skill point threshold
                                    state.powerups.push(Powerup {
                                        x: (rand() % screen_w) as f32,
                                        y: (rand() % screen_h) as f32,
                                        width: 8,
                                        height: 8,
                                        effect: PowerupEffect::DamageBoost, // TODO: maybe randomize
                                        movement: PowerupMovement::Floating(0.5),
                                    });
                                }
                            }
                        }
                    }
                    // Additional behavior based on projectile type
                    match projectile.projectile_type {
                        ProjectileType::Basic => {
                            // ...
                        }
                        ProjectileType::Splatter => {
                            // Splatter creates fragments on impact, affecting a wider area
                            let splash_angles = [45.0, 135.0, 225.0, 315.0]; // Diagonal angles
                            for &angle in splash_angles.iter() {
                                splashes.push(Projectile {
                                    x: projectile.x,
                                    y: projectile.y,
                                    width: projectile.width / 2,
                                    height: projectile.height / 2,
                                    velocity: projectile.velocity / 2.0, // Reduced velocity for splash projectiles
                                    angle,
                                    damage: projectile.damage / 2, // Reduced damage for splash projectiles
                                    projectile_type: ProjectileType::Fragment,
                                    projectile_owner: ProjectileOwner::Player,
                                    ttl: Some(10),
                                });
                            }
                        }
                        ProjectileType::Fragment => {
                            // ...
                        }
                        ProjectileType::Laser => {
                            // ...
                        }
                        ProjectileType::Bomb => {
                            // ...
                        }
                    }
                }
                enemy.health > 0
            });
            projectile_active
        });
    }

    // Handle collisions between enemy projectiles and the player
    for i in 0..state.players.len() {
        let player = &mut state.players[i];
        state.projectiles.retain(|projectile| {
            let mut projectile_active = true;
            if projectile.projectile_owner != ProjectileOwner::Enemy {
                return projectile_active;
            }
            let did_collide = check_collision(
                projectile.x,
                projectile.y,
                projectile.width,
                projectile.height,
                player.x,
                player.y,
                player.width,
                player.height,
            );
            if did_collide && state.hit_timer == 0 && player.health > 0 {
                let prev_hp = player.health;
                player.health = player.health.saturating_sub(projectile.damage);
                state.hit_timer = match (prev_hp, player.health) {
                    (prev, 0) if prev > 0 => DEAD_FRAMES,         // just died
                    (_, curr) if curr > 0 => INVULNERABLE_FRAMES, // damaged
                    _ => state.hit_timer,                         // been dead
                };
                projectile_active = false // Remove the projectile on collision
            }

            projectile_active
        });
    }

    // Add projectile splashes
    for projectile in splashes {
        state.projectiles.push(projectile);
    }

    // Update projectiles
    for projectile in &mut state.projectiles {
        // projectile.y -= projectile.velocity;
        let radian_angle = projectile.angle.to_radians();
        projectile.x += projectile.velocity * radian_angle.cos();
        projectile.y += projectile.velocity * radian_angle.sin();

        if let Some(ttl) = &mut projectile.ttl {
            *ttl = ttl.saturating_sub(1);
        }
    }

    // Remove expired and out-of-bounds projectiles
    state.projectiles.retain(|projectile| {
        let is_alive = projectile.ttl.map_or(true, |ttl| ttl > 0);
        let is_in_bounds = !(projectile.y < -(projectile.height as f32)
            || projectile.x < -(projectile.width as f32)
            || projectile.x > screen_w as f32
            || projectile.y > screen_h as f32);
        is_alive && is_in_bounds
    });

    // Check enemy x player collisions
    for i in 0..state.players.len() {
        let player = &mut state.players[i];
        state.enemies.retain(|enemy| {
            let did_collide = check_collision(
                player.x,
                player.y,
                player.width,
                player.height,
                enemy.x,
                enemy.y,
                enemy.width,
                enemy.height,
            );
            if did_collide && state.hit_timer == 0 && player.health > 0 {
                let prev_hp = player.health;
                player.health = player.health.saturating_sub(enemy.attack + 1);
                state.hit_timer = match (prev_hp, player.health) {
                    (prev, 0) if prev > 0 => DEAD_FRAMES,         // just died
                    (_, curr) if curr > 0 => INVULNERABLE_FRAMES, // damaged
                    _ => state.hit_timer,                         // been dead
                };
            }
            return enemy.y < screen_h as f32;
        });
    }

    for i in 0..state.players.len() {
        let player = &mut state.players[i];
        for enemy in &mut state.enemies {
            match enemy.strategy {
                EnemyStrategy::TargetPlayer(intensity, speed, size) => {
                    // Logic for attacking with specified intensity
                    enemy.y += enemy.speed;
                    if rand() % (250 / intensity as u32) == 0 {
                        // Calculate angle from enemy to player
                        let angle = ((player.y - enemy.y).atan2(player.x - enemy.x) * 180.0)
                            / std::f32::consts::PI;

                        // Create and shoot projectiles from enemy towards the player
                        state.projectiles.push(Projectile {
                            x: enemy.x + (enemy.width as f32 * 0.5) - (size as f32 * 0.5),
                            y: enemy.y + (enemy.height as f32),
                            width: size,
                            height: size,
                            velocity: speed,
                            angle: angle,
                            // damage: intensity as u32, // Damage based on attack intensity
                            damage: 1 + enemy.attack,
                            projectile_type: ProjectileType::Laser, // Assuming enemy uses Laser
                            projectile_owner: ProjectileOwner::Enemy,
                            ttl: None,
                        });
                    }
                }
                EnemyStrategy::ShootDown(intensity, speed, size) => {
                    // Logic for attacking with specified intensity
                    enemy.y += enemy.speed;
                    if rand() % (250 / intensity as u32) == 0 {
                        // Create and shoot projectiles from enemy towards the player
                        state.projectiles.push(Projectile {
                            x: enemy.x + (enemy.width as f32 * 0.5) - (size as f32 * 0.5),
                            y: enemy.y + (enemy.height as f32),
                            width: size,
                            height: size,
                            velocity: speed,
                            angle: 90.0,
                            // damage: intensity as u32, // Damage based on attack intensity
                            damage: 1 + enemy.attack,
                            projectile_type: ProjectileType::Laser, // Assuming enemy uses Laser
                            projectile_owner: ProjectileOwner::Enemy,
                            ttl: None,
                        });
                    }
                }
                EnemyStrategy::MoveDown => {
                    enemy.y += enemy.speed;
                }
                EnemyStrategy::RandomZigZag(angle) => {
                    // Logic for dodging attacks, using angle to determine movement
                    enemy.x += enemy.speed * enemy.angle.cos();
                    enemy.y += enemy.speed;
                    // Reverse direction when heading out of bounds
                    if enemy.x < 0.0 || enemy.x > screen_w as f32 {
                        enemy.angle = std::f32::consts::PI - enemy.angle;
                    }
                    // 5% chance to randomly change angle
                    else if rand() % 20 == 0 {
                        enemy.angle += std::f32::consts::PI / angle; // Change angle
                    }
                }
            }
        }
    }

    // Update power-up positions based on their movement patterns
    for powerup in &mut state.powerups {
        match powerup.movement {
            PowerupMovement::Floating(speed) => {
                powerup.y += speed;
                // Optionally, reverse the direction if it reaches the screen bounds
                if powerup.y <= 0.0 || powerup.y >= screen_h as f32 {
                    powerup.movement = PowerupMovement::Floating(-speed);
                }
            }
            PowerupMovement::Drifting(speed) => {
                powerup.x += speed;
                // Optionally, reverse the direction if it reaches the screen bounds
                if powerup.x <= 0.0 || powerup.x >= screen_w as f32 {
                    powerup.movement = PowerupMovement::Drifting(-speed);
                }
            }
            PowerupMovement::Static => {
                // Static powerups do not move
            }
        }
    }

    // Countdown player powerup timer
    for i in 0..state.players.len() {
        let player = &mut state.players[i];
        player.powerups.retain_mut(|(effect, time)| {
            *time = time.saturating_sub(1);
            let done = *time == 0;
            if done {
                match effect {
                    PowerupEffect::DamageBoost => {
                        state.notifications.push("Damage Boost Ended".to_string());
                    }
                    PowerupEffect::SpeedBoost => {
                        state.notifications.push("Speed Boost Ended".to_string());
                    }
                    _ => {}
                }
            }
            !done
        });
    }

    // Enable skills
    for i in 0..state.players.len() {
        let player = &mut state.players[i];
        if state.score > 100 && !player.skills.speed_boost {
            player.skills.speed_boost = true;
        }
        if state.score > 200 && !player.skills.double_damage {
            player.skills.double_damage = true;
        }
    }

    // Notifications timer
    if state.notifications.len() > 0 {
        state.notification_timer += 1;
        if state.notification_timer >= 120 - 1 {
            state.notification_timer = 0;
            let _ = state.notifications.remove(0);
        }
    }

    // hit timer
    state.hit_timer = state.hit_timer.saturating_sub(1);
}

// Define a function for rendering game elements
fn draw_game_screen(state: &GameState) {
    let [screen_w, screen_h] = resolution();

    if state.hit_timer > 0 {
        canvas::camera::set_xy(rand() as i32 % 3, rand() as i32 % 3)
    } else {
        canvas::camera::set_xy(128, 193);
    }

    // Drawing the character with customization
    let len = state.players.len();
    for i in 0..len {
        let player = &state.players[i];
        if player.health > 0 {
            draw_player(&player, len > 1);
        }
    }

    // Draw enemies
    for enemy in &state.enemies {
        draw_enemy(enemy);
    }

    // Draw projectiles
    for projectile in &state.projectiles {
        draw_projectile(projectile);
    }

    // Draw powerups
    for powerup in &state.powerups {
        draw_powerup(powerup, state.tick);
    }

    // Reset camera
    canvas::camera::set_xy(128, 193);

    // Render notifications
    draw_notifications(state, screen_w, screen_h);

    // Game over text
    let is_game_over = state.players.iter().all(|p| p.health == 0);
    if is_game_over {
        draw_game_over(state, screen_w, screen_h);
    }

    // Draw game HUD
    draw_hud(state, screen_w);
}

fn draw_stars(state: &GameState, screen_w: u32, screen_h: u32) {
    // Define star layers with different speeds
    let star_layers = [
        (54321, 1, 0.15, 10),
        (12345, 1, 0.25, 10),
        (67890, 2, 0.35, 10),
    ];

    for &(seed, size, speed, count) in star_layers.iter() {
        for i in 0..count {
            let rand_x = rand_with_seed(seed + i + state.tick / 10) % screen_w;
            let rand_y = (rand_with_seed(seed + i + state.tick / 10) / screen_w) % screen_h;

            // Adjust position slightly based on player movement
            let adjust_x = state.players[0].x * speed / -5.0;
            let adjust_y = state.players[0].y * speed / -5.0;

            let x = rand_x as i32 + adjust_x as i32;
            let y = (state.tick as f32 * speed) as i32 + rand_y as i32 + adjust_y as i32;

            // Draw the star
            circ!(
                x = x % screen_w as i32,
                y = y % screen_h as i32,
                d = size,
                color = 0xFFFFFF44
            ); // Adjust star size and color if needed
        }
    }
}

fn draw_player(player: &Player, show_number: bool) {
    sprite!(
        "player",
        x = player.x as i32,
        y = player.y as i32,
        color = player.color
    );
    if show_number {
        text!(
            "{}", player.id + 1;
            x = player.x as i32 + 8,
            y = player.y as i32 + 24,
            font = "small",
        );
    }
    if let Some(accessory) = &player.accessory {
        sprite!(accessory, x = player.x as i32, y = player.y as i32);
    }
}

fn draw_enemy(enemy: &Enemy) {
    let x = enemy.x as i32;
    let y = enemy.y as i32;
    rect!(
        color = 0x333333ff,
        w = 10,
        h = 2,
        x = x + (enemy.width / 2) as i32 - 5,
        y = y - 4
    );
    let percent_hp = enemy.health as f32 / enemy.max_health as f32;
    let color = match percent_hp {
        n if n <= 0.25 => 0xff0000ffu32,
        n if n <= 0.5 => 0xff9900ff,
        _ => 0x00ff00ff,
    };
    rect!(
        color = color,
        w = (enemy.health as f32 / enemy.max_health as f32) * 10.,
        h = 2,
        x = x + (enemy.width / 2) as i32 - 5,
        y = y - 4
    );
    sprite!(&enemy.sprite, x = x, y = y);
}

fn draw_projectile(projectile: &Projectile) {
    match projectile.projectile_type {
        ProjectileType::Splatter => {
            sprite!("projectile_ketchup", x = projectile.x, y = projectile.y);
        }
        ProjectileType::Fragment => {
            let color = 0xff0000ffu32;
            ellipse!(
                x = projectile.x,
                y = projectile.y,
                w = projectile.width,
                h = projectile.height,
                color = color
            );
        }
        _ => {
            let color = 0xffff00ffu32;
            ellipse!(
                x = projectile.x,
                y = projectile.y,
                w = projectile.width,
                h = projectile.height,
                color = color
            );
        }
    }
}

fn draw_powerup(powerup: &Powerup, tick: u32) {
    let n = (tick as f32 * 0.15).cos() * 8.0;
    let o = n / 2.;
    let x = (powerup.x - o) as i32;
    let y = (powerup.y - o) as i32;
    ellipse!(
        x = x + 1,
        y = y + 1,
        w = (powerup.width as f32 + n) as u32,
        h = (powerup.height as f32 + n) as u32,
        color = match powerup.effect {
            PowerupEffect::Heal => 0x00ff6666u32,
            PowerupEffect::MaxHealthUp => 0x00ffff66,
            PowerupEffect::DamageBoost => 0xff006666,
            PowerupEffect::SpeedBoost => 0x6600ff66,
        },
        border_color = match powerup.effect {
            PowerupEffect::Heal => 0x00ff6699u32,
            PowerupEffect::MaxHealthUp => 0x00ffff99,
            PowerupEffect::DamageBoost => 0xff006699,
            PowerupEffect::SpeedBoost => 0x6600ff99,
        },
        border_size = 1,
    );
    let key = match powerup.effect {
        PowerupEffect::Heal => "powerup_heal",
        PowerupEffect::MaxHealthUp => "powerup_max_health_up",
        PowerupEffect::DamageBoost => "powerup_damage_boost",
        PowerupEffect::SpeedBoost => "powerup_speed_boost",
    };
    sprite!(key, x = powerup.x, y = powerup.y);
}

fn draw_hud(state: &GameState, screen_w: u32) {
    let center = (screen_w / 2) as i32;
    let left = center - 128;
    sprite!("ui_bar", x = left);

    // Displaying game information on the HUD
    let text_color = 0x181425ff; // White text color

    // Display Health
    for i in 0..state.players[0].max_health {
        let n = (i as i32) * 8;
        sprite!("hp", x = left + 26 + n, y = 4, color = 0x222222aa);
    }
    for i in 0..state.players[0].health {
        let n = (i as i32) * 8;
        sprite!("hp", x = left + 26 + n, y = 4);
    }

    // Display Skill Points
    let skill_points_text = format!("{:0>5}", state.players[0].skill_points);
    text!(
        &skill_points_text,
        x = center + 68,
        y = 6,
        font = "large",
        color = text_color
    );
}

fn draw_notifications(state: &GameState, screen_w: u32, _screen_h: u32) {
    // Render notifications
    for notif in &state.notifications {
        let len = notif.chars().count();
        let w = len * 8;
        let x = (screen_w as usize / 2) - (w / 2);
        rect!(
            w = w as u32 + 4,
            h = 14,
            x = x - 2,
            y = 24 - 2,
            color = 0x68386cff,
            border_radius = 4,
            border_size = 1,
            border_color = 0xb55088ff,
        );
        text!(notif, x = x, y = 26, font = "large", color = 0xf6757aff);
        break;
    }
}

fn draw_game_over(state: &GameState, screen_w: u32, screen_h: u32) {
    text!(
        "GAME OVER",
        x = (screen_w / 2) - 32,
        y = (screen_h / 2) - 4,
        font = "large"
    );
    if state.hit_timer == 0 {
        if state.tick / 4 % 8 < 4 {
            text!(
                "PRESS START",
                x = (screen_w / 2) - 24,
                y = (screen_h / 2) - 4 + 16,
                font = "medium"
            );
        }
    }
}

// Function to check collision between two rectangular objects
#[rustfmt::skip]
fn check_collision(x1: f32, y1: f32, w1: u32, h1: u32, x2: f32, y2: f32, w2: u32, h2: u32) -> bool {
    let x1 = x1 as i32;
    let y1 = y1 as i32;
    let w1 = w1 as i32;
    let h1 = h1 as i32;
    let x2 = x2 as i32;
    let y2 = y2 as i32;
    let w2 = w2 as i32;
    let h2 = h2 as i32;
    x1 < x2 + w2 && x1 + w1 > x2 &&
    y1 < y2 + h2 && y1 + h1 > y2
}

// Pseudo-random number generator
fn rand_with_seed(seed: u32) -> u32 {
    (seed * 1103515245 + 12345) % 2147483648
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, PartialEq)]
// Struct for Player properties
struct Player {
    id: u32,
    x: f32,
    y: f32,
    width: u32,
    height: u32,
    health: u32,
    max_health: u32,
    speed: f32,
    color: u32,
    projectile_type: ProjectileType,
    projectile_damage: u32,
    accessory: Option<String>,
    skill_points: u32,
    skills: Skills,
    powerups: Vec<(PowerupEffect, u32)>, // effect + time remaining
    metrics: PlayerMetrics,
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, PartialEq)]
// Struct for Player metrics that can be used to deterministically compute player achievements
struct PlayerMetrics {
    longest_run_seconds: f32,
    num_projectiles_collected: u32,
    num_enemies_defeated: u32,
}
impl PlayerMetrics {
    pub fn new() -> Self {
        Self {
            longest_run_seconds: 0.0,
            num_projectiles_collected: 0,
            num_enemies_defeated: 0,
        }
    }
}

#[derive(Debug, Copy, Clone, BorshDeserialize, BorshSerialize, PartialEq)]
struct Skills {
    speed_boost: bool,
    double_damage: bool,
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, PartialEq)]
// Struct for Projectiles shot by the player
struct Projectile {
    x: f32,
    y: f32,
    width: u32,
    height: u32,
    velocity: f32,
    angle: f32,
    damage: u32,
    projectile_type: ProjectileType,
    projectile_owner: ProjectileOwner,
    ttl: Option<u32>,
}

#[derive(Debug, Copy, Clone, BorshDeserialize, BorshSerialize, PartialEq)]
enum ProjectileType {
    Basic,
    Splatter,
    Fragment,
    Laser,
    Bomb,
}

#[derive(Debug, Copy, Clone, BorshDeserialize, BorshSerialize, PartialEq)]
enum ProjectileOwner {
    Enemy,
    Player,
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, PartialEq)]
// Struct for Enemies
struct Enemy {
    sprite: String,
    x: f32,
    y: f32,
    width: u32,
    height: u32,
    health: u32,
    max_health: u32,
    attack: u32,
    speed: f32,
    angle: f32,
    points: u32,
    strategy: EnemyStrategy,
}
impl Enemy {
    pub fn tank() -> Self {
        let [screen_w, _] = resolution();
        Self {
            sprite: "enemy_tank".to_string(),
            x: (rand() % screen_w - 32) as f32,
            y: -32.0,
            width: 30,
            height: 33,
            attack: 1,
            health: 6,
            max_health: 6,
            speed: 0.5,
            angle: 0.0,
            points: 50,
            strategy: EnemyStrategy::TargetPlayer(1.0, 2.5, 16),
        }
    }
    pub fn shooter() -> Self {
        let [screen_w, _] = resolution();
        Self {
            sprite: "enemy_shooter".to_string(),
            x: (rand() % screen_w - 16) as f32,
            y: -16.0,
            width: 22,
            height: 17,
            health: 4,
            max_health: 4,
            attack: 0,
            speed: 1.0,
            angle: 0.0,
            points: 30,
            strategy: EnemyStrategy::TargetPlayer(3.0, 2.0, 4),
        }
    }
    pub fn turret() -> Self {
        let [screen_w, _] = resolution();
        Self {
            sprite: "enemy_turret".to_string(),
            x: (rand() % screen_w - 16) as f32,
            y: -8.0,
            width: 11,
            height: 19,
            health: 3,
            max_health: 3,
            attack: 0,
            speed: 1.5,
            angle: 0.0,
            points: 30,
            strategy: EnemyStrategy::ShootDown(2.0, 2.5, 2),
        }
    }
    pub fn zipper() -> Self {
        let [screen_w, _] = resolution();
        Self {
            sprite: "enemy_zipper".to_string(),
            x: (rand() % screen_w - 16) as f32,
            y: -16.0,
            width: 14,
            height: 16,
            health: 3,
            max_health: 3,
            attack: 0,
            speed: 0.5,
            angle: 0.0,
            points: 20,
            strategy: EnemyStrategy::RandomZigZag(1.0),
        }
    }
    pub fn meteor() -> Self {
        let [screen_w, _] = resolution();
        Self {
            sprite: "enemy_meteor".to_string(),
            x: (rand() % screen_w - 8) as f32,
            y: -8.0,
            width: 9,
            height: 9,
            health: 2,
            max_health: 2,
            attack: 0,
            speed: 3.0,
            angle: 0.0,
            points: 20,
            strategy: EnemyStrategy::MoveDown,
        }
    }
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, PartialEq)]
// AI States for enemy behavior
enum EnemyStrategy {
    TargetPlayer(f32, f32, u32), // Moves down. Attacks with given intensity, speed, and size
    ShootDown(f32, f32, u32),    // Moves down. Attacks with given intensity, speed, and size
    MoveDown,                    // Moves down. Nothing fancy
    RandomZigZag(f32),           // Moves in a random zig zag pattern with a given angle
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, PartialEq)]
struct Level {
    id: u32,
    name: String,
    difficulty: DifficultyLevel,
}

#[derive(Debug, Copy, Clone, BorshDeserialize, BorshSerialize, PartialEq)]
enum DifficultyLevel {
    Easy,
    Medium,
    Hard,
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, PartialEq)]
struct Powerup {
    x: f32,
    y: f32,
    width: u32,
    height: u32,
    effect: PowerupEffect,
    movement: PowerupMovement,
}

#[derive(Debug, Copy, Clone, BorshDeserialize, BorshSerialize, PartialEq)]
enum PowerupEffect {
    Heal,        // Heals the player when interacted with
    MaxHealthUp, // Increases max health
    SpeedBoost,  // Temporarily increases player's speed
    DamageBoost, // Temporarily increases projectile's damage
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, PartialEq)]
enum PowerupMovement {
    Static,
    Floating(f32), // Vertical floating speed
    Drifting(f32), // Horizontal drifting speed
}
