use crate::*;

pub const TOTAL_ROUNDS: usize = 6;
const UNIT_RATINGS: [(UnitType, u8); 20] = [
    // Basic units
    (UnitType::Axeman, 4),
    (UnitType::Blade, 2),
    (UnitType::Hunter, 2),
    (UnitType::Pyro, 1),
    (UnitType::Bigpound, 2),
    (UnitType::Deathray, 3),
    (UnitType::Cosmo, 3),
    (UnitType::Zombie, 1),
    (UnitType::Shield, 3),
    (UnitType::Serpent, 1),
    // Advanced units
    (UnitType::Sabre, 4),
    (UnitType::Flameboi, 5),
    (UnitType::Bazooka, 5),
    (UnitType::Draco, 6),
    (UnitType::Saucer, 5),
    (UnitType::Tanker, 7),
    (UnitType::Catapult, 7),
    (UnitType::Darkknight, 6),
    (UnitType::Yeti, 5),
    (UnitType::Igor, 4),
];

pub fn title_screen_unit(rng: &mut RNG, data_store: &UnitDataStore) -> WalkingUnitPreview {
    //let is_left_side = (rng.next() % 2) == 0;
    let x_pos = -64.0;
    let y_pos = rng.next_in_range(0, 160) as f32;
    let pos = (x_pos, y_pos);
    let available_types: Vec<&UnitType> = data_store.data.keys().collect();
    let index = rng.next_in_range(0, available_types.len() as u32 - 1) as usize;
    let unit_type = available_types[index].clone();
    let data = data_store.get_unit_data(&unit_type).unwrap();
    let s_w = data.sprite_width;
    let speed = data.speed;
    let mut a = AnimatedSprite::new(pos, true);
    let anim_name = unit_type.as_string() + "_walk";
    a.set_anim(anim_name, s_w, 4, UNIT_ANIM_SPEED, true);
    let w = WalkingUnitPreview::new(unit_type, a, pos, speed as f32, false);
    w
}

fn draw_checkerboard_background() {
    let checker_size = 16; // Size of each square
    let light_color: usize = 0x8f8cacff;
    let dark_color: usize = 0x7a7795ff; // Slightly darker version

    for row in 0..(232 / checker_size) {
        for col in 0..(384 / checker_size) {
            let checker_x = col * checker_size;
            let checker_y = row * checker_size;

            // Alternate colors based on position
            let color = if (row + col) % 2 == 0 {
                light_color
            } else {
                dark_color
            };

            rect!(
                x = checker_x,
                y = checker_y,
                w = checker_size as f32,
                h = checker_size as f32,
                color = color
            );
        }
    }
}

fn draw_textured_background() {
    let tile_size: i32 = 2;
    let base_color: u32 = 0x8f8cacff;
    let variation = 8; // Reduced variation for subtlety

    // Extend the drawing area 50 pixels in each direction
    for row in -50..(216 / tile_size + 50) {
        for col in -50..(384 / tile_size + 50) {
            let x = (col * tile_size) as usize;
            let y = (row * tile_size) as usize;

            // Gentler pattern
            let offset = (((x as u32) * 11 + (y as u32) * 7) % variation) as u32;
            let color = base_color.wrapping_sub(offset * 0x00010101); // Smaller color change

            rect!(
                x = x,
                y = y,
                w = tile_size as f32,
                h = tile_size as f32,
                color = color
            );
        }
    }
}

fn draw_diagonal_background() {
    let tile_size = 2;
    let light_color: usize = 0x8f8cacff;
    let dark_color: usize = 0x7a7795ff;

    for row in 0..(232 / tile_size) {
        for col in 0..(384 / tile_size) {
            let x = col * tile_size;
            let y = row * tile_size;

            // Diagonal pattern
            let color = if ((row + col) / 2) % 2 == 0 {
                light_color
            } else {
                dark_color
            };

            rect!(
                x = x,
                y = y,
                w = tile_size as f32,
                h = tile_size as f32,
                color = color
            );
        }
    }
}

fn draw_organic_background() {
    let base_size = 8;
    let light_color: usize = 0x8f8cacff;
    let dark_color: usize = 0x7a7795ff;

    let mut y = 0;
    while y < 232 {
        let mut x = 0;
        while x < 384 {
            let size = base_size + (((x + y) / 32) % 8); // Subtle size variation

            let color = if ((x / base_size + y / base_size) % 2) == 0 {
                light_color
            } else {
                dark_color
            };

            rect!(
                x = x,
                y = y,
                w = size as f32,
                h = size as f32,
                color = color
            );

            x += size;
        }
        y += base_size;
    }
}

pub fn team_selection(state: &mut GameState) {
    draw_team_info_and_buttons(state);
    let gp = gamepad(0);
    if gp.start.just_pressed() {
        state.phase = Phase::PreBattle;
    }

    //move camera if you press up and down
    if gp.down.pressed() {
        set_cam!(y = cam!().1 + 3);
    } else if gp.up.pressed() {
        set_cam!(y = cam!().1 - 3);
    }
}

pub fn dbgo(state: &mut GameState) {
    clear!(0x8f8cacff);
    //draw_checkerboard_background();
    //draw_organic_background();
    //draw_diagonal_background();
    draw_textured_background();
    let gp = gamepad(0);
    let m = mouse(0);
    //get the data store
    if state.data_store.is_none() {
        match UnitDataStore::load_from_csv(UNIT_DATA_CSV) {
            Ok(loaded_store) => {
                state.data_store = Some(loaded_store);
            }
            Err(e) => {
                eprintln!("Failed to load UnitDataStore: {}", e);
                state.data_store = Some(UnitDataStore::new());
            }
        }
        //print out unit powers
    }
    let mut ready_to_transition = false;
    if let Some(transition) = &mut state.transition {
        if transition.ready_for_scene_change {
            ready_to_transition = true;
            transition.start_transition_out(&mut state.rng);
        }
    }

    match state.dbphase {
        DBPhase::ParticleTest => {
            // let config = BurstConfig {
            //     center: (160., 120.), // Center of screen
            //     radius: 5.0,
            //     direction: 0.0,               // Right
            //     spread: std::f32::consts::PI, // 180 degrees
            //     speed: 2.0,
            //     speed_var: 1.0,
            //     color: 0xFF0000FF, // Red
            //     lifetime: 1.0,
            //     count: 20,
            // };
            // let config = BurstConfig {
            //     center: (160., 120.),
            //     radius: 2.0,
            //     direction: 0.0,
            //     spread: std::f32::consts::TAU, // 180 degree spread
            //     speed: 0.8,                    // Slower speed
            //     speed_var: 0.2,                // Less variation
            //     color: ACID_GREEN as u32,
            //     lifetime: 0.3, // Longer lifetime to see slow particles
            //     count: 10,
            // };
            // let mut config = BurstConfig::circular_burst((160., 120.), ACID_GREEN as u32);
            // config.lifetime = 0.3;
            // config.speed = 0.5;
            // config.count = 10;
            // let config = BurstConfig::confetti((140., 0.), DAMAGE_TINT_RED as u32);
            if gamepad(0).a.just_pressed() {
                start_end_game_particles(&mut state.particle_manager);
            }
            state.particle_manager.update();
            state.particle_manager.draw();
        }
        DBPhase::Title => {
            if state.round != 1 && state.round != 7 {
                state.dbphase = DBPhase::Shop;
            } else {
                if state.title_screen_units.len() == 0 {
                    //make some title screen units
                    for _ in 0..3 {
                        let w =
                            title_screen_unit(&mut state.rng, state.data_store.as_ref().unwrap());
                        state.title_screen_units.push(w);
                    }
                } else {
                    // Create a new unit every 60 ticks
                    if tick() % 60 == 0 {
                        let w =
                            title_screen_unit(&mut state.rng, state.data_store.as_ref().unwrap());
                        state.title_screen_units.push(w);
                    }
                    //TODO: Base this on foot position not top left
                    // First sort the units by y-position
                    state
                        .title_screen_units
                        .sort_by(|a, b| a.pos.1.partial_cmp(&b.pos.1).unwrap());

                    // Then update them in order
                    state.title_screen_units.retain_mut(|t| {
                        t.update();
                        true
                    });
                }
                //PIXEL WARS
                sprite!("pixelwars_title_static", x = 128, y = 31);
                if state.round == 1 {
                    power_text!(
                        "TINY UI BATTLES FOR",
                        x = 0,
                        y = 108,
                        center_width = 384,
                        font = Font::S,
                        drop_shadow = SHADOW_COLOR
                    );
                    power_text!(
                        "YOUR VIEWING PLEASURE",
                        x = 0,
                        y = 116,
                        center_width = 384,
                        font = Font::S,
                        drop_shadow = SHADOW_COLOR
                    );
                    power_text!(
                        "- POWERED BY TURBO OS -",
                        x = 0,
                        y = 129,
                        center_width = 384,
                        font = Font::S,
                        drop_shadow = SHADOW_COLOR
                    );
                } else {
                    power_text!(
                        "YOU WON THE PIXEL WARS!",
                        x = 0,
                        y = 108,
                        center_width = 384,
                        font = Font::L,
                        drop_shadow = SHADOW_COLOR
                    );
                    power_text!(
                        "- POWERED BY TURBO OS -",
                        x = 0,
                        y = 129,
                        center_width = 384,
                        font = Font::S,
                        drop_shadow = SHADOW_COLOR
                    );
                }
                let opacity = if (tick() % 90) < 75 { 0xFF } else { 0x00 };

                let color = 0xFFFFFF00 | opacity as u32;
                let drop_shadow = 0x69668200 | opacity as u32;
                power_text!(
                    "Click to Start",
                    x = 0,
                    y = 184,
                    center_width = 384,
                    font = Font::S,
                    underline = true,
                    color = color,
                    drop_shadow = drop_shadow
                );
            }
            if m.left.just_pressed() {
                if state.round == TOTAL_ROUNDS as u8 + 1 {
                    *state = GameState::default();
                } else {
                    state.dbphase = DBPhase::Shop;
                }
            }
            if gp.right.just_pressed() {
                state.teams.clear();
                state.teams.push(Team::new(
                    "Battle Bois".to_string(),
                    state.data_store.as_ref().unwrap().clone(),
                ));
                state.teams.push(Team::new(
                    "Pixel Peeps".to_string(),
                    state.data_store.as_ref().unwrap().clone(),
                ));
                state.dbphase = DBPhase::Sandbox;
            }
        }

        DBPhase::Shop => {
            //get the data store
            if state.data_store.is_none() {
                match UnitDataStore::load_from_csv(UNIT_DATA_CSV) {
                    Ok(loaded_store) => {
                        state.data_store = Some(loaded_store);
                    }
                    Err(e) => {
                        eprintln!("Failed to load UnitDataStore: {}", e);
                        state.data_store = Some(UnitDataStore::new());
                    }
                }
            }
            let ds = state.data_store.as_ref().unwrap();

            //create the enemy team
            if state.enemy_team_placeholder == None {
                let team_name =
                    TEAM_NAMES[(state.rng.next() as usize) % TEAM_NAMES.len()].to_string();
                let t = generate_team_db(
                    &state.data_store.as_ref().unwrap(),
                    &mut state.rng,
                    None,
                    team_name,
                    get_power_level_for_round(state.round),
                    state.round,
                );
                state.enemy_team_placeholder = Some(t);
            }

            if state.shop.len() == 0 {
                //let player revive units if they lost 1/3 of units in battle after round 2
                let mut fallen_units = None;
                let num_units = state.last_round_dead_units.len();
                turbo::println!("NUM DEAD UNITS: {}", num_units);
                if num_units > 0 && num_units * 2 > state.teams[0].units.len() && state.round > 2 {
                    let percent_to_include = 80;
                    let num_to_include = (num_units * percent_to_include / 100).max(1); // Ensure at least 1 unit

                    // Clone the fallen units
                    let mut shuffled_units = state.last_round_dead_units.clone();

                    // Custom shuffle using state.rng
                    for i in (0..shuffled_units.len()).rev() {
                        let j = state.rng.next() as usize % (i + 1);
                        shuffled_units.swap(i, j);
                    }

                    // Take the first num_to_include units
                    fallen_units = Some(shuffled_units[0..num_to_include].to_vec());
                }

                //get the unit types on the current team
                let mut unit_types = Vec::new();
                if state.teams.len() != 0 {
                    unit_types = state.teams[0].units.clone();
                    unit_types.extend(state.last_round_dead_units.clone());
                    unit_types.sort();
                    unit_types.dedup();
                }

                //if unit_types contains an upgradeable type, then send that into the create unit packs function
                let mut upgradeable_unit = None;
                if unit_types.contains(&UnitType::Tanker) {
                    upgradeable_unit = Some((UnitType::Tanker, UnitType::GoldenTank));
                } else if unit_types.contains(&UnitType::Yeti) {
                    upgradeable_unit = Some((UnitType::Yeti, UnitType::BloodYeti));
                }
                let num_artifacts = if state.round == 1 { 0 } else { 1 };

                let player_artifacts: Vec<Artifact> = state
                    .artifacts
                    .iter()
                    .filter(|artifact| artifact.team == 0)
                    .cloned()
                    .collect();

                let mut team_upgrades = Vec::new();
                if state.teams.len() > 0 {
                    team_upgrades = state.teams[0].upgraded_units.clone();
                }

                //TODO: maybe ask ai to make this make more sense
                state.shop = create_unit_packs(
                    4,
                    num_artifacts,
                    &ds,
                    &mut state.rng,
                    state.round,
                    fallen_units,
                    unit_types,
                    team_upgrades,
                    &player_artifacts,
                    upgradeable_unit,
                );
                if state.round == 1 {
                    state.num_picks = 3;
                } else {
                    state.num_picks = 2;
                }
            }
            let m = mouse(0);
            let m_pos = (m.position[0], m.position[1]);
            for (i, u) in state.shop.iter_mut().enumerate() {
                u.draw(m_pos);
                if m.left.just_pressed()
                    && u.is_hovered(m_pos)
                    && !u.is_picked
                    && state.num_picks > 0
                {
                    select_unit_pack(i, state);
                    state.num_picks -= 1;
                    if state.num_picks == 0 {
                        if let Some(enemy_team) = &state.enemy_team_placeholder {
                            state.teams.push(enemy_team.clone());
                        }
                        state.units = create_units_for_all_teams(
                            &mut state.teams,
                            &mut state.rng,
                            &state.data_store.as_ref().unwrap(),
                        );
                        // if state.round == 2 || state.round == 4 {
                        //     state.dbphase = DBPhase::ArtifactShop;
                        // } else {
                        if state.transition.is_none() {
                            state.transition = Some(Transition::new(&mut state.rng));
                            //}
                        }
                    }
                    break;
                }
            }
            let txt = format!("Choose {}", state.num_picks);
            power_text!(
                &txt,
                x = 0,
                y = 20,
                font = Font::L,
                drop_shadow = SHADOW_COLOR,
                center_width = 384,
                underline = true,
            );

            if state.teams.len() != 0 {
                draw_current_team(&state.teams[0], &state.data_store.as_ref().unwrap(), false);
            }
            if let Some(enemy_team) = &state.enemy_team_placeholder {
                draw_current_team(enemy_team, &state.data_store.as_ref().unwrap(), true);
            }
            if state.num_picks == 0 {
                if ready_to_transition {
                    state.dbphase = DBPhase::Battle;
                }
            }
            //enter sandbox mode
            if gp.right.just_pressed() {
                state.teams.clear();
                state.teams.push(Team::new(
                    "Battle Bois".to_string(),
                    state.data_store.as_ref().unwrap().clone(),
                ));
                state.teams.push(Team::new(
                    "Pixel Peeps".to_string(),
                    state.data_store.as_ref().unwrap().clone(),
                ));
                state.dbphase = DBPhase::Sandbox;
            }
        }
        DBPhase::ArtifactShop => {
            state.dbphase = DBPhase::Battle;
            // let mut should_end = false;
            // if gp.a.just_pressed() || (state.round != 2 && state.round != 4) {
            //     should_end = true;
            //     ready_to_transition = true;
            // }

            // let m = mouse(0);
            // let m_pos = (m.position[0], m.position[1]);
            // //generate 2 choices
            // if state.artifact_shop.len() == 0 {
            //     let player_artifacts: Vec<Artifact> = state
            //         .artifacts
            //         .iter()
            //         .filter(|artifact| artifact.team == 0)
            //         .cloned()
            //         .collect();

            //     state.artifact_shop = create_artifact_shop(2, &mut state.rng, &player_artifacts);
            // }
            // for (i, a) in state.artifact_shop.iter_mut().enumerate() {
            //     let pos = (100 + (i as i32 * 100), 50);
            //     if m.left.just_pressed() && a.is_hovered(pos, m_pos) {
            //         state.artifacts.push(a.clone());
            //         if state.transition.is_none() {
            //             state.transition = Some(Transition::new(&mut state.rng));
            //         }
            //     } else {
            //         if !should_end {
            //             //a.draw_card(pos, m_pos);
            //         }
            //     }
            // }
            // if !should_end {
            //     let txt = format!("Choose An Artifact");
            //     power_text!(
            //         &txt,
            //         x = 0,
            //         y = 20,
            //         font = Font::L,
            //         drop_shadow = SHADOW_COLOR,
            //         center_width = 384,
            //         underline = true,
            //     );
            //     if state.teams.len() != 0 {
            //         draw_current_team(&state.teams[0], &state.data_store.as_ref().unwrap(), false);
            //     }
            //     if let Some(enemy_team) = &state.enemy_team_placeholder {
            //         draw_current_team(enemy_team, &state.data_store.as_ref().unwrap(), true);
            //     }
            // }
            // if ready_to_transition {
            //     state.dbphase = DBPhase::Battle;
            //     //TODO: maybe turn this into a clearer transition step
            //     //also make this into more of a game loop (like they get more in later levels)
            //     //TODO: Figure out what to do with enemy team artifacts better than this
            //     // let enemy_artifact_kinds = choose_artifacts_for_enemy_team(2, &mut state.rng);
            //     // let enemy_artifacts: Vec<Artifact> = enemy_artifact_kinds
            //     //     .into_iter()
            //     //     .map(|kind| Artifact::new(kind, 1))
            //     //     .collect();
            //     // state.artifacts.extend(enemy_artifacts);
            // }
        }
        DBPhase::Sandbox => {
            draw_team_info_and_buttons(state);
            if gp.start.just_pressed() {
                state.units = create_units_for_all_teams(
                    &mut state.teams,
                    &mut state.rng,
                    &state.data_store.as_ref().unwrap(),
                );
                state.is_playing_sandbox_game = true;
                state.dbphase = DBPhase::Battle;
                reset_cam();
            }
            if gp.down.pressed() {
                set_cam!(y = cam!().1 + 3);
            } else if gp.up.pressed() {
                set_cam!(y = cam!().1 - 3);
            } else if gp.left.just_pressed() {
                *state = GameState::default();

                reset_cam();
            }
        }
        DBPhase::Battle => {
            if state.battle_countdown_timer == BATTLE_COUNTDOWN_TIME {
                //set up the map if it has traps
                //TODO: Set up a system for this based on round number or something
                //state.traps = setup_level(0, &mut state.rng);
                //handle start of battle artifacts
                apply_start_of_battle_artifacts(
                    &mut state.units,
                    &mut state.traps,
                    &mut state.rng,
                    &mut state.artifacts,
                );
                for u in &mut state.units {
                    if u.pos.0 > 100. {
                        if let Some(display) = u.display.as_mut() {
                            display.is_facing_left = true;
                        }
                    }
                    //Do any special sequencing stuff here
                    //u.set_march_position();
                    //probably give them a target, set to moving, and give them a new state like (marching in),
                }
            }
            //figure out the length of the simulation
            if state.simulation_result.is_none() {
                simulate_battle_locally(state);
            }
            // } else {
            //     log!(
            //         "NUM Frames in sim: {:?}",
            //         state.simulation_result.as_ref().unwrap().num_frames
            //     );
            // }
            if state.battle_countdown_timer > 0 {
                //this happens once at the start of battle phase

                // for u in &mut state.units {
                //     u.start_cheering();
                //     u.update();
                // }
                state.battle_countdown_timer -= 1;

                //show text
                draw_prematch_timer(state.battle_countdown_timer);
            } else {
                let mut speed = 1;
                if gp.right.pressed() {
                    speed = 8;
                }
                for _ in 0..speed {
                    step_through_battle(
                        &mut state.units,
                        &mut state.attacks,
                        &mut state.traps,
                        &mut state.explosions,
                        &mut state.craters,
                        &mut state.rng,
                        &mut state.artifacts,
                        &mut state.particle_manager,
                        false,
                    );
                    state.elapsed_frames += 1;
                }
            }

            let winner_idx = has_some_team_won(&state.units);
            if winner_idx.is_some() {
                let needs_update_complete = !state.is_battle_complete;
                if needs_update_complete {
                    apply_end_of_battle_artifacts(
                        winner_idx.unwrap() as usize,
                        &mut state.units,
                        &mut state.rng,
                        &mut state.artifacts,
                    );
                    state.is_battle_complete = true;
                    // Other updates
                }
            }

            //check if you are near the end but not finished.
            //if so zoom into one of the surviving units
            let max_zoom = 2.0;
            let zoom_duration: usize = 10;
            let easing = Easing::EaseInQuad;

            if let Some(s_r) = &state.simulation_result {
                if state.elapsed_frames > 0
                    && state.elapsed_frames < s_r.num_frames
                    && state.elapsed_frames + 120 > s_r.num_frames
                {
                    if let Some(winning_team) = s_r.winning_team {
                        if let Some(unit) = state
                            .units
                            .iter()
                            .find(|u| u.team != winning_team && u.state != UnitState::Dead)
                        {
                            //set the tweens if they are 0
                            if state.zoom_tween_x.get() == 0.0 {
                                let s = cam!().0 as f32;
                                state.zoom_tween_x = Tween::new(s);
                                state.zoom_tween_x.set(unit.pos.0);
                                state.zoom_tween_x.set_duration(zoom_duration);
                                state.zoom_tween_x.set_ease(easing);
                            }
                            if state.zoom_tween_y.get() == 0.0 {
                                let s = cam!().1 as f32;
                                state.zoom_tween_y = Tween::new(s);
                                state.zoom_tween_y.set(unit.pos.1);
                                state.zoom_tween_y.set_duration(zoom_duration);
                                state.zoom_tween_y.set_ease(easing);
                            }
                            if state.zoom_tween_z.get() == 0.0 {
                                state.zoom_tween_z = Tween::new(1.0);
                                state.zoom_tween_z.set(max_zoom);
                                state.zoom_tween_z.set_duration(zoom_duration);
                                state.zoom_tween_z.set_ease(easing);
                            }
                            let (x, y, z) = (
                                state.zoom_tween_x.get(),
                                state.zoom_tween_y.get(),
                                state.zoom_tween_z.get(),
                            );

                            set_cam!(x = x, y = y, z = z);
                        }
                        let (x, y, z) = cam!();
                        //flag for shader
                        if z == max_zoom {
                            rect!(
                                color = 0xe3e3ffff,
                                w = 2,
                                h = 2,
                                x = x as f32 - 96.0,
                                y = y as f32 - 54.0
                            );
                        }
                    }
                } else if state.elapsed_frames > s_r.num_frames {
                    //TODO: Turn this into something that makes sense
                    let (x, y, z) = cam!();
                    if state.elapsed_frames > s_r.num_frames + 40
                        && state.elapsed_frames < s_r.num_frames + 50
                        && z == max_zoom
                    {
                        let easing = Easing::EaseOutQuad;
                        log!("RESETTING TWEENS");
                        let s = cam!().0 as f32;
                        state.zoom_tween_x = Tween::new(s);
                        state.zoom_tween_x.set(192.0);
                        state.zoom_tween_x.set_duration(zoom_duration);
                        state.zoom_tween_x.set_ease(easing);

                        let s = cam!().1 as f32;
                        state.zoom_tween_y = Tween::new(s);
                        state.zoom_tween_y.set(108.0);
                        state.zoom_tween_y.set_duration(zoom_duration);
                        state.zoom_tween_y.set_ease(easing);

                        state.zoom_tween_z = Tween::new(max_zoom);
                        state.zoom_tween_z.set(1.0);
                        state.zoom_tween_z.set_duration(zoom_duration);
                        state.zoom_tween_z.set_ease(easing);
                    }
                    let (x, y, z) = (
                        state.zoom_tween_x.get(),
                        state.zoom_tween_y.get(),
                        state.zoom_tween_z.get(),
                    );
                    set_cam!(x = x, y = y, z = z);
                }
            }

            /////////////
            //Draw Code//
            /////////////

            //Draw craters beneath everything
            for c in &state.craters {
                c.draw();
            }

            //Draw footprints beneath units
            for u in &mut state.units {
                for fp in &mut u.display.as_mut().unwrap().footprints {
                    fp.draw();
                }
            }
            for t in &mut state.traps {
                t.draw();
            }
            //DRAW UNITS
            let mut indices: Vec<usize> = (0..state.units.len()).collect();

            indices.sort_by(|&a, &b| {
                let unit_a = &state.units[a];
                let unit_b = &state.units[b];

                // First, sort by dead/alive status
                match (
                    unit_a.state == UnitState::Dead,
                    unit_b.state == UnitState::Dead,
                ) {
                    (true, false) => return Ordering::Less,
                    (false, true) => return Ordering::Greater,
                    _ => {}
                }

                // If both are alive or both are dead, sort by y-position
                if unit_a.state != UnitState::Dead {
                    unit_a
                        .pos
                        .1
                        .partial_cmp(&unit_b.pos.1)
                        .unwrap_or(Ordering::Equal)
                } else {
                    Ordering::Equal
                }
            });

            // Draw units in the sorted order
            for &index in &indices {
                state.units[index].draw();
            }
            //draw explosions
            state.explosions.retain_mut(|explosion| {
                explosion.update();
                !explosion.animator.is_done()
            });
            for explosion in &mut state.explosions {
                explosion.draw();
            }
            //draw particles
            state.particle_manager.update();
            state.particle_manager.draw();
            //draw health bar on hover
            //get mouse posisiton
            let m = mouse(0);
            let mpos = (m.position[0] as f32, m.position[1] as f32);

            //for unit, if mouse position is in bounds, then draw health bar
            for u in &mut state.units {
                if u.state != UnitState::Dead && u.is_point_in_bounds(mpos) {
                    u.draw_health_bar();
                }
            }

            //hide health bar if camera zoom isn't 1.0
            let z = cam!().2;
            if z == 1.0 {
                draw_ui(state);
            }
            //TODO: Move all this into the wrap_up game state and transition on winner = some
            //Also just generally make this code a little bit more flexible
            let mut text = "Click to Play Again";
            if let Some(winner_idx) = has_some_team_won(&state.units) {
                for u in &mut state.units {
                    u.start_cheering();
                }
                if z == 1.0 {
                    //draw end game stats
                    draw_end_stats(&state.units, &state.data_store.as_ref().unwrap());
                    //move to the next round, of if in sandbox go back to sandbox and keep the
                    //current settings
                    if m.left.just_pressed() {
                        if state.is_playing_sandbox_game {
                            // Return to sandbox with current teams
                            let teams = state.teams.clone();
                            let artifacts = state.artifacts.clone();
                            *state = GameState::default();
                            state.teams = teams;
                            state.artifacts = artifacts;
                            state.dbphase = DBPhase::Sandbox;
                        } else {
                            // Handle win/loss
                            if winner_idx == 0 {
                                // Win: proceed to next round
                                let mut your_team = state.teams[0].clone();
                                let living_unit_types: Vec<UnitType> = state
                                    .units
                                    .iter()
                                    .filter(|unit| unit.team == 0 && unit.health > 0.0)
                                    .map(|unit| unit.unit_type.clone())
                                    .collect();

                                your_team.units = living_unit_types;
                                let dead_unit_types: Vec<UnitType> = state
                                    .units
                                    .iter()
                                    .filter(|unit| unit.team == 0 && unit.health <= 0.0)
                                    .map(|unit| unit.unit_type.clone())
                                    .collect();

                                //store player artifacts for next round
                                let player_artifacts = state
                                    .artifacts
                                    .iter()
                                    .filter(|artifact| artifact.team == 0)
                                    .cloned()
                                    .collect();
                                let r = state.round + 1;
                                *state = GameState::default();
                                state.teams.push(your_team);
                                state.round = r;
                                state.artifacts = player_artifacts;
                                //using this for reviving the dead units
                                state.last_round_dead_units = dead_unit_types;
                                turbo::println!("{:?}", state.last_round_dead_units);
                            } else {
                                // Loss: reset game
                                *state = GameState::default();
                            }
                        }
                    }

                    // Draw appropriate end animation and text
                    if winner_idx == 0 {
                        if state.particle_manager.bursts.len() == 0 {
                            //start_end_game_particles(&mut state.particle_manager);
                        }
                        draw_end_animation(Some(true));
                        text = "Click to Continue";
                    } else {
                        draw_end_animation(Some(false));
                    }

                    power_text!(
                        &text,
                        x = 0,
                        y = 140,
                        drop_shadow = SHADOW_COLOR,
                        center_width = 384,
                        font = Font::L
                    );
                }
            }
        }
        DBPhase::WrapUp => {
            // Post-battle cleanup and results
        }
    }
    //handle event queue
    while let Some(event) = state.event_queue.pop() {
        match event {
            GameEvent::AddUnitToTeam(team_index, unit_type) => {
                state.teams[team_index].add_unit(unit_type);
            }
            GameEvent::RemoveUnitFromTeam(team_index, unit_type) => {
                state.teams[team_index].remove_unit(unit_type);
            }
            GameEvent::AddArtifactToTeam(team_index, artifact_kind) => {
                if let Some(kind) = ARTIFACT_KINDS.iter().find(|&&k| {
                    std::mem::discriminant(&k) == std::mem::discriminant(&artifact_kind)
                }) {
                    state.artifacts.push(Artifact::new(*kind, team_index as u8));
                    turbo::println!("ADDING ARTIFACT: {:?}", kind);
                }
            }
            GameEvent::RemoveArtifactFromTeam(team_index, artifact_kind) => {
                state.artifacts.retain(|a| {
                    !(std::mem::discriminant(&a.artifact_kind)
                        == std::mem::discriminant(&artifact_kind)
                        && a.team == team_index as u8)
                });
            }
            GameEvent::ChooseTeam(team_num) => {
                let mut team_choice_counter = TeamChoiceCounter {
                    team_0: 0,
                    team_1: 0,
                };
                if state.selected_team_index.is_some() {
                    if state.selected_team_index == Some(0) && team_num == 1 {
                        team_choice_counter.team_0 = -1;
                        team_choice_counter.team_1 = 1;
                    } else if state.selected_team_index == Some(1) && team_num == 0 {
                        team_choice_counter.team_0 = 1;
                        team_choice_counter.team_1 = -1;
                    }
                } else {
                    if team_num == 0 {
                        team_choice_counter.team_0 = 1;
                        team_choice_counter.team_1 = 0;
                    } else if team_num == 1 {
                        team_choice_counter.team_0 = 0;
                        team_choice_counter.team_1 = 1;
                    }
                }

                let bytes = borsh::to_vec(&team_choice_counter).unwrap();
                os::client::exec("pixel_wars", "choose_team", &bytes);
            }
            GameEvent::RestartGame() => {
                *state = GameState::default();
                //retain these values between rounds
            }
        }
    }
    if let Some(transition) = &mut state.transition {
        transition.update();
        transition.draw();
        if transition.complete {
            state.transition = None;
        }
    }
}

// pub fn zoom_cam_to_center_point(point: (f32, f32)) {
//     let (current_x, current_y, z) = cam!();
//     let dx = point.0 - current_x as f32;
//     let dy = point.1 - current_y as f32;
//     let distance = (dx * dx + dy * dy).sqrt();

//     if distance > 0.5 {
//         let new_x = current_x as f32 + dx.signum() * 0.5;
//         let new_y = current_y as f32 + dy.signum() * 0.5;
//         set_cam!(x = new_x, y = new_y);
//     } else {
//         set_cam!(x = point.0, y = point.1);
//     }
//     zoom_cam();
// }

// pub fn zoom_cam() {

//     set_cam!(z = z);
// }

pub fn reset_cam() {
    set_cam!(x = 192, y = 108, z = 1);
}

pub fn generate_team_db(
    data_store: &UnitDataStore,
    rng: &mut RNG,
    match_team: Option<&Team>,
    team_name: String,
    power_level: f32,
    round: u8, // Added round parameter
) -> Team {
    // Get available unit types based on round
    let mut available_types = get_available_units(round, data_store, Vec::new());

    // If matching a team, remove its unit types from available options
    if let Some(team) = match_team {
        available_types.retain(|unit_type| !team.units.contains(*unit_type));
    }

    let num_types = match round {
        1..=2 => 2,
        3..=5 => 3,
        6 => 4,
        _ => 2,
    };
    let selected_types = select_random_unit_types(&available_types, num_types, rng);

    // Calculate all unit powers
    let unit_powers: HashMap<UnitType, f32> = data_store
        .data
        .iter()
        .map(|(unit_type, unit_data)| (unit_type.clone(), calculate_single_unit_power(unit_data)))
        .collect();

    //UNIT POWER LEVELS

    // Calculate target power
    let target_power = match match_team {
        Some(team) => get_team_total_power(team),
        None => calculate_team_power_target(&unit_powers, power_level),
    };

    // Create and return the team
    let mut team = Team::new(team_name, data_store.clone());
    create_team(&mut team, &selected_types, &unit_powers, target_power, rng);
    team
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum DBPhase {
    Title,
    Shop,
    ArtifactShop,
    Battle,
    WrapUp,
    Sandbox,
    ParticleTest,
}

pub fn initialize_first_team(data_store: UnitDataStore) -> Team {
    Team::new("You".to_string(), data_store)
}

pub fn create_unit_packs(
    total_packs: u8,
    num_artifact_packs: u8,
    data_store: &UnitDataStore,
    rng: &mut RNG,
    round: u8,
    fallen_units: Option<Vec<UnitType>>,
    current_team_unit_types: Vec<UnitType>,
    team_upgrades: Vec<UnitType>,
    existing_artifacts: &Vec<Artifact>,
    upgraded_unit_pack: Option<(UnitType, UnitType)>,
) -> Vec<UnitPack> {
    let mut unitpacks = Vec::new();
    let pack_height = 50.0;
    let pack_width = 80.0;
    let pack_margin = 13.0;
    let start_x = pack_margin;
    let start_y = 50.0;

    // Track our current position
    let mut next_x = start_x;

    // 1. Add fallen units pack if needed
    if let Some(ref fallen_unit_types) = fallen_units {
        let pos = (next_x, start_y);
        let fallen_pack = UnitPack::new_fallen_units(fallen_unit_types.clone(), pos);
        unitpacks.push(fallen_pack);

        // Move to next position
        next_x += pack_width + pack_margin;
    }

    // 2. Add unit upgrade pack if available
    if let Some(unit_type) = upgraded_unit_pack {
        let pos = (next_x, start_y);
        let original_unit_data: &UnitData = data_store.get_unit_data(&unit_type.0).unwrap();
        let original_unit_preview =
            UnitPreview::new(unit_type.0.clone(), original_unit_data.clone(), pos, false);
        let new_unit_data: &UnitData = data_store.get_unit_data(&unit_type.1).unwrap();
        let new_unit_preview =
            UnitPreview::new(unit_type.1.clone(), new_unit_data.clone(), pos, false);
        //let unitpack = UnitPack::new_normal(unit_type.clone(), unit_preview, unit_count, pos);
        let upgrade_pack = UnitPack {
            //create the unit previews
            pack_type: UnitPackType::Transform {
                original_unit_type: unit_type.0,
                original_unit_preview: original_unit_preview,
                new_unit_type: unit_type.1,
                new_unit_preview: new_unit_preview,
                is_upgrade: true,
            },
            is_picked: false,
            pos,
            width: pack_width as u32,
            height: pack_height as u32,
        };
        unitpacks.push(upgrade_pack);

        // Move to next position
        next_x += pack_width + pack_margin;
    }

    // 3. Filter artifact kinds to only include those not already owned
    let available_kinds: Vec<ArtifactKind> = ARTIFACT_KINDS
        .iter()
        .filter(|kind| {
            !existing_artifacts
                .iter()
                .any(|artifact| artifact.artifact_kind == **kind)
        })
        .copied()
        .collect();

    // Only add artifact packs if we have available kinds
    let actual_artifact_packs = if available_kinds.is_empty() {
        0
    } else {
        num_artifact_packs.min(available_kinds.len() as u8)
    };

    // Add artifact packs
    for _ in 0..actual_artifact_packs {
        let index = rng.next_in_range(0, available_kinds.len() as u32 - 1);
        let artifact_kind = available_kinds[index as usize].clone();

        let pack = UnitPack {
            pack_type: UnitPackType::Artifact {
                kind: artifact_kind,
            },
            is_picked: false,
            pos: (next_x, start_y),
            width: pack_width as u32,
            height: pack_height as u32,
        };

        unitpacks.push(pack);

        // Move to next position
        next_x += pack_width + pack_margin;
    }

    // 4. Calculate remaining unit packs
    let fallen_packs = if fallen_units.is_some() { 1 } else { 0 };
    let upgrade_packs = if upgraded_unit_pack.is_some() { 1 } else { 0 };
    let num_normal_packs = total_packs - fallen_packs - actual_artifact_packs - upgrade_packs;

    // 5. Add normal unit packs
    let available_types = get_available_units(round, data_store, current_team_unit_types);
    let mut types = select_random_unit_types(&available_types, num_normal_packs as usize, rng);
    //types[0] = UnitType::Yeti;
    // Check if any types can be upgraded
    // Check if any types can be upgraded
    for unit_type in types.iter_mut() {
        // Check if team has the upgrade for this unit type
        if team_upgrades.contains(unit_type) {
            // Replace with upgraded version
            match *unit_type {
                UnitType::Yeti => *unit_type = UnitType::BloodYeti,
                UnitType::Tanker => *unit_type = UnitType::GoldenTank,
                // Add more upgrade mappings here as needed
                _ => {} // No upgrade for this unit type
            }
        }
    }
    for unit_type in types.iter() {
        let pos = (next_x, start_y);

        let data: &UnitData = data_store.get_unit_data(unit_type).unwrap();
        let unit_count = get_unit_count(round, *unit_type);
        let unit_preview = UnitPreview::new(unit_type.clone(), data.clone(), pos, false);
        let unitpack = UnitPack::new_normal(unit_type.clone(), unit_preview, unit_count, pos);

        unitpacks.push(unitpack);

        // Move to next position
        next_x += pack_width + pack_margin;
    }

    unitpacks
}

pub fn get_available_units(
    round: u8,
    data_store: &UnitDataStore,
    current_team_unit_types: Vec<UnitType>,
) -> Vec<&UnitType> {
    let all_types: Vec<&UnitType> = data_store
        .data
        .keys()
        .filter(|unit_type| {
            // Get the unit data for this type
            let unit_data = data_store.data.get(*unit_type).unwrap();

            // Check if it does NOT have the Evolved attribute
            !unit_data.attributes.contains(&Attribute::Evolved)
        })
        .collect();
    let basic_units = vec![
        UnitType::Axeman,
        UnitType::Serpent,
        UnitType::Blade,
        UnitType::Hunter,
        UnitType::Pyro,
        UnitType::Bigpound,
        UnitType::Deathray,
        UnitType::Cosmo,
        UnitType::Zombie,
        UnitType::Shield,
    ];

    let advanced_units = vec![
        UnitType::Sabre,
        UnitType::Flameboi,
        UnitType::Bazooka,
        UnitType::Draco,
        UnitType::Saucer,
        UnitType::Tanker,
        UnitType::Catapult,
        UnitType::Darkknight,
        UnitType::Yeti,
        UnitType::Igor,
    ];

    // If the team already has 6 or more unique unit types,
    // only offer those types
    if current_team_unit_types.len() >= 6 {
        return all_types
            .iter()
            .filter(|&&unit| current_team_unit_types.contains(unit))
            .copied()
            .collect();
    }

    match round {
        1..=2 => all_types
            .iter()
            .filter(|&&unit| basic_units.contains(unit))
            .copied()
            .collect(),
        3..=6 => all_types.iter().copied().collect(),
        _ => all_types
            .iter()
            .filter(|&&unit| basic_units.contains(unit))
            .copied()
            .collect(),
    }
}

fn get_unit_count(round: u8, unit_type: UnitType) -> u32 {
    let rating = get_unit_strength_rating(unit_type);
    //TODO: add a little randomness here
    match round {
        //only basic units tier 1-3 in rounds 1 and 2
        1..=2 => match rating {
            1 => 8,
            2 => 6,
            3 => 4,
            4 => 2,
            _ => 1,
        },
        3..=4 => match rating {
            1 => 15,
            2 => 10,
            3 => 8,
            4 => 4,
            5 => 3,
            6 => 2,
            7 => 1,
            _ => 1,
        },
        5..=8 => match rating {
            1 => 25,
            2 => 18,
            3 => 14,
            4 => 8,
            5 => 6,
            6 => 4,
            7 => 2,
            _ => 1,
        },
        _ => 1,
    }
}

fn get_unit_strength_rating(unit_type: UnitType) -> u8 {
    match UNIT_RATINGS.iter().find(|(unit, _)| *unit == unit_type) {
        Some((_, rating)) => *rating,
        None => 5, // default rating
    }
}

pub fn get_power_level_for_round(round: u8) -> f32 {
    match round {
        1 => 3.,
        2 => 5.,
        3 => 8.,
        4 => 10.,
        5 => 13.,
        6 => 18.,
        7 => 23.,
        8 => 29.,
        _ => round as f32 * 7.0, // Default case, though shouldn't happen in 6-round game
    }
}

pub fn create_artifact_shop(
    num: usize,
    rng: &mut RNG,
    existing_artifacts: &Vec<Artifact>,
) -> Vec<Artifact> {
    // Get all possible artifact kinds and filter out existing ones
    let available_kinds: Vec<ArtifactKind> = ARTIFACT_KINDS
        .iter()
        .filter(|kind| {
            !existing_artifacts
                .iter()
                .any(|artifact| artifact.artifact_kind == **kind)
        })
        .copied()
        .collect();

    // Convert to slice of references
    let available_kinds_refs: Vec<&ArtifactKind> = available_kinds.iter().collect();

    // Determine how many artifacts to generate
    let num_types = std::cmp::min(num, available_kinds.len());

    if num_types == 0 {
        return Vec::new();
    }

    // Select random kinds and create artifacts
    select_random_artifact_kinds(&available_kinds_refs, num_types, rng)
        .into_iter()
        .map(|kind| Artifact::new(kind, 0))
        .collect()
}

pub fn select_random_artifact_kinds(
    available_kinds: &[&ArtifactKind],
    num_kinds: usize,
    rng: &mut RNG,
) -> Vec<ArtifactKind> {
    // Returning owned Strings
    let mut selected_kinds = Vec::new();
    let mut remaining_attempts = 1000;

    while selected_kinds.len() < num_kinds && remaining_attempts > 0 {
        let index = rng.next_in_range(0, available_kinds.len() as u32 - 1) as usize;
        let artifact_kind = available_kinds[index].clone(); // Clone to get owned String

        if !selected_kinds.contains(&artifact_kind) {
            selected_kinds.push(artifact_kind);
        }

        remaining_attempts -= 1;
    }

    selected_kinds
}

pub fn choose_artifacts_for_enemy_team(num_kinds: usize, rng: &mut RNG) -> Vec<ArtifactKind> {
    // Convert ARTIFACT_KINDS to slice of references
    let available_kinds = ARTIFACT_KINDS.iter().collect::<Vec<_>>();

    // Use the existing select_random_artifact_kinds function
    select_random_artifact_kinds(&available_kinds, num_kinds, rng)
}

pub fn split_text_at_spaces(text: &str) -> Vec<String> {
    let target_length = 11;
    let mut result = Vec::new();
    let mut remaining = text;

    while !remaining.is_empty() {
        if remaining.len() <= target_length {
            result.push(remaining.to_string());
            break;
        }

        // Look at the substring up to target_length + 5 to find closest space
        let search_range = std::cmp::min(remaining.len(), target_length + 5);
        let substring = &remaining[..search_range];

        // Find the last space in our search range
        let split_index = match substring.rfind(' ') {
            Some(index) => index + 1, // Split after the space
            None => {
                // If no space found, force split at target_length
                target_length
            }
        };

        result.push(remaining[..split_index].trim().to_string());
        remaining = &remaining[split_index..];
    }

    result
}

pub fn draw_current_team(team: &Team, data_store: &UnitDataStore, facing_left: bool) {
    //Draw header
    if facing_left {
        power_text!("ENEMY TEAM", x = 320, y = 140, underline = true,);
    } else {
        power_text!("YOUR TEAM", x = 10, y = 140, underline = true);
    }

    // Create a vec to store (unit_type, count)
    let mut type_counts: Vec<(&UnitType, u32)> = Vec::new();

    // Count occurrences of each unit type while maintaining order
    for unit_type in &team.units {
        // Check if we already have this type
        if let Some(entry) = type_counts.iter_mut().find(|(t, _)| *t == unit_type) {
            entry.1 += 1;
        } else {
            type_counts.push((unit_type, 1));
        }
    }

    // Sort by unit type to ensure consistent order
    type_counts.sort_by(|a, b| a.0.cmp(b.0));

    // Calculate positions and draw
    let start_x = if facing_left { 320 } else { 10 };
    let start_y = 160;
    let vertical_spacing = 20;
    let horizontal_spacing = 40;
    let max_rows = 3;

    // Draw each unit type count
    for (i, (unit_type, count)) in type_counts.iter().enumerate() {
        // Calculate position
        let row = i % max_rows;
        let column = i / max_rows;

        let x = if facing_left {
            start_x - (column * horizontal_spacing)
        } else {
            start_x + (column * horizontal_spacing)
        };
        let y = start_y + (row * vertical_spacing);

        // Draw count
        let txt = format!("{}x ", count);
        text!(txt.as_str(), x = x, y = y);

        // Draw sprite
        let unit_name = unit_type.as_string();
        let txt = format!("{}_idle", unit_name);
        let data = data_store.data.get(unit_type);
        let x_adj = data.unwrap().bounding_box.0;
        let y_adj = data.unwrap().bounding_box.1;
        let sw = data.unwrap().sprite_width;
        sprite!(
            &txt,
            x = x - x_adj as usize + 16,
            y = y - y_adj as usize,
            sw = sw,
            flip_x = facing_left,
        );
    }
}
