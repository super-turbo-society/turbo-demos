use crate::*;

pub fn dbgo(state: &mut GameState) {
    clear!(0x8f8cacff);
    let gp = gamepad(0);

    match state.dbphase {
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
            if state.shop.len() == 0 {
                state.shop = create_unit_packs(4, &ds, &mut state.rng);
                if state.round == 0 {
                    state.num_picks = 3;
                } else {
                    state.num_picks = 1;
                }
            }
            let m = mouse(0);
            let m_pos = (m.position[0], m.position[1]);
            for (i, u) in state.shop.iter_mut().enumerate() {
                u.draw(m_pos);
                if m.left.just_pressed() && u.is_hovered(m_pos) {
                    select_unit_pack(i, state);
                    state.num_picks -= 1;
                    break;
                }
            }
            let txt = format!("Choose {} Unit Sets", state.num_picks);
            text!(&txt, x = 20, y = 20, font = Font::L);
            if state.teams.len() != 0 {
                draw_current_team(&state.teams[0], &state.data_store.as_ref().unwrap());
                //let txt = format!("Your Team: {:?}", state.teams[0].units);
                //text!(&txt, x = 10, y = 180);
            }
            //do something to start the battle
            //TODO: Turn this into a button
            if gp.a.just_pressed() && state.num_picks == 0 {
                //todo: figure out how to generate a fair team
                let t = generate_team_db(
                    &state.data_store.as_ref().unwrap(),
                    &mut state.rng,
                    Some(&state.teams[0]),
                    "Bad Bois".to_string(),
                    (20 * (state.round + 1)) as f32,
                );
                state.teams.push(t);
                state.units = create_units_for_all_teams(
                    &mut state.teams,
                    &mut state.rng,
                    &state.data_store.as_ref().unwrap(),
                );
                state.dbphase = DBPhase::ArtifactShop;
            }
        }
        DBPhase::ArtifactShop => {
            let m = mouse(0);
            let m_pos = (m.position[0], m.position[1]);
            //generate 2 choices
            if state.artifact_shop.len() == 0 {
                state.artifact_shop = create_artifact_shop(2, &mut state.rng, &state.artifacts);
            }
            for (i, a) in state.artifact_shop.iter_mut().enumerate() {
                let pos = (100 + (i as i32 * 100), 72);
                a.draw_card(pos, m_pos);
                if m.left.just_pressed() && a.is_hovered(pos, m_pos) {
                    state.artifacts.push(a.clone());

                    state.dbphase = DBPhase::Battle;
                }
            }
            let txt = format!("Choose An Artifact");
            text!(&txt, x = 20, y = 20, font = Font::L);
            if state.teams.len() != 0 {
                //TODO: Turn this into a function
                //let txt = format!("Your Team: {:?}", state.teams[0].units);
                text!(&txt, x = 10, y = 180);
            }
            if gp.a.just_pressed() || state.round != 0 {
                state.dbphase = DBPhase::Battle;
            }
        }
        DBPhase::Battle => {
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
                for u in &mut state.units {
                    u.update();
                }
                state.battle_countdown_timer -= 1;

                //show text
                draw_prematch_timer(state.battle_countdown_timer);
            } else {
                step_through_battle(
                    &mut state.units,
                    &mut state.attacks,
                    &mut state.traps,
                    &mut state.explosions,
                    &mut state.craters,
                    &mut state.rng,
                    &state.artifacts,
                );
            }

            /////////////
            //Draw Code//
            /////////////

            //Draw craters beneath everything
            for c in &state.craters {
                c.draw();
            }
            //sprite!("crater_01", x=100, y=100, color = 0xFFFFFF80);
            //Draw footprints beneath units
            for u in &mut state.units {
                for fp in &mut u.display.as_mut().unwrap().footprints {
                    fp.draw();
                    //format!()
                }
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
            if let Some(winner_idx) = has_some_team_won(&state.units) {
                if winner_idx == 0 {
                    //then you win
                    draw_end_animation(Some(true));
                    //need to do some type of
                } else {
                    //then you lose
                    draw_end_animation(Some(false));
                }
                if gp.start.just_pressed() {
                    //go to next round
                    let your_team = state.teams[0].clone();
                    let artifacts = state.artifacts.clone();
                    let r = state.round + 1;
                    *state = GameState::default();
                    state.teams.push(your_team);
                    state.round = r;
                    state.artifacts = artifacts;
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
}

pub fn generate_team_db(
    data_store: &UnitDataStore,
    rng: &mut RNG,
    match_team: Option<&Team>,
    team_name: String,
    power_level: f32,
) -> Team {
    // Get available unit types as Vec<&String>
    let mut available_types: Vec<&String> = data_store.data.keys().collect();

    // If matching a team, remove its unit types from available options
    if let Some(team) = match_team {
        available_types.retain(|unit_type| !team.units.contains(*unit_type));
    }

    // Select 2 random unit types for this team
    let selected_types = select_random_unit_types(&available_types, 2, rng);

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
    Shop,
    ArtifactShop,
    Battle,
    WrapUp,
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
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, EnumIter)]

pub enum ArtifactKind {
    StrenghtOfTheFallen,
    SnipersFocus,
    FlameWard,
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
                String::from("Increase damage by 1% for each dead unit on your team"),
            ),
            ArtifactKind::SnipersFocus => (
                ArtifactConfig::DistanceDamageBoost {
                    percent_per_pixel: 0.5,
                },
                String::from("Increase damage for all ranged attacks by .5% per pixel away"),
            ),
            ArtifactKind::FlameWard => (
                ArtifactConfig::FireResistance {
                    resistance_percent: 50.0,
                },
                String::from("Give all of your units 50% fire resistance"),
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
        let mut color = ACID_GREEN;
        match self.artifact_kind {
            ArtifactKind::StrenghtOfTheFallen => {
                color = ACID_GREEN;
            }
            ArtifactKind::SnipersFocus => {
                color = OFF_BLACK as usize;
            }
            ArtifactKind::FlameWard => {
                color = DAMAGE_TINT_RED;
            }
        }
        //match on the artifact type to get the sprite
        circ!(color = color, x = pos.0, y = pos.1, d = 12);
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
) -> Vec<UnitPack> {
    //choose some number of packs to make
    //add them to the game state
    let mut unitpacks = Vec::new();
    let available_types: Vec<&String> = data_store.data.keys().collect();
    let types = select_random_unit_types(&available_types, num_types, rng);
    let mut i = 0;
    while i < num_types {
        //create a pack
        //TODO: update this with a unit pack new function
        //and make a unit preview based on the type
        //we already have the data store so it shouldn't be too hard
        let unit_type = types[i].clone();
        let pos = ((i * 90 + 20) as f32, 72.);
        let data = data_store.get_unit_data(&unit_type).unwrap();
        let unit_power = calculate_single_unit_power(data);
        let unit_count = 2500 as f32 / unit_power;
        let unit_preview = UnitPreview::new(unit_type, data.clone(), pos, false);
        let unitpack = UnitPack::new(types[i].clone(), unit_count as u32, unit_preview, pos);
        unitpacks.push(unitpack);
        i += 1;
    }

    unitpacks
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
    let mut remaining_attempts = 100;

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

pub fn draw_current_team(team: &Team, data_store: &UnitDataStore) {
    //Draw header
    text!("YOUR TEAM: ", x = 10, y = 140);

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
    let start_x = 10;
    let start_y = 160;
    let vertical_spacing = 20;
    let horizontal_spacing = 40;
    let max_rows = 3;

    // Draw each unit type count
    for (i, (unit_type, count)) in type_counts.iter().enumerate() {
        // Calculate position
        let row = i % max_rows;
        let column = i / max_rows;

        let x = start_x + (column * horizontal_spacing);
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
            x = x + x_adj as usize + 12,
            y = y - y_adj as usize,
            sw = sw,
        );
    }
}
