turbo::cfg! {r#"
    name = "Space Shooter"
    version = "2.5.0"
    author = "Turbo"
    description = "A thrilling projectile combat adventure in space!"
    [settings]
    resolution = [512, 512]
"#}

turbo::init! {
    struct GameState {
        tick: u32,
        notification_timer: u32,
        hit_timer: u32,

        // Game elements
        score: u32,
        tutorial_active: bool,
        help_messages: Vec<String>,
        current_quest: Option<Quest>,
        notifications: Vec<String>,
        unlockables: Unlockables,

        // Entities
        player: Player,
        boss: Option<Boss>,
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
            tick: 0,
            notification_timer: 0,
            hit_timer: 0,
            score: 0,
            tutorial_active: true,
            help_messages: vec![
                String::from("Use arrow keys to move"),
                String::from("Press A to shoot projectiles"),
            ],
            current_quest: Some(Quest::defeat_boss(
                "Defeat the First Boss",
                "A quest to defeat the infamous first boss!",
            )),
            notifications: vec![
                "Use arrow keys to move.".to_string(),
                "Press SPACE or A to shoot.".to_string(),
                "Defeat enemies and collect powerups.".to_string(),
                "Try to not die. Good luck!".to_string(),
            ],
            unlockables: Unlockables::new(),
            player: Player {
                x: ((screen_w / 2) - 8) as f32,
                y: (screen_h - 64) as f32,
                width: 16,
                height: 16,
                health: 3,
                max_health: 3,
                speed: 2.0,
                color: 0xFF00FFFF,
                accessory: None,
                projectile_type: ProjectileType::Splatter,
                projectile_damage: 1,
                skill_points: 0,
                skills: Skills {
                    speed_boost: false,
                    double_damage: false,
                },
                metrics: PlayerMetrics::new(),
            },
            boss: None,
            projectiles: vec![],
            enemies: vec![],
            powerups: vec![],
        }
    }
}

turbo::go! {
    let mut state = GameState::load();
    if gamepad(0).b.pressed() {
        let health = state.player.health;
        let xy = (state.player.x, state.player.y);
        let score = state.score;
        let res = canvas_size!();
        log!("- Health = {health}\n- Position: {xy:?}\n- Score: {score}\n- Resolution: {res:?}");
    }

    let [screen_w, screen_h] = resolution();

    // Drawing all game elements, including player, enemies, environment, and UI
    draw_game_elements(&state);

    if state.player.health == 0 {
        // Restart
        if state.hit_timer == 0 && gamepad(0).start.just_pressed() || gamepad(0).a.just_pressed() {
            state = GameState::new();
        }
    } else {
        // Player movement handling
        if gamepad(0).up.pressed() {
            state.player.y = (state.player.y - state.player.speed).max(0.0); // Move up
        }
        if gamepad(0).down.pressed() {
            state.player.y = (state.player.y + state.player.speed).min((screen_h - state.player.height) as f32); // Move down
        }
        if gamepad(0).left.pressed() {
            state.player.x = (state.player.x - state.player.speed).max(0.0); // Move left
        }
        if gamepad(0).right.pressed() {
            state.player.x = (state.player.x + state.player.speed).min((screen_w - state.player.width) as f32); // Move right
        }

        // Shooting projectiles
        if gamepad(1).start.just_pressed() || gamepad(1).a.just_pressed() {
            state.projectiles.push(Projectile {
                x: state.player.x + ((state.player.width / 2) as f32) - 2.0,
                y: state.player.y,
                width: 8,
                height: 8,
                velocity: 5.0,
                angle: -90.0,
                damage: state.player.projectile_damage,
                projectile_type: state.player.projectile_type,
                projectile_owner: ProjectileOwner::Player,
                ttl: None,
            });
        }
    }

    // Every 30s, spawn a heal at a random location
    if state.tick % (60 * 30) == 0 {
        state.powerups.push(Powerup {
            x: (rand() % screen_w) as f32,
            y: 24.0 + (rand() % screen_h / 2) as f32,
            width: 8,
            height: 8,
            effect: PowerupEffect::Heal,
            movement: PowerupMovement::Drifting(0.75),
        });
    }

    // Spawn a heal every 10s when player's health is low
    if state.tick % (60 * 10) == 0 && state.player.health == 1 {
        state.powerups.push(Powerup {
            x: (rand() % screen_w) as f32,
            y: (rand() % screen_h) as f32,
            width: 8,
            height: 8,
            effect: PowerupEffect::MaxHealthUp,
            movement: PowerupMovement::Floating(0.5),
        });
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
            initial_spawn_rate.saturating_sub(state.tick / speed_up_rate)
        );
        if state.player.health > 0 {
            text!(&format!("spawn rate: {spawn_rate}"), x = 4, y = 22, font = Font::S);
        }
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
                _ => unreachable!()
            });
        }
    }
    // Handle player picking up power-ups
    state.powerups.retain(|powerup| {
        if check_collision(powerup.x, powerup.y, powerup.width, powerup.height,
                        state.player.x, state.player.y, state.player.width, state.player.height) {
            match powerup.effect {
                PowerupEffect::Heal => {
                    state.player.health = (state.player.health + 1).min(state.player.max_health);
                    state.player.skill_points += 1;
                    state.notifications.push("+1 Health".to_string());
                },
                PowerupEffect::MaxHealthUp => {
                    state.player.max_health = (state.player.max_health + 1).min(10);
                    state.player.health = state.player.max_health;
                    state.player.skill_points += 1;
                    state.notifications.push("Max Health +1".to_string());
                },
                PowerupEffect::SpeedBoost => {
                    state.player.speed *= 1.1;
                    state.player.skill_points += 1;
                    state.notifications.push("1.1x Speed Boost".to_string());
                },
                PowerupEffect::DamageBoost(projectile_type) => {
                    if state.player.projectile_type == projectile_type {
                        state.notifications.push(format!("+1 {projectile_type:?} Damage"));
                        state.player.skill_points += 1;
                        state.player.projectile_damage = (state.player.projectile_damage + 1).min(2);
                    }
                }
            }
            false // Remove the power-up after it's picked up
        } else {
            true
        }
    });

    // Update projectiles and check for collisions with enemies
    let mut splashes = vec![];
    state.projectiles.retain(|projectile| {
        let mut projectile_active = true;
        if projectile.projectile_owner != ProjectileOwner::Player {
            return projectile_active;
        }
        state.enemies.retain_mut(|enemy| {
            let did_collide = check_collision(
                projectile.x, projectile.y, projectile.width, projectile.height,
                enemy.x, enemy.y, enemy.width, enemy.height
            );
            if did_collide {
                enemy.health = enemy.health.saturating_sub(projectile.damage);
                projectile_active = false; // Remove projectile on collision
                if enemy.health == 0 {
                    state.score += 1;
                    state.player.skill_points += enemy.points; // To ensure this triggers only once per threshold
                    if rand() % 10 == 0 {
                        state.powerups.push(Powerup {
                            x: enemy.x,
                            y: enemy.y,
                            width: 8,
                            height: 8,
                            effect: PowerupEffect::SpeedBoost, // TODO: maybe randomize
                            movement: PowerupMovement::Floating(0.1),
                        });
                        // Spawn additional power-up when player reaches skill point threshold
                        if state.player.skill_points > 500 { // Adjust the skill point threshold
                            state.powerups.push(Powerup {
                                x: (rand() % screen_w) as f32,
                                y: (rand() % screen_h) as f32,
                                width: 8,
                                height: 8,
                                effect: PowerupEffect::DamageBoost(ProjectileType::Splatter), // TODO: maybe randomize
                                movement: PowerupMovement::Floating(0.5),
                            });
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
                        let splash_angles = [45.0, 135.0, 225.0, 315.0];  // Diagonal angles
                        for &angle in splash_angles.iter() {
                            splashes.push(Projectile {
                                x: projectile.x,
                                y: projectile.y,
                                width: projectile.width,
                                height: projectile.height,
                                velocity: projectile.velocity / 2.0,  // Reduced velocity for splash projectiles
                                angle,
                                damage: projectile.damage / 2,  // Reduced damage for splash projectiles
                                projectile_type: ProjectileType::Fragment,
                                projectile_owner: ProjectileOwner::Player,
                                ttl: Some(10)
                            });
                        }
                    },
                    ProjectileType::Fragment => {
                        // ...
                    },
                    ProjectileType::Laser => {
                        // ...
                    },
                    ProjectileType::Bomb => {
                        // ...
                    },
                }
            }
            enemy.health > 0
        });
        projectile_active
    });

    // Check if a boss is present and handle collision with projectiles
    state.projectiles.retain(|projectile| {
        let mut projectile_active = true;
        if projectile.projectile_owner != ProjectileOwner::Player {
            return projectile_active;
        }
        if let Some(boss) = &mut state.boss {
            let did_collide = check_collision(
                projectile.x, projectile.y, projectile.width, projectile.height,
                boss.enemy.x, boss.enemy.y, boss.enemy.width, boss.enemy.height
            );
            if did_collide {
                boss.enemy.health -= projectile.damage;
                projectile_active = false; // Remove projectile on collision
                if boss.enemy.health == 0 {
                    state.score += 10; // Award more points for defeating a boss
                    state.boss = None; // Remove boss on defeat
                }
                // Additional behavior based on projectile type
                match projectile.projectile_type {
                    ProjectileType::Basic => {
                        // ...
                    }
                    ProjectileType::Splatter => {
                        // Splatter creates fragments on impact, affecting a wider area
                        let splash_angles = [45.0, 135.0, 225.0, 315.0];  // Diagonal angles
                        for &angle in splash_angles.iter() {
                            splashes.push(Projectile {
                                x: projectile.x,
                                y: projectile.y,
                                width: projectile.width,
                                height: projectile.height,
                                velocity: projectile.velocity / 2.0,  // Reduced velocity for splash projectiles
                                angle,
                                damage: projectile.damage / 2,  // Reduced damage for splash projectiles
                                projectile_type: ProjectileType::Fragment,
                                projectile_owner: ProjectileOwner::Player,
                                ttl: Some(10)
                            });
                        }
                    },
                    ProjectileType::Fragment => {
                        // ...
                    },
                    ProjectileType::Laser => {
                        // ...
                    },
                    ProjectileType::Bomb => {
                        // ...
                    },
                }
            }
        }
        return projectile_active;
    });

    // Handle collisions between enemy projectiles and the player
    state.projectiles.retain(|projectile| {
        let mut projectile_active = true;
        if projectile.projectile_owner != ProjectileOwner::Enemy {
            return projectile_active;
        }
        let did_collide = check_collision(
            projectile.x, projectile.y, projectile.width, projectile.height,
            state.player.x, state.player.y, state.player.width, state.player.height
        );
        if did_collide {
            let prev_hp = state.player.health;
            state.player.health = state.player.health.saturating_sub(projectile.damage);
            // hit timer is longer on final hit
            state.hit_timer = if prev_hp > 0 && state.player.health == 0 { 240 } else { 10 };
            projectile_active = false // Remove the projectile on collision
        }

        projectile_active
    });


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
        projectile.ttl.map_or(true, |ttl| ttl > 0) ||
        projectile.y < -(projectile.height as f32) ||
        projectile.x < -(projectile.width as f32) ||
        projectile.x > screen_w as f32 ||
        projectile.y > screen_h as f32
    });

    // Check enemy x player collisions
    state.enemies.retain(|enemy| {
        let did_collide = check_collision(
            state.player.x, state.player.y, state.player.width, state.player.height,
            enemy.x, enemy.y, enemy.width, enemy.height
        );
        if did_collide {
            // Collision detected, reduce player health
            state.player.health = state.player.health.saturating_sub(1); // Adjust damage as needed
            // return false;
        }
        return enemy.y < screen_h as f32;
    });

    for enemy in &mut state.enemies {
        match enemy.strategy {
            EnemyStrategy::TargetPlayer(intensity, speed, size) => {
                // Logic for attacking with specified intensity
                enemy.y += enemy.speed;
                if rand() % (250 / intensity as u32) == 0 {
                    // Calculate angle from enemy to player
                    let angle = ((state.player.y - enemy.y).atan2(state.player.x - enemy.x) * 180.0) / std::f32::consts::PI;

                    // Create and shoot projectiles from enemy towards the player
                    state.projectiles.push(Projectile {
                        x: enemy.x + (enemy.width as f32 * 0.5) - (size as f32 * 0.5),
                        y: enemy.y + (enemy.height as f32),
                        width: size,
                        height: size,
                        velocity: speed,
                        angle: angle,
                        // damage: intensity as u32, // Damage based on attack intensity
                        damage: 1,
                        projectile_type: ProjectileType::Laser, // Assuming enemy uses Laser
                        projectile_owner: ProjectileOwner::Enemy,
                        ttl: None,
                    });
                }
            },
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
                        damage: 1,
                        projectile_type: ProjectileType::Laser, // Assuming enemy uses Laser
                        projectile_owner: ProjectileOwner::Enemy,
                        ttl: None,
                    });
                }
            },
            EnemyStrategy::MoveDown => {
                enemy.y += enemy.speed;
            },
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
            },
        }
    }

    if let Some(boss) = &mut state.boss {
        // Logic for attacking with specified intensity
        let intensity = 4.0;
        if rand() % (100 / intensity as u32) == 0 {
            // Calculate angle from enemy to player
            let angle = ((state.player.y - boss.enemy.y).atan2(state.player.x - boss.enemy.x) * 180.0) / std::f32::consts::PI;

            // Create and shoot projectiles from enemy towards the player
            state.projectiles.push(Projectile {
                x: boss.enemy.x,
                y: boss.enemy.y,
                width: 4,
                height: 4,
                velocity: intensity * 2.0, // Velocity based on attack intensity
                angle: angle,
                damage: intensity as u32, // Damage based on attack intensity
                projectile_type: ProjectileType::Laser, // Assuming enemy uses Laser
                projectile_owner: ProjectileOwner::Enemy,
                ttl: None,
            });
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
            },
            PowerupMovement::Drifting(speed) => {
                powerup.x += speed;
                // Optionally, reverse the direction if it reaches the screen bounds
                if powerup.x <= 0.0 || powerup.x >= screen_w as f32 {
                    powerup.movement = PowerupMovement::Drifting(-speed);
                }
            },
            PowerupMovement::Static => {
                // Static powerups do not move
            },
        }
    }

    // Check for quest completion
    if let Some(quest) = &mut state.current_quest {
        if !quest.completed {
            match quest.objective {
                QuestObjective::DefeatBoss => {
                    if state.boss.as_ref().map_or(false, |b| b.enemy.health == 0) {
                        quest.completed = true;
                        let boss = state.boss.as_ref().unwrap();
                        state.player.metrics.bosses_defeated.push(boss.boss_type.clone());
                        state.player.metrics.completed_quests.push(quest.clone());
                        // Check and update boss-gated unlockables
                        match boss.boss_type {
                            BossType::FirstBoss => {
                                state.unlockables.special_ability = Some(SpecialAbility::Slow);
                            }
                        }
                        state.notifications.push(format!("Quest completed: {}", quest.title));
                    }
                }
                QuestObjective::CollectProjectiles(num_projectiles) => {
                    if state.player.metrics.num_projectiles_collected >= num_projectiles {
                        quest.completed = true;
                        state.player.metrics.completed_quests.push(quest.clone());
                        state.notifications.push(format!("Quest completed: {}", quest.title));
                    }
                }
                QuestObjective::DefeatEnemies(num_enemies) => {
                    if state.player.metrics.num_enemies_defeated >= num_enemies {
                        quest.completed = true;
                        state.player.metrics.completed_quests.push(quest.clone());
                        state.notifications.push(format!("Quest completed: {}", quest.title));
                    }
                }
                QuestObjective::SkillPoints(num_skill_points) => {
                    if state.player.skill_points >= num_skill_points {
                        quest.completed = true;
                        state.player.metrics.completed_quests.push(quest.clone());
                        state.notifications.push(format!("Quest completed: {}", quest.title));
                    }
                }
            }
        }
    }

    // Enable skills
    if state.score > 100 && !state.player.skills.speed_boost {
        state.player.skills.speed_boost = true;
    }
    if state.score > 200 && !state.player.skills.double_damage {
        state.player.skills.double_damage = true;
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

    state.tick += 1;
    state.save();
}

// Define a function for rendering game elements
fn draw_game_elements(state: &GameState) {
    let [screen_w, screen_h] = resolution();

    if state.hit_timer > 0 {
        set_camera(rand() as i32 % 3, rand() as i32 % 3);
    } else {
        set_camera(0, 0);
    }

    // Draw moving parallax stars in the background
    draw_stars(state, screen_w, screen_h);

    // Drawing the character with customization
    if state.player.health > 0 {
        draw_player(&state.player);
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
    set_camera(0, 0);

    // Render notifications
    draw_notifications(state, screen_w, screen_h);

    // Game over text
    if state.player.health == 0 {
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
            let adjust_x = state.player.x * speed / -5.0;
            let adjust_y = state.player.y * speed / -5.0;

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

fn draw_player(player: &Player) {
    rect!(
        x = player.x,
        y = player.y,
        w = player.width,
        h = player.height,
        color = player.color
    );
    if let Some(accessory) = &player.accessory {
        sprite!(accessory, x = player.x, y = player.y);
    }
}

fn draw_enemy(enemy: &Enemy) {
    rect!(
        x = enemy.x,
        y = enemy.y,
        w = enemy.width,
        h = enemy.height,
        color = 0xaaaaaaff
    );
}

fn draw_projectile(projectile: &Projectile) {
    let color = match projectile.projectile_type {
        ProjectileType::Splatter | ProjectileType::Fragment => 0xff000ff,
        ProjectileType::Basic | ProjectileType::Bomb | ProjectileType::Laser => 0xfff00ff,
    };
    circ!(
        x = projectile.x,
        y = projectile.y,
        d = projectile.width.max(projectile.height),
        color = color
    );
}

fn draw_powerup(powerup: &Powerup, tick: u32) {
    let n = (tick as f32 * 0.15).cos() * 3.0;
    circ!(
        x = (powerup.x - (n * 0.5)) as i32,
        y = (powerup.y - (n * 0.5)) as i32,
        d = powerup.width.max(powerup.height) + n as u32,
        color = match powerup.effect {
            PowerupEffect::Heal => 0x00ff66ff,
            PowerupEffect::MaxHealthUp => 0x00ffffff,
            PowerupEffect::DamageBoost(_) => 0xff0066f,
            PowerupEffect::SpeedBoost => 0x6600ffff,
        }
    );
    sprite!("powerup_sprite", x = powerup.x as i32, y = powerup.y as i32);
}

fn draw_hud(state: &GameState, screen_w: u32) {
    // Drawing the HUD panel
    let hud_height = 16; // Height of the HUD panel
    rect!(
        x = 0,
        y = 0,
        w = screen_w,
        h = hud_height,
        color = 0x000000ff
    ); // Black background for the HUD

    // Drawing borders for the HUD section
    rect!(
        x = 0,
        y = hud_height,
        w = screen_w,
        h = 1,
        // border = 1,
        color = 0xffffffff
    ); // White border

    // Displaying game information on the HUD
    let hud_padding = 4; // Padding inside the HUD
    let text_color = 0xfffffff; // White text color

    // Display Score
    let score_text = format!("LVL: 1");
    text!(
        &score_text,
        x = hud_padding,
        y = hud_padding,
        font = Font::L,
        color = text_color
    );

    // Display Health
    let health_text = format!("HP: {}", state.player.health);
    let health_text_x = (screen_w as i32 / 2) - ((health_text.chars().count() as i32 * 8) / 2);
    text!(
        &health_text,
        x = health_text_x,
        y = hud_padding,
        font = Font::L,
        color = text_color
    );

    // Display Skill Points
    let skill_points_text = format!("XP: {:0>5}", state.player.skill_points);
    let skill_points_text_x =
        screen_w as i32 - (skill_points_text.chars().count() as i32 * 8) - hud_padding;
    text!(
        &skill_points_text,
        x = skill_points_text_x,
        y = hud_padding,
        font = Font::L,
        color = text_color
    );
}

fn draw_notifications(state: &GameState, screen_w: u32, screen_h: u32) {
    // Render notifications
    for notif in &state.notifications {
        let len = notif.chars().count();
        let w = len * 5;
        let x = (screen_w as usize / 2) - (w / 2);
        rect!(w = w + 4, h = 10, x = x - 2, y = 24 - 2, color = 0x22aaaaff);
        text!(notif, x = x, y = 24, font = Font::M, color = 0xffffffff);
        break;
    }
}

fn draw_game_over(state: &GameState, screen_w: u32, screen_h: u32) {
    text!(
        "GAME OVER",
        x = (screen_w / 2) - 32,
        y = (screen_h / 2) - 4,
        font = Font::L
    );
    if state.tick / 4 % 8 < 4 {
        text!(
            "PRESS START",
            x = (screen_w / 2) - 24,
            y = (screen_h / 2) - 4 + 16,
            font = Font::M
        );
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
    metrics: PlayerMetrics,
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, PartialEq)]
// Struct for Player metrics that can be used to deterministically compute player achievements
struct PlayerMetrics {
    longest_run_seconds: f32,
    num_projectiles_collected: u32,
    num_enemies_defeated: u32,
    completed_quests: Vec<Quest>,
    bosses_defeated: Vec<BossType>,
}
impl PlayerMetrics {
    pub fn new() -> Self {
        Self {
            longest_run_seconds: 0.0,
            num_projectiles_collected: 0,
            num_enemies_defeated: 0,
            completed_quests: vec![],
            bosses_defeated: vec![],
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
// Struct for Boss that is basically an enemy with a type
struct Boss {
    boss_type: BossType,
    enemy: Enemy,
}

#[derive(Debug, Copy, Clone, BorshDeserialize, BorshSerialize, PartialEq)]
enum BossType {
    FirstBoss,
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, PartialEq)]
// Struct for Enemies
struct Enemy {
    x: f32,
    y: f32,
    width: u32,
    height: u32,
    health: u32,
    speed: f32,
    angle: f32,
    points: u32,
    strategy: EnemyStrategy,
}
impl Enemy {
    pub fn tank() -> Self {
        let [screen_w, _] = resolution();
        Self {
            x: (rand() % screen_w - 32) as f32,
            y: -32.0,
            width: 32,
            height: 32,
            health: 5,
            speed: 0.5,
            angle: 0.0,
            points: 50,
            strategy: EnemyStrategy::TargetPlayer(1.0, 2.5, 16),
        }
    }
    pub fn shooter() -> Self {
        let [screen_w, _] = resolution();
        Self {
            x: (rand() % screen_w - 16) as f32,
            y: -16.0,
            width: 16,
            height: 16,
            health: 3,
            speed: 1.0,
            angle: 0.0,
            points: 30,
            strategy: EnemyStrategy::TargetPlayer(3.0, 2.0, 4),
        }
    }
    pub fn turret() -> Self {
        let [screen_w, _] = resolution();
        Self {
            x: (rand() % screen_w - 16) as f32,
            y: -8.0,
            width: 16,
            height: 8,
            health: 3,
            speed: 1.5,
            angle: 0.0,
            points: 30,
            strategy: EnemyStrategy::ShootDown(2.0, 2.5, 2),
        }
    }
    pub fn zipper() -> Self {
        let [screen_w, _] = resolution();
        Self {
            x: (rand() % screen_w - 16) as f32,
            y: -16.0,
            width: 16,
            height: 16,
            health: 2,
            speed: 0.5,
            angle: 0.0,
            points: 20,
            strategy: EnemyStrategy::RandomZigZag(1.0),
        }
    }
    pub fn meteor() -> Self {
        let [screen_w, _] = resolution();
        Self {
            x: (rand() % screen_w - 8) as f32,
            y: -8.0,
            width: 8,
            height: 8,
            health: 2,
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
// Struct for Quests in the game
struct Quest {
    title: String,
    description: String,
    completed: bool,
    objective: QuestObjective,
}
impl Quest {
    fn defeat_boss(title: &str, description: &str) -> Self {
        Self {
            title: title.to_string(),
            description: description.to_string(),
            completed: false,
            objective: QuestObjective::DefeatBoss,
        }
    }
}

#[derive(Debug, Copy, Clone, BorshDeserialize, BorshSerialize, PartialEq)]
enum QuestObjective {
    DefeatBoss,              // Defeat the boss
    DefeatEnemies(u32),      // Number of enemies to defeat
    CollectProjectiles(u32), // Number of items to collect
    SkillPoints(u32),        // Number of skill points to obtain
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, PartialEq)]
// Struct for Unlockable content
struct Unlockables {
    special_ability: Option<SpecialAbility>,
    extra_levels: Vec<Level>,
    cosmetic_items: Vec<String>,
}
impl Unlockables {
    fn new() -> Self {
        Self {
            special_ability: None,
            extra_levels: vec![],
            cosmetic_items: vec![],
        }
    }
}

#[derive(Debug, Copy, Clone, BorshDeserialize, BorshSerialize, PartialEq)]
enum SpecialAbility {
    ChainDamage,
    AutomaticWeapons,
    Armor(u32),
    Regen,
    Vampire,
    Lucky,
    Slow,
    Freeze,
    Poison,
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

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, PartialEq)]
enum PowerupEffect {
    Heal,                        // Heals the player when interacted with
    MaxHealthUp,                 // Increases max health
    SpeedBoost,                  // Temporarily increases player's speed
    DamageBoost(ProjectileType), // Temporarily increases projectile's damage
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, PartialEq)]
enum PowerupMovement {
    Static,
    Floating(f32), // Vertical floating speed
    Drifting(f32), // Horizontal drifting speed
}
