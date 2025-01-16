use crate::*;

const TOTAL_ROUNDS: usize = 6;
const UNIT_RATINGS: [(&str, u8); 18] = [
    // Basic units
    ("axeman", 4),
    ("blade", 2),
    ("hunter", 2),
    ("pyro", 1),
    ("bigpound", 2),
    ("deathray", 3),
    ("cosmo", 3),
    ("zombie", 1),
    ("shield", 3),
    // Advanced units
    ("sabre", 4),
    ("flameboi", 5),
    ("bazooka", 5),
    ("draco", 7),
    ("saucer", 6),
    ("tanker", 7),
    ("catapult", 5),
    ("darkknight", 7),
    ("yeti", 5),
];

pub fn title_screen_unit(rng: &mut RNG, data_store: &UnitDataStore) -> WalkingUnitPreview {
    let is_left_side = (rng.next() % 2) == 0;
    let x_pos = -64.0;
    let y_pos = rng.next_in_range(0, 160) as f32;
    let pos = (x_pos, y_pos);
    let available_types: Vec<&String> = data_store.data.keys().collect();
    let index = rng.next_in_range(0, available_types.len() as u32 - 1) as usize;
    let unit_type = available_types[index].clone();
    let data = data_store.get_unit_data(&unit_type).unwrap();
    let s_w = data.sprite_width;
    let speed = data.speed;
    let mut a = AnimatedSprite::new(pos, true);
    let anim_name = unit_type.to_string() + "_walk";
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
    let tile_size: usize = 2;
    let base_color: u32 = 0x8f8cacff;
    let variation = 8; // Reduced variation for subtlety

    for row in 0..(216 / tile_size) {
        for col in 0..(384 / tile_size) {
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
    }
    let mut ready_to_transition = false;
    if let Some(transition) = &mut state.transition {
        if transition.ready_for_scene_change {
            ready_to_transition = true;
            transition.start_transition_out(&mut state.rng);
        }
    }

    match state.dbphase {
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
                if state.round == 7 {
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
                let t = generate_team_db(
                    &state.data_store.as_ref().unwrap(),
                    &mut state.rng,
                    None,
                    "Bad Bois".to_string(),
                    get_power_level_for_round(state.round),
                    state.round,
                );
                state.enemy_team_placeholder = Some(t);
            }

            if state.shop.len() == 0 {
                state.shop = create_unit_packs(4, &ds, &mut state.rng, state.round);
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
                        if state.round == 2 || state.round == 4 {
                            state.dbphase = DBPhase::ArtifactShop;
                        } else {
                            if state.transition.is_none() {
                                state.transition = Some(Transition::new(&mut state.rng));
                            }
                        }
                    }
                    break;
                }
            }
            let txt = format!("Choose {} Unit Sets", state.num_picks);
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
                    state.dbphase = DBPhase::ArtifactShop;
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
            let mut should_end = false;
            if gp.a.just_pressed() || (state.round != 2 && state.round != 4) {
                should_end = true;
                ready_to_transition = true;
            }

            let m = mouse(0);
            let m_pos = (m.position[0], m.position[1]);
            //generate 2 choices
            if state.artifact_shop.len() == 0 {
                state.artifact_shop = create_artifact_shop(2, &mut state.rng, &state.artifacts);
            }
            for (i, a) in state.artifact_shop.iter_mut().enumerate() {
                let pos = (100 + (i as i32 * 100), 50);
                if m.left.just_pressed() && a.is_hovered(pos, m_pos) {
                    state.artifacts.push(a.clone());
                    if state.transition.is_none() {
                        state.transition = Some(Transition::new(&mut state.rng));
                    }
                } else {
                    if !should_end {
                        a.draw_card(pos, m_pos);
                    }
                }
            }
            if !should_end {
                let txt = format!("Choose An Artifact");
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
            }
            if ready_to_transition {
                // Handle FlameWard effects
                if state
                    .artifacts
                    .iter()
                    .any(|a| a.artifact_kind == ArtifactKind::FlameWard)
                {
                    for unit in &mut state.units {
                        if unit.team == 0
                            && !unit.data.attributes.contains(&Attribute::FireResistance)
                        {
                            unit.data.attributes.push(Attribute::FireResistance);
                        }
                    }
                }

                // Handle TrapArtist effects
                if state
                    .artifacts
                    .iter()
                    .any(|a| a.artifact_kind == ArtifactKind::TrapArtist)
                {
                    let trap_artifact = state
                        .artifacts
                        .iter()
                        .find(|a| a.artifact_kind == ArtifactKind::TrapArtist)
                        .unwrap();

                    if let ArtifactConfig::TrapBoard { num_traps } = trap_artifact.config {
                        let mut i = 0;
                        while i < num_traps {
                            let t = create_trap(&mut state.rng);
                            state.traps.push(t);
                            i += 1;
                        }
                    }
                }
                if state
                    .artifacts
                    .iter()
                    .any(|a| a.artifact_kind == ArtifactKind::ShotOutACannon)
                {
                    for unit in &mut state.units {
                        if unit.team == 0 {
                            unit.start_haste();
                        }
                    }
                }
                state.dbphase = DBPhase::Battle;
            }
        }
        DBPhase::Sandbox => {
            draw_team_info_and_buttons(state);
            if gp.start.just_pressed() {
                state.units = create_units_for_all_teams(
                    &mut state.teams,
                    &mut state.rng,
                    &state.data_store.as_ref().unwrap(),
                );
                state.dbphase = DBPhase::Battle;
            }
            if gp.down.pressed() {
                set_cam!(y = cam!().1 + 3);
            } else if gp.up.pressed() {
                set_cam!(y = cam!().1 - 3);
            }
        }
        DBPhase::Battle => {
            set_cam!(x = 192, y = 108);
            if state.battle_countdown_timer > 0 {
                if state.battle_countdown_timer == BATTLE_COUNTDOWN_TIME {
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
                    );
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
                    //format!()
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

            //Draw round indicator
            let txt = format!("Round {}/{}", state.round, TOTAL_ROUNDS);
            let len = txt.len();
            let char_length = 8;
            let center_x = canvas_size!()[0] / 2;
            let text_width = len as u32 * char_length;
            let x = center_x - (text_width / 2);
            text!(&txt, x = x, y = 10, font = Font::L, color = OFF_BLACK);

            //Draw team health bars
            let mut team0_base_health = 0.0;
            let mut team0_current_health = 0.0;
            let mut team1_base_health = 0.0;
            let mut team1_current_health = 0.0;

            for unit in &state.units {
                if unit.team == 0 {
                    team0_base_health += unit.data.max_health as f32;
                    team0_current_health += unit.health as f32;
                } else {
                    team1_base_health += unit.data.max_health as f32;
                    team1_current_health += unit.health as f32;
                }
            }
            let mut is_chosen_team = false;
            if state.selected_team_index == Some(0) {
                is_chosen_team = true;
            }
            let (team_0_pos, team_1_pos) = ((24.0, 20.0), (232.0, 20.0));
            // Draw health bar for team 0
            draw_team_health_bar(
                team0_base_health,
                team0_current_health,
                team_0_pos,
                &state.teams[0].name.to_uppercase(),
                true,
                is_chosen_team,
            );
            //draw team0 artifacts start from left side of the bar
            for (i, a) in state.artifacts.clone().iter().enumerate() {
                let x_offset = i as i32 * 16; // 16 pixels right for each item
                let pos = (team_0_pos.0 as i32 + x_offset, team_0_pos.1 as i32 + 14);
                a.draw_sprite_scaled(pos, 0.5);
            }
            is_chosen_team = false;
            if state.selected_team_index == Some(1) {
                is_chosen_team = true;
            }
            // Draw health bar for team 1
            draw_team_health_bar(
                team1_base_health,
                team1_current_health,
                team_1_pos,
                &state.teams[1].name.to_uppercase(),
                false,
                is_chosen_team,
            );
            let mut text = "Click to Play Again";
            if let Some(winner_idx) = has_some_team_won(&state.units) {
                for u in &mut state.units {
                    u.start_cheering();
                }
                if winner_idx == 0 {
                    //then you win
                    draw_end_animation(Some(true));

                    if m.left.just_pressed() {
                        //go to next round
                        let mut your_team = state.teams[0].clone();

                        // Create a new Vec with just the living units' types
                        let living_unit_types: Vec<String> = state
                            .units
                            .iter()
                            .filter(|unit| unit.team == 0 && unit.health > 0.0)
                            .map(|unit| unit.unit_type.clone())
                            .collect();

                        // Replace your_team's units with just the living ones
                        your_team.units = living_unit_types;
                        let artifacts = state.artifacts.clone();
                        let r = state.round + 1;
                        *state = GameState::default();
                        state.teams.push(your_team);
                        state.round = r;
                        state.artifacts = artifacts;
                    }
                    text = "Click to Continue";
                } else {
                    //then you lose
                    draw_end_animation(Some(false));
                    if m.left.just_pressed() {
                        *state = GameState::default();
                    }
                }
                //show some text for what to do next
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

pub fn generate_team_db(
    data_store: &UnitDataStore,
    rng: &mut RNG,
    match_team: Option<&Team>,
    team_name: String,
    power_level: f32,
    round: u32, // Added round parameter
) -> Team {
    // Get available unit types based on round
    let mut available_types = get_available_units(round, data_store);

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
    let unit_powers: HashMap<String, f32> = data_store
        .data
        .iter()
        .map(|(unit_type, unit_data)| (unit_type.clone(), calculate_single_unit_power(unit_data)))
        .collect();

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
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct UnitPack {
    pub unit_type: String,
    pub unit_count: u32,
    pub unit_preview: UnitPreview,
    pub is_picked: bool,
    pub pos: (f32, f32),
    pub width: u32,
    pub height: u32,
}

impl UnitPack {
    pub fn new(
        unit_type: String,
        unit_count: u32,
        unit_preview: UnitPreview,
        pos: (f32, f32),
    ) -> Self {
        UnitPack {
            unit_type,
            unit_count,
            unit_preview,
            is_picked: false, // Default value
            pos,
            width: 80,  // Default value
            height: 80, // Default value
        }
    }

    pub fn capitalize(s: &str) -> String {
        let mut c = s.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }

    pub fn is_hovered(&self, mouse_pos: (i32, i32)) -> bool {
        let (mouse_x, mouse_y) = mouse_pos;
        let (pack_x, pack_y) = self.pos;

        mouse_x >= pack_x as i32
            && mouse_x <= pack_x as i32 + self.width as i32
            && mouse_y >= pack_y as i32
            && mouse_y <= pack_y as i32 + self.height as i32
    }

    pub fn on_picked(&mut self) {
        //do something
    }

    pub fn draw_pack_card(&self, mouse_pos: (i32, i32)) {
        //create a panel
        let pw = 80; // Made panel wider to accommodate text
        let ph = 80;
        let border_color = OFF_BLACK;
        let mut panel_color = DARK_GRAY;
        if self.is_hovered(mouse_pos) {
            panel_color = LIGHT_GRAY;
        }
        let px = self.pos.0;
        let py = self.pos.1;
        rect!(
            x = px,
            y = py,
            h = ph,
            w = pw,
            color = panel_color,
            border_color = border_color,
            border_radius = 6,
            border_width = 2
        );

        // Header
        let c = Self::capitalize(&self.unit_type);
        let txt = format!("{} {}s", self.unit_count, c);
        text!(&txt, x = px + 5., y = py + 5.);

        // Stats rows - each line is 15 pixels apart
        let damage_text = format!("DAMAGE: {}", self.unit_preview.data.damage);
        let speed_text = format!("SPEED: {}", self.unit_preview.data.speed);
        let health_text = format!("HEALTH: {}", self.unit_preview.data.max_health);

        text!(&damage_text, x = px + 5., y = py + 25.);
        text!(&speed_text, x = px + 5., y = py + 35.);
        text!(&health_text, x = px + 5., y = py + 45.);
    }

    pub fn draw(&mut self, mouse_pos: (i32, i32)) {
        if !self.is_picked {
            self.unit_preview.pos.0 = self.pos.0 + 30.;
            self.unit_preview.pos.1 = self.pos.1 + self.height as f32 - 10.;
            self.unit_preview.update();
            self.draw_pack_card(mouse_pos);
            self.unit_preview.draw();
        }
    }
    //draw unit preview
}

pub fn select_unit_pack(pack_index: usize, state: &mut GameState) {
    //get team 0
    let pack = &mut state.shop[pack_index];
    if state.teams.len() == 0 {
        let team = initialize_first_team(state.data_store.as_ref().unwrap().clone());
        state.teams.push(team);
    }
    //add the units to team 0
    let num = pack.unit_count;
    let mut i = 0;
    while i < num {
        state.teams[0].add_unit(pack.unit_type.clone());
        i += 1;
    }
    pack.is_picked = true;
}

pub fn initialize_first_team(data_store: UnitDataStore) -> Team {
    Team {
        name: ("YOU".to_string()),
        units: (Vec::new()),
        data: (data_store),
        win_streak: (0),
    }
}

//artifacts
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum ArtifactConfig {
    DeadUnitDamageBoost { percent_per_unit: f32 },
    DistanceDamageBoost { percent_per_pixel: f32 },
    FireResistance { resistance_percent: f32 },
    TrapBoard { num_traps: u8 },
    TeamHaste,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, EnumIter)]

pub enum ArtifactKind {
    StrenghtOfTheFallen,
    SnipersFocus,
    FlameWard,
    TrapArtist,
    ShotOutACannon,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct Artifact {
    pub artifact_kind: ArtifactKind,
    pub config: ArtifactConfig,
    pub text: String,
}

impl Artifact {
    pub fn new(kind: ArtifactKind) -> Self {
        // Match the name to get the preconfigured artifact
        let (config, text) = match kind {
            ArtifactKind::StrenghtOfTheFallen => (
                ArtifactConfig::DeadUnitDamageBoost {
                    percent_per_unit: 1.0,
                },
                String::from("Increase damage for each dead unit on your team"),
            ),
            ArtifactKind::SnipersFocus => (
                ArtifactConfig::DistanceDamageBoost {
                    percent_per_pixel: 0.5,
                },
                String::from("Increase damage for all ranged attacks"),
            ),
            ArtifactKind::FlameWard => (
                ArtifactConfig::FireResistance {
                    resistance_percent: 50.0,
                },
                String::from("Give all of your units fire resistance"),
            ),
            ArtifactKind::TrapArtist => (
                ArtifactConfig::TrapBoard { num_traps: 12 },
                String::from("Add Traps to the Board"),
            ),
            ArtifactKind::ShotOutACannon => (
                ArtifactConfig::TeamHaste,
                String::from("All your units start with Haste"),
            ),
            _ => panic!("Unknown artifact kind"),
        };

        Self {
            artifact_kind: kind,
            config,
            text,
        }
    }

    pub fn is_hovered(&self, pos: (i32, i32), mouse_pos: (i32, i32)) -> bool {
        let (mouse_x, mouse_y) = mouse_pos;
        let (pack_x, pack_y) = pos;
        let width = 80;
        let height = 80;
        mouse_x >= pack_x as i32
            && mouse_x <= pack_x as i32 + width as i32
            && mouse_y >= pack_y as i32
            && mouse_y <= pack_y as i32 + height as i32
    }

    pub fn draw_card(&self, pos: (i32, i32), mouse_pos: (i32, i32)) {
        //do some card stuff, with a position
        let pw = 80; // Made panel wider to accommodate text
        let ph = 80;
        let border_color = OFF_BLACK;
        let mut panel_color = DARK_GRAY;
        if self.is_hovered(pos, mouse_pos) {
            panel_color = LIGHT_GRAY;
        }
        let px = pos.0;
        let py = pos.1;
        rect!(
            x = px,
            y = py,
            h = ph,
            w = pw,
            color = panel_color,
            border_color = border_color,
            border_radius = 6,
            border_width = 2
        );
        let sprite_pos = (pos.0 + 32, pos.1 + 10);
        let text_pos = (pos.0 + 3, pos.1 + 36);
        self.draw_sprite(sprite_pos);
        self.draw_effect_text(text_pos);
    }

    pub fn draw_sprite(&self, pos: (i32, i32)) {
        self.draw_sprite_scaled(pos, 1.0);
    }

    pub fn draw_sprite_scaled(&self, pos: (i32, i32), scale: f32) {
        let mut color = ACID_GREEN;
        match self.artifact_kind {
            ArtifactKind::StrenghtOfTheFallen => {
                color = POO_BROWN;
            }
            ArtifactKind::SnipersFocus => {
                color = OFF_BLACK as usize;
            }
            ArtifactKind::FlameWard => {
                color = DAMAGE_TINT_RED;
            }
            ArtifactKind::TrapArtist => {
                color = WHITE;
            }
            ArtifactKind::ShotOutACannon => {
                color = ACID_GREEN;
            }
        }
        //match on the artifact type to get the sprite
        let d = 12.0 * scale;
        circ!(color = color, x = pos.0, y = pos.1, d = d);
    }

    pub fn draw_effect_text(&self, pos: (i32, i32)) {
        let texts = split_text_at_spaces(&self.text);

        for (i, line) in texts.iter().enumerate() {
            let y_offset = pos.1 + (i as i32 * 8);
            text!(line, x = pos.0, y = y_offset);
        }
    }
}

pub fn create_unit_packs(
    num_types: usize,
    data_store: &UnitDataStore,
    rng: &mut RNG,
    round: u32,
) -> Vec<UnitPack> {
    //choose some number of packs to make
    //add them to the game state
    let mut unitpacks = Vec::new();
    let available_types = get_available_units(round, data_store);
    let types = select_random_unit_types(&available_types, num_types, rng);
    let mut i = 0;
    while i < num_types {
        //create a pack
        //TODO: update this with a unit pack new function
        let unit_type = types[i].clone();
        let pos = ((i * 90 + 20) as f32, 50.);
        let data: &UnitData = data_store.get_unit_data(&unit_type).unwrap();
        let unit_count = get_unit_count(round, &unit_type);
        let unit_preview = UnitPreview::new(unit_type, data.clone(), pos, false);
        let unitpack = UnitPack::new(types[i].clone(), unit_count as u32, unit_preview, pos);
        unitpacks.push(unitpack);
        i += 1;
    }

    unitpacks
}

pub fn get_available_units(round: u32, data_store: &UnitDataStore) -> Vec<&String> {
    let all_types: Vec<&String> = data_store.data.keys().collect();

    let basic_units = vec![
        "axeman", "blade", "hunter", "pyro", "bigpound", "deathray", "cosmo", "zombie", "shield",
    ];
    let advanced_units = vec![
        "sabre",
        "flameboi",
        "bazooka",
        "draco",
        "saucer",
        "tanker",
        "catapult",
        "darkknight",
        "yeti",
    ];

    match round {
        1..=2 => all_types
            .iter()
            .filter(|&&unit| basic_units.contains(&unit.as_str()))
            .copied()
            .collect(),
        3..=6 => all_types.iter().copied().collect(),
        _ => all_types
            .iter()
            .filter(|&&unit| basic_units.contains(&unit.as_str()))
            .copied()
            .collect(),
    }
}

fn get_unit_count(round: u32, unit_type: &str) -> u32 {
    let rating = get_unit_strength_rating(unit_type);
    //TODO: add a little randomness here
    match round {
        //only basic units tier 1-3 in rounds 1 and 2
        1..=2 => match rating {
            1 => 12,
            2 => 9,
            3 => 6,
            4 => 4,
            _ => 1,
        },
        3..=4 => match rating {
            1 => 28,
            2 => 20,
            3 => 14,
            4 => 8,
            5 => 5,
            6 => 3,
            7 => 2,
            _ => 1,
        },
        5..=6 => match rating {
            1 => 55,
            2 => 38,
            3 => 28,
            4 => 16,
            5 => 11,
            6 => 5,
            7 => 4,
            _ => 1,
        },
        _ => 1,
    }
}

fn get_unit_strength_rating(unit_type: &str) -> u8 {
    match UNIT_RATINGS.iter().find(|(unit, _)| *unit == unit_type) {
        Some((_, rating)) => *rating,
        None => 5, // default rating
    }
}

pub fn get_power_level_for_round(round: u32) -> f32 {
    match round {
        1 => 4.0,
        2 => 7.0,
        3 => 11.0,
        4 => 16.0,
        5 => 26.0,
        6 => 50.0,
        _ => round as f32 * 10.0, // Default case, though shouldn't happen in 6-round game
    }
}

pub fn create_artifact_shop(
    num: usize,
    rng: &mut RNG,
    existing_artifacts: &Vec<Artifact>,
) -> Vec<Artifact> {
    // Get all possible artifact kinds and filter out existing ones
    let available_kinds: Vec<ArtifactKind> = ArtifactKind::iter()
        .filter(|kind| {
            !existing_artifacts
                .iter()
                .any(|artifact| artifact.artifact_kind == *kind)
        })
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
        .map(|kind| Artifact::new(kind))
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
    let mut type_counts: Vec<(&String, u32)> = Vec::new();

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
        let txt = format!("{}_idle", unit_type);
        let data = data_store.data.get(&**unit_type);
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
