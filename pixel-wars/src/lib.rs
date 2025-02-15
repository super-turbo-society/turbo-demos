mod artifact;
mod attribute;
mod backend;
mod colors;
mod deckbuilder;
mod particles;
mod rng;
mod trap;
mod unit;

use artifact::*;
use attribute::*;
use backend::*;
use colors::*;
use csv::{Reader, ReaderBuilder};
use deckbuilder::*;
use os::server;
use particles::*;
use rng::*;
use std::cmp::{max, Ordering};
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::f32::consts::E;
use std::fmt::{format, Display};
use std::str::FromStr;
use strum::IntoEnumIterator;
use strum_macros::Display;
use strum_macros::EnumIter;
use trap::*;
use unit::*;

const UNIT_DATA_CSV: &[u8] = include_bytes!("../resources/unit-data.csv");
const DAMAGE_EFFECT_TIME: u32 = 12;
//avg number of units to balance each generated team around
const TEAM_POWER_MULTIPLIER: f32 = 25.0;
const TEAM_SELECTION_TIME: u32 = 3600;
const BATTLE_COUNTDOWN_TIME: u32 = 1;
const TEAM_NAMES: [&str; 12] = [
    "Pixel Peeps",
    "Battle Bois",
    "Mighty Militia",
    "Combat Crew",
    "Warrior Wreckers",
    "Fighting Fellas",
    "Savage Squad",
    "Rowdy Raiders",
    "Brawling Bunch",
    "Tragic Troops",
    "Danger Dudes",
    "Rumble Rookies",
];

const UNIT_ANIM_SPEED: u8 = 8;
//This is global unit speed (how many frames it takes to go one pixel/unit speed). Lower is faster
const MOVEMENT_DIVISOR: f32 = 16.0;

//TODO: Figure out if we want to use this again
const MAX_Y_ATTACK_DISTANCE: f32 = 10.;
const FOOTPRINT_LIFETIME: u32 = 240;
const MAP_BOUNDS: (f32, f32, f32, f32) = (10.0, 340.0, 0.0, 200.0);

//TODO: Add back turbo OS later
turbo::cfg! {r#"
    name = "Pixel Wars"
    version = "1.0.0"
    author = "Turbo"
    description = "Epic Fantasy Battles of All Time"
    [settings]
    resolution = [384, 216]
"#}

turbo::init! {
    struct GameState {
        transition: Option<Transition>,
        dbphase: DBPhase,
        title_screen_units: Vec<WalkingUnitPreview>,
        shop: Vec<UnitPack>,
        round: u8,
        num_picks: u8,
        artifacts: Vec<Artifact>,
        artifact_shop: Vec<Artifact>,
        enemy_team_placeholder: Option<Team>,
        phase: Phase,
        units: Vec<Unit>,
        next_id: u32,
        teams: Vec<Team>,
        unit_previews: Vec<UnitPreview>,
        attacks: Vec<Attack>,
        event_queue: Vec<GameEvent>,
        rng: RNG,
        data_store: Option<UnitDataStore>,
        traps: Vec<Trap>,
        explosions: Vec<AnimatedSprite>,
        craters: Vec<AnimatedSprite>,
        game_over_anim: AnimatedSprite,
        selected_team_index: Option<i32>,
        simulation_result: Option<SimulationResult>,
        elapsed_frames: u32,
        zoom_tween_x: Tween<f32>,
        zoom_tween_y: Tween<f32>,
        zoom_tween_z: Tween<f32>,
        auto_assign_teams: bool,
        user: UserStats,
        last_winning_team: Option<Team>,
        last_round_dead_units: Vec<String>,
        team_selection_timer: u32,
        team_generation_requested: bool,
        previous_battle: Option<Battle>,
        battle_countdown_timer: u32,
        battle_simulation_requested: bool,
        is_playing_sandbox_game: bool,
        is_battle_complete: bool,
        particle_manager: ParticleManager,
    } = {
        Self {
            transition: None,
            dbphase: DBPhase::Title,
            title_screen_units: Vec::new(),
            phase: Phase::TeamSetUp,
            round: 1,
            num_picks: 0,
            units: Vec::new(),
            shop: Vec::new(),
            artifacts: Vec::new(),
            artifact_shop: Vec::new(),
            enemy_team_placeholder: None,
            //this starts at 1 so if any unit has 0 id it is unassigned or a bug.
            next_id: 1,
            teams: Vec::new(),
            attacks: Vec::new(),
            event_queue: Vec::new(),
            traps: Vec::new(),
            unit_previews: Vec::new(),
            explosions: Vec::new(),
            craters: Vec::new(),
            game_over_anim: AnimatedSprite::new((0.,100.), false),
            //replace this number with a TURBO OS number later
            rng: RNG::new(12345),
            data_store: None,
            auto_assign_teams: true,
            selected_team_index: None,
            simulation_result: None,
            elapsed_frames: 0,
            zoom_tween_x: Tween::new(0.0),
            zoom_tween_y: Tween::new(0.0),
            zoom_tween_z: Tween::new(0.0),
            user: UserStats{points: 100},
            last_winning_team: None,
            last_round_dead_units: Vec::new(),
            team_selection_timer: TEAM_SELECTION_TIME,//TODO: This should come from TURBO OS
            team_generation_requested: false,
            previous_battle: None,
            battle_countdown_timer: BATTLE_COUNTDOWN_TIME,
            battle_simulation_requested: false,
            is_playing_sandbox_game: false,
            is_battle_complete: false,
            particle_manager: ParticleManager::new(),
        }
    }
}

turbo::go!({
    let mut state = GameState::load();
    dbgo(&mut state);
    let gp = gamepad(0);
    if gp.a.just_pressed() {
        let t = create_trap(&mut state.rng, None, TrapSide::Middle);
        state.traps.push(t);
    }
    if gp.b.just_pressed() {}
    state.save();
});

fn old_go(mut state: &mut GameState) {
    clear!(0x8f8cacff);
    let gp = gamepad(0);
    if gp.select.just_pressed() {
        os::client::exec("pixel-wars", "simulate_battle", &[]);
    }
    if gp.a.just_pressed() {
        os::client::exec("pixel-wars", "generate_team_seed", &[]);
    }
    match state.phase {
        Phase::TeamSetUp => {
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
                state.previous_battle = os::client::watch_file("pixel-wars", "current_battle")
                    .data
                    .and_then(|file| Battle::try_from_slice(&file.contents).ok());
            }
            if state.auto_assign_teams {
                //generate teams one time
                if !state.team_generation_requested {
                    turbo::println!("Generate teams");
                    os::client::exec("pixel-wars", "generate_teams", &[]);
                    //TODO: Ask josiah if this is bad form bc you don't know what finished first
                    os::client::exec("pixel-wars", "reset_choice_data", &[]);
                    state.team_generation_requested = true;
                }
                //now wait until we have new teams, and if we do, create the unit previews
                let data_store = state
                    .data_store
                    .as_ref()
                    .expect("Data store should be loaded");
                let battle: Option<Battle> = os::client::watch_file("pixel-wars", "current_battle")
                    .data
                    .and_then(|file| Battle::try_from_slice(&file.contents).ok());
                // {
                //     Some(battle) => battle,
                //     None => {
                //         //TODO: Figure out what to do
                //         //if you can't find a battle for some reason
                //         //should not happen though
                //         return;
                //     }
                //};
                let battle_clone = battle.clone();
                if battle != state.previous_battle {
                    let b = battle_clone.unwrap();
                    let team_0 = b.team_0;
                    let team_1 = b.team_1;
                    state
                        .unit_previews
                        .extend(create_unit_previews(&team_0, false, data_store));
                    state
                        .unit_previews
                        .extend(create_unit_previews(&team_1, true, data_store));
                    state.teams = Vec::new();
                    state.teams.push(team_0);
                    state.teams.push(team_1);
                    state.phase = Phase::SelectionScreen;
                }
            } else {
                //make two blank teams
                let data_store = state
                    .data_store
                    .as_ref()
                    .expect("Data store should be loaded");

                state
                    .teams
                    .push(Team::new("Battle Bois".to_string(), data_store.clone()));
                state
                    .teams
                    .push(Team::new("Pixel Peeps".to_string(), data_store.clone()));
                state.phase = Phase::SelectionScreen;
            }
        }

        Phase::SelectionScreen => {
            if state.auto_assign_teams {
                //run the timer
                if state.team_selection_timer > 0 {
                    state.team_selection_timer -= 1;
                } else {
                    state.phase = Phase::PreBattle;
                }
                let userid = os::client::user_id();
                let seed = get_seed_from_turbo_os();
                let file_path = format!("users/{}/choice/{}", userid.unwrap(), seed);
                let num: Option<i32> = os::client::watch_file("pixel-wars", &file_path)
                    .data
                    .and_then(|file| i32::try_from_slice(&file.contents).ok());
                state.selected_team_index = num;
                draw_team_selection_timer(state.team_selection_timer);
                //draw each unit based on the teams
                draw_assigned_team_info(&mut state);
                //get team choice counter from server
                let choices = os::client::watch_file("pixel-wars", "global_choice_counter")
                    .data
                    .and_then(|file| TeamChoiceCounter::try_from_slice(&file.contents).ok())
                    .unwrap_or(TeamChoiceCounter {
                        team_0: 0,
                        team_1: 0,
                    });
                draw_team_choice_numbers(choices);

                for u in &mut state.unit_previews {
                    u.update();
                    u.draw();
                }

                //Draw healthbar when you hover the unit
                let m = mouse(0);
                let mpos = (m.position[0] as f32, m.position[1] as f32);

                for u in &mut state.unit_previews {
                    if u.is_point_in_bounds(mpos) {
                        u.draw_unit_details();
                    }
                }

                let userid = os::client::user_id();

                let file_path = format!("users/{}/stats", userid.unwrap());
                let stats = os::client::watch_file("pixel-wars", &file_path)
                    .data
                    .and_then(|file| UserStats::try_from_slice(&file.contents).ok())
                    .unwrap_or(UserStats { points: 100 });

                draw_points_prebattle_screen(stats.points);
            }

            if !state.auto_assign_teams {
                draw_team_info_and_buttons(&mut state);
            }
            //input
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
        Phase::PreBattle => {
            if !state.battle_simulation_requested {
                //TODO: This will move to some other system once the cron is working
                os::client::exec("pixel-wars", "simulate_battle", &[]);
                state.battle_simulation_requested = true;
            } else {
                if let Some(battle) = os::client::watch_file("pixel-wars", "current_battle")
                    .data
                    .and_then(|file| Battle::try_from_slice(&file.contents).ok())
                {
                    if battle.battle_seed.is_some() {
                        state.rng = RNG::new(battle.battle_seed.unwrap());
                        state.units = create_units_for_all_teams(
                            &mut state.teams,
                            &mut state.rng,
                            state.data_store.as_ref().unwrap(),
                        );
                        set_cam!(x = 192, y = 108);
                        state.phase = Phase::Battle;
                        state.battle_simulation_requested = false;
                        //commit points
                        os::client::exec("pixel-wars", "commit_points", &[]);
                    } else {
                        log!("WAITING FOR SIM RESULT");
                    }
                }
            }
        }
        Phase::Battle => {
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
                log!("TESTING");
                let mut speed = 1;
                if gp.right.pressed() {
                    log!("RIGHT PRESSED");
                    speed = 3;
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
                }
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
            // Draw end game state
            if let Some(winner_idx) = has_some_team_won(&state.units) {
                let sim_result = os::client::watch_file("pixel-wars", "current_result")
                    .data
                    .and_then(|file| SimulationResult::try_from_slice(&file.contents).ok());
                //turbo::println!("Checking sim result");
                turbo::println!("SIM RESULT 1: {:?}", sim_result);
                let result = sim_result.unwrap_or(SimulationResult {
                    seed: 0,
                    living_units: Vec::new(),
                    winning_team: None,
                    num_frames: 1,
                });
                turbo::println!("SIM RESULT 2: {:?}", result);
                //turbo::println!("Unwrapped sim result");
                let seed = result.seed;
                let winning_team_index = result.winning_team;
                let userid = os::client::user_id();
                let file_path = format!("users/{}/choice/{}", userid.unwrap(), seed);
                let choice = os::client::watch_file("pixel-wars", &file_path)
                    .data
                    .and_then(|file| u8::try_from_slice(&file.contents).ok());
                let mut is_win: Option<bool> = None;
                if choice.is_some() {
                    if choice == winning_team_index {
                        is_win = Some(true);
                    } else {
                        is_win = Some(false);
                    }
                }
                // Draw end game visuals
                draw_end_animation(is_win);
                let userid = os::client::user_id();

                let file_path = format!("users/{}/stats", userid.unwrap());
                let stats = os::client::watch_file("pixel-wars", &file_path)
                    .data
                    .and_then(|file| UserStats::try_from_slice(&file.contents).ok())
                    .unwrap_or(UserStats { points: 100 });
                draw_points_end_screen(stats.points, calculate_points_change(is_win));

                // Draw and handle restart button
                let restart_button = Button::new(
                    String::from("AGAIN!"),
                    (20., 175.),
                    (50., 25.),
                    GameEvent::RestartGame(),
                );
                restart_button.draw();
                restart_button.handle_click(&mut state);

                // Start victory animations for living units
                state
                    .units
                    .iter_mut()
                    .filter(|unit| unit.state != UnitState::Dead)
                    .for_each(|unit| unit.start_cheering());

                // Check if simulation matches
                let living_units = all_living_units(&state.units);
                // "current_result"
                //watch current result file
                let sim_result = match os::client::watch_file("pixel-wars", "current_result")
                    .data
                    .and_then(|file| SimulationResult::try_from_slice(&file.contents).ok())
                {
                    Some(sim_result) => sim_result,
                    None => {
                        //TODO: Figure out what to do
                        //if you can't find a battle for some reason
                        //should not happen though
                        return;
                    }
                };
                //and check to make sure it matches our result

                if living_units.len() != sim_result.living_units.len() {
                    text!(
                        "SIMULATION DOES NOT MATCH",
                        x = 50,
                        y = 50,
                        color = DAMAGE_TINT_RED
                    );
                    // turbo::println!("{}", living_units.len());
                    // turbo::println!("{}", sim_result.living_units.len());
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
        }
        Phase::WrapUp => {
            // Post-battle cleanup and results
        }
    }

    //alert drawing
    // Watch for alerts
    if let Some(event) = os::client::watch_events("pixel-wars", Some("alert")).data {
        // Display an alert banner for notifications that are < 10s old
        let duration = 10_000;
        let millis_since_event = time::now() - event.created_at as u64 * 1000;
        if millis_since_event < duration {
            if let Ok(msg) = std::str::from_utf8(&event.data) {
                text!(msg, x = 10, y = 100, font = Font::L);
            }
        }
    }
}

fn draw_end_stats(units: &Vec<Unit>, data_store: &UnitDataStore) {
    let mut team_units: BTreeMap<(String, u8), Vec<&Unit>> = BTreeMap::new();
    for unit in units {
        team_units
            .entry((unit.unit_type.clone(), unit.team))
            .or_default()
            .push(unit);
    }

    for team in [0, 1] {
        let pos = if team == 0 { (10, 80) } else { (270, 80) };

        rect!(
            x = pos.0,
            y = pos.1,
            h = 100,
            w = 100,
            color = DARK_GRAY,
            border_color = OFF_BLACK,
            border_radius = 6,
            border_width = 2
        );

        let mut y_offset = pos.1 + 10;

        power_text!("Unit", x = pos.0 + 5, y = y_offset, underline = true);
        power_text!("Damage", x = pos.0 + 50, y = y_offset, underline = true);
        y_offset += 15;

        for ((unit_type, t), units) in team_units.iter() {
            if *t != team {
                continue;
            }

            let data = data_store.data.get(&**unit_type);
            let x_adj = data.unwrap().bounding_box.0;
            let y_adj = data.unwrap().bounding_box.1;
            let sw = data.unwrap().sprite_width;

            let total_count = units.len();
            let living_count = units.iter().filter(|u| u.health > 0.0).count();
            let total_damage: u32 = units.iter().map(|u| u.stats.damage_dealt).sum();

            let count_text: String = if total_count == living_count {
                format!("")
            } else {
                format!("{}", total_count)
            };
            let living_text: String = if living_count == 0 {
                format!("")
            } else {
                format!("{}", living_count)
            };

            let sprite_name = format!("{}_attack", unit_type);
            sprite!(
                &sprite_name,
                x = pos.0 - x_adj as usize + 24,
                y = y_offset - y_adj as usize,
                sw = sw,
            );
            let damage_text = format!("{}", total_damage);

            power_text!(
                &count_text,
                x = pos.0 + 5,
                y = y_offset,
                strikethrough = true,
                color = DAMAGE_TINT_RED,
            );
            text!(&living_text, x = pos.0 + 12, y = y_offset);
            text!(&damage_text, x = pos.0 + 60, y = y_offset);
            y_offset += 12;
        }
    }
}

fn draw_end_animation(is_win: Option<bool>) {
    if is_win.is_some() {
        let center_x = canvas_size!()[0] / 2;
        let center_y = canvas_size()[1] / 2 - 16;
        if is_win.unwrap() {
            sprite!(
                "you_win_loop_01",
                x = center_x - 48,
                y = center_y,
                sw = 32,
                fps = fps::FAST
            );
            sprite!(
                "you_win_loop_02",
                x = center_x - 16,
                y = center_y,
                scale = 2.0,
                sw = 32,
                fps = fps::FAST
            );
            sprite!(
                "you_win_loop_03",
                x = center_x + 16,
                y = center_y,
                sw = 32,
                fps = fps::FAST
            );
        } else {
            sprite!(
                "you_lose_loop_01",
                x = center_x - 48,
                y = center_y,
                sw = 32,
                fps = fps::FAST
            );
            sprite!(
                "you_lose_loop_02",
                x = center_x - 16,
                y = center_y,
                sw = 32,
                fps = fps::FAST
            );
            sprite!(
                "you_lose_loop_03",
                x = center_x + 16,
                y = center_y,
                sw = 32,
                fps = fps::FAST
            );
        }
    }
}

fn start_end_game_particles(pm: &mut ParticleManager) {
    let colors = [0x87CEFAFF, 0xFF0000FF, 0x00FF00FF];

    for &color in &colors {
        let config = BurstConfig {
            source: BurstSource::Box {
                min: (0., 0.),
                max: (340., 0.),
            },
            direction: std::f32::consts::PI / 2.0,
            spread: std::f32::consts::PI / 4.0,
            speed: 0.4,
            speed_var: 0.3,
            color,
            lifetime: 12.0 + (rand() as f32 / u32::MAX as f32) * 4.0,
            count: 80,
        };
        pm.create_burst(&config);
    }
}

//local sim
fn simulate_battle_locally(state: &mut GameState) {
    // Store initial state
    let initial_state = state.clone();

    // Run simulation with fresh RNG
    state.rng = RNG::new(state.rng.seed);
    let mut i = 0;
    let winning_team_index = loop {
        step_through_battle(
            &mut state.units,
            &mut state.attacks,
            &mut state.traps,
            &mut state.explosions, //look into a callback to replace this
            &mut state.craters,    //look into a callback to replace this
            &mut state.rng,
            &mut state.artifacts,
            &mut state.particle_manager,
            true,
        );
        i += 1;
        if i > 30000 {
            log!("Simulation is taking too long!!");
            panic!("ITS TAKING TOO LONG");
        }
        if let Some(winner_idx) = has_some_team_won(&state.units) {
            break winner_idx;
        }
    };
    apply_end_of_battle_artifacts(
        winning_team_index as usize,
        &mut state.units,
        &mut state.rng,
        &mut state.artifacts,
    );
    // Create simulation result
    let simulation_result = SimulationResult {
        living_units: all_living_units(&state.units),
        seed: state.rng.seed,
        winning_team: Some(winning_team_index),
        num_frames: i,
    };

    //let updated_user = state.user.clone();
    // Reset state
    *state = initial_state;
    state.simulation_result = Some(simulation_result);
    //state.user = updated_user;

    // Reset RNG to match simulation
    state.rng = RNG::new(state.rng.seed);
}

fn step_through_battle(
    units: &mut Vec<Unit>,
    attacks: &mut Vec<Attack>,
    traps: &mut Vec<Trap>,
    explosions: &mut Vec<AnimatedSprite>,
    craters: &mut Vec<AnimatedSprite>,
    rng: &mut RNG,
    artifacts: &mut Vec<Artifact>,
    particle_manager: &mut ParticleManager,
    sim: bool,
) {
    let units_clone = units.clone();
    //=== MOVEMENT AND ATTACKING ===
    //go through each unit, see what it wants to do, and handle all actions from here
    for unit in &mut *units {
        if unit.state == UnitState::Idle {
            //check if you want to switch to healing when you are idle
            if traps.iter().any(|trap| trap.trap_type == TrapType::Healing)
                && unit.health / unit.data.max_health <= 0.5
            {
                //TODO: we should roll here to make it does not always happen
                unit.attack_strategy = AttackStrategy::Heal;
            }

            // //check for trample
            if unit.data.has_attribute(&Attribute::Trample) {
                //TODO: make this depend on enemy locations
                let trample_chance = 12;
                if rng.next() % trample_chance == 0 {
                    unit.attack_strategy = AttackStrategy::Trample { target: None };
                }
            }

            apply_idle_artifacts(unit, rng, artifacts, sim);

            match unit.attack_strategy {
                AttackStrategy::AttackClosest => {
                    //find closest enemy
                    if let Some(closest_target) =
                        find_target(&units_clone, unit.team, unit.pos, TargetPriority::Closest)
                    {
                        if unit.is_unit_in_range(closest_target) {
                            // In range - determine best target
                            let target = if unit.data.splash_area > 0.0
                                && unit.data.has_attribute(&Attribute::Ranged)
                            {
                                find_target(
                                    &units_clone,
                                    unit.team,
                                    unit.pos,
                                    TargetPriority::AreaDensity {
                                        attack_range: unit.data.range,
                                        splash_range: unit.data.splash_area,
                                    },
                                )
                                .unwrap_or(closest_target)
                            } else {
                                closest_target
                            };

                            // Attack logic
                            let mut attack = unit.start_attack(target.id);
                            attack = modify_damage_from_artifacts(attack, &units_clone, artifacts);
                            attacks.push(attack);

                            // Face direction
                            if let Some(display) = unit.display.as_mut() {
                                display.is_facing_left = unit.pos.0 > target.pos.0;
                            }
                            unit.target_id = target.id;
                        } else {
                            // Not in range - move toward closest
                            unit.set_new_target_move_position(&closest_target.pos, rng);
                            unit.target_id = closest_target.id;
                        }
                    }
                }
                AttackStrategy::TargetLowestHealth => {
                    //check if target id is dead or none
                    let mut target_unit = find_unit_by_id(&units_clone, Some(unit.target_id));
                    if target_unit.is_some() && target_unit.unwrap().health > 0. {
                        if unit.is_unit_in_range(&target_unit.unwrap()) {
                            //attacks.push(unit.start_attack();
                            let mut attack = unit.start_attack(target_unit.unwrap().id);
                            attack = modify_damage_from_artifacts(attack, &units_clone, artifacts);

                            attacks.push(attack);
                            //assign the units target id as this unit now
                            unit.target_id = target_unit.unwrap().id;
                            if unit.pos.0 > target_unit.unwrap().pos.0 {
                                if let Some(display) = unit.display.as_mut() {
                                    display.is_facing_left = true;
                                }
                            } else {
                                if let Some(display) = unit.display.as_mut() {
                                    display.is_facing_left = false;
                                }
                            }
                        } else {
                            unit.set_new_target_move_position(&target_unit.unwrap().pos, rng);
                        }
                    } else {
                        //find a unit with lowest health and set it as your target and move toward that position
                        target_unit = find_target(
                            &units_clone,
                            unit.team,
                            unit.pos,
                            TargetPriority::Backline,
                        );
                        if target_unit.is_some() {
                            unit.set_new_target_move_position(&target_unit.unwrap().pos, rng);
                            unit.target_id = target_unit.unwrap().id;
                        }
                    }
                }
                AttackStrategy::Flank { ref mut stage } => {
                    //if target is none, choose lowest health enemy and set target
                    let mut target_unit = find_unit_by_id(&units_clone, Some(unit.target_id));
                    let max_dist = 20.0;
                    if target_unit.is_none() || target_unit.unwrap().health == 0. {
                        target_unit = find_target(
                            &units_clone,
                            unit.team,
                            unit.pos,
                            TargetPriority::Backline,
                        );
                    }
                    if target_unit.is_some() {
                        //if you have a target, move to a position at the bottom of the screen, underneath it
                        //first check if you have reached the top or bottom of the screen. If not, then set target as top of bottom
                        let mut target_pos = target_unit.unwrap().pos;
                        if *stage == FlankStage::Vertical {
                            if unit.pos.1 < 100. {
                                target_pos.1 = MAP_BOUNDS.2;
                            } else {
                                target_pos.1 = MAP_BOUNDS.3;
                            }
                            //give a small adjust to unit.pos so they don't go backwards
                            if unit.pos.0 < 100. {
                                target_pos.0 = unit.pos.0 + 10.;
                            } else {
                                target_pos.0 = unit.pos.0 - 10.;
                            }
                        } else {
                            target_pos.1 = unit.pos.1;
                        }

                        if distance_between(unit.pos, target_pos) > max_dist {
                            unit.set_new_target_move_position(&target_pos, rng);
                        } else {
                            if *stage == FlankStage::Vertical {
                                *stage = FlankStage::Horizontal;
                            } else {
                                unit.attack_strategy = AttackStrategy::TargetLowestHealth;
                            }
                        }
                        //if you can't find a target for some reason, transition to attacking the lowest health enemies
                    } else {
                        unit.attack_strategy = AttackStrategy::TargetLowestHealth;
                    }
                }
                AttackStrategy::SeekTarget => {
                    //set target unit to closest enemy one time
                    let mut target_unit = find_unit_by_id(&units_clone, Some(unit.target_id));
                    if target_unit.is_none() || target_unit.unwrap().health == 0. {
                        target_unit =
                            find_target(&units_clone, unit.team, unit.pos, TargetPriority::Closest);
                        if target_unit.is_some() {
                            unit.target_id = target_unit.unwrap().id;
                        } else {
                            unit.attack_strategy = AttackStrategy::AttackClosest;
                        }
                    }
                    //if you already have a target unit, then try to fight it
                    else {
                        if unit.is_unit_in_range(&target_unit.unwrap()) {
                            let mut attack = unit.start_attack(target_unit.unwrap().id);
                            attack = modify_damage_from_artifacts(attack, &units_clone, artifacts);

                            attacks.push(attack);
                            //assign the units target id as this unit now
                            unit.target_id = target_unit.unwrap().id;
                            if unit.pos.0 > target_unit.unwrap().pos.0 {
                                if let Some(display) = unit.display.as_mut() {
                                    display.is_facing_left = true;
                                }
                            } else {
                                if let Some(display) = unit.display.as_mut() {
                                    display.is_facing_left = false;
                                }
                            }
                        } else {
                            unit.set_new_target_move_position(&target_unit.unwrap().pos, rng);
                        }
                    }
                }
                AttackStrategy::Flee { ref mut timer } => {
                    //if timer is 0, then start trying to fight again
                    if *timer <= 0 {
                        unit.attack_strategy = AttackStrategy::AttackClosest;
                    } else {
                        *timer -= 1;
                        //if not, figure out which way to run away
                        let dir = if unit.team == 0 { -1 } else { 1 };
                        //choose a spot X units in that dir
                        let flee_dist = 50 * dir;
                        //TODO: add some extra randomness to the Y value here
                        let new_target = (unit.pos.0 + flee_dist as f32, unit.pos.1);
                        //move to that target position
                        unit.set_new_target_move_position(&new_target, rng);
                    }
                }
                AttackStrategy::Defend {
                    ref mut timer,
                    ref mut defended_unit_id,
                } => {
                    // Extract the data we need for combat checks before any borrowing
                    let unit_position = unit.pos;
                    let unit_range = unit.data.range;

                    // Check for enemies using our new function that doesn't require borrowing the whole unit
                    if let Some(target_unit) =
                        find_target(&units_clone, unit.team, unit.pos, TargetPriority::Closest)
                    {
                        let enemy = target_unit;
                        if is_in_range_with_data(unit_position, unit_range, enemy.pos) {
                            let mut attack = unit.start_attack(target_unit.id);
                            attack = modify_damage_from_artifacts(attack, &units_clone, artifacts);
                            attacks.push(attack);
                            //unit.attack_strategy = AttackStrategy::AttackClosest;
                            if let Some(display) = unit.display.as_mut() {
                                display.is_facing_left = unit_position.0 > target_unit.pos.0;
                            }
                            continue;
                        }
                    }

                    // If we get here, we're not in combat, so handle defense positioning
                    if defended_unit_id.is_none() {
                        let defended_id = can_defend(&units_clone, unit.team, unit.pos, rng);
                        if defended_id.is_none() {
                            unit.attack_strategy = AttackStrategy::AttackClosest;
                            continue;
                        } else {
                            *defended_unit_id = defended_id;
                        }
                    }

                    if let Some(defended_id) = *defended_unit_id {
                        if let Some(defended_unit) =
                            find_unit_by_id(&units_clone, Some(defended_id))
                        {
                            // Check if the defended unit is still alive
                            if defended_unit.health <= 0.0 {
                                unit.attack_strategy = AttackStrategy::AttackClosest;
                                continue;
                            }

                            // Calculate our desired defensive position based on team
                            let x_offset = if unit.team == 0 { 20.0 } else { -20.0 };
                            let defense_position =
                                (defended_unit.pos.0 + x_offset, defended_unit.pos.1);

                            // Only move if we're not already close to our desired position
                            let distance_to_target = distance_between(unit.pos, defense_position);
                            if distance_to_target > 10.0 {
                                unit.set_new_target_move_position(&defense_position, rng);
                            } else {
                                unit.state = UnitState::Defending;
                            }
                        } else {
                            *defended_unit_id = None;
                            unit.attack_strategy = AttackStrategy::AttackClosest;
                            continue;
                        }
                    }
                }
                AttackStrategy::Heal => {
                    //either you are 'healing' and you have healing status
                    //or you are looking for a health pack
                    // so i want to find the closest health pack
                    //then set that as new target move position
                    if unit.status_effects.contains(&Status::Healing) {
                        //then do nothing and continue healing. This will automaticaly change when they finish healing
                    } else {
                        let pos = closest_health_pack(unit.pos, &traps);
                        if pos.is_some() {
                            // adjust position to add unit foot position on top
                            let d_y = unit.data.bounding_box.3 as f32 / 2.;
                            let pos = pos.unwrap();
                            let pos = (pos.0, pos.1 - d_y);
                            unit.set_exact_move_position(pos);
                        } else {
                            unit.attack_strategy = AttackStrategy::AttackClosest;
                        }
                    }
                }
                AttackStrategy::Trample { ref mut target } => match *target {
                    Some(_) => {
                        unit.attack_strategy = AttackStrategy::AttackClosest;
                    }
                    None => {
                        let mut pos_x = unit.pos.0;
                        let trample_dist = 60.0;
                        let adj = if unit.team == 1 {
                            -trample_dist
                        } else {
                            trample_dist
                        };
                        pos_x = (pos_x + adj).clamp(MAP_BOUNDS.0, MAP_BOUNDS.1);
                        let target_pos = (pos_x, unit.pos.1);
                        *target = Some(target_pos);
                        unit.set_exact_move_position(target_pos);
                    }
                },
                _ => {
                    panic!("Unexpected Attack Strategy!!");
                }
            }
        }

        if let AttackStrategy::Trample { ref mut target } = unit.attack_strategy {
            let collision_indices = overlapping_enemy_indices(&unit, &units_clone);

            if collision_indices
                .iter()
                .any(|&i| units_clone[i].data.has_attribute(&Attribute::Large))
            {
                unit.attack_strategy = AttackStrategy::AttackClosest;
                unit.state = UnitState::Idle;
            } else {
                for &i in &collision_indices {
                    let attack = Attack::new(
                        Some(unit.id),
                        units_clone[i].id,
                        2.,
                        unit.pos,
                        10000.0,
                        0.0,
                        1,
                        Vec::new(),
                    );
                    // let attack = unit.start_attack(units_clone[i].id);
                    // let modified_attack =
                    //     modify_damage_from_artifacts(attack, &units_clone, artifacts);
                    attacks.push(attack);
                }
            }
        }
        unit.update();
        //check if the unit is on top of a trap
        for trap in &mut *traps {
            if is_unit_on_trap(unit, trap) && trap.is_active() {
                if trap.trap_type == TrapType::Poop {
                    if let Some(display) = unit.display.as_mut() {
                        display.footprint_status = FootprintStatus::Poopy;
                    }
                } else if trap.trap_type == TrapType::Acidleak {
                    let attack = Attack::new(
                        None,
                        unit.id,
                        1.,
                        trap.pos,
                        trap.damage,
                        0.,
                        1,
                        vec![Attribute::PoisonAttack],
                    );
                    unit.take_attack(&attack, rng, particle_manager);
                    if let Some(display) = unit.display.as_mut() {
                        display.footprint_status = FootprintStatus::Acid;
                    }
                } else if trap.trap_type == TrapType::Landmine {
                    if let Some(closest_unit_index) =
                        closest_unit_to_position(trap.pos, &units_clone)
                    {
                        let attack = Attack::new(
                            None,
                            units_clone[closest_unit_index].id,
                            1.,
                            trap.pos,
                            trap.damage,
                            8.,
                            1,
                            vec![Attribute::ExplosiveAttack],
                        );
                        attacks.push(attack);
                        trap.set_inactive();
                        turbo::println!("TRAP POS {}, {}", trap.pos.0, trap.pos.1);
                    }
                } else if trap.trap_type == TrapType::Healing {
                    //check if unit is not at max health
                    if unit.health != unit.data.max_health {
                        unit.start_healing();
                        //and erase the trap
                        trap.set_inactive();
                    }
                }
            }
        }
    }

    if let Some(_winning_team) = has_some_team_won(units) {
        //clear all attacks if there's a winner so we don't kill someone after the simulation ended.
        attacks.clear();
        //do end of game artifacts one time
    } else {
        //go through attacks and update, then draw
        attacks.retain_mut(|attack| {
            let should_keep = !attack.update(&units_clone);
            attack.draw();
            if !should_keep {
                let mut total_damage = 0.0;
                let mut kills = 0;
                //deal the actual damage here
                if attack.splash_area == 0. {
                    if let Some(unit_index) =
                        units.iter().position(|u| u.id == attack.target_unit_id)
                    {
                        let unit = &mut units[unit_index];
                        //TODO: If any artifacts effect damage on this end, add them in here
                        let damage = unit.take_attack(&attack, rng, particle_manager);
                        total_damage += damage;
                        if unit.health <= 0. {
                            kills += 1;
                            if unit.data.has_attribute(&Attribute::ExplodeOnDeath) {
                                let explosion_offset = (-8., -24.);
                                // if unit.flip_x() {
                                //     explosion_offset.0 = -8.;
                                // }
                                let explosion_pos = (
                                    unit.pos.0 + explosion_offset.0,
                                    unit.pos.1 + explosion_offset.1,
                                );
                                let mut explosion = AnimatedSprite::new(explosion_pos, false);
                                explosion.set_anim("explosion".to_string(), 32, 14, 5, false);
                                explosions.push(explosion);
                            }
                        }
                    }
                }
                //if it has splash area, then look for all enemy units within range
                if attack.splash_area > 0. {
                    // let team = find_unit_by_id(units, Some(attack.target_unit_id))
                    //     .unwrap()
                    //     .team;
                    for unit in &mut *units {
                        if distance_between(attack.pos, unit.pos) <= attack.splash_area
                            && unit.state != UnitState::Dead
                        // && unit.team == team
                        {
                            let damage = unit.take_attack(&attack, rng, particle_manager);
                            total_damage += damage;

                            if unit.health <= 0.0 {
                                kills += 1;
                                if unit.data.has_attribute(&Attribute::ExplodeOnDeath) {
                                    let explosion_offset = (-8., -24.);
                                    // if unit.flip_x() {
                                    //     explosion_offset.0 = -24.;
                                    // }
                                    let explosion_pos = (
                                        unit.pos.0 + explosion_offset.0,
                                        unit.pos.1 + explosion_offset.1,
                                    );
                                    let mut explosion = AnimatedSprite::new(explosion_pos, false);
                                    explosion.set_anim("explosion".to_string(), 32, 14, 5, false);
                                    explosions.push(explosion);
                                }
                            }
                        }
                    }
                }
                if attack.attributes.contains(&Attribute::ExplosiveAttack) {
                    //create explosion
                    let explosion_offset = (-8., -24.);
                    let explosion_pos = (
                        attack.pos.0 + explosion_offset.0,
                        attack.pos.1 + explosion_offset.1,
                    );
                    let mut explosion = AnimatedSprite::new(explosion_pos, false);
                    explosion.set_anim("explosion".to_string(), 32, 14, 5, false);
                    explosions.push(explosion);
                    //make a crater
                    let crater_pos = (explosion_pos.0, explosion_pos.1 + 16.0);
                    let mut crater = AnimatedSprite::new(crater_pos, false);

                    crater.set_anim("crater_01".to_string(), 16, 1, 1, true);
                    crater.animator.change_tint_color(0xFFFFFF80);
                    craters.push(crater);
                }
                if let Some(attacker) = find_mutable_unit_by_id(units, attack.owner_id) {
                    //if you have the blood sucker artifact, add 10% of damage to unit health
                    let team = attacker.team;
                    // Check for bloodsucker artifact
                    if let Some(bloodsucker) = artifacts.iter_mut().find(|a| {
                        matches!(a.artifact_kind, ArtifactKind::BloodSucker { .. })
                            && a.team == team
                    }) {
                        if let ArtifactKind::BloodSucker { steal_factor } =
                            bloodsucker.artifact_kind
                        {
                            if attacker.health > 0.0 && attacker.health < attacker.data.max_health {
                                let heal_amount = total_damage as f32 * steal_factor;
                                attacker.health =
                                    (attacker.health + heal_amount).min(attacker.data.max_health);
                                bloodsucker.play_effect();
                            }
                        }
                    }

                    attacker.stats.damage_dealt += total_damage as u32;
                    let old_kills = attacker.stats.kills;
                    attacker.stats.kills += kills;
                    //check if kills put you over 3 to trigger Berserk

                    if attacker.stats.kills >= 3
                        && old_kills < 3
                        && attacker.data.has_attribute(&Attribute::Berserk)
                    {
                        let status = Status::Berserk { timer: (600) };
                        attacker.status_effects.push(status);
                    }
                }
            }

            should_keep
        });
    }
    //go through traps, update and draw
    for trap in traps {
        trap.update();
    }
}

fn apply_start_of_battle_artifacts(
    units: &mut Vec<Unit>,
    traps: &mut Vec<Trap>,
    rng: &mut RNG,
    artifacts: &Vec<Artifact>,
) {
    let team_artifacts: Vec<(ArtifactKind, u8)> = artifacts
        .iter()
        .map(|a| (a.artifact_kind, a.team))
        .collect();

    for unit in units.iter_mut() {
        if team_artifacts.iter().any(|&(kind, team)| {
            matches!(kind, ArtifactKind::FlameWard { .. }) && team == unit.team
        }) {
            unit.data.attributes.push(Attribute::FireResistance);
        }
        if team_artifacts.iter().any(|&(kind, team)| {
            matches!(kind, ArtifactKind::SpeedRunner { .. }) && team == unit.team
        }) {
            unit.start_haste();
        }
    }

    if team_artifacts
        .iter()
        .any(|&(kind, team)| matches!(kind, ArtifactKind::DoctorsIn { .. }))
    {
        for &(kind, team) in team_artifacts.iter() {
            if let ArtifactKind::DoctorsIn { num_kits } = kind {
                let side = if team == 0 {
                    TrapSide::Left
                } else {
                    TrapSide::Right
                };
                for _ in 0..num_kits {
                    traps.push(create_trap(rng, Some(TrapType::Healing), side));
                }
            }
        }
    }
    // // Handle traps separately
    // if let Some(trap_artifact) = artifacts
    //     .iter()
    //     .find(|a| a.artifact_kind == ArtifactKind::TrapArtist)
    // {
    //     if let ArtifactConfig::TrapBoard { num_traps } = trap_artifact.config {
    //         for _ in 0..num_traps {
    //             traps.push(create_trap(rng));
    //         }
    //     }
    // }
}

fn apply_idle_artifacts(unit: &mut Unit, rng: &mut RNG, artifacts: &mut Vec<Artifact>, sim: bool) {
    for artifact in artifacts {
        if let ArtifactKind::SeeingGhosts { chance_to_occur } = artifact.artifact_kind {
            if artifact.team != unit.team && rng.next() % chance_to_occur == 0 {
                unit.attack_strategy = AttackStrategy::Flee { timer: 5 };
                artifact.play_effect();
            }
        }
    }
}

fn modify_damage_from_artifacts(
    mut attack: Attack,
    units: &Vec<Unit>,
    artifacts: &mut Vec<Artifact>,
) -> Attack {
    if let Some(attacker) = find_unit_by_id(units, attack.owner_id) {
        for artifact in artifacts {
            if artifact.team != attacker.team {
                continue;
            }
            match artifact.artifact_kind {
                ArtifactKind::StrengthOfTheFallen { percent_per_unit } => {
                    let dead_count = units
                        .iter()
                        .filter(|u| u.health <= 0. && u.team == artifact.team)
                        .count();

                    let damage_multiplier = 1.0 + (dead_count as f32 * percent_per_unit / 100.0);
                    attack.damage *= damage_multiplier;
                    artifact.play_effect();
                }

                ArtifactKind::SnipersFocus { percent_per_pixel } => {
                    if attack.attributes.contains(&Attribute::Ranged) {
                        if let Some(target_unit) =
                            find_unit_by_id(units, Some(attack.target_unit_id))
                        {
                            let dx = (target_unit.pos.0 - attack.pos.0) as f32;
                            let dy = (target_unit.pos.1 - attack.pos.1) as f32;
                            let distance = (dx * dx + dy * dy).sqrt();

                            let damage_multiplier = 1.0 + (distance * percent_per_pixel / 100.0);
                            attack.damage *= damage_multiplier;
                            artifact.play_effect();
                        }
                    }
                }

                ArtifactKind::GiantSlayer { boost_factor } => {
                    if let Some(target_unit) = find_unit_by_id(units, Some(attack.target_unit_id)) {
                        if target_unit.data.has_attribute(&Attribute::Large) {
                            attack.damage *= boost_factor;
                            //turbo::println!("BF: {}", boost_factor);
                            artifact.play_effect();
                        }
                    }
                }
                _ => {}
            }
        }
    }
    attack
}

fn apply_end_of_battle_artifacts(
    winner_idx: usize,
    units: &mut Vec<Unit>,
    rng: &mut RNG,
    artifacts: &mut Vec<Artifact>,
) {
    for artifact in artifacts {
        if let ArtifactKind::Necromancer { revival_chance } = artifact.artifact_kind {
            if artifact.team == winner_idx as u8 {
                for unit in units.iter_mut() {
                    if unit.state == UnitState::Dead && unit.team == artifact.team {
                        if rng.next() % 100 < revival_chance as u32 {
                            unit.revive_unit();
                            artifact.play_effect();
                        } else {
                        }
                    }
                }
            }
        }
    }
}

fn find_unit_by_id(units: &Vec<Unit>, id: Option<u32>) -> Option<&Unit> {
    if units.is_empty() {
        return None;
    }

    match id {
        Some(target_id) => {
            let result = units.iter().find(|&unit| unit.id == target_id);
            match result {
                Some(unit) => Some(unit),
                None => None,
            }
        }
        None => None,
    }
}

fn find_mutable_unit_by_id(units: &mut Vec<Unit>, id: Option<u32>) -> Option<&mut Unit> {
    if units.is_empty() {
        return None;
    }

    match id {
        Some(target_id) => units.iter_mut().find(|unit| unit.id == target_id),
        None => None,
    }
}

fn lowest_health_enemy_unit(units: &Vec<Unit>, team: u8) -> Option<&Unit> {
    if units.is_empty() {
        return None;
    }

    units
        .iter()
        //filter to keep living units not on this team
        .filter(|unit| unit.team != team && unit.health > 0.0)
        .min_by(|a, b| {
            a.data
                .max_health
                .partial_cmp(&b.data.max_health)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

fn closest_health_pack(start_pos: (f32, f32), traps: &Vec<Trap>) -> Option<(f32, f32)> {
    // Look through all traps and find the closest healing trap
    let mut closest_pos = None;
    let mut closest_distance = f32::MAX;

    for trap in traps {
        if trap.trap_type == TrapType::Healing && trap.is_active() {
            let distance = distance_between(start_pos, trap.pos);
            if distance < closest_distance {
                closest_distance = distance;
                closest_pos = Some(trap.pos);
            }
        }
    }

    closest_pos
}

fn is_unit_on_trap(unit: &Unit, trap: &Trap) -> bool {
    // Get the bottom line segment of the unit
    let unit_width = unit.data.bounding_box.2;
    let bottom_left = (
        unit.foot_position().0 - unit_width as f32 / 2.0,
        unit.foot_position().1,
    );
    let bottom_right = (
        unit.foot_position().0 + unit_width as f32 / 2.0,
        unit.foot_position().1,
    );

    // Calculate the nearest point on the line segment to the trap's center
    let trap_to_left = (trap.pos.0 - bottom_left.0, trap.pos.1 - bottom_left.1);
    let segment = (
        bottom_right.0 - bottom_left.0,
        bottom_right.1 - bottom_left.1,
    );
    let segment_length_squared = segment.0 * segment.0 + segment.1 * segment.1;

    // Find the closest point on the line segment to the trap center
    let t = ((trap_to_left.0 * segment.0 + trap_to_left.1 * segment.1) / segment_length_squared)
        .max(0.0)
        .min(1.0);

    let closest_point = (bottom_left.0 + t * segment.0, bottom_left.1 + t * segment.1);

    // Check if this closest point is within the trap's radius
    distance_between(closest_point, trap.pos) < trap.size / 2.0
}

// fn lowest_health_closest_enemy_unit(
//     units: &Vec<Unit>,
//     team: i32,
//     pos: (f32, f32),
// ) -> Option<&Unit> {
//     if units.is_empty() {
//         return None;
//     }

//     units
//         .iter()
//         .filter(|unit| {
//             unit.team != team
//                 && unit.health > 0.0
//                 && !unit
//                     .status_effects
//                     .iter()
//                     .any(|status| matches!(status, Status::Invisible { .. }))
//         })
//         .min_by(|&a, &b| {
//             match a.data.max_health.partial_cmp(&b.data.max_health) {
//                 Some(std::cmp::Ordering::Equal) => {
//                     // If health is equal, compare distances
//                     let dist_a = distance_between(pos, a.pos);
//                     let dist_b = distance_between(pos, b.pos);
//                     dist_a
//                         .partial_cmp(&dist_b)
//                         .unwrap_or(std::cmp::Ordering::Equal)
//                 }
//                 Some(ordering) => ordering,
//                 None => std::cmp::Ordering::Equal,
//             }
//         })
// }

fn draw_ui(state: &mut GameState) {
    let (team0_health, team1_health) = calculate_team_healths(&state.units);
    draw_round_indicator(state.round);

    let (team_0_pos, team_1_pos) = ((24.0, 20.0), (232.0, 20.0));

    // Team 0
    let is_chosen = state.selected_team_index != Some(1);
    draw_team_health_bar(
        team0_health.0,
        team0_health.1,
        team_0_pos,
        &state.teams[0].name.to_uppercase(),
        true,
        is_chosen,
    );
    draw_team_artifacts(state, 0, team_0_pos, false);

    // Team 1
    let is_chosen = state.selected_team_index == Some(1);
    draw_team_health_bar(
        team1_health.0,
        team1_health.1,
        team_1_pos,
        &state.teams[1].name.to_uppercase(),
        false,
        is_chosen,
    );
    draw_team_artifacts(state, 1, team_1_pos, true);
}

fn calculate_team_healths(units: &[Unit]) -> ((f32, f32), (f32, f32)) {
    units.iter().fold(((0.0, 0.0), (0.0, 0.0)), |acc, unit| {
        if unit.team == 0 {
            (
                (
                    acc.0 .0 + unit.data.max_health as f32,
                    acc.0 .1 + unit.health as f32,
                ),
                acc.1,
            )
        } else {
            (
                acc.0,
                (
                    acc.1 .0 + unit.data.max_health as f32,
                    acc.1 .1 + unit.health as f32,
                ),
            )
        }
    })
}

fn draw_round_indicator(round: u8) {
    let txt = format!("{}/{}", round, TOTAL_ROUNDS);
    let center_x = canvas_size!()[0] / 2;
    let x = center_x - ((txt.len() as u32 * 8) / 2);
    text!(&txt, x = x, y = 10, font = Font::L, color = OFF_BLACK);
}

fn draw_team_artifacts(state: &mut GameState, team: u8, pos: (f32, f32), right_aligned: bool) {
    for (i, a) in state
        .artifacts
        .iter_mut()
        .filter(|a| a.team == team)
        .enumerate()
    {
        let x_offset = i as i32 * 16;
        let x = if right_aligned {
            pos.0 as i32 + x_offset + 60
        } else {
            pos.0 as i32 + x_offset
        };
        let y = pos.1 as i32 + 12;
        a.draw_sprite_scaled((x, y), 0.5);
        //artifact icon hover code
        let mp = mouse(0).position;
        if a.icon_is_hovered((x, y), (mp[0], mp[1])) {
            a.draw_name((x, y + 20));
        }
    }
}

fn can_defend(units: &Vec<Unit>, team: u8, pos: (f32, f32), rng: &mut RNG) -> Option<u32> {
    // First check if there's at least one enemy non-ranged unit
    let has_enemy_melee = units.iter().any(|unit| {
        unit.team != team && unit.health > 0.0 && !unit.data.has_attribute(&Attribute::Ranged)
    });

    // If there are no enemy melee units, no need to defend
    if !has_enemy_melee {
        return None;
    }

    // Find all friendly ranged units and collect their IDs
    let friendly_ranged_ids: Vec<u32> = units
        .iter()
        .filter(|unit| {
            unit.team == team && unit.health > 0.0 && unit.data.has_attribute(&Attribute::Ranged)
        })
        .map(|unit| unit.id)
        .collect();

    // If we have no ranged units to defend with, return None
    if friendly_ranged_ids.is_empty() {
        return None;
    }

    // Choose a random ID from our collection of friendly ranged units
    let random_index = rng.next_in_range(0, friendly_ranged_ids.len() as u32 - 1);
    Some(friendly_ranged_ids[random_index as usize])
}

fn lowest_health_ranged_enemy_unit(units: &Vec<Unit>, team: u8, pos: (f32, f32)) -> Option<&Unit> {
    if units.is_empty() {
        return None;
    }

    units
        .iter()
        .filter(|unit| {
            unit.team != team && unit.health > 0.0 && unit.data.has_attribute(&Attribute::Ranged)
        })
        .min_by(|&a, &b| {
            match a.data.max_health.partial_cmp(&b.data.max_health) {
                Some(std::cmp::Ordering::Equal) => {
                    // If health is equal, compare distances
                    let dist_a = distance_between(pos, a.pos);
                    let dist_b = distance_between(pos, b.pos);
                    dist_a
                        .partial_cmp(&dist_b)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }
                Some(ordering) => ordering,
                None => std::cmp::Ordering::Equal,
            }
        })
}

//TODO: Make this based on streak or something like that
fn calculate_points_change(is_won: Option<bool>) -> i32 {
    if is_won.is_some() {
        if is_won.unwrap() {
            return 10;
        } else {
            return -10;
        }
    }
    0
}

fn commit_points_change(user: &mut UserStats, is_win: bool) {
    //user.points += points_change;
    os::client::exec("pixel-wars", "commit_points", &[is_win as u8]);
}

fn draw_points_prebattle_screen(points: i32) {
    //print text showing points at position
    let pos = (20.0, 160.0);
    let txt = format!("Points: {}", points);
    text!(txt.as_str(), x = pos.0, y = pos.1, font = Font::L);
}

fn draw_points_end_screen(points: i32, points_change: i32) {
    let center_x = canvas_size!()[0] / 2;
    let center_y = canvas_size()[1] / 2 - 16;
    let sign = if points_change >= 0 { "+" } else { "-" };
    //draw the points under You Win/You Lose
    let txt = format!("Points: {} ({}{})", points, sign, points_change.abs());
    text!(
        txt.as_str(),
        x = center_x - 64,
        y = center_y + 24,
        font = Font::L
    );
    //then draw the change (plus or minus)
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
enum Phase {
    TeamSetUp, //get the teams from the server
    SelectionScreen,
    PreBattle,
    Battle,
    WrapUp,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
struct AnimatedSprite {
    animator: Animator,
    pos: (f32, f32),
    flip_x: bool,
}

impl AnimatedSprite {
    fn new(pos: (f32, f32), flip_x: bool) -> Self {
        Self {
            //placeholder animation
            animator: Animator::new(Animation {
                name: "placeholder".to_string(),
                s_w: 16,
                num_frames: 0,
                loops_per_frame: 0,
                is_looping: true,
            }),
            pos,
            flip_x,
        }
    }

    fn set_anim(
        &mut self,
        name: String,
        s_w: u8,
        num_frames: u8,
        loops_per_frame: u8,
        is_looping: bool,
    ) {
        self.animator.set_cur_anim(Animation {
            name,
            s_w,
            num_frames,
            loops_per_frame,
            is_looping,
        });
    }
    fn update(&mut self) {
        self.animator.update();
    }

    fn draw(&self) {
        self.animator.draw(self.pos, self.flip_x)
    }
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
struct Attack {
    owner_id: Option<u32>,
    target_unit_id: u32,
    speed: f32,
    pos: (f32, f32),
    damage: f32,
    splash_area: f32,
    size: i32,
    attributes: Vec<Attribute>,
    start_pos: (f32, f32),
    elapsed_frames: f32,
    initial_distance: f32,
}

impl Attack {
    //new
    fn new(
        owner_id: Option<u32>,
        target_unit_id: u32,
        speed: f32,
        pos: (f32, f32),
        damage: f32,
        splash_area: f32,
        size: i32,
        attributes: Vec<Attribute>,
    ) -> Self {
        Self {
            owner_id,
            target_unit_id,
            speed,
            pos,
            damage,
            splash_area,
            size,
            attributes,
            start_pos: pos,
            elapsed_frames: 0.0,
            initial_distance: 0.0,
        }
    }
    fn update(&mut self, units: &Vec<Unit>) -> bool {
        // Get the target unit's position
        let target_unit = find_unit_by_id(units, Some(self.target_unit_id));
        if target_unit.is_some() {
            if self.attributes.contains(&Attribute::ParabolicAttack) {
                let target_position = target_unit.unwrap().pos;

                if self.elapsed_frames == 0.0 {
                    let dx = target_position.0 - self.pos.0;
                    let dy = target_position.1 - self.pos.1;
                    self.initial_distance = (dx * dx + dy * dy).sqrt();
                }

                self.elapsed_frames += 1.0;

                let current_dx = target_position.0 - self.start_pos.0;
                let current_dy = target_position.1 - self.start_pos.1;
                let current_distance = (current_dx * current_dx + current_dy * current_dy).sqrt();
                let flight_duration = current_distance / self.speed;

                let t = self.elapsed_frames / flight_duration;
                let eased_t = t;

                if t >= 1.0 {
                    // Add one extra frame at end position
                    if t < 1.1 {
                        self.pos = target_position;
                        return false;
                    }
                    self.pos = target_position;
                    return true;
                }

                self.pos.0 = self.start_pos.0 + (current_dx * eased_t);

                let height_factor = -0.6;
                let parabola =
                    4.0 * height_factor * current_distance * (eased_t - eased_t * eased_t);
                self.pos.1 = self.start_pos.1 + parabola;
            } else {
                let target_position = target_unit.unwrap().pos;

                // Calculate the direction vector towards the target
                let direction_x = target_position.0 - self.pos.0;
                let direction_y = target_position.1 - self.pos.1;

                // Calculate the distance to the target
                let distance = (direction_x * direction_x + direction_y * direction_y).sqrt();

                // Normalize the direction vector and scale by speed
                if distance > 0.0 {
                    self.pos.0 += self.speed * (direction_x / distance);
                    self.pos.1 += self.speed * (direction_y / distance);
                }
                //if distance is less than speed, we want to remove the attack and deal the damage
                return distance <= self.speed;
            }
        }

        false
    }

    fn draw(&self) {
        //this is for the catapult, a 4x gray circle
        if self.attributes.contains(&Attribute::ParabolicAttack) {
            circ!(
                x = self.pos.0 as i32,
                y = self.pos.1 as i32,
                d = 4 * self.size,
                color = LIGHT_GRAY
            );
            //other ranged attacks get a 2x gray circle
        } else if self.attributes.contains(&Attribute::Ranged) {
            circ!(
                x = self.pos.0 as i32,
                y = self.pos.1 as i32,
                d = 2 * self.size,
                color = LIGHT_GRAY
            );
        }
    }
}

pub fn shuffle<T>(rng: &mut RNG, array: &mut [T]) {
    let len = array.len();
    for i in (1..len).rev() {
        // Generate a random index between 0 and i (inclusive)
        let j = rng.next_in_range(0, i as u32) as usize;
        array.swap(i, j);
    }
}

enum TargetPriority {
    Closest,             // Attack nearest enemy
    Backline,            // Attack ranged units first, then lowest health
    SpecificTarget(u32), // Attack specific unit by ID
    AreaDensity {
        // Attack unit with most other units within splash range
        attack_range: f32,
        splash_range: f32,
    },
}

// Main targeting function
fn find_target(
    units: &Vec<Unit>,
    team: u8,
    pos: (f32, f32),
    priority: TargetPriority,
) -> Option<&Unit> {
    match priority {
        TargetPriority::Backline => {
            // Non-frozen ranged
            units
                .iter()
                .filter(|u| {
                    is_valid_target(u, team)
                        && !is_frozen(u)
                        && u.data.has_attribute(&Attribute::Ranged)
                })
                .min_by(|a, b| {
                    a.health
                        .partial_cmp(&b.health)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .or_else(|| {
                    // Non-frozen non-ranged
                    units
                        .iter()
                        .filter(|u| is_valid_target(u, team) && !is_frozen(u))
                        .min_by(|a, b| {
                            a.health
                                .partial_cmp(&b.health)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                })
                .or_else(|| {
                    // Frozen units
                    units
                        .iter()
                        .filter(|u| is_valid_target(u, team))
                        .min_by(|a, b| {
                            a.health
                                .partial_cmp(&b.health)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                })
        }
        TargetPriority::Closest => {
            // Non-frozen first
            units
                .iter()
                .filter(|u| is_valid_target(u, team) && !is_frozen(u))
                .min_by(|a, b| {
                    let dist_a = distance_between(pos, a.pos);
                    let dist_b = distance_between(pos, b.pos);
                    dist_a
                        .partial_cmp(&dist_b)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .or_else(|| {
                    // Then frozen
                    units
                        .iter()
                        .filter(|u| is_valid_target(u, team))
                        .min_by(|a, b| {
                            let dist_a = distance_between(pos, a.pos);
                            let dist_b = distance_between(pos, b.pos);
                            dist_a
                                .partial_cmp(&dist_b)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                })
        }
        TargetPriority::SpecificTarget(id) => {
            // Non-frozen first
            units
                .iter()
                .find(|u| u.id == id && is_valid_target(u, team) && !is_frozen(u))
                .or_else(|| {
                    // Then frozen
                    units
                        .iter()
                        .find(|u| u.id == id && is_valid_target(u, team))
                })
        }
        TargetPriority::AreaDensity {
            attack_range,
            splash_range,
        } => units
            .iter()
            .filter(|u| is_valid_target(u, team) && !is_frozen(u))
            .map(|unit| {
                let nearby_count = units
                    .iter()
                    .filter(|other| {
                        is_valid_target(other, team)
                            && distance_between(unit.pos, other.pos) <= splash_range
                    })
                    .count();
                (unit, nearby_count)
            })
            .filter(|(unit, _)| distance_between(pos, unit.pos) <= attack_range)
            .max_by_key(|(_, count)| *count)
            .map(|(unit, _)| unit),
    }
}

fn is_valid_target(unit: &Unit, team: u8) -> bool {
    unit.team != team
        && unit.health > 0.0
        && !unit
            .status_effects
            .iter()
            .any(|s| matches!(s, Status::Invisible { .. }))
}

fn is_frozen(unit: &Unit) -> bool {
    unit.status_effects
        .iter()
        .any(|s| matches!(s, Status::Freeze { .. }))
}

// fn closest_enemy_unit(units: &Vec<Unit>, team: i32, pos: (f32, f32)) -> Option<&Unit> {
//     units
//         .iter()
//         .filter(|unit| {
//             unit.team != team
//                 && unit.health > 0.0
//                 && !unit
//                     .status_effects
//                     .iter()
//                     .any(|status| matches!(status, Status::Invisible { .. }))
//         })
//         .min_by(|&a, &b| {
//             let dist_a = distance_between(pos, a.pos);
//             let dist_b = distance_between(pos, b.pos);
//             dist_a
//                 .partial_cmp(&dist_b)
//                 .unwrap_or(std::cmp::Ordering::Equal)
//         })
// }

fn is_in_range_with_data(position: (f32, f32), range: f32, target_position: (f32, f32)) -> bool {
    // Calculate distance between positions
    let distance = distance_between(position, target_position);
    // Check if target is within range
    distance <= range
}

// fn closest_enemy_index_with_data(
//     team: i32,
//     position: (f32, f32),
//     unit_id: u32, // Using unit ID instead of a pointer
//     units: &Vec<Unit>,
// ) -> Option<usize> {
//     units
//         .iter()
//         .enumerate()
//         .filter(|(_, other_unit)| {
//             other_unit.team != team &&        // Filter out units on the same team
//             other_unit.health > 0.0 &&        // Filter out dead units
//             other_unit.id != unit_id && // Filter out the unit itself using ID comparison
//             !other_unit.status_effects.iter().any(|status| matches!(status, Status::Invisible { .. }))

//         })
//         .min_by(|(_, a), (_, b)| {
//             let dist_a = distance_between(position, a.pos);
//             let dist_b = distance_between(position, b.pos);
//             dist_a
//                 .partial_cmp(&dist_b)
//                 .unwrap_or(std::cmp::Ordering::Equal)
//         })
//         .map(|(index, _)| index)
// }

// fn closest_enemy_index(unit: &Unit, units: &Vec<Unit>) -> Option<usize> {
//     closest_enemy_index_with_data(unit.team, unit.pos, unit.id, units)
// }

fn closest_unit_to_position(position: (f32, f32), units: &Vec<Unit>) -> Option<usize> {
    units
        .iter()
        .enumerate()
        .filter(|(_, unit)| {
            unit.health > 0.0 // Filter out dead units
        })
        .min_by(|(_, a), (_, b)| {
            let dist_a = distance_between(position, a.pos);
            let dist_b = distance_between(position, b.pos);
            dist_a
                .partial_cmp(&dist_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(index, _)| index)
}

fn random_enemy_id(unit: &Unit, units: &Vec<Unit>, rng: &mut RNG) -> Option<u32> {
    let enemy_ids: Vec<u32> = units
        .iter()
        .filter(|other_unit| {
            other_unit.team != unit.team && // Filter out units on the same team
            other_unit.health > 0.0 &&      // Filter out dead units
            other_unit.id != unit.id // Filter out the unit itself
        })
        .map(|other_unit| other_unit.id)
        .collect();

    if enemy_ids.is_empty() {
        None
    } else {
        let random_index = rng.next_in_range(0, enemy_ids.len() as u32 - 1 as u32);
        Some(enemy_ids[random_index as usize])
    }
}

fn distance_between(pos1: (f32, f32), pos2: (f32, f32)) -> f32 {
    let dx = pos1.0 - pos2.0;
    let dy = pos1.1 - pos2.1;
    (dx * dx + dy * dy).sqrt()
}

fn has_some_team_won(units: &Vec<Unit>) -> Option<u8> {
    let all_team_0_dead = units
        .iter()
        .filter(|unit| unit.team == 0)
        .all(|unit| unit.state == UnitState::Dead);
    let all_team_1_dead = units
        .iter()
        .filter(|unit| unit.team == 1)
        .all(|unit| unit.state == UnitState::Dead);

    if all_team_0_dead {
        return Some(1);
    } else if all_team_1_dead {
        return Some(0);
    }
    None
}

fn overlapping_enemy_indices(unit: &Unit, units: &Vec<Unit>) -> Vec<usize> {
    let (left1, right1, top1, bottom1) = get_bounds(unit);

    units
        .iter()
        .enumerate()
        .filter(|(_, other)| {
            other.id != unit.id && other.team != unit.team && {
                let (left2, right2, top2, bottom2) = get_bounds(other);
                !(left1 > right2 || right1 < left2 || top1 > bottom2 || bottom1 < top2)
            }
        })
        .map(|(i, _)| i)
        .collect()
}

fn get_bounds(unit: &Unit) -> (f32, f32, f32, f32) {
    let half_width = unit.data.bounding_box.2 as f32 / 2.0;
    let half_height = unit.data.bounding_box.3 as f32 / 2.0;

    let left = unit.pos.0 - half_width;
    let right = unit.pos.0 + half_width;
    let top = unit.pos.1 - half_height;
    let bottom = unit.pos.1 + half_height;

    (left, right, top, bottom)
}

fn all_living_units(units: &Vec<Unit>) -> Vec<String> {
    let mut living_units: Vec<String> = Vec::new();
    for u in units {
        if u.state != UnitState::Dead {
            living_units.push(u.unit_type.to_string());
        }
    }
    living_units
}

fn draw_prematch_timer(time: u32) {
    let text = time / 60;
    let text = format!("{}", text);
    power_text!(
        text.as_str(),
        x = 0,
        y = 90,
        font = Font::XL,
        center_width = 384,
        drop_shadow = SHADOW_COLOR,
        underline = true,
    );
}

fn draw_team_selection_timer(time: u32) {
    //turn ticks into seconds format
    let text = format_time(time);
    let text = format!("Next Battle in: {}", text);
    text!(text.as_str(), x = 90, y = 10, font = Font::L);
}

fn format_time(ticks: u32) -> String {
    let total_seconds = ticks / 60; // Convert ticks to seconds
    let minutes = total_seconds / 60;
    let remaining_seconds = total_seconds % 60;
    format!("{}:{:02}", minutes, remaining_seconds)
}

fn draw_assigned_team_info(state: &mut GameState) {
    let pos_0 = 20;
    let pos_1 = 200;
    let y_start = 30;
    let button_width = 80;
    let button_height = 20;

    for (team_index, pos) in [(0, pos_0), (1, pos_1)].iter() {
        let team = &mut state.teams[*team_index].clone();
        let mut y_pos = y_start;

        // Draw team name
        let name_text = format!("{}", team.name);
        text!(
            name_text.as_str(),
            x = *pos,
            y = y_pos,
            font = Font::L,
            color = 0xADD8E6ff
        );
        if team.win_streak > 0 {
            let streak_text = format!("{} Win Streak", team.win_streak);
            text!(
                streak_text.as_str(),
                x = *pos,
                y = y_pos + 10,
                font = Font::L,
                color = ACID_GREEN,
            );
        }
        let team_summary = team.get_unit_summary();
        for (unit_type, count) in team_summary {
            let text = format!("{} {}s", count, unit_type);
            y_pos += 30;
            text!(text.as_str(), x = *pos, y = y_pos, font = Font::L);
            //figure out which unit type is in each time and how many
        }
        text!("AND", x = *pos + 24, y = y_start + 45);

        //Make a button for this team
        let team_button = Button::new(
            String::from("CHOOSE"),
            (*pos as f32, y_pos as f32 + 20.),
            (button_width as f32, button_height as f32),
            GameEvent::ChooseTeam(*team_index as i32),
        );
        team_button.draw();
        team_button.handle_click(state);
        //check if it is the selected team, and if it is put a border around it
        if let Some(selected) = state.selected_team_index {
            if *team_index == selected as usize {
                //draw highlight around button
                rect!(
                    x = *pos,
                    y = y_pos + 20,
                    w = button_width,
                    h = button_height,
                    border_radius = 3,
                    border_width = 1,
                    border_color = 0xe6e7f0ff,
                    color = 0x00000000,
                );
            }
        }
    }
    text!(
        "VS.",
        x = 150,
        y = y_start + 45,
        font = Font::L,
        color = 0xADD8E6ff
    );
}

fn draw_team_choice_numbers(choices: TeamChoiceCounter) {
    //print text for each choice at the correct position
    let team_0_text = format!("Backed by {} players", choices.team_0);
    let team_1_text = format!("Backed by {} players", choices.team_1);
    let (x_0, x_1) = (20, 200);
    let y = 134;
    text!(&team_0_text, x = x_0, y = y);
    text!(&team_1_text, x = x_1, y = y);
}

fn draw_team_info_and_buttons(state: &mut GameState) {
    let pos_0 = 20;
    let pos_1 = 200;
    let y_start = 20;
    let y_spacing = 20;
    let button_width = 20;
    let button_height = 10;

    let data_store = state
        .data_store
        .as_ref()
        .expect("Data store should be loaded");
    let mut unit_types = data_store.get_all_unit_types();
    unit_types.sort(); // Sort the unit types alphabetically

    for (team_index, pos) in [(0, pos_0), (1, pos_1)].iter() {
        let team = &mut state.teams[*team_index].clone();
        let mut y_pos = y_start;

        // Draw team name
        let name_text = format!("{}:", team.name);
        text!(name_text.as_str(), x = *pos, y = y_pos);
        y_pos += y_spacing;

        // Draw unit info and buttons
        for unit_type in &unit_types {
            let num_units = team.num_unit(unit_type.clone());
            let unit_type_capitalized = unit_type
                .chars()
                .next()
                .unwrap()
                .to_uppercase()
                .collect::<String>()
                + &unit_type[1..];
            let unit_text = format!("[{}] {}", num_units, unit_type_capitalized);
            text!(unit_text.as_str(), x = *pos, y = y_pos, font = Font::L);

            // Plus Button
            let plus_button = Button::new(
                String::from("+"),
                (*pos as f32 + 100.0, y_pos as f32),
                (button_width as f32, button_height as f32),
                GameEvent::AddUnitToTeam(*team_index, unit_type.clone()),
            );
            plus_button.draw();
            plus_button.handle_click(state);

            // Minus Button
            let minus_button = Button::new(
                String::from("-"),
                (*pos as f32 + 130.0, y_pos as f32),
                (button_width as f32, button_height as f32),
                GameEvent::RemoveUnitFromTeam(*team_index, unit_type.clone()),
            );
            minus_button.draw();
            minus_button.handle_click(state);

            y_pos += y_spacing;
        }
    }
    let final_y_pos = {
        let mut y = y_start;
        y += y_spacing; // Team name
        y += unit_types.len() as i32 * y_spacing;
        y
    };
    draw_artifact_info_and_buttons(state, final_y_pos);
}

fn draw_artifact_info_and_buttons(state: &mut GameState, y_start_pos: i32) {
    let pos_0 = 20;
    let pos_1 = 200;
    let y_spacing = 20;
    let button_width = 20;
    let button_height = 10;

    for (team_index, pos) in [(0, pos_0), (1, pos_1)].iter() {
        let mut y_pos = y_start_pos;

        text!("Artifacts:", x = *pos, y = y_pos);
        y_pos += y_spacing;

        for artifact_kind in ArtifactKind::iter() {
            let has_artifact = state.artifacts.iter().any(|a| {
                // Compare only the variant type, ignore the fields
                std::mem::discriminant(&a.artifact_kind) == std::mem::discriminant(&artifact_kind)
                    && a.team == *team_index as u8
            });
            let debug_str = format!("{:?}", artifact_kind);
            let name = debug_str.split('{').next().unwrap_or("");
            let artifact_text = format!("[{}] {}", if has_artifact { 1 } else { 0 }, name);
            text!(artifact_text.as_str(), x = *pos, y = y_pos, font = Font::M);

            let plus_button = Button::new(
                String::from("+"),
                (*pos as f32 + 100.0, y_pos as f32),
                (button_width as f32, button_height as f32),
                GameEvent::AddArtifactToTeam(*team_index, artifact_kind),
            );
            plus_button.draw();
            plus_button.handle_click(state);

            let minus_button = Button::new(
                String::from("-"),
                (*pos as f32 + 130.0, y_pos as f32),
                (button_width as f32, button_height as f32),
                GameEvent::RemoveArtifactFromTeam(*team_index, artifact_kind),
            );
            minus_button.draw();
            minus_button.handle_click(state);

            y_pos += y_spacing;
        }
    }
}

fn draw_text_box(
    text: String,
    pos: (f32, f32),
    size: (f32, f32),
    background_color: i32,
    text_color: i32,
) {
    //draw a border around the box with rect!
    rect!(
        x = pos.0,
        y = pos.1,
        w = size.0,
        h = size.1,
        color = background_color,
        border_color = 0x000000ff,
        border_radius = 2,
        border_width = 2,
    );
    let text_width = text.len() * 5;
    let text_height = 8; // Assuming 8 pixels high for the text

    // Calculate centered position for text
    let text_x = pos.0 as i32 + (size.0 as i32 - text_width as i32) / 2;
    let text_y = pos.1 as i32 + (size.1 as i32 - text_height as i32) / 2;

    // Draw centered text
    text!(
        text.as_str(),
        x = text_x,
        y = text_y + 1,
        color = text_color
    ); // Centered button label
       //text!(&text, x = pos.0, y = pos.1, color = text_color);
}

fn draw_team_health_bar(
    total_base_health: f32,
    current_health: f32,
    pos: (f32, f32),
    team_name: &str,
    right_allign: bool,
    is_chosen_team: bool,
) {
    let x = pos.0;
    let y = pos.1;
    let x_bar = x;
    let y_bar = y;
    let w_bar = 128.;
    let h_bar = 10;
    let inner_border_color: u32 = 0x696682ff;
    let outer_border_color: u32 = 0xc5c7ddff;
    let selected_border_color: u32 = 0xe6e7f0ff;
    let mut health_width = (current_health / total_base_health * w_bar) as i32;
    health_width = health_width.max(0);

    let checker_size = 2; // Size of each checker square
    let rows = (h_bar as f32 / checker_size as f32).ceil() as i32;
    let cols = (w_bar as f32 / checker_size as f32).ceil() as i32;

    // Colors for the checkerboard pattern
    let main_color_dark: u32 = 0xadb834ff;
    let main_color_light: u32 = 0xd5dc1dff;
    let back_color_dark: u32 = 0xf1641fff;
    let back_color_light: u32 = 0xfca570ff;

    // Draw checkerboard pattern
    for row in 0..rows {
        for col in 0..cols {
            let checker_x = x_bar + (col * checker_size) as f32;
            let checker_y = y_bar + (row * checker_size) as f32;
            let is_light = (row + col) % 2 == 0;
            let is_health = (col * checker_size) < health_width;

            let color = if is_health {
                if is_light {
                    main_color_light
                } else {
                    main_color_dark
                }
            } else {
                if is_light {
                    back_color_light
                } else {
                    back_color_dark
                }
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

    // Draw health bar inner border
    rect!(
        w = w_bar + 2.,
        h = h_bar + 2,
        x = x_bar - 1.,
        y = y_bar - 1.,
        color = 0,
        border_color = inner_border_color,
        border_width = 2,
        border_radius = 5
    );

    //draw outer border
    rect!(
        w = w_bar + 4.,
        h = h_bar + 5,
        x = x_bar - 2.,
        y = y_bar - 2.,
        color = 0,
        border_color = outer_border_color,
        border_width = 2,
        border_radius = 5
    );
    //draw selected_team_border
    if is_chosen_team {
        rect!(
            w = w_bar + 6.,
            h = h_bar + 7,
            x = x_bar - 3.,
            y = y_bar - 3.,
            color = 0,
            border_color = selected_border_color,
            border_width = 2,
            border_radius = 5
        );
    }
    let mut text_adj = 0.;
    if right_allign {
        text_adj = (128 - team_name.len() * 5) as f32;
    }
    //put team name in white below the bar
    text!(
        team_name,
        x = x_bar + text_adj,
        y = y_bar + h_bar as f32 + 8.,
        font = Font::M,
        color = SHADOW_COLOR
    );
    text!(
        team_name,
        x = x_bar + text_adj,
        y = y_bar + h_bar as f32 + 7.,
        font = Font::M,
        color = WHITE
    );
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
struct Team {
    name: String,
    units: Vec<String>,
    data: UnitDataStore,
    win_streak: u32,
}

impl Team {
    fn new(name: String, data: UnitDataStore) -> Self {
        Self {
            name,
            units: Vec::new(),
            data,
            win_streak: 0,
        }
    }

    fn add_unit(&mut self, unit: String) {
        self.units.push(unit);
    }

    fn num_unit(&self, unit_type: String) -> i32 {
        // Return the number of units of a specific UnitType in self.units
        self.units.iter().filter(|&unit| *unit == unit_type).count() as i32
    }

    fn remove_unit(&mut self, unit_type: String) -> bool {
        // Remove the last unit of the specified UnitType, only if there is at least one
        if let Some(pos) = self.units.iter().rposition(|unit| *unit == unit_type) {
            self.units.remove(pos);
            true
        } else {
            false
        }
    }

    fn capitalize(s: &str) -> String {
        let mut c = s.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }

    fn get_unit_summary(&self) -> Vec<(String, usize)> {
        let mut sorted_units = self.units.clone();
        sorted_units.sort();

        let mut summary = Vec::new();
        let mut current_unit = String::new();
        let mut count = 0;

        for unit in sorted_units {
            if unit != current_unit {
                if !current_unit.is_empty() {
                    summary.push((Self::capitalize(&current_unit), count));
                }
                current_unit = unit;
                count = 1;
            } else {
                count += 1;
            }
        }

        if !current_unit.is_empty() {
            summary.push((Self::capitalize(&current_unit), count));
        }

        summary
    }
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
enum GameEvent {
    AddUnitToTeam(usize, String),
    RemoveUnitFromTeam(usize, String),
    AddArtifactToTeam(usize, ArtifactKind),
    RemoveArtifactFromTeam(usize, ArtifactKind),
    ChooseTeam(i32),
    RestartGame(),
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
struct Animator {
    //current animation
    cur_anim: Animation,
    anim_timer: u16,
    next_anim: Option<Animation>,
    tint_color: usize,
}

impl Animator {
    fn new(cur_anim: Animation) -> Self {
        Animator {
            cur_anim,
            anim_timer: 0,
            next_anim: None,
            tint_color: WHITE,
        }
    }

    fn update(&mut self) {
        // if !self.is_done(){
        //     self.anim_timer += 1;
        // }
        self.anim_timer += 1;
        if self.is_done() {
            if self.cur_anim.is_looping {
                self.anim_timer = 0;
            } else if let Some(next_anim) = self.next_anim.take() {
                self.cur_anim = next_anim;
                self.anim_timer = 0;
            }
        }
    }

    fn is_done(&self) -> bool {
        if self.anim_timer >= self.cur_anim.total_animation_time() {
            return true;
        }
        false
    }

    fn change_tint_color(&mut self, color: usize) {
        self.tint_color = color;
    }

    fn draw(&self, pos: (f32, f32), flip_x: bool) {
        let name = self.cur_anim.name.as_str();
        let mut frame_index = self.anim_timer / self.cur_anim.loops_per_frame as u16; // Calculate the frame index
        frame_index = frame_index.clamp(0, self.cur_anim.num_frames as u16 - 1);
        let sx = (frame_index * self.cur_anim.s_w as u16).clamp(
            0,
            self.cur_anim.s_w as u16 * (self.cur_anim.num_frames as u16 - 1),
        ); // Calculate the sprite X coordinate

        sprite!(
            name,
            x = pos.0,
            y = pos.1,
            sx = sx,
            flip_x = flip_x,
            sw = self.cur_anim.s_w,
            color = self.tint_color,
        );
    }

    fn set_cur_anim(&mut self, new_anim: Animation) {
        if self.cur_anim != new_anim {
            self.cur_anim = new_anim;
            self.anim_timer = 0;
        }
    }

    fn set_next_anim(&mut self, next_anim: Option<Animation>) {
        self.next_anim = next_anim;
    }
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
struct Animation {
    name: String,
    s_w: u8,
    num_frames: u8,
    loops_per_frame: u8,
    is_looping: bool,
}

impl Animation {
    fn total_animation_time(&self) -> u16 {
        return self.num_frames as u16 * self.loops_per_frame as u16;
    }
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
struct Button {
    label: String,
    position: (f32, f32),
    size: (f32, f32),
    event: GameEvent,
}

impl Button {
    fn new(label: String, position: (f32, f32), size: (f32, f32), event: GameEvent) -> Self {
        Self {
            label,
            position,
            size,
            event,
        }
    }

    fn is_clicked(&self, click_position: [i32; 2]) -> bool {
        let (x, y) = self.position;
        let (width, height) = self.size;
        let (click_x, click_y) = (click_position[0] as f32, click_position[1] as f32);

        click_x >= x && click_x <= x + width && click_y >= y && click_y <= y + height
    }

    fn handle_click(&self, game_state: &mut GameState) {
        if mouse(0).left.just_pressed() && self.is_clicked(mouse(0).position) {
            game_state.event_queue.push(self.event.clone());
        }
    }

    fn draw(&self) {
        // Drawing logic for the button
        rect!(
            x = self.position.0,
            y = self.position.1,
            w = self.size.0,
            h = self.size.1,
            color = 0x808080ff,
            border_radius = 5,
            border_width = 2,
            border_color = 0x000000ff,
        ); // Example button background
           // Calculate text dimensions
        let text_width = self.label.len() * 5; // Assuming 4 pixels per character
        let text_height = 8; // Assuming 8 pixels high for the text

        // Calculate centered position for text
        let text_x = self.position.0 as i32 + (self.size.0 as i32 - text_width as i32) / 2;
        let text_y = self.position.1 as i32 + (self.size.1 as i32 - text_height as i32) / 2;

        // Draw centered text
        text!(self.label.as_str(), x = text_x, y = text_y + 1); // Centered button label
    }
}

fn create_units_for_all_teams(
    teams: &mut Vec<Team>,
    rng: &mut RNG,
    data_store: &UnitDataStore,
) -> Vec<Unit> {
    // Define clump positions
    // Define clump positions with more spacing
    let left_clumps = vec![
        (20.0, 60.0),  // Front left
        (20.0, 110.0), // Mid-low left
        (40.0, 160.0), // Back left
        (70.0, 80.0),  // Mid-high right
        (55.0, 130.0), // Center-ish
        (25.0, 180.0), // Lower back
    ];

    let right_clumps = vec![
        (334.0, 60.0),  // Front right
        (349.0, 110.0), // Mid-low right
        (324.0, 160.0), // Back right
        (294.0, 80.0),  // Mid-high left
        (309.0, 130.0), // Center-ish
        (339.0, 180.0), // Lower back
    ];
    let overflow_positions = vec![(-30.0, 100.0), (384.0, 100.0)];

    let mut units = Vec::new();
    let mut id = 1;

    // Process each team
    for (team_index, team) in teams.iter().enumerate() {
        // Group units by type
        let mut unit_groups: HashMap<String, Vec<String>> = HashMap::new();
        for unit_type in &team.units {
            unit_groups
                .entry(unit_type.clone())
                .or_insert(Vec::new())
                .push(unit_type.clone());
        }

        // Get clump positions for this team
        let clump_positions = if team_index == 0 {
            &left_clumps
        } else {
            &right_clumps
        };
        let mut next_clump = 0;

        // Process each unit type group
        for (_unit_type, group) in unit_groups {
            // Split into subgroups of 20 if needed
            let chunks = group.chunks(20).collect::<Vec<_>>();

            for chunk in chunks {
                if next_clump >= clump_positions.len() {
                    // Use overflow position if we run out of clump spots
                    let overflow_pos = overflow_positions[team_index];
                    create_clump(
                        chunk,
                        overflow_pos,
                        team_index,
                        &mut id,
                        data_store,
                        rng,
                        &mut units,
                    );
                } else {
                    let clump_pos = clump_positions[next_clump];
                    create_clump(
                        chunk, clump_pos, team_index, &mut id, data_store, rng, &mut units,
                    );
                    next_clump += 1;
                }
            }
        }
    }

    units
}

fn create_clump(
    unit_types: &[String],
    base_pos: (f32, f32),
    team_index: usize,
    id: &mut u32,
    data_store: &UnitDataStore,
    rng: &mut RNG,
    units: &mut Vec<Unit>,
) {
    let row_height = 12.0; // Reduced from 16
    let row_width = 8.0; // Reduced from 20
    let units_per_row = 3; // Reduced from 5 to make more vertical formations

    for (i, unit_type) in unit_types.iter().enumerate() {
        let row = i / units_per_row;
        let col = i % units_per_row;

        // Smaller random offsets
        let offset_x = rng.next_in_range(0, 3) as f32 - 1.5;
        let offset_y = rng.next_in_range(0, 3) as f32 - 1.5;

        let x = base_pos.0 + (col as f32 * row_width) + offset_x;
        let y = base_pos.1 + (row as f32 * row_height) + offset_y;

        let mut unit = Unit::new(unit_type.clone(), (x, y), team_index as u8, data_store, *id);
        unit.set_starting_strategy(rng);
        units.push(unit);
        *id += 1;
    }
}

fn generate_team(
    data_store: &UnitDataStore,
    rng: &mut RNG,
    match_team: Option<&Team>,
    team_name: String,
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
        None => calculate_team_power_target(&unit_powers, TEAM_POWER_MULTIPLIER),
    };

    // Create and return the team
    let mut team = Team::new(team_name, data_store.clone());
    create_team(&mut team, &selected_types, &unit_powers, target_power, rng);
    team
}

//TODO: Make this system more fair
fn calculate_single_unit_power(unit_data: &UnitData) -> f32 {
    // Calculate base DPS
    let dps = unit_data.damage / (unit_data.attack_time as f32 / 60.0);

    // Range multiplier
    let range_multiplier = if unit_data.range > 125. {
        8.0
    } else if unit_data.range > 50.0 {
        6.0
    } else if unit_data.range > 20.0 {
        3.0
    } else {
        1.0
    };

    // Splash multiplier
    let splash_multiplier = if unit_data.splash_area > 9.0 {
        10.0
    } else if unit_data.splash_area > 0.0 {
        3.0
    } else {
        1.0
    };

    // Combine all factors
    let power_level =
        unit_data.max_health + (dps * range_multiplier * splash_multiplier) + unit_data.speed;

    power_level
}

pub fn calculate_team_power_target(
    power_levels: &HashMap<String, f32>,
    team_size_multiplier: f32,
) -> f32 {
    let average_power = calculate_average_unit_power(power_levels);
    average_power * team_size_multiplier
}

pub fn calculate_average_unit_power(power_levels: &HashMap<String, f32>) -> f32 {
    if power_levels.is_empty() {
        return 0.0;
    }
    power_levels.values().sum::<f32>() / power_levels.len() as f32
}

pub fn get_team_total_power(team: &Team) -> f32 {
    team.units
        .iter()
        .map(|unit_type| {
            if let Some(unit_data) = team.data.data.get(unit_type) {
                calculate_single_unit_power(unit_data)
            } else {
                0.0
            }
        })
        .sum()
}

fn select_random_unit_types(
    available_types: &[&String], // Taking references as input
    num_types: usize,
    rng: &mut RNG,
) -> Vec<String> {
    // Returning owned Strings
    let mut selected_types = Vec::new();
    let mut remaining_attempts = 100;

    while selected_types.len() < num_types && remaining_attempts > 0 {
        let index = rng.next_in_range(0, available_types.len() as u32 - 1) as usize;
        let unit_type = available_types[index].clone(); // Clone to get owned String

        if !selected_types.contains(&unit_type) {
            selected_types.push(unit_type);
        }

        remaining_attempts -= 1;
    }

    selected_types
}

fn create_team(
    team: &mut Team,
    unit_types: &[String],
    power_levels: &HashMap<String, f32>,
    target_power: f32,
    rng: &mut RNG,
) {
    let mut current_power = 0.0;

    // Get power levels for all unit types
    let powers: Vec<&f32> = unit_types
        .iter()
        .map(|unit_type| &power_levels[unit_type])
        .collect();

    // Generate random weights for each unit type that sum to 1.0
    let mut weights: Vec<f32> = (0..unit_types.len()).map(|_| rng.next_f32()).collect();
    let sum: f32 = weights.iter().sum();
    weights.iter_mut().for_each(|w| *w /= sum);

    while current_power < target_power {
        let remaining_power = target_power - current_power;

        // Generate random number for weighted selection
        let mut random = rng.next_f32();
        let mut selected_index = 0;

        // Select unit type based on weights
        for (i, &weight) in weights.iter().enumerate() {
            random -= weight;
            if random <= 0.0 {
                selected_index = i;
                break;
            }
        }

        // Try to add selected unit if power allows
        if remaining_power >= *powers[selected_index] {
            team.units.push(unit_types[selected_index].clone());
            current_power += powers[selected_index];
        } else {
            // Try other unit types if selected one doesn't fit
            let mut found_fit = false;
            for i in 0..unit_types.len() {
                if remaining_power >= *powers[i] {
                    team.units.push(unit_types[i].clone());
                    current_power += powers[i];
                    found_fit = true;
                    break;
                }
            }
            // If no unit type fits, stop adding units
            if !found_fit {
                break;
            }
        }
    }

    // Ensure at least one of each unit type
    for unit_type in unit_types {
        if !team.units.contains(unit_type) {
            team.units.push(unit_type.clone());
        }
    }
}

fn create_unit_previews(
    team: &Team,
    is_facing_left: bool,
    data_store: &UnitDataStore,
) -> Vec<UnitPreview> {
    let team_summary = team.get_unit_summary();
    let mut unit_previews = Vec::new();
    let mut y_start = 60.;
    let mut x = 124.;
    if is_facing_left {
        x += 60.;
    }
    for (unit_type, _count) in team_summary {
        let unit_type = unit_type.to_lowercase();
        //let s_w = data_store.get_sprite_width(&unit_type).unwrap();
        let data = data_store.get_unit_data(&unit_type).unwrap();
        let u_p = UnitPreview::new(unit_type, data.clone(), (x, y_start), is_facing_left);
        unit_previews.push(u_p);
        y_start += 30.;
    }
    unit_previews
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Copy)]
enum TrapSide {
    Left,
    Middle,
    Right,
}

fn create_trap(rng: &mut RNG, trap_type: Option<TrapType>, trap_side: TrapSide) -> Trap {
    let trap_type = trap_type.unwrap_or_else(|| match rng.next_in_range(0, 4) {
        0 => TrapType::Poop,
        1 => TrapType::Acidleak,
        2 => TrapType::Landmine,
        3 => TrapType::Healing,
        4 => TrapType::Spikes,
        _ => unreachable!(),
    });

    let x_range = match trap_side {
        TrapSide::Left => (40, 140),
        TrapSide::Middle => (120, 264),
        TrapSide::Right => (244, 344),
    };

    let x = rng.next_in_range(x_range.0, x_range.1) as f32;
    let y = rng.next_in_range(40, 176) as f32;

    Trap::new((x, y), trap_type)
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
struct UnitDataStore {
    data: HashMap<String, UnitData>,
}

impl UnitDataStore {
    fn new() -> Self {
        UnitDataStore {
            data: HashMap::new(),
        }
    }

    fn add_unit_data(&mut self, data: UnitData) {
        self.data.insert(data.unit_type.clone(), data);
    }

    fn get_unit_data(&self, unit_type: &String) -> Option<&UnitData> {
        self.data.get(unit_type)
    }

    pub fn get_all_unit_types(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }

    pub fn get_sprite_width(&self, unit_type: &str) -> Option<u8> {
        self.data
            .get(unit_type)
            .map(|unit_data| unit_data.sprite_width)
    }

    pub fn load_from_csv(file_path: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let mut store = UnitDataStore::new();
        let mut reader = ReaderBuilder::new()
            .has_headers(false)
            .flexible(true)
            .from_reader(file_path);
        for record in reader.records().skip(1) {
            let record = record?;
            let unit_type = record.get(0).ok_or("Missing damage")?.parse::<String>()?;
            let damage = record.get(1).ok_or("Missing damage")?.parse::<f32>()?;
            let max_health = record.get(2).ok_or("Missing max health")?.parse::<f32>()?;
            let speed = record.get(3).ok_or("Missing speed")?.parse::<f32>()?;
            let range = record.get(4).ok_or("Missing range")?.parse::<f32>()?;
            let attack_time = record.get(5).ok_or("Missing attack time")?.parse::<u16>()?;
            let splash_area = record.get(6).ok_or("Missing splash area")?.parse::<f32>()?;
            let sprite_width = record.get(7).ok_or("Missing sprite width")?.parse::<u8>()?;
            let box_x = record.get(8).ok_or("Missing box_x")?.parse::<u8>()?;
            let box_y = record.get(9).ok_or("Missing box_y")?.parse::<u8>()?;
            let box_w = record.get(10).ok_or("Missing box_w")?.parse::<u8>()?;
            let box_h = record.get(11).ok_or("Missing box_h")?.parse::<u8>()?;
            let bounding_box = (box_x, box_y, box_w, box_h);
            let attributes = record
                .get(12)
                .map(|s| {
                    s.split(',')
                        .filter_map(|attr| attr.parse::<Attribute>().ok())
                        .collect()
                })
                .unwrap_or_else(Vec::new);
            let unit_data = UnitData {
                unit_type,
                damage,
                max_health,
                speed,
                range,
                attack_time,
                splash_area,
                sprite_width,
                bounding_box,
                attributes,
            };
            store.add_unit_data(unit_data);
        }
        Ok(store)
    }
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
struct SimulationResult {
    seed: u32,
    living_units: Vec<String>,
    winning_team: Option<u8>,
    num_frames: u32,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
struct Battle {
    team_0: Team,
    team_1: Team,
    team_seed: u32,
    battle_seed: Option<u32>,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
struct UserStats {
    points: i32,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
struct MatchData {
    timer: u32,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
struct TeamChoiceCounter {
    pub team_0: i32,
    pub team_1: i32,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct ScreenSquare {
    pub x: i32,
    pub y: i32,
    pub opacity: f32, // 0.0 is clear, 1.0 is black
    pub index: usize, // To help with randomization
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]

pub struct Transition {
    pub squares: Vec<ScreenSquare>,
    pub squares_to_change: Vec<usize>, // Indexes of squares in order
    pub square_size: i32,
    pub current_timer: u32,
    pub timer_max: u32,         // How many frames between each square changing
    pub transitioning_in: bool, // true = going to black, false = going to clear
    pub ready_for_scene_change: bool,
    pub complete: bool,
}

impl Transition {
    pub fn new(rng: &mut RNG) -> Self {
        let square_size = 4; // Changed from 24 to 4
        let cols = 384 / square_size;
        let rows = 216 / square_size;

        // Create all squares
        let mut squares = Vec::new();
        let mut index = 0;
        for row in 0..rows {
            for col in 0..cols {
                squares.push(ScreenSquare {
                    x: col * square_size,
                    y: row * square_size,
                    opacity: 0.0,
                    index,
                });
                index += 1;
            }
        }

        // Create randomized order for squares
        let mut squares_to_change: Vec<usize> = (0..squares.len()).collect();
        for i in (1..squares_to_change.len()).rev() {
            let j: usize = rng.next_in_range(0, i as u32) as usize;
            squares_to_change.swap(i, j);
        }

        Transition {
            squares,
            squares_to_change,
            square_size,
            current_timer: 0,
            timer_max: 1, // Change to 1 since we'll do multiple per frame
            transitioning_in: true,
            ready_for_scene_change: false,
            complete: false,
        }
    }

    pub fn update(&mut self) {
        if self.complete {
            return;
        }

        self.current_timer += 1;
        if self.current_timer >= self.timer_max {
            self.current_timer = 0;

            // Change multiple squares per frame
            for _ in 0..150 {
                if !self.squares_to_change.is_empty() {
                    let square_index = self.squares_to_change.pop().unwrap();
                    self.squares[square_index].opacity =
                        if self.transitioning_in { 1.0 } else { 0.0 };
                } else if self.transitioning_in {
                    // Everything is black now
                    self.ready_for_scene_change = true;
                    break;
                } else {
                    self.complete = true;
                    break;
                }
            }
        }
    }

    pub fn start_transition_out(&mut self, rng: &mut RNG) {
        self.transitioning_in = false;
        self.ready_for_scene_change = false;
        self.squares_to_change = (0..self.squares.len()).collect();
        // Rerandomize order for transition out
        for i in (1..self.squares_to_change.len()).rev() {
            let j = rng.next_in_range(0, i as u32) as usize;
            self.squares_to_change.swap(i, j);
        }
    }

    pub fn draw(&self) {
        for square in &self.squares {
            let mut color: usize = 0x3637f0ff;
            if square.opacity == 0.0 {
                color = 0x00000000;
            } else {
                color = 0x696682ff;
            }
            rect!(
                x = square.x,
                y = square.y,
                w = self.square_size as f32,
                h = self.square_size as f32,
                color = color,
            );
        }
    }
}

#[macro_export]
macro_rules! power_text {
   // Basic version without formatting args
   ($text:expr) => {{
       $crate::power_text!($text,)
   }};

   // Version with named parameters
   ($text:expr, $( $key:ident = $val:expr ),* $(,)*) => {{
       let mut x: i32 = 0;
       let mut y: i32 = 0;
       let mut font: Font = Font::M;
       let mut color: u32 = 0xffffffff;
       let mut absolute: bool = false;
       let mut drop_shadow: Option<u32> = None;
       let mut underline: bool = false;
       let mut strikethrough: bool = false;
       let mut center_width: Option<i32> = None;  // Width of area to center within

       $($crate::paste::paste!{ [< $key >] = power_text!(@coerce $key, $val); })*


       let char_width = match font {
        Font::S => 5,
        Font::M => 5,
        Font::L => 8,
        Font::XL => 16,
     };
     let char_height = match font {
        Font::S => 5,
        Font::M => 7,
        Font::L => 8,
        Font::XL => 16,
     };
     let drop_shadow_distance = match font {
        Font::S => 1,
        Font::M => 1,
        Font::L => 2,
        Font::XL => 2,
     };
       // Handle centering if specified
       if let Some(width) = center_width {

           let text_width = (char_width * $text.len() as i32);
           x += (width - text_width) / 2;
       }
        // Handle drop shadow if specified
        if let Some(shadow_color) = drop_shadow {
            text!($text,
                x = x,
                y = y + drop_shadow_distance,
                color = shadow_color,
                font = font,
                absolute = absolute
            );
        }
       // Draw main text
       text!($text,
           x = x,
           y = y,
           color = color,
           font = font,
           absolute = absolute
       );

       // Handle underline if specified
       if underline {
        let text_width = char_width * $text.len() as i32;
        rect!(x = x, y=y+char_height, color=color, w = text_width, h = 1);
        if let Some(shadow_color) = drop_shadow{
            rect!(x = x, y=y+char_height+1, color=shadow_color, w = text_width, h = 1);

        }
       }

       if strikethrough{
        if $text.len() != 0{
        let  text_width = char_width * $text.len() as i32 + 1;

        rect!(x =x-1, y = y + char_height/2, color = color, w = text_width, h = 1);
        }
       }
   }};

   // Add coercion rules for new parameters
   (@coerce x, $val:expr) => { $val as i32 };
   (@coerce y, $val:expr) => { $val as i32 };
   (@coerce absolute, $val:expr) => { $val as bool };
   (@coerce font, $val:expr) => { $val as Font };
   (@coerce color, $val:expr) => { $val as u32 };
   (@coerce drop_shadow, $val:expr) => { Some($val as u32) };
   (@coerce underline, $val:expr) => { $val as bool };
   (@coerce strikethrough, $val:expr) => { $val as bool };
   (@coerce center_width, $val:expr) => { Some($val as i32) };

}
