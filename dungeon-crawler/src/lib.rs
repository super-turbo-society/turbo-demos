use borsh::{BorshDeserialize, BorshSerialize};
use os::ReadFileError;

turbo::cfg! {r#"
    name = "Dungeon Crawler"
    [settings]
    resolution = [384, 256]
    [turbo-os]
    api-url = "https://os.turbo.computer"
    # api-url = "http://localhost:8000"
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
const TURN_DUR: usize = 16;
const FLOOR_DUR: usize = 32;
const MOVE_DUR: usize = 8;
const MOVE_Y_OFFSET: i32 = 4;
const MOVE_X_OFFSET: i32 = 4;
const EXEC_TIMEOUT_DUR: usize = 32;
const SHADOW_COLOR: u32 = 0x000000dd;

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
        opacity = 0.001,
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
                            if dungeon.is_player(monster.x, monster.y - 1) {
                                entity.offset_y.set(-MOVE_Y_OFFSET);
                            }
                        }
                        Direction::Down => {
                            if dungeon.is_player(monster.x, monster.y + 1) {
                                entity.offset_y.set(MOVE_Y_OFFSET);
                            }
                        }
                        Direction::Left => {
                            if dungeon.is_player(monster.x - 1, monster.y) {
                                entity.offset_x.set(-MOVE_X_OFFSET);
                            }
                        }
                        Direction::Right => {
                            if dungeon.is_player(monster.x + 1, monster.y) {
                                entity.offset_x.set(MOVE_X_OFFSET);
                            }
                        }
                    }
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
            else if gp.up.pressed() {
                DungeonCrawlerProgram::move_player(MovePlayerCommandInput {
                    direction: Direction::Up,
                });
                state.last_exec_at = tick();
                state.last_exec_turn = Some(dungeon.turn);
                if dungeon.is_position_blocked(dungeon.player.x, dungeon.player.y - 1) {
                    state.player.offset_y.set(-MOVE_Y_OFFSET);
                }
            } else if gp.down.pressed() {
                DungeonCrawlerProgram::move_player(MovePlayerCommandInput {
                    direction: Direction::Down,
                });
                state.last_exec_at = tick();
                state.last_exec_turn = Some(dungeon.turn);
                if dungeon.is_position_blocked(dungeon.player.x, dungeon.player.y + 1) {
                    state.player.offset_y.set(MOVE_Y_OFFSET);
                }
            } else if gp.left.pressed() {
                DungeonCrawlerProgram::move_player(MovePlayerCommandInput {
                    direction: Direction::Left,
                });
                state.last_exec_at = tick();
                state.last_exec_turn = Some(dungeon.turn);
                if dungeon.is_position_blocked(dungeon.player.x - 1, dungeon.player.y) {
                    state.player.offset_x.set(-MOVE_X_OFFSET);
                }
            } else if gp.right.pressed() {
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

        // Center camera on player
        set_cam!(
            x = state.player.x.get() + (TILE_SIZE / 2) + 64,
            y = state.player.y.get() + (TILE_SIZE / 2)
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
            "floor",
            w = dungeon_w,
            h = dungeon_h,
            repeat = true,
            opacity = 0.01,
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
                ObstacleKind::StoneBlock => {
                    sprite!(
                        "wall",
                        x = obstacle.x * TILE_SIZE,
                        y = obstacle.y * TILE_SIZE,
                    );
                }
                ObstacleKind::Firepit => {
                    sprite!(
                        "firepit",
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
            let y = y + state.player.offset_y.get() - 6;
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
                    sprite!("blue_blob", x = x, y = y + 3, fps = fps::FAST);
                }
                MonsterKind::OrangeGoblin => {
                    ellipse!(
                        x = x + 2,
                        y = y + 8,
                        w = TILE_SIZE - 4,
                        h = TILE_SIZE - 4,
                        color = SHADOW_COLOR,
                    );
                    sprite!("orange_goblin", x = x, y = y, fps = fps::FAST);
                }
                _ => {
                    ellipse!(
                        x = x + 2,
                        y = y + 8,
                        w = TILE_SIZE - 4,
                        h = TILE_SIZE - 4,
                        color = SHADOW_COLOR,
                    );
                    sprite!("goblin", x = x, y = y, fps = fps::FAST);
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

            // Draw health bar foreground (green)
            rect!(
                x = x,
                y = y - 4,
                w = TILE_SIZE as f32 * (monster.health as f32 / monster.max_health as f32),
                h = 2,
                color = 0x00ff00ff,
            );
        }

        // Draw UI
        let [w, h] = canvas_size!();
        rect!(absolute = true, w = 256, h = 28, color = 0x0000fd);

        // // Left side
        // rect!(
        //     absolute = true,
        //     x = 8,
        //     y = 0,
        //     w = 78,
        //     h = h,
        //     color = 0x000000ff,
        //     border_color = 0x83758bff,
        //     border_width = 2,
        //     border_radius = 4,
        // );

        // // Top-left panel
        // rect!(
        //     absolute = true,
        //     x = 8,
        //     y = 16,
        //     w = 78,
        //     h = h - 78 - 16,
        //     color = 0x000000ff,
        //     border_color = 0x83758bff,
        //     border_width = 2,
        //     border_radius = 4,
        // );
        // let mut x = 8 + 5;
        // let mut y = 140;
        // text!(
        //     "FLOOR",
        //     absolute = true,
        //     font = Font::S,
        //     x = x,
        //     y = y,
        //     color = 0xff00ffff
        // );
        // text!("           {:0>3}", dungeon.floor; absolute = true, font = Font::S, x = x, y = y);
        // y += 7;
        // text!(
        //     "GOLD",
        //     absolute = true,
        //     font = Font::S,
        //     x = x,
        //     y = y,
        //     color = 0xffff00ff
        // );
        // text!("           {:0>3}", dungeon.player.gold; absolute = true, font = Font::S, x = x, y = y);
        // y += 7;
        // text!("HP", absolute = true, font = Font::S, x = x, y = y,);
        // text!("         {:0>2}/{:0>2}", dungeon.player.health, dungeon.player.max_health; absolute = true, font = Font::S, x = x, y = y, color = match dungeon.player.health as f32 / dungeon.player.max_health as f32 {
        //     0.75..=1.0 => 0x00ff00ff,
        //     0.25..=0.75 => 0xffff00ff,
        //     _ => 0xff0000ff,
        // });
        // y += 7;
        // text!("ATK         {:0>2}", 1; absolute = true, font = Font::S, x = x, y = y);
        // y += 7;
        // text!("DEF         {:0>2}", 0; absolute = true, font = Font::S, x = x, y = y);

        // // Bottom-left panel
        // rect!(
        //     absolute = true,
        //     x = 8,
        //     y = h - 78,
        //     w = 78,
        //     h = 78,
        //     color = 0x000000ff,
        //     border_color = 0x83758bff,
        //     border_width = 2,
        //     border_radius = 4,
        // );
        // let xy = (8., h as f32 - 78.);
        // for i in 0..4 {
        //     for j in 0..4 {
        //         rect!(
        //             absolute = true,
        //             x = xy.0 + (i as f32 * 18.) + 4.,
        //             y = xy.1 + (j as f32 * 18.) + 4.,
        //             w = 16,
        //             h = 16,
        //             color = 0x00000000,
        //             border_color = 0x524c52ff,
        //             border_width = 1,
        //             border_radius = 3,
        //         );
        //     }
        // }
        // circ!(
        //     absolute = true,
        //     d = 12,
        //     x = 72,
        //     y = h - 15,
        //     color = 0xff00ff88
        // );
        // circ!(
        //     absolute = true,
        //     d = 12,
        //     x = 72,
        //     y = h - 16,
        //     color = 0xff00ffff
        // );
        // text!("B", absolute = true, x = 72 + 4, y = h - 16 + 3,);

        // Top-right panel
        rect!(
            absolute = true,
            x = 256,
            y = 0,
            w = 128,
            h = h,
            color = 0x000000ff,
            border_color = 0x83758bff,
            border_width = 2,
            border_radius = 4,
        );
        text!(
            "ADVENTURE LOG: FLOOR {0:<2}", dungeon.floor + 1;
            absolute = true,
            x = 256 + 8,
            y = 4,
            font = Font::M,
            color = 0xffffffff
        );
        path!(
            absolute = true,
            color = 0x83758bff,
            width = 2,
            start = (256, 14),
            end = (w, 14)
        );
        let mut i = 2;
        for log in &dungeon.logs {
            text!(&log, absolute = true, x = 256 + 8, y = i * 10);
            i += 1;
        }

        // Bottom-right panel
        // rect!(
        //     absolute = true,
        //     x = 256,
        //     y = h / 2,
        //     w = 128,
        //     h = h / 2,
        //     color = 0x000000ff,
        //     border_color = 0x83758bff,
        //     border_width = 2,
        //     border_radius = 4,
        // );

        // outer background
        // let color = 0xde599cff;
        // let color = 0x293c8bff;
        // rect!(
        //     absolute = true,
        //     w = w,
        //     h = 16,
        //     x = 0,
        //     y = 0,
        //     // color = 0x293c8bff,
        //     color = color
        // );
        // rect!(
        //     absolute = true,
        //     w = w,
        //     h = h + 8,
        //     x = 0,
        //     y = 0,
        //     color = 0x00000000,
        //     border_color = color,
        //     border_width = 8,
        //     border_radius = 12,
        // );
        // rect!(
        //     absolute = true,
        //     w = w,
        //     h = h + 8,
        //     x = 0,
        //     y = 8,
        //     color = 0x00000000,
        //     border_color = color,
        //     border_width = 8,
        //     border_radius = 12,
        // );

        // Display text if player dies
        if dungeon.player.health == 0 {
            text!(
                "YOU DIED. GAME OVER!",
                absolute = true,
                x = 8,
                y = 8,
                font = Font::L,
                color = 0xff0000ff
            );
        }
        // Display text if all enemies are defeated
        else if dungeon.is_exit(dungeon.player.x, dungeon.player.y) {
            text!(
                "GO TO NEXT FLOOR? PRESS START!",
                absolute = true,
                x = 8,
                y = 8,
                font = Font::L,
                color = 0xffffffff
            );
        } else {
            // Draw Life
            text!("- LIFE -", absolute = true, x = 48, y = 4, font = Font::L);
            for i in 0..dungeon.player.max_health {
                if dungeon.player.health > i {
                    sprite!("full_heart", absolute = true, x = i * 16, y = 12)
                } else {
                    sprite!("empty_heart", absolute = true, x = i * 16, y = 12)
                }
            }

            // Draw Gold
            text!("- GOLD -", absolute = true, x = 180, y = 4, font = Font::L);
            text!("  ${:0>3}  ", dungeon.player.gold; absolute = true, x = 180, y = 16, font = Font::L);
        }

        // Swipe transition
        let p = state.floor.elapsed as f64 / FLOOR_DUR as f64;
        {
            let xo = p * w as f64;
            rect!(absolute = true, x = xo, w = w, h = h, color = 0x000000ff);
            rect!(absolute = true, x = -xo, w = w, h = h, color = 0x000000ff);
        }

        // circ!(absolute=true, d = 32, x = 32, y = 32, color = if is_ready_to_exec { 0x00ff00ff } else { 0xff0000ff });
    }

    // If no existing dungeon, allow player to create one
    if let Err(err) = &dungeon {
        // Handle user input
        let gp = gamepad(0);
        if gp.start.just_pressed() {
            DungeonCrawlerProgram::create_new_dungeon(CreateNewDungeonCommandInput { reset: true });
        }

        let t = tick() as f32;
        sprite!("night_sky", w = w, sw = w, tx = tick(), repeat = true);
        // sprite!("night_sky");
        if t % 2. == 0. {
            sprite!(
                "clouds_3",
                y = ((t / 16.).cos() * 2.) + 1.,
                w = w,
                sw = w,
                tx = tick() / 2,
                repeat = true,
                opacity = 0.5
            );
        }
        sprite!(
            "clouds_0",
            y = ((t / 10.).cos() * 2.) + 1.,
            w = w,
            sw = w,
            tx = tick() / 8,
            repeat = true
        );
        sprite!("title_b", y = ((t / 32.).cos() * 2.) - 2., x = 64);
        sprite!(
            "clouds_1",
            y = ((t / 24.).cos() * 2.) + 1.,
            w = w,
            sw = w,
            tx = tick() / 4,
            repeat = true
        );
        sprite!(
            "clouds_2",
            y = ((t / 8.).cos() * 2.) + 1.,
            w = w,
            sw = w,
            tx = tick() / 2,
            repeat = true
        );
        sprite!(
            "title_text",
            y = 114.,
            x = 128 - 9,
            w = 146,
            h = 93,
            color = 0x000000ff,
            opacity = 0.75
        );
        sprite!("title_text", y = 112., x = 128 - 9, w = 146, h = 93);

        // rect!(absolute = true, y = 232 + 15, w = w, h = 1, color = 0x000000ff);
        if os::user_id().is_some() {
            rect!(absolute = true, y = 232, w = w, h = 32, color = 0x222034ff);
            if t / 2. % 32. < 16. {
                text!(
                    "PRESS START",
                    x = 149,
                    y = 240,
                    color = 0xffffffff,
                    font = Font::L
                );
            }
        }

        // text!("PRESS START {:?}", os::user_id(););
        // text!("Reason: {}", err; y = 32);
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

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
enum MonsterKind {
    GreenGoblin,
    OrangeGoblin,
    YellowBlob,
    BlueBlob,
    RedBlob,
}
impl MonsterKind {
    pub fn abbrev<'a>(&self) -> &'a str {
        match self {
            Self::BlueBlob => "B. Blob",
            Self::RedBlob => "R. Blob",
            Self::YellowBlob => "Y. Blob",
            Self::GreenGoblin => "G. Goblin",
            Self::OrangeGoblin => "O. Goblin",
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

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
enum ObstacleKind {
    StoneBlock,
    Firepit,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
struct Obstacle {
    x: i32,
    y: i32,
    kind: ObstacleKind,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
struct Dungeon {
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
                let monster_name = monster.kind.abbrev();
                let msg = format!("P1 attacks {}!", monster_name);
                log(&msg);
                self.logs.push(msg);
                let amount = self.player.strength;
                let msg = format!("P1 did {amount} damage.");
                log(&msg);
                self.logs.push(msg);
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

            // Move monster towards the player
            let dx = player.x - mx;
            let dy = player.y - my;
            let (dir, mx, my) = match (dx.abs() > dy.abs(), dx > 0, dy > 0) {
                // move up
                (false, _, false) => (Direction::Up, mx, my - 1),
                // move down
                (false, _, true) => (Direction::Down, mx, my + 1),
                // move left
                (true, false, _) => (Direction::Left, mx - 1, my),
                // move right
                (true, true, _) => (Direction::Right, mx + 1, my),
            };

            if self.is_out_of_bounds(mx, my) {
                return true;
            }

            if self.is_position_occupied(mx, my) {
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
        let filepath = Self::get_dungeon_filepath(&user_id);
        let file = os::read_file(Self::PROGRAM_ID, &filepath)?;
        Dungeon::try_from_slice(&file.contents)
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
            Dungeon {
                floor: 0,
                turn: 0,
                width: 8,
                height: 8,
                player: Player {
                    x: program::random_number::<i32>().abs() % 8,
                    y: program::random_number::<i32>().abs() % 8,
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
            // Embiggen every other floor
            if dungeon.floor % 2 == 0 {
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
        let num_monsters = magic_ratio;
        while dungeon.monsters.len() < num_monsters {
            let x = program::random_number::<i32>().abs() % max_x;
            let y = program::random_number::<i32>().abs() % max_y;
            if !dungeon.is_position_occupied(x, y) {
                dungeon
                    .monsters
                    .push(match program::random_number::<u32>() % 10 {
                        0..=1 => Monster {
                            x,
                            y,
                            health: 5,
                            max_health: 5,
                            strength: 1,
                            direction: Direction::Down,
                            kind: MonsterKind::OrangeGoblin,
                        },
                        2..=5 => Monster {
                            x,
                            y,
                            health: 1,
                            max_health: 1,
                            strength: 2,
                            direction: Direction::Down,
                            kind: MonsterKind::BlueBlob,
                        },
                        _ => Monster {
                            x,
                            y,
                            health: 3,
                            max_health: 3,
                            strength: 1,
                            direction: Direction::Down,
                            kind: MonsterKind::GreenGoblin,
                        },
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
                    ObstacleKind::Firepit
                } else {
                    // 90% chance for stone block
                    ObstacleKind::StoneBlock
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

            divide(grid, walls, x, y, wall_x - x, height);
            divide(grid, walls, wall_x + 1, y, x + width - wall_x - 1, height);
        }
    }

    divide(&mut grid, &mut walls, 0, 0, width, height);
    walls
}
