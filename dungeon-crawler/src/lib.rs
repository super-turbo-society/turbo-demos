use borsh::{BorshDeserialize, BorshSerialize};
use os::ReadFileError;

turbo::cfg! {r#"
    name = "Dungeon Crawler"
    [settings]
    resolution = [132, 224]
    [turbo-os]
    # api-url = "https://os.turbo.computer"
    api-url = "http://localhost:8000"
"#}

turbo::init! {
    struct LocalState {
        floor: Tween<u32>,
        turn: Tween<u32>,
        last_exec_at: usize,
        last_exec_turn: Option<u32>,
        player: struct Entity {
            x: Tween<i32>,
            y: Tween<i32>,
            offset_x: Tween<i32>,
            offset_y: Tween<i32>,
        },
        monsters: Vec<Entity>,
        leaderboard_kind: enum LeaderboardKind {
            Floor,
            Gold,
        },
    } = {
        Self {
            floor: Tween::new(0).duration(FLOOR_DUR),
            turn: Tween::new(0).duration(TURN_DUR),
            last_exec_at: 0,
            last_exec_turn: None,
            player: Entity {
                x: Tween::new(0).duration(MOVE_DUR).ease(Easing::EaseInOutQuad),
                y: Tween::new(0).duration(MOVE_DUR).ease(Easing::EaseInOutQuad),
                offset_x: Tween::new(0).duration(MOVE_DUR / 2).ease(Easing::EaseInOutQuad),
                offset_y: Tween::new(0).duration(MOVE_DUR / 2).ease(Easing::EaseInOutQuad),
            },
            monsters: vec![],
            leaderboard_kind: LeaderboardKind::Floor,
        }
    }
}

impl Entity {
    fn is_idle(&mut self) -> bool {
        let is_x_done = self.x.done();
        let is_y_done = self.y.done();
        let is_offset_x_done = self.offset_x.done();
        let is_offset_y_done = self.offset_y.done();
        is_x_done && is_y_done && is_offset_x_done && is_offset_y_done
    }
}

const TILE_SIZE: i32 = 16;
const TURN_DUR: usize = 20;
const FLOOR_DUR: usize = 32;
const MOVE_DUR: usize = 10;
const MOVE_Y_OFFSET: i32 = 6;
const MOVE_X_OFFSET: i32 = 6;
const EXEC_TIMEOUT_DUR: usize = 32;
const SHADOW_COLOR: u32 = 0x000000dd;

// floor, dark_floor, wood_floor
// wall, metal_block, crate
// firepit,
const THEMES: [(&'static str, &'static str, &'static str); 3] = [
    // Fortress
    ("floor", "wall", "firepit"),
    // Crypt
    ("dark_floor", "metal_block", "metal_crate"),
    // Pirate
    ("wood_floor", "crate", "barrel"),
];

turbo::go!({
    // Load the game state
    let mut state = LocalState::load();

    // Clear the screen
    clear(0x000000ff);
    // clear(0x1b1126ff);

    let [w, h] = canvas_size!();
    sprite!(
        "dotted_tile_border",
        w = w,
        h = h,
        tx = (tick() / 4) % w as usize,
        ty = (tick() / 4) % h as usize,
        opacity = 0.002,
        repeat = true,
        absolute = true,
    );

    // Load dungeon
    let user_id = os::user_id();
    // log!("USER ID {:?}", user_id.clone());

    let dungeon = user_id
        .ok_or_else(|| "Not logged in".to_string())
        .and_then(|user_id| {
            DungeonCrawlerProgram::fetch_player_dungeon(&user_id).map_err(|err| err.to_string())
        });
    // log!("DUNGEON {:?}", dungeon);

    if let Ok(dungeon) = &dungeon {
        state.turn.set(dungeon.turn);

        // Update player tweens
        state.player.x.set(dungeon.player.x * TILE_SIZE);
        state.player.y.set(dungeon.player.y * TILE_SIZE);

        // Player "nudge" animation
        if (!state.player.y.done() || !state.player.x.done()) && state.player.offset_y.done() {
            state.player.offset_y.set(-MOVE_Y_OFFSET);
        }
        if state.player.offset_x.done() && state.player.offset_x.get() != 0 {
            state.player.offset_x.set(0);
        }
        if state.player.offset_y.done() && state.player.offset_y.get() != 0 {
            state.player.offset_y.set(0);
        }

        // Update monster tweens
        if state.floor.get() != dungeon.floor || state.monsters.len() != dungeon.monsters.len() {
            state.floor.set(dungeon.floor);
            state.monsters.clear();
            for monster in &dungeon.monsters {
                state.monsters.push(Entity {
                    x: Tween::new(monster.x * TILE_SIZE)
                        .duration(MOVE_DUR)
                        .ease(Easing::EaseOutSine),
                    y: Tween::new(monster.y * TILE_SIZE)
                        .duration(MOVE_DUR)
                        .ease(Easing::EaseOutSine),
                    offset_x: Tween::new(0)
                        .duration(MOVE_DUR / 2)
                        .ease(Easing::EaseInOutQuad),
                    offset_y: Tween::new(0)
                        .duration(MOVE_DUR / 2)
                        .ease(Easing::EaseInOutQuad),
                })
            }
        }
        if state.player.is_idle() {
            for (monster, entity) in dungeon
                .monsters
                .iter()
                .zip(state.monsters.iter_mut())
                .collect::<Vec<(_, _)>>()
            {
                entity.x.set(monster.x * TILE_SIZE);
                entity.y.set(monster.y * TILE_SIZE);

                // Monster "nudge" animation
                if !state.turn.done() && entity.x.done() && entity.y.done() {
                    match monster.direction {
                        Direction::Up => {
                            if dungeon.is_player(monster.x, monster.y - 1) && monster.stun_dur == 0
                            {
                                entity.offset_y.set(-MOVE_Y_OFFSET);
                            }
                        }
                        Direction::Down => {
                            if dungeon.is_player(monster.x, monster.y + 1) && monster.stun_dur == 0
                            {
                                entity.offset_y.set(MOVE_Y_OFFSET);
                            }
                        }
                        Direction::Left => {
                            if dungeon.is_player(monster.x - 1, monster.y) && monster.stun_dur == 0
                            {
                                entity.offset_x.set(-MOVE_X_OFFSET);
                            }
                        }
                        Direction::Right => {
                            if dungeon.is_player(monster.x + 1, monster.y) && monster.stun_dur == 0
                            {
                                entity.offset_x.set(MOVE_X_OFFSET);
                            }
                        }
                    }
                }
                if (!entity.y.done() || !entity.x.done()) && entity.offset_y.done() {
                    entity.offset_y.set(-MOVE_Y_OFFSET);
                }
                if entity.offset_x.done() && entity.offset_x.get() != 0 {
                    entity.offset_x.set(0);
                }
                if entity.offset_y.done() && entity.offset_y.get() != 0 {
                    entity.offset_y.set(0);
                }
            }
        }

        let did_turn_transition_end = state.turn.done();
        let was_last_exec_on_diff_turn = state.last_exec_turn.map_or(true, |t| t != dungeon.turn);
        let did_exec_timeout = (tick() - state.last_exec_at) >= EXEC_TIMEOUT_DUR;
        let is_ready_to_exec =
            did_turn_transition_end && (was_last_exec_on_diff_turn || did_exec_timeout);
        let is_alive = dungeon.player.health > 0;

        // Handle player input
        let gp = gamepad(0);

        // Hard reset game
        if gp.start.just_pressed() && gp.select.pressed() {
            DungeonCrawlerProgram::create_new_dungeon(CreateNewDungeonCommandInput { reset: true });
            state.last_exec_at = tick();
            state.last_exec_turn = Some(dungeon.turn);
        }
        // Dungeon controls
        else if is_ready_to_exec {
            // Next floor or restart
            if gp.start.just_pressed() {
                DungeonCrawlerProgram::create_new_dungeon(CreateNewDungeonCommandInput {
                    reset: dungeon.player.health == 0,
                });
                state.last_exec_at = tick();
                state.last_exec_turn = Some(dungeon.turn);
            }
            // Move
            else if gp.up.pressed() && is_alive {
                DungeonCrawlerProgram::move_player(MovePlayerCommandInput {
                    direction: Direction::Up,
                });
                state.last_exec_at = tick();
                state.last_exec_turn = Some(dungeon.turn);
                if dungeon.is_position_blocked(dungeon.player.x, dungeon.player.y - 1) {
                    state.player.offset_y.set(-MOVE_Y_OFFSET);
                }
            } else if gp.down.pressed() && is_alive {
                DungeonCrawlerProgram::move_player(MovePlayerCommandInput {
                    direction: Direction::Down,
                });
                state.last_exec_at = tick();
                state.last_exec_turn = Some(dungeon.turn);
                if dungeon.is_position_blocked(dungeon.player.x, dungeon.player.y + 1) {
                    state.player.offset_y.set(MOVE_Y_OFFSET);
                }
            } else if gp.left.pressed() && is_alive {
                DungeonCrawlerProgram::move_player(MovePlayerCommandInput {
                    direction: Direction::Left,
                });
                state.last_exec_at = tick();
                state.last_exec_turn = Some(dungeon.turn);
                if dungeon.is_position_blocked(dungeon.player.x - 1, dungeon.player.y) {
                    state.player.offset_x.set(-MOVE_X_OFFSET);
                }
            } else if gp.right.pressed() && is_alive {
                DungeonCrawlerProgram::move_player(MovePlayerCommandInput {
                    direction: Direction::Right,
                });
                state.last_exec_at = tick();
                state.last_exec_turn = Some(dungeon.turn);
                if dungeon.is_position_blocked(dungeon.player.x + 1, dungeon.player.y) {
                    state.player.offset_x.set(MOVE_X_OFFSET);
                }
            }
        }

        let theme = match dungeon.theme {
            DungeonTheme::Fortress => THEMES[0],
            DungeonTheme::Crypt => THEMES[1],
            DungeonTheme::Pirate => THEMES[2],
        };

        // Size constants
        let [w, h] = canvas_size!();
        let menubar_h = 40;

        // Center camera on player
        set_cam!(
            x = state.player.x.get() + (TILE_SIZE / 2),
            y = state.player.y.get() + (TILE_SIZE / 2) + menubar_h,
        );

        let dungeon_w = dungeon.width * TILE_SIZE as u32;
        let dungeon_h = dungeon.height * TILE_SIZE as u32;

        // Draw dungeon floor and border
        rect!(
            w = dungeon_w,
            h = dungeon_h,
            color = 0x000000ff,
            border_radius = 4,
        );
        sprite!(
            theme.0,
            w = dungeon_w,
            h = dungeon_h,
            repeat = true,
            // opacity = 0.01,
            border_radius = 4,
        );
        rect!(
            w = dungeon_w + 16,
            h = dungeon_h + 16 + 4,
            x = -8,
            y = -4 - 4,
            // color = 0x83758bff,
            color = 0x00000000,
            border_color = 0x83758bff,
            // border_color = 0xbd59deff,
            // border_color = 0x000000ff,
            border_width = 8,
            border_radius = 4,
        );
        rect!(
            w = dungeon_w + 16,
            h = dungeon_h + 16 + 4,
            x = -8,
            y = -8 - 4,
            // color = 0x83758bff,
            color = 0x00000000,
            border_color = 0x524c52ff,
            // border_color = 0x7b34bdff,
            // border_color = 0x000000ff,
            border_width = 8,
            border_radius = 4,
        );

        // Draw exit
        if let Some(exit) = &dungeon.exit {
            sprite!(
                "stairs_down",
                x = exit.0 * TILE_SIZE,
                y = exit.1 * TILE_SIZE
            )
        }

        // Draw obstacles
        for obstacle in &dungeon.obstacles {
            match obstacle.kind {
                ObstacleKind::WallA => {
                    sprite!(
                        theme.1,
                        x = obstacle.x * TILE_SIZE,
                        y = obstacle.y * TILE_SIZE,
                    );
                }
                ObstacleKind::WallB => {
                    sprite!(
                        theme.2,
                        x = obstacle.x * TILE_SIZE,
                        y = obstacle.y * TILE_SIZE,
                        fps = fps::SLOW
                    );
                }
            }
        }

        // Draw player
        if dungeon.player.health > 0 {
            let x = state.player.x.get();
            let y = state.player.y.get();
            sprite!(
                "dotted_tile_border",
                x = x,
                y = y,
                opacity = 0.1,
                fps = fps::FAST,
            );
            let x = x + state.player.offset_x.get();
            // let y = y - 9;
            ellipse!(
                x = x + 2,
                y = y + 3,
                w = TILE_SIZE - 4,
                h = TILE_SIZE - 4,
                color = SHADOW_COLOR,
            );
            let y = y + state.player.offset_y.get() - 4;
            sprite!("hero", x = x, y = y, fps = fps::FAST,);
        } else {
            sprite!(
                "tombstone",
                x = state.player.x.get(),
                y = state.player.y.get() - 5,
            );
        }

        // Draw monsters
        for (monster, entity) in dungeon
            .monsters
            .iter()
            .zip(state.monsters.iter_mut())
            .collect::<Vec<(_, _)>>()
        {
            if monster.health == 0 {
                continue;
            }
            if monster.stun_dur > 0 && tick() % 16 < 8 {
                continue;
            }
            let opacity = if monster.stun_dur > 0 { 0.5 } else { 1.0 };
            let x = entity.x.get() + entity.offset_x.get();
            let y = entity.y.get() + entity.offset_y.get() - 6;
            match monster.kind {
                MonsterKind::BlueBlob => {
                    ellipse!(
                        x = x + 1,
                        y = y + 9,
                        w = TILE_SIZE - 2,
                        h = TILE_SIZE - 6,
                        color = SHADOW_COLOR,
                    );
                    sprite!(
                        "blue_blob",
                        x = x,
                        y = y + 3,
                        fps = fps::FAST,
                        opacity = opacity
                    );
                }
                MonsterKind::YellowBlob => {
                    ellipse!(
                        x = x + 1,
                        y = y + 9,
                        w = TILE_SIZE - 2,
                        h = TILE_SIZE - 6,
                        color = SHADOW_COLOR,
                    );
                    sprite!(
                        "yellow_blob",
                        x = x,
                        y = y + 3,
                        fps = fps::FAST,
                        opacity = opacity
                    );
                }
                MonsterKind::RedBlob => {
                    ellipse!(
                        x = x + 1,
                        y = y + 9,
                        w = TILE_SIZE - 2,
                        h = TILE_SIZE - 6,
                        color = SHADOW_COLOR,
                    );
                    sprite!(
                        "red_blob",
                        x = x,
                        y = y + 3,
                        fps = fps::FAST,
                        opacity = opacity
                    );
                }
                MonsterKind::OrangeGoblin => {
                    ellipse!(
                        x = x + 2,
                        y = y + 8,
                        w = TILE_SIZE - 4,
                        h = TILE_SIZE - 4,
                        color = SHADOW_COLOR,
                    );
                    sprite!(
                        "orange_goblin",
                        x = x,
                        y = y,
                        fps = fps::FAST,
                        opacity = opacity
                    );
                }
                MonsterKind::GreenGoblin => {
                    ellipse!(
                        x = x + 2,
                        y = y + 8,
                        w = TILE_SIZE - 4,
                        h = TILE_SIZE - 4,
                        color = SHADOW_COLOR,
                    );
                    sprite!("goblin", x = x, y = y, fps = fps::FAST, opacity = opacity);
                }
                MonsterKind::Shade => {
                    ellipse!(
                        x = x + 2,
                        y = y + 8,
                        w = TILE_SIZE - 4,
                        h = TILE_SIZE - 4,
                        color = SHADOW_COLOR,
                    );
                    sprite!(
                        "shade",
                        x = x,
                        y = y,
                        fps = fps::FAST,
                        opacity = if dungeon.is_obstacle(monster.x, monster.y) {
                            0.5
                        } else {
                            opacity
                        }
                    );
                }
                MonsterKind::Spider => {
                    ellipse!(
                        x = x + 2,
                        y = y + 8,
                        w = TILE_SIZE - 4,
                        h = TILE_SIZE - 4,
                        color = SHADOW_COLOR,
                    );
                    sprite!(
                        "spider",
                        x = x,
                        y = y + 2,
                        fps = fps::MEDIUM,
                        opacity = opacity
                    );
                }
                _ => {
                    ellipse!(
                        x = x + 2,
                        y = y + 8,
                        w = TILE_SIZE - 4,
                        h = TILE_SIZE - 4,
                        color = SHADOW_COLOR,
                    );
                    sprite!(
                        "generic_monster",
                        x = x,
                        y = y,
                        fps = fps::FAST,
                        opacity = opacity
                    );
                }
            }
        }

        let t = tick() as f32;

        // Draw exit key
        if let Some(exit_key) = &dungeon.exit_key {
            let y = exit_key.1 * TILE_SIZE - 4;
            let y_offset = ((t / 10.).cos() * 2.) - 1.;
            ellipse!(
                x = (exit_key.0 * TILE_SIZE + 5) as f32 + (y_offset / 4.),
                y = (y + 12) as f32 + (y_offset / 4.),
                w = (TILE_SIZE - 9) as f32 - (y_offset / 2.),
                h = (TILE_SIZE - 12) as f32 - (y_offset / 2.),
                color = SHADOW_COLOR,
            );
            sprite!(
                "boss_key",
                x = exit_key.0 * TILE_SIZE,
                y = y as f32 + y_offset - 4.
            )
        }

        // Draw treasures
        for treasure in &dungeon.treasures {
            let y = treasure.y * TILE_SIZE - 5;
            let y_offset = ((t / 10.).cos() * 2.) - 1.;
            ellipse!(
                x = (treasure.x * TILE_SIZE + 5) as f32 + (y_offset / 4.),
                y = (y + 12) as f32 + (y_offset / 4.),
                w = (TILE_SIZE - 9) as f32 - (y_offset / 2.),
                h = (TILE_SIZE - 12) as f32 - (y_offset / 2.),
                color = SHADOW_COLOR,
            );
            match treasure.kind {
                TreasureKind::Gold => {
                    if treasure.value > 1 {
                        sprite!(
                            "purple_gem",
                            x = treasure.x * TILE_SIZE,
                            y = y as f32 + y_offset - 4.,
                        );
                    } else {
                        sprite!("coin", x = treasure.x * TILE_SIZE, y = y as f32 + y_offset,);
                    }
                }
                TreasureKind::Heal => {
                    sprite!(
                        "full_heart",
                        x = treasure.x * TILE_SIZE,
                        y = y as f32 + y_offset,
                    );
                }
            }
        }

        // Draw monsters health bars
        for (monster, entity) in dungeon
            .monsters
            .iter()
            .zip(state.monsters.iter_mut())
            .collect::<Vec<(_, _)>>()
        {
            if monster.health == 0 {
                continue;
            }
            let x = entity.x.get();
            let y = entity.y.get() - 8;

            // Draw health bar background (black)
            rect!(x = x, y = y - 4, w = TILE_SIZE, h = 3, color = 0x000000ff,);

            // Calculate segment width and spacing
            let total_segments = monster.max_health as i32;
            let spacing = 1 as i32;
            let segment_width =
                ((TILE_SIZE - 1) as i32 - spacing * (total_segments - 1)) / total_segments;

            // Draw health bar foreground in segments with spacing
            for i in 0..monster.health {
                rect!(
                    x = 1 + x + (i as i32 * (segment_width + spacing)),
                    y = y - 4,
                    w = segment_width,
                    h = 2,
                    color = 0x00ff00ff,
                );
            }
        }

        // Button menubar
        let menubar_y = (h - menubar_h as u32) as i32;
        let y = menubar_y;

        if dungeon.player.health == 0 && state.turn.done() {
            if let Some(user_id) = &os::user_id() {
                if gp.left.just_pressed() || gp.right.just_pressed() {
                    state.leaderboard_kind = match state.leaderboard_kind {
                        LeaderboardKind::Floor => LeaderboardKind::Gold,
                        LeaderboardKind::Gold => LeaderboardKind::Floor,
                    }
                }
                if let Ok(leaderboard) = DungeonCrawlerProgram::fetch_leaderboard() {
                    rect!(absolute = true, w = w, h = 40, color = 0x000000fa);
                    rect!(
                        absolute = true,
                        w = w,
                        h = h, // - menubar_h as u32,
                        y = 32,
                        // color = 0x293c8bff,
                        color = 0x1a1932ff,
                        border_radius = 8,
                    );

                    let slide_dot_y = h as i32 - (menubar_h + 12);
                    circ!(
                        absolute = true,
                        d = 6,
                        x = (w / 2) - 8,
                        y = slide_dot_y,
                        color = 0xacaabdff,
                        border_width = if state.leaderboard_kind == LeaderboardKind::Floor {
                            0
                        } else {
                            1
                        },
                        border_color = 0,
                    );
                    circ!(
                        absolute = true,
                        d = 6,
                        x = (w / 2),
                        y = slide_dot_y,
                        color = 0xacaabdff,
                        border_width = if state.leaderboard_kind == LeaderboardKind::Gold {
                            0
                        } else {
                            1
                        },
                        border_color = 0,
                    );

                    let leaderboard_x = 0;
                    text!(
                        "LEADERBOARD",
                        absolute = true,
                        x = leaderboard_x + 8,
                        y = 7,
                        font = Font::L
                    );
                    let y = 8;
                    match state.leaderboard_kind {
                        LeaderboardKind::Floor => {
                            let mut i = 2;
                            text!(
                                "Highest Floor",
                                absolute = true,
                                x = leaderboard_x + 8,
                                y = i * 10
                            );
                            i += 1;
                            text!("#  PLAYER {:>7} FLOOR", ""; absolute = true, x = leaderboard_x + 8, y = y + i * 10);
                            i += 1;
                            let mut n = 0;
                            let mut prev_floor = 0;
                            for (id, floor) in leaderboard.highest_floor {
                                if prev_floor != floor {
                                    n += 1;
                                    prev_floor = floor;
                                }
                                text!("{}  {:.8} {:>11} ", n, id, floor + 1; absolute = true, x = leaderboard_x + 8, y = y +  i * 10, color = if id == *user_id {
                                    0x6ecb62ff
                                } else {
                                    0xacaabdff
                                });
                                i += 1;
                                if i - 3 >= 10 {
                                    break;
                                }
                            }
                        }
                        LeaderboardKind::Gold => {
                            let mut i = 2;
                            text!(
                                "Most Gold",
                                absolute = true,
                                x = leaderboard_x + 8,
                                y = i * 10
                            );
                            i += 1;
                            text!("#  PLAYER {:>8} GOLD", ""; absolute = true, x = leaderboard_x + 8, y = y + i * 10);
                            i += 1;
                            let mut n = 0;
                            let mut prev_gold = 0;
                            for (id, gold) in leaderboard.most_gold {
                                if prev_gold != gold {
                                    n += 1;
                                    prev_gold = gold;
                                }
                                text!("{}  {:.8} {:>11} ", n, id, &format!("${}", gold); absolute = true, x = leaderboard_x + 8, y = y + i * 10, color = if id == *user_id {
                                    0x6ecb62ff
                                } else {
                                    0xacaabdff
                                });
                                i += 1;
                                if i - 3 >= 10 {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Menubar background
        rect!(
            absolute = true,
            w = w,
            h = menubar_h,
            y = y,
            color = 0x000000ff
        );
        let y = y + 2;

        // HP
        sprite!("full_heart", absolute = true, y = y);
        let y = y + 4;
        let hp_color = match dungeon.player.health as f32 / dungeon.player.max_health as f32 {
            0.75..=1.0 => 0x71f341ff,
            0.25..=0.75 => 0xffa200ff,
            _ => 0xb41c39ff,
        };
        text!("  {:0>2}/  ", dungeon.player.health; absolute = true, x = 0, y = y, font = Font::L, color = hp_color);
        text!("    /{:0>2}", dungeon.player.max_health; absolute = true, x = 0, y = y, font = Font::L);
        let y = y + 8;

        // Gold
        sprite!("coin", absolute = true, y = y);
        text!("  ${:0>4}", dungeon.player.gold; absolute = true, x = 0, y = y + 5, font = Font::L);

        if dungeon.player.health == 0 {
            let t = tick() as f32;
            let cos_16 = ((t / 16.).cos()) + 1.;
            let action_btn_x = w / 2;
            let action_btn_y = (menubar_y + 3) - (cos_16 as i32);
            let action_btn_w = (w / 2) - 4;
            let action_btn_h = 24;
            rect!(
                absolute = true,
                w = action_btn_w,
                h = action_btn_h,
                x = action_btn_x,
                y = action_btn_y + 1 + (cos_16 as i32),
                color = 0x81090aaa,
                border_radius = 4,
            );
            rect!(
                absolute = true,
                w = action_btn_w,
                h = action_btn_h,
                x = action_btn_x,
                y = action_btn_y,
                color = 0x81090aff,
                border_radius = 4,
                border_width = cos_16,
                border_color = 0xb41c39ff,
            );
            let action_btn_text = "GAME OVER";
            let action_btn_text_len = action_btn_text.len() as u32;
            let action_btn_text_w = action_btn_text_len * 5;
            let action_btn_text_x = 1 + action_btn_x + (action_btn_w / 2) - (action_btn_text_w / 2);
            let action_btn_text_y = action_btn_y + 5;
            text!(
                action_btn_text,
                absolute = true,
                // color = 0x000000ff,
                x = action_btn_text_x,
                y = action_btn_text_y,
                font = Font::M,
            );
            let action_btn_text = "Try again?";
            let action_btn_text_len = action_btn_text.len() as u32;
            let action_btn_text_w = action_btn_text_len * 5;
            let action_btn_text_x = 1 + action_btn_x + (action_btn_w / 2) - (action_btn_text_w / 2);
            let action_btn_text_y = action_btn_y + 13;
            text!(
                action_btn_text,
                absolute = true,
                x = action_btn_text_x,
                y = action_btn_text_y,
                font = Font::M,
            );

            // Handle next floor click / tap
            let m = mouse(0);
            let [mx, my] = m.position;
            let mx = (mx - (cam!().0)) + (w / 2) as i32;
            let my = (my - (cam!().1)) + (h / 2) as i32;
            let hit_x0 = action_btn_x as i32;
            let hit_x1 = (action_btn_x + action_btn_w) as i32;
            let hit_y0 = action_btn_y as i32;
            let hit_y1 = (action_btn_y + action_btn_h) as i32;
            let is_in_btn = mx >= hit_x0 && mx < hit_x1 && my >= hit_y0 && my < hit_y1;
            if m.left.just_pressed() && is_in_btn {
                DungeonCrawlerProgram::create_new_dungeon(CreateNewDungeonCommandInput {
                    reset: true,
                });
            }
        }
        // Next floor button
        else if dungeon.is_exit(dungeon.player.x, dungeon.player.y) {
            let t = tick() as f32;
            let cos_16 = ((t / 16.).cos()) + 1.;
            let action_btn_x = w / 2;
            let action_btn_y = (menubar_y + 3) - (cos_16 as i32);
            let action_btn_w = (w / 2) - 4;
            let action_btn_h = 24;
            rect!(
                absolute = true,
                w = action_btn_w,
                h = action_btn_h,
                x = action_btn_x,
                y = action_btn_y + 1 + (cos_16 as i32),
                color = 0x7b34bdaa,
                border_radius = 4,
            );
            rect!(
                absolute = true,
                w = action_btn_w,
                h = action_btn_h,
                x = action_btn_x,
                y = action_btn_y,
                color = 0x7b34bdff,
                border_radius = 4,
                border_width = cos_16,
                border_color = 0xbd59deff,
            );
            let action_btn_text = "ENTER";
            let action_btn_text_len = action_btn_text.len() as u32;
            let action_btn_text_w = action_btn_text_len * 8;
            let action_btn_text_x = 1 + action_btn_x + (action_btn_w / 2) - (action_btn_text_w / 2);
            let action_btn_text_y = action_btn_y + 5;
            text!(
                action_btn_text,
                absolute = true,
                // color = 0x000000ff,
                x = action_btn_text_x,
                y = action_btn_text_y,
                font = Font::L,
            );
            let action_btn_text = "NEXT FLOOR";
            let action_btn_text_len = action_btn_text.len() as u32;
            let action_btn_text_w = action_btn_text_len * 5;
            let action_btn_text_x = 1 + action_btn_x + (action_btn_w / 2) - (action_btn_text_w / 2);
            let action_btn_text_y = action_btn_y + 13;
            text!(
                action_btn_text,
                absolute = true,
                x = action_btn_text_x,
                y = action_btn_text_y,
                font = Font::M,
            );

            // Handle next floor click / tap
            let m = mouse(0);
            let [mx, my] = m.position;
            let mx = (mx - (cam!().0)) + (w / 2) as i32;
            let my = (my - (cam!().1)) + (h / 2) as i32;
            let hit_x0 = action_btn_x as i32;
            let hit_x1 = (action_btn_x + action_btn_w) as i32;
            let hit_y0 = action_btn_y as i32;
            let hit_y1 = (action_btn_y + action_btn_h) as i32;
            let is_in_btn = mx >= hit_x0 && mx < hit_x1 && my >= hit_y0 && my < hit_y1;
            if m.left.just_pressed() && is_in_btn {
                DungeonCrawlerProgram::create_new_dungeon(CreateNewDungeonCommandInput {
                    reset: false,
                });
            }
        }
        // CTA: Find exit
        else if dungeon.exit.is_some() {
            let cta_x = w / 2;
            let cta_y = menubar_y + 4;
            let cta_w = (w / 2) - 4;
            let cta_text = "~TASK~";
            let cta_text_len = cta_text.len() as u32;
            let cta_text_w = cta_text_len * 8;
            let cta_text_x = 1 + cta_x + (cta_w / 2) - (cta_text_w / 2);
            let cta_text_y = cta_y + 5;
            text!(
                cta_text,
                absolute = true,
                color = 0x524c52ff,
                x = cta_text_x,
                y = cta_text_y,
                font = Font::L,
            );
            let cta_text = "Find exit";
            let cta_text_len = cta_text.len() as u32;
            let cta_text_w = cta_text_len * 5;
            let cta_text_x = 1 + cta_x + (cta_w / 2) - (cta_text_w / 2);
            let cta_text_y = cta_y + 13;
            text!(
                cta_text,
                absolute = true,
                color = 0x524c52ff,
                x = cta_text_x,
                y = cta_text_y,
                font = Font::M,
            );
        }
        // CTA: Get the key
        else {
            let cta_x = w / 2;
            let cta_y = menubar_y + 4;
            let cta_w = (w / 2) - 4;
            let cta_text = "~TASK~";
            let cta_text_len = cta_text.len() as u32;
            let cta_text_w = cta_text_len * 8;
            let cta_text_x = 1 + cta_x + (cta_w / 2) - (cta_text_w / 2);
            let cta_text_y = cta_y + 5;
            text!(
                cta_text,
                absolute = true,
                color = 0x524c52ff,
                x = cta_text_x,
                y = cta_text_y,
                font = Font::L,
            );
            let cta_text = "Get the key";
            let cta_text_len = cta_text.len() as u32;
            let cta_text_w = cta_text_len * 5;
            let cta_text_x = 1 + cta_x + (cta_w / 2) - (cta_text_w / 2);
            let cta_text_y = cta_y + 13;
            text!(
                cta_text,
                absolute = true,
                color = 0x524c52ff,
                x = cta_text_x,
                y = cta_text_y,
                font = Font::M,
            );
        }

        // Bottom info bar
        let info_bar_y = menubar_y + 32;
        if let Some(user_id) = &os::user_id() {
            rect!(
                absolute = true,
                w = w,
                h = 8,
                y = info_bar_y,
                color = 0x293c8bff,
            );
            rect!(
                absolute = true,
                w = w / 2,
                h = 8,
                y = info_bar_y,
                color = 0x524c52ff,
            );
            let id_text = format!("ID:{:.8}", user_id);
            text!(
                &id_text,
                absolute = true,
                x = 4,
                y = info_bar_y + 2,
                font = Font::S,
                color = 0xacaabdff
            );
            let floor_text = format!("FLOOR:{:0>2}", dungeon.floor + 1);
            let floor_text_len = floor_text.len() as u32;
            let floor_text_w = floor_text_len * 5;
            text!(
                &floor_text,
                absolute = true,
                x = w - floor_text_w - 4,
                y = info_bar_y + 2,
                font = Font::S,
                color = 0x4181c5ff
            );
        }

        // Swipe transition
        let p = state.floor.elapsed as f64 / FLOOR_DUR as f64;
        {
            let xo = p * w as f64;
            rect!(absolute = true, x = xo, w = w, h = h, color = 0x000000ff);
            rect!(absolute = true, x = -xo, w = w, h = h, color = 0x000000ff);
        }
    }

    // If no existing dungeon, allow player to create one
    if let Err(err) = &dungeon {
        // Handle user input
        let gp = gamepad(0);
        if gp.start.just_pressed() {
            DungeonCrawlerProgram::create_new_dungeon(CreateNewDungeonCommandInput { reset: true });
        }

        // Reset camera position
        reset_cam!();

        // Current tick and timers
        let t = tick() as f32;
        let cos_32 = ((t / 32.).cos()) * 2. + 1.;
        let cos_24 = (t / 24.).cos();
        let cos_16 = (t / 16.).cos();
        let cos_10 = (t / 10.).cos();
        let cos_08 = (t / 08.).cos();

        // Calculate y offset and base y position
        let v_offset = if h < 256 { h } else { 256 };
        let y = (h - v_offset) as f32;

        // Draw background sky and clouds
        sprite!("night_sky", y = y, w = w, sw = w, tx = t, repeat = true);
        if t % 2. == 0. {
            sprite!(
                "clouds_3",
                y = y + (cos_16 * 2.) + 1.,
                w = w,
                sw = w,
                tx = t / 2.,
                repeat = true,
                opacity = 0.5
            );
        }
        sprite!(
            "clouds_0",
            y = y + (cos_10 * 2.) + 1.,
            w = w,
            sw = w,
            tx = t / 8.,
            repeat = true
        );

        // Draw background castle
        let castle_scale = 0.5;
        let castle_h = 256. * castle_scale;
        let castle_w = 256. * castle_scale;
        let castle_x = (w as f32 / 2.) - (castle_w / 2.);
        let castle_y = h as f32 - castle_h - cos_32;
        sprite!("title_b", scale = castle_scale, x = castle_x, y = castle_y);

        // Draw foreground clouds
        sprite!(
            "clouds_1",
            y = y + (cos_24 * 2.) + 1.,
            w = w,
            sw = w,
            tx = t / 4.,
            repeat = true
        );
        sprite!(
            "clouds_2",
            y = y + (cos_08 * 2.) + 1.,
            w = w,
            sw = w,
            tx = t / 2.,
            repeat = true
        );

        // Draw title text
        let title_scale = 0.75;
        let title_h = 93. * title_scale;
        let title_w = 146. * title_scale;
        let title_x = (w as f32 / 2.) - (title_w / 2.);
        let title_y = h as f32 - (title_h * 3.);
        sprite!(
            "title_text",
            scale = title_scale,
            y = title_y + 2.,
            x = title_x,
            color = 0x000000ff,
            opacity = 0.75
        );
        sprite!("title_text", scale = title_scale, y = title_y, x = title_x,);

        if os::user_id().is_some() {
            if mouse(0).left.just_pressed() {
                DungeonCrawlerProgram::create_new_dungeon(CreateNewDungeonCommandInput {
                    reset: true,
                });
            }
            rect!(
                absolute = true,
                y = h - 32,
                w = w,
                h = 32,
                color = 0x222034ff
            );
            if t / 2. % 32. < 16. {
                let text = "TAP TO START";
                let text_len = text.len() as u32;
                let text_w = text_len * 8;
                text!(
                    text,
                    x = (w / 2) - (text_w / 2),
                    y = h - 20,
                    color = 0xffffffff,
                    font = Font::L
                );
            }
        }

        // text!("PRESS START {:?}", os::user_id(););
        // let msg = format!("{}", err).replace("ParsingError(", "").replace(")", "");
        // text!("{}", msg; y = 0, font = Font::S);
        // text!("PRESS START");
    }

    state.save();
});

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
struct Player {
    x: i32,
    y: i32,
    health: u32,
    max_health: u32,
    strength: u32,
    gold: u32,
    direction: Direction,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy)]
enum MonsterKind {
    GreenGoblin,
    OrangeGoblin,
    YellowBlob,
    BlueBlob,
    RedBlob,
    Shade,
    Spider,
}
impl MonsterKind {
    pub fn abbrev<'a>(&self) -> &'a str {
        match self {
            Self::BlueBlob => "B. Blob",
            Self::RedBlob => "R. Blob",
            Self::YellowBlob => "Y. Blob",
            Self::GreenGoblin => "G. Goblin",
            Self::OrangeGoblin => "O. Goblin",
            Self::Shade => "Shade",
            Self::Spider => "Spider",
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
struct Monster {
    x: i32,
    y: i32,
    health: u32,
    max_health: u32,
    strength: u32,
    direction: Direction,
    kind: MonsterKind,
    stun_dur: u32,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
enum TreasureKind {
    Gold,
    Heal,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
struct Treasure {
    x: i32,
    y: i32,
    value: u32,
    kind: TreasureKind,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy)]
enum ObstacleKind {
    WallA,
    WallB,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
struct Obstacle {
    x: i32,
    y: i32,
    kind: ObstacleKind,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy)]
enum DungeonTheme {
    Fortress,
    Crypt,
    Pirate,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
struct Dungeon {
    theme: DungeonTheme,
    floor: u32,
    turn: u32,
    width: u32,
    height: u32,
    player: Player,
    monsters: Vec<Monster>,
    treasures: Vec<Treasure>,
    obstacles: Vec<Obstacle>,
    logs: Vec<String>,
    exit_key: Option<(i32, i32)>,
    exit: Option<(i32, i32)>,
}
impl Dungeon {
    fn move_player(&mut self, direction: Direction, log: fn(&str)) -> bool {
        if self.player.health == 0 {
            log("P1 is dead.");
            return false;
        }

        let Player { x, y, .. } = self.player;
        let (new_x, new_y) = match direction {
            Direction::Up => (x, y - 1),
            Direction::Down => (x, y + 1),
            Direction::Left => (x - 1, y),
            Direction::Right => (x + 1, y),
        };

        if self.is_out_of_bounds(new_x, new_y) {
            log("P1 cannot move out-of-bounds");
            return false;
        }

        if self.is_obstacle(new_x, new_y) {
            log("P1 cannot move through obstacle");
            return false;
        }

        if self.is_monster(new_x, new_y) {
            if let Some(monster) = self
                .monsters
                .iter_mut()
                .find(|m| m.x == new_x && m.y == new_y && m.health > 0)
            {
                // Swap positions with the stunned monsters
                if monster.stun_dur > 0 {
                    std::mem::swap(&mut self.player.x, &mut monster.x);
                    std::mem::swap(&mut self.player.y, &mut monster.y);
                    return true;
                }

                let monster_name = monster.kind.abbrev();
                let msg = format!("P1 attacks {}!", monster_name);
                log(&msg);
                self.logs.push(msg);
                let amount = self.player.strength;
                let msg = format!("P1 did {amount} damage.");
                log(&msg);
                self.logs.push(msg);
                monster.stun_dur = 2; //program::random_number::<u32>() % 3; // 0 - 2
                monster.health = monster.health.saturating_sub(amount);
                if monster.health <= 0 {
                    let msg = format!("{} defeated!", monster_name);
                    log(&msg);
                    self.logs.push(msg);
                }
            }
            return true; // Player doesn't move into the monster's position
        }

        // Player moved
        let msg = format!("P1 moved {direction:?}.");
        log(&msg);
        // self.logs.push(msg);
        self.player.x = new_x;
        self.player.y = new_y;
        self.player.direction = direction;

        // Player collected treasure
        if self.is_treasure(new_x, new_y) {
            if let Some(treasure) = self.treasures.iter().find(|m| m.x == new_x && m.y == new_y) {
                let amount = treasure.value;
                match treasure.kind {
                    TreasureKind::Gold => {
                        self.player.gold += amount;
                        let msg = format!("Got treasure! +${amount}");
                        log(&msg);
                        self.logs.push(msg);
                    }
                    TreasureKind::Heal => {
                        let prev_player_health = self.player.health;
                        self.player.health =
                            (self.player.health + amount).min(self.player.max_health);
                        let recovered_health = prev_player_health.abs_diff(self.player.health);
                        let msg = format!("Recovered {} HP!", recovered_health);
                        log(&msg);
                        self.logs.push(msg);
                    }
                }
            }
            self.treasures.retain_mut(|t| t.x != new_x || t.y != new_y);
        }

        if self.is_exit_key(new_x, new_y) {
            let msg = "Found exit key.".to_string();
            log(&msg);
            self.logs.push(msg);
            self.exit_key = None;
            let (max_x, max_y) = self.bounds();
            // Initialize exit position at least 8 tiles away from player
            let min_distance = (self.width.min(self.height) / 2) as i32;
            loop {
                let x = program::random_number::<i32>().abs() % max_x;
                let y = program::random_number::<i32>().abs() % max_y;
                let dx = (x - new_x).abs();
                let dy = (y - new_y).abs();
                if dx + dy >= min_distance && !self.is_position_occupied(x, y) {
                    self.exit = Some((x, y));
                    break;
                }
            }
            let msg = "Hidden stairs appeared!".to_string();
            log(&msg);
            self.logs.push(msg);
        }

        return true;
    }
    fn move_monsters(&mut self, log: fn(&str)) {
        let mut player = self.player.clone();
        let mut monsters = self.monsters.clone();
        let mut n = 0;
        monsters.retain_mut(|monster| {
            n += 1;
            let i = n - 1;

            // Skip dead monsters
            if monster.health == 0 {
                return true;
            }

            // Skip stunned monsters
            if monster.stun_dur > 0 {
                monster.stun_dur = monster.stun_dur.saturating_sub(1);
                return true;
            }

            // If the monster is adjacent to the player, it attacks
            let (mx, my) = (monster.x, monster.y);
            if (mx - player.x).abs() + (my - player.y).abs() == 1 {
                let monster_name = monster.kind.abbrev();
                let msg = format!("{} attacks!", monster_name);
                log(&msg);
                self.logs.push(msg);
                if self.is_player(mx, my - 1) {
                    monster.direction = Direction::Up;
                }
                if self.is_player(mx, my + 1) {
                    monster.direction = Direction::Down;
                }
                if self.is_player(mx - 1, my) {
                    monster.direction = Direction::Left;
                }
                if self.is_player(mx + 1, my) {
                    monster.direction = Direction::Right;
                }
                let prev_player_health = player.health;
                player.health = player.health.saturating_sub(monster.strength);
                let damage = prev_player_health.abs_diff(player.health);
                let msg = format!("{} did {} damage.", monster_name, damage);
                log(&msg);
                self.logs.push(msg);
                if player.health == 0 {
                    let msg = "P1 died.".to_string();
                    log(&msg);
                    self.logs.push(msg);
                }
                return true;
            }

            // Movement based on monster kind
            let (dir, mx, my) = match monster.kind {
                MonsterKind::BlueBlob | MonsterKind::YellowBlob | MonsterKind::RedBlob => {
                    let dx = player.x - mx;
                    let dy = player.y - my;

                    // When player is 2 or fewer spaces away, chase them
                    if dx.abs() <= 2 && dy.abs() <= 2 {
                        let (dir, mx, my) = match (dx.abs() > dy.abs(), dx > 0, dy > 0) {
                            (false, _, false) => (Direction::Up, mx, my - 1),
                            (false, _, true) => (Direction::Down, mx, my + 1),
                            (true, false, _) => (Direction::Left, mx - 1, my),
                            (true, true, _) => (Direction::Right, mx + 1, my),
                        };
                        if self.is_position_occupied(mx, my) {
                            return true;
                        }
                        (dir, mx, my)
                    }
                    // Otherwise, move in a random direction
                    else {
                        let (dir, mx, my) = match program::random_number::<usize>() % 4 {
                            0 => (Direction::Up, mx, my - 1),
                            1 => (Direction::Down, mx, my + 1),
                            2 => (Direction::Left, mx - 1, my),
                            _ => (Direction::Right, mx + 1, my),
                        };
                        if self.is_position_occupied(mx, my) {
                            return true;
                        }
                        (dir, mx, my)
                    }
                }
                MonsterKind::Spider => {
                    // Moves up to 3 spaces in one direction towards the player every 4 turns
                    if self.turn % 3 != 0 {
                        return true;
                    }

                    let dx = player.x - mx;
                    let dy = player.y - my;

                    // Attempt to move up to 3 spaces towards player
                    let steps = 3.min(dx.abs().max(dy.abs()));

                    let mut new_mx = mx;
                    let mut new_my = my;
                    let mut dir = monster.direction;

                    for s in (1..=steps).rev() {
                        let (dir_next, mx_next, my_next) =
                            match (dx.abs() > dy.abs(), dx > 0, dy > 0) {
                                (false, _, false) => (Direction::Up, mx, my - s),
                                (false, _, true) => (Direction::Down, mx, my + s),
                                (true, false, _) => (Direction::Left, mx - s, my),
                                (true, true, _) => (Direction::Right, mx + s, my),
                            };

                        if !self.is_position_occupied(mx_next, my_next) {
                            new_mx = mx_next;
                            new_my = my_next;
                            dir = dir_next;
                            break;
                        }
                    }

                    (dir, new_mx, new_my)
                }
                MonsterKind::Shade => {
                    // Moves towards the player every other turn
                    // Can phase through obstacles
                    if self.turn % 2 != 0 {
                        return true;
                    }

                    let dx = player.x - mx;
                    let dy = player.y - my;

                    let steps = 1;

                    let (dir, mx, my) = match (dx.abs() > dy.abs(), dx > 0, dy > 0) {
                        (false, _, false) => (Direction::Up, mx, my - steps),
                        (false, _, true) => (Direction::Down, mx, my + steps),
                        (true, false, _) => (Direction::Left, mx - steps, my),
                        (true, true, _) => (Direction::Right, mx + steps, my),
                    };

                    if self.is_monster(mx, my) {
                        return true;
                    }

                    (dir, mx, my)
                }
                _ => {
                    // Moves towards the player each turn
                    let dx = player.x - mx;
                    let dy = player.y - my;

                    let move_y = || {
                        if dy < 0 {
                            (Direction::Up, mx, my - 1)
                        } else {
                            (Direction::Down, mx, my + 1)
                        }
                    };
                    let move_x = || {
                        if dx < 0 {
                            (Direction::Left, mx - 1, my)
                        } else {
                            (Direction::Right, mx + 1, my)
                        }
                    };
                    let all = if dx.abs() > dy.abs() {
                        [move_x(), move_y()]
                    } else {
                        [move_y(), move_x()]
                    };
                    if let Some(a) = all.iter().find(|a| !self.is_position_occupied(a.1, a.2)) {
                        *a
                    } else {
                        return true;
                    }
                }
            };

            if self.is_out_of_bounds(mx, my) {
                return true;
            }

            self.monsters[i].x = mx;
            self.monsters[i].y = my;
            self.monsters[i].direction = dir;
            monster.x = mx;
            monster.y = my;
            monster.direction = dir;

            return true;
        });
        self.player = player;
        self.monsters = monsters;
    }
    fn is_player(&self, x: i32, y: i32) -> bool {
        self.player.x == x && self.player.y == y
    }
    fn bounds(&self) -> (i32, i32) {
        let max_x = self.width as i32 - 1;
        let max_y = self.height as i32 - 1;
        (max_x, max_y)
    }
    fn is_out_of_bounds(&self, x: i32, y: i32) -> bool {
        let (min_x, min_y) = (0, 0);
        let (max_x, max_y) = self.bounds();
        x < min_x || y < min_y || x > max_x || y > max_y
    }
    fn is_obstacle(&self, x: i32, y: i32) -> bool {
        self.obstacles.iter().any(|obs| obs.x == x && obs.y == y)
    }
    fn is_monster(&self, x: i32, y: i32) -> bool {
        self.monsters
            .iter()
            .any(|mon| mon.x == x && mon.y == y && mon.health > 0)
    }
    fn is_treasure(&self, x: i32, y: i32) -> bool {
        self.treasures.iter().any(|t| t.x == x && t.y == y)
    }
    fn is_exit_key(&self, x: i32, y: i32) -> bool {
        self.exit_key.map_or(false, |a| a.0 == x && a.1 == y)
    }
    fn is_exit(&self, x: i32, y: i32) -> bool {
        self.exit.map_or(false, |a| a.0 == x && a.1 == y)
    }
    fn is_position_blocked(&self, x: i32, y: i32) -> bool {
        self.is_obstacle(x, y) || self.is_monster(x, y) || self.is_player(x, y)
    }
    fn is_position_occupied(&self, x: i32, y: i32) -> bool {
        self.is_position_blocked(x, y)
            || self.is_treasure(x, y)
            || self.is_exit_key(x, y)
            || self.is_exit(x, y)
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
struct Leaderboard {
    highest_floor: Vec<(String, u32)>,
    most_gold: Vec<(String, u32)>,
}
impl Leaderboard {
    fn new() -> Self {
        Self {
            highest_floor: vec![],
            most_gold: vec![],
        }
    }
    fn update(&mut self, user_id: String, floor: u32, gold: u32) {
        self.highest_floor.push((user_id.clone(), floor));
        self.most_gold.push((user_id, gold));

        self.highest_floor.sort_by(|a, b| b.1.cmp(&a.1));
        self.most_gold.sort_by(|a, b| b.1.cmp(&a.1));

        if self.highest_floor.len() > 10 {
            self.highest_floor.truncate(10);
        }
        if self.most_gold.len() > 10 {
            self.most_gold.truncate(10);
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
struct CreateNewDungeonCommandInput {
    reset: bool,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
struct MovePlayerCommandInput {
    direction: Direction,
}

struct DungeonCrawlerProgram;
impl DungeonCrawlerProgram {
    pub const PROGRAM_ID: &'static str = "dungeon_crawler";
    pub const VERSION: usize = 1;
    pub fn fetch_player_dungeon(user_id: &str) -> Result<Dungeon, os::ReadFileError> {
        // return Err(ReadFileError::ParsingError("lol nerd".to_string()));
        let filepath = Self::get_dungeon_filepath(&user_id);
        let file = os::read_file(Self::PROGRAM_ID, &filepath)?;
        Dungeon::try_from_slice(&file.contents)
            .map_err(|err| ReadFileError::ParsingError(err.to_string()))
    }
    pub fn fetch_leaderboard() -> Result<Leaderboard, os::ReadFileError> {
        let filepath = "leaderboard";
        let file = os::read_file(Self::PROGRAM_ID, &filepath)?;
        Leaderboard::try_from_slice(&file.contents)
            .map_err(|err| ReadFileError::ParsingError(err.to_string()))
    }
    pub fn create_new_dungeon(input: CreateNewDungeonCommandInput) -> String {
        os::exec(
            Self::PROGRAM_ID,
            "create_new_dungeon",
            &input.try_to_vec().unwrap(),
        )
    }
    pub fn move_player(input: MovePlayerCommandInput) -> String {
        os::exec(
            Self::PROGRAM_ID,
            "move_player",
            &input.try_to_vec().unwrap(),
        )
    }
    fn get_dungeon_filepath(user_id: &str) -> String {
        format!("v{}/{}.dungeon", Self::VERSION, user_id)
    }
    fn load_player_dungeon(user_id: &str) -> Result<Dungeon, &'static str> {
        let filepath = Self::get_dungeon_filepath(user_id);
        let data = program::read_file(&filepath)?;
        match Dungeon::try_from_slice(&data) {
            Ok(dungeon) => Ok(dungeon),
            Err(err) => {
                program::log(&err.to_string());
                Err("Failed to deserialize dungeon")
            }
        }
    }
    fn save_player_dungeon(user_id: &str, dungeon: &Dungeon) {
        let filepath = Self::get_dungeon_filepath(user_id);
        let data = dungeon.try_to_vec().unwrap();
        program::write_file(&filepath, &data).unwrap()
    }

    #[export_name = "turbo/create_new_dungeon"]
    unsafe extern "C" fn on_create_new_dungeon() -> usize {
        // Parse command input
        program::log("Parsing command input...");
        let input_bytes = program::get_input_data();
        let input = match CreateNewDungeonCommandInput::try_from_slice(&input_bytes) {
            Ok(input) => input,
            Err(err) => {
                program::log(&err.to_string());
                return program::CANCEL;
            }
        };

        // Create a default dungeon
        let mut dungeon = if input.reset {
            let w = 6;
            let h = 6;
            Dungeon {
                theme: DungeonTheme::Fortress,
                floor: 0,
                turn: 0,
                width: w,
                height: h,
                player: Player {
                    x: program::random_number::<i32>().abs() % w as i32,
                    y: program::random_number::<i32>().abs() % h as i32,
                    health: 10,
                    max_health: 10,
                    strength: 1,
                    gold: 0,
                    direction: Direction::Down,
                },
                monsters: vec![],
                treasures: vec![],
                obstacles: vec![],
                logs: vec![],
                exit: None,
                exit_key: None,
            }
        } else {
            // Load player dungeon
            program::log("Loading the dungeon...");
            let user_id = program::get_user_id();
            let mut dungeon = match Self::load_player_dungeon(&user_id) {
                Ok(dungeon) => dungeon,
                Err(err) => {
                    program::log(err);
                    return program::CANCEL;
                }
            };

            if !dungeon.is_exit(dungeon.player.x, dungeon.player.y) {
                program::log("P1 has not reached the exit.");
                return program::CANCEL;
            }

            // Remove exit
            dungeon.exit = None;
            // Clear monsters, treasures, and obstacles
            dungeon.monsters.clear();
            dungeon.treasures.clear();
            dungeon.obstacles.clear();
            // Clear logs
            dungeon.logs.clear();
            // Recover one heart
            dungeon.player.health = (dungeon.player.health + 1).min(dungeon.player.max_health);
            // Increase floor
            dungeon.floor += 1;
            // Update dungeon theme
            if dungeon.floor == 10 {
                dungeon.theme = DungeonTheme::Crypt;
            }
            // Embiggen every 3 floors
            if dungeon.floor % 3 == 0 {
                dungeon.width += 2;
                dungeon.height += 2;
            }
            // Reset turn
            dungeon.turn = 0;

            dungeon
        };

        // Get the dungeon bounds
        let (max_x, max_y) = dungeon.bounds();

        let magic_ratio = ((max_x * max_y) / 32) as usize;

        // Randomize monsters
        program::log("Randomizing monsters...");
        let num_monsters = 2 + magic_ratio;
        // Define monsters and their weights
        let monster_weights: &[(u32, MonsterKind)] = match dungeon.theme {
            DungeonTheme::Crypt => &[
                (3, MonsterKind::GreenGoblin),
                (1, MonsterKind::OrangeGoblin),
                (1, MonsterKind::Spider),
                (1, MonsterKind::Shade),
                (1, MonsterKind::RedBlob),
            ],
            _ => &[
                (3, MonsterKind::BlueBlob),
                (1, MonsterKind::YellowBlob),
                (1, MonsterKind::RedBlob),
                (1, MonsterKind::GreenGoblin),
            ],
        };
        let total_weight: u32 = monster_weights.iter().map(|(weight, _)| *weight).sum();

        while dungeon.monsters.len() < num_monsters {
            let x = program::random_number::<i32>().abs() % max_x;
            let y = program::random_number::<i32>().abs() % max_y;
            if !dungeon.is_position_occupied(x, y) {
                // Generate a random number within the total weight
                let mut rng = program::random_number::<u32>() % total_weight;
                let mut selected_monster = MonsterKind::GreenGoblin;
                // Select the monster based on weighted probability
                for (weight, monster_kind) in monster_weights {
                    if rng < *weight {
                        selected_monster = *monster_kind;
                        break;
                    }
                    rng -= *weight;
                }
                // Define monster stats based on the selected kind
                let (health, strength) = match selected_monster {
                    MonsterKind::OrangeGoblin => (5, 1),
                    MonsterKind::Spider => (4, 2),
                    MonsterKind::Shade => (3, 2),
                    MonsterKind::GreenGoblin => (2, 1),
                    MonsterKind::RedBlob => (3, 2),
                    MonsterKind::YellowBlob => (2, 1),
                    _ => (1, 1),
                };
                dungeon.monsters.push(Monster {
                    x,
                    y,
                    health,
                    max_health: health,
                    strength,
                    direction: Direction::Down,
                    kind: selected_monster,
                    stun_dur: 0,
                });
            }
        }

        // Randomize treasures
        program::log("Randomizing treasures...");
        let num_treasures = magic_ratio + (dungeon.floor as usize / 2);
        while dungeon.treasures.len() < num_treasures {
            let x = program::random_number::<i32>().abs() % max_x;
            let y = program::random_number::<i32>().abs() % max_y;
            if !dungeon.is_position_occupied(x, y) {
                // Last treasure is a healing item
                if dungeon.treasures.len() == num_treasures - 1 {
                    dungeon.treasures.push(Treasure {
                        x,
                        y,
                        value: 2,
                        kind: TreasureKind::Heal,
                    })
                }
                // Every other treasure gives the player gold
                else {
                    let n = program::random_number::<u32>() % 10;
                    dungeon.treasures.push(if n < 9 {
                        // 90% chance for $1 gold treasure
                        Treasure {
                            x,
                            y,
                            value: 1,
                            kind: TreasureKind::Gold,
                        }
                    } else {
                        // 10% chance for $10 gold treasure
                        Treasure {
                            x,
                            y,
                            value: 10,
                            kind: TreasureKind::Gold,
                        }
                    });
                }
            }
        }

        // Initialize exit_key position at least 8 tiles away from player
        program::log("Initializing exit key position...");
        let min_distance = (dungeon.width.min(dungeon.height) / 2) as i32;
        loop {
            let x = program::random_number::<i32>().abs() % max_x;
            let y = program::random_number::<i32>().abs() % max_y;
            let dx = (x - dungeon.player.x).abs();
            let dy = (y - dungeon.player.y).abs();
            if dx + dy >= min_distance && !dungeon.is_position_occupied(x, y) {
                dungeon.exit_key = Some((x, y));
                break;
            }
        }

        // Randomize obstacles
        program::log("Randomizing obstacles...");
        for (x, y) in generate_maze(max_x as usize, max_y as usize) {
            // 1/3 chance to skip a obstacle placement
            if program::random_number::<u8>() % 3 == 0 {
                continue;
            }
            // Make sure spot is empty
            if dungeon.is_position_occupied(x, y) {
                continue;
            }
            dungeon.obstacles.push(Obstacle {
                x,
                y,
                kind: if program::random_number::<usize>() % 10 == 9 {
                    // 10% chance for firepit
                    ObstacleKind::WallB
                } else {
                    // 90% chance for stone block
                    ObstacleKind::WallA
                },
            });
        }

        // Save the dungeon
        program::log("Saving dungeon...");
        let user_id = program::get_user_id();
        Self::save_player_dungeon(&user_id, &dungeon);

        // Commit the command result
        return program::COMMIT;
    }

    #[export_name = "turbo/move_player"]
    unsafe extern "C" fn on_move_player() -> usize {
        // Parse command input
        program::log("Parsing command input...");
        let input_bytes = program::get_input_data();
        let input = match MovePlayerCommandInput::try_from_slice(&input_bytes) {
            Ok(input) => input,
            Err(err) => {
                program::log(&err.to_string());
                return program::CANCEL;
            }
        };

        // Load player dungeon
        program::log("Loading the dungeon...");
        let user_id = program::get_user_id();
        let mut dungeon = match Self::load_player_dungeon(&user_id) {
            Ok(dungeon) => dungeon,
            Err(err) => {
                program::log(err);
                return program::CANCEL;
            }
        };

        // Cancel command if player has already won or lost
        program::log("Checking game over conditions...");
        if dungeon.player.health == 0 {
            program::log("P1 has died. Game over.");
            return program::CANCEL;
        }

        // Move player
        program::log("Moving player...");
        if !dungeon.move_player(input.direction, program::log) {
            return program::CANCEL;
        }

        // Move monsters if player has not reached the exit
        if !dungeon.is_exit(dungeon.player.x, dungeon.player.y) {
            program::log("Moving monsters...");
            dungeon.move_monsters(program::log);
        } else {
            let msg = "P1 reached exit.".to_string();
            program::log(&msg);
            dungeon.logs.push(msg);
        }

        // Truncate dungeon logs
        program::log("Truncating logs...");
        let num_logs = dungeon.logs.len();
        let log_limit = 23;
        if num_logs > log_limit {
            dungeon.logs = dungeon.logs.split_off(num_logs - log_limit);
        }

        // Increment turn
        program::log("Incrementing turn number...");
        dungeon.turn += 1;

        // Save the dungeon
        program::log("Saving the dungeon...");
        Self::save_player_dungeon(&user_id, &dungeon);

        // If player died, try adding their leaderboard entry
        if dungeon.player.health == 0 {
            let filepath = "leaderboard";
            program::log("About to read the leaderboard file.");
            let mut leaderboard = match program::read_file(&filepath) {
                Ok(bytes) => {
                    program::log("Read the leaderboard file.");
                    Leaderboard::try_from_slice(&bytes).unwrap()
                }
                Err(err) => {
                    program::log("Could not deserialize leaderboard.");
                    program::log(err);
                    Leaderboard::new()
                }
            };
            program::log("Updating leaderboard...");
            leaderboard.update(user_id, dungeon.floor, dungeon.player.gold);
            program::log("Updated leaderboard!");
            let data = leaderboard.try_to_vec().unwrap();
            program::log("Writing leaderboard file...");
            program::write_file(&filepath, &data).unwrap();
        }

        // Commit the command result
        return program::COMMIT;
    }
}

fn generate_maze(width: usize, height: usize) -> Vec<(i32, i32)> {
    let mut grid = vec![vec![false; width]; height];
    let mut walls = vec![];

    fn divide(
        grid: &mut Vec<Vec<bool>>,
        walls: &mut Vec<(i32, i32)>,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) {
        if width <= 2 || height <= 2 {
            return;
        }

        let horizontal = program::random_number::<u8>() % 2 == 0;

        if horizontal {
            let wall_y = y + 1 + (program::random_number::<usize>() % ((height / 2).max(1))) * 2;
            for i in x..x + width {
                grid[wall_y][i] = true;
                walls.push((i as i32, wall_y as i32));
            }
            let passage_x = x + program::random_number::<usize>() % width;
            grid[wall_y][passage_x] = false;
            walls.retain(|&(wx, wy)| !(wx == passage_x as i32 && wy == wall_y as i32));

            // Ensure at least one passage in the adjacent walls
            if !grid[wall_y - 1][passage_x] && !grid[wall_y + 1][passage_x] {
                grid[wall_y][passage_x] = false;
                walls.retain(|&(wx, wy)| !(wx == passage_x as i32 && wy == wall_y as i32));
            }

            divide(grid, walls, x, y, width, wall_y - y);
            divide(grid, walls, x, wall_y + 1, width, y + height - wall_y - 1);
        } else {
            let wall_x = x + 1 + (program::random_number::<usize>() % ((width / 2).max(1))) * 2;
            for i in y..y + height {
                grid[i][wall_x] = true;
                walls.push((wall_x as i32, i as i32));
            }
            let passage_y = y + program::random_number::<usize>() % height;
            grid[passage_y][wall_x] = false;
            walls.retain(|&(wx, wy)| !(wx == wall_x as i32 && wy == passage_y as i32));

            // Ensure at least one passage in the adjacent walls
            if !grid[passage_y][wall_x - 1] && !grid[passage_y][wall_x + 1] {
                grid[passage_y][wall_x] = false;
                walls.retain(|&(wx, wy)| !(wx == wall_x as i32 && wy == passage_y as i32));
            }

            divide(grid, walls, x, y, wall_x - x, height);
            divide(grid, walls, wall_x + 1, y, x + width - wall_x - 1, height);
        }
    }

    divide(&mut grid, &mut walls, 0, 0, width, height);
    walls
}
