mod rng;
mod trap;
mod unit;

use csv::{Reader, ReaderBuilder};
use rng::*;
use std::cmp::{max, Ordering};
use std::collections::HashMap;
use std::fmt::{format, Display};
use std::str::FromStr;
use trap::*;
use unit::*;

const UNIT_DATA_CSV: &[u8] = include_bytes!("../resources/unit-data.csv");
const DAMAGE_EFFECT_TIME: u32 = 12;
//avg number of units to balance each generated team around
const TEAM_POWER_MULTIPLIER: f32 = 25.0;
const PREMATCH_TIME: u32 = 3600;

const UNIT_ANIM_SPEED: i32 = 8;
const MAX_Y_ATTACK_DISTANCE: f32 = 10.;
const FOOTPRINT_LIFETIME: u32 = 240;
const MAP_BOUNDS: (f32, f32, f32, f32) = (10.0, 340.0, 0.0, 200.0);

//colors
const POO_BROWN: usize = 0x654321FF;
const ACID_GREEN: usize = 0x32CD32FF;
const WHITE: usize = 0xffffffff;
const DAMAGE_TINT_RED: usize = 0xb9451dff;

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
        selected_team_index: i32,
        simulation_result: Option<SimulationResult>,
        //test variables
        auto_assign_teams: bool,
        user: UserStats,
        last_winning_team: Option<Team>,
        prematch_timer: u32,
    } = {
        Self {
            phase: Phase::SelectionScreen,
            units: Vec::new(),
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
            //replace this number with a program number later
            rng: RNG::new(12345),
            data_store: None,
            auto_assign_teams: true,
            selected_team_index: 0,
            simulation_result: None,
            user: UserStats{points: 100},
            last_winning_team: None,
            //TODO: This should maybe come from TURBO OS
            prematch_timer: PREMATCH_TIME,
        }
    }
}

turbo::go!({
    let mut state = GameState::load();
    clear!(0x8f8cacff);
    if state.phase == Phase::SelectionScreen {
        //initialize the data store if it is blank
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
            //set the seed for the rng as a random number. TODO: get this from turbo os
            state.rng = RNG::new(rand());
        }
        //if teams are not assigned, check if we should auto assign or not
        if state.teams.len() == 0 {
            if state.auto_assign_teams {
                let data_store = state
                    .data_store
                    .as_ref()
                    .expect("Data store should be loaded");

                let (team1, team2) = if let Some(winning_team) = &state.last_winning_team {
                    // Use winning team and generate a matching opponent
                    (
                        winning_team.clone(),
                        generate_team(
                            &data_store,
                            &mut state.rng,
                            Some(winning_team),
                            "Challenger".to_string(),
                        ),
                    )
                } else {
                    // Generate two fresh teams
                    (
                        generate_team(&data_store, &mut state.rng, None, "Pixel Peeps".to_string()),
                        //TODO: This team needs to ensure it isn't including same unit type as the other team
                        generate_team(&data_store, &mut state.rng, None, "Battle Bois".to_string()),
                    )
                };
                state
                    .unit_previews
                    .extend(create_unit_previews(&team1, false, data_store));
                state
                    .unit_previews
                    .extend(create_unit_previews(&team2, true, data_store));
                state.teams = Vec::new();
                state.teams.push(team1);
                state.teams.push(team2);
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
            }
        }
        if state.auto_assign_teams {
            if state.prematch_timer > 0 {
                state.prematch_timer -= 1;
            } else {
                start_match(&mut state);
            }
            draw_prematch_timer(state.prematch_timer);
            //draw each unit based on the teams
            draw_assigned_team_info(&mut state);
            for u in &mut state.unit_previews {
                u.update();
                u.draw();
            }
            draw_points_prebattle_screen(state.user.points);
        }
        if !state.auto_assign_teams {
            draw_team_info_and_buttons(&mut state);
        }
        let gp = gamepad(0);
        if gp.start.just_pressed() {
            start_match(&mut state);
        }

        //move camera if you press up and down
        if gp.down.pressed() {
            set_cam!(y = cam!().1 + 3);
        } else if gp.up.pressed() {
            set_cam!(y = cam!().1 - 3);
        }
    } else if state.phase == Phase::Battle {
        //run the simulation once.
        //TODO: This might explode if there's a tie so lets find a better way to do this
        if state.simulation_result.is_none() {
            simulate_battle(&mut state);
            // //store the state somehow
            // let stored_state = state.clone();
            // let mut winning_team = None;
            // state.rng = RNG::new(state.rng.seed);

            // while winning_team.is_none() {
            //     step_through_battle(&mut state);
            //     winning_team = has_some_team_won(&state.units);
            // }
            // let simulation_result = SimulationResult {
            //     living_units: all_living_units(&state.units),
            //     seed: state.rng.seed,
            // };
            // //commit points change
            // //TODO: Make this on turbo OS
            // let is_won = winning_team.unwrap() == state.selected_team_index;
            // let points_change = calculate_points_change(is_won);
            // commit_points_change(&mut state.user, points_change);
            // let u = state.user;
            // //reset the state here
            // state = stored_state;
            // //and assign the simulation result. Then we'll do the actual simulation
            // state.simulation_result = simulation_result;
            // //carry over the points change
            // state.user = u;
            // //assign the winning team to last_winning_team so it stays for the next round
            // state.last_winning_team = winning_team.map(|index| state.teams[index as usize].clone());
            // if let Some(winning_team) = &mut state.last_winning_team {
            //     winning_team.win_streak += 1;
            // }
            // //assign the rng to the same seed you used for the simulation, so it matches
            // state.rng = RNG::new(state.rng.seed);
        } else {
            //after we did the simulation, step through one frame at a time until it's over
            step_through_battle(&mut state);
        }
        //some testing code to try out attack strategies
        let gp = gamepad(0);
        if gp.a.just_pressed() {
            //get the units on team 1 and make them flee
            for u in &mut state.units {
                if u.team == 1 {
                    u.attack_strategy = AttackStrategy::Flee { timer: 3 };
                    //u.attack_strategy = AttackStrategy::AttackClosest;
                }
            }
        }

        //temp code to create traps
        // let gp = gamepad(0);
        // if gp.a.just_pressed() {
        //     state.traps.push(create_trap(&mut state.rng));
        // }
        // if gp.b.just_pressed() {
        //     state.traps.push(create_trap(&mut state.rng));
        //     state.traps.push(create_trap(&mut state.rng));
        //     state.traps.push(create_trap(&mut state.rng));
        //     state.traps.push(create_trap(&mut state.rng));
        //     state.traps.push(create_trap(&mut state.rng));
        //     state.traps.push(create_trap(&mut state.rng));
        //     state.traps.push(create_trap(&mut state.rng));
        //     state.traps.push(create_trap(&mut state.rng));
        //     state.traps.push(create_trap(&mut state.rng));
        //     state.traps.push(create_trap(&mut state.rng));
        // }
        ///////////////DRAW CODE//////////////

        //Draw craters beneath everything
        for c in &state.craters {
            c.draw();
        }
        //sprite!("crater_01", x=100, y=100, color = 0xFFFFFF80);
        //Draw footprints beneath units
        for u in &mut state.units {
            for fp in &mut u.footprints {
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
        //draw end game text
        let mut winning_team = has_some_team_won(&state.units);
        if winning_team.is_some() {
            let index: usize = winning_team.take().unwrap_or(-1) as usize;
            let mut is_win = true;
            if index != state.selected_team_index as usize {
                is_win = false;
            }
            draw_end_animation(is_win);
            //TODO: Add some delay here
            let points_change = calculate_points_change(is_win);
            draw_points_end_screen(state.user.points, points_change);
            //add a restart game button here
            let restart_button = Button::new(
                String::from("AGAIN!"),
                (20., 175.),
                (50., 25.),
                GameEvent::RestartGame(),
            );
            restart_button.draw();
            restart_button.handle_click(&mut state);
            for unit in &mut state.units {
                if unit.state != UnitState::Dead {
                    unit.start_cheering();
                }
            }
            let living_units = all_living_units(&state.units);
            if let Some(sim_result) = &state.simulation_result {
                if living_units.len() != sim_result.living_units.len() {
                    text!(
                        "SIMULATION DOES NOT MATCH",
                        x = 50,
                        y = 50,
                        color = DAMAGE_TINT_RED
                    );
                }
            }
        }
        //TODO: clean this up
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
        if state.selected_team_index == 0 {
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
        if state.selected_team_index == 1 {
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
                state.selected_team_index = team_num;
                //TODO: Do something with turbo OS here
                // create_units_for_all_teams(&mut state);
                // state.phase = Phase::Battle;
            }
            GameEvent::RestartGame() => {
                let t = state.last_winning_team;
                let u = state.user;
                state = GameState::default();
                //retain these values between rounds
                state.last_winning_team = t;
                state.user = u;
            }
        }
    }
    let gp = gamepad(0);
    if gp.right.just_pressed() {
        state = GameState::default();
        state.auto_assign_teams = false;
    }
    if gp.left.just_pressed() {
        state = GameState::default();
        state.auto_assign_teams = true;
    }
    state.save();
});

fn draw_end_animation(is_win: bool) {
    let center_x = canvas_size!()[0] / 2;
    let center_y = canvas_size()[1] / 2 - 16;
    if is_win {
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

fn simulate_battle(state: &mut GameState) {
    // Store initial state
    let initial_state = state.clone();

    // Run simulation with fresh RNG
    state.rng = RNG::new(state.rng.seed);

    let winning_team_index = loop {
        step_through_battle(state);
        if let Some(winner_idx) = has_some_team_won(&state.units) {
            break winner_idx;
        }
    };

    // Create simulation result
    let simulation_result = SimulationResult {
        living_units: all_living_units(&state.units),
        seed: state.rng.seed,
    };

    // Handle points
    let is_won = winning_team_index == state.selected_team_index;
    let points_change = calculate_points_change(is_won);
    commit_points_change(&mut state.user, points_change);
    let updated_user = state.user.clone();

    // Reset state but keep necessary changes
    *state = initial_state;
    state.simulation_result = Some(simulation_result);
    state.user = updated_user;

    // Store winning team data using the index
    state.last_winning_team = Some(state.teams[winning_team_index as usize].clone());

    // Update win streak
    if let Some(winning_team) = &mut state.last_winning_team {
        winning_team.win_streak += 1;
    }

    // Reset RNG to match simulation
    state.rng = RNG::new(state.rng.seed);
}

fn step_through_battle(state: &mut GameState) {
    let units_clone = state.units.clone();
    //=== MOVEMENT AND ATTACKING ===
    //go through each unit, see what it wants to do, and handle all actions from here
    for unit in &mut state.units {
        if unit.state == UnitState::Idle {
            match unit.attack_strategy {
                AttackStrategy::AttackClosest => {
                    //find closest enemy
                    if let Some(index) = closest_enemy_index(&unit, &units_clone) {
                        if unit.is_unit_in_range(&units_clone[index]) {
                            state.attacks.push(unit.start_attack(units_clone[index].id));
                            if unit.pos.0 > units_clone[index].pos.0 {
                                unit.is_facing_left = true;
                            } else {
                                unit.is_facing_left = false;
                            }
                        } else {
                            unit.set_new_target_move_position(
                                &units_clone[index].pos,
                                &mut state.rng,
                            );
                        }
                        unit.target_id = units_clone[index].id;
                    }
                }
                AttackStrategy::TargetLowestHealth => {
                    //check if target id is dead or none
                    let mut target_unit = find_unit_by_id(&units_clone, Some(unit.target_id));
                    if target_unit.is_some() && target_unit.unwrap().health > 0. {
                        if unit.is_unit_in_range(&target_unit.unwrap()) {
                            state
                                .attacks
                                .push(unit.start_attack(target_unit.unwrap().id));
                            //assign the units target id as this unit now
                            unit.target_id = target_unit.unwrap().id;
                            if unit.pos.0 > target_unit.unwrap().pos.0 {
                                unit.is_facing_left = true;
                            } else {
                                unit.is_facing_left = false;
                            }
                        } else {
                            unit.set_new_target_move_position(
                                &target_unit.unwrap().pos,
                                &mut state.rng,
                            );
                        }
                    } else {
                        //find a unit with lowest health and set it as your target and move toward that position
                        target_unit =
                            lowest_health_closest_enemy_unit(&units_clone, unit.team, unit.pos);
                        if target_unit.is_some() {
                            unit.set_new_target_move_position(
                                &target_unit.unwrap().pos,
                                &mut state.rng,
                            );
                            unit.target_id = target_unit.unwrap().id;
                        }
                    }
                }
                AttackStrategy::Flank { ref mut stage } => {
                    //if target is none, choose lowest health enemy and set target
                    let mut target_unit = find_unit_by_id(&units_clone, Some(unit.target_id));
                    let max_dist = 20.0;
                    if target_unit.is_none() || target_unit.unwrap().health == 0. {
                        target_unit =
                            lowest_health_closest_enemy_unit(&units_clone, unit.team, unit.pos);
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
                            unit.set_new_target_move_position(&target_pos, &mut state.rng);
                        } else {
                            if *stage == FlankStage::Vertical {
                                *stage = FlankStage::Horizontal;
                            } else {
                                unit.attack_strategy = AttackStrategy::TargetLowestHealth;
                            }
                        }
                    }
                }
                AttackStrategy::SeekTarget => {
                    //set target unit to closest enemy one time
                    let mut target_unit = find_unit_by_id(&units_clone, Some(unit.target_id));
                    if target_unit.is_none() || target_unit.unwrap().health == 0. {
                        target_unit = closest_enemy_unit(&units_clone, unit.team, unit.pos);
                        if target_unit.is_some() {
                            unit.target_id = target_unit.unwrap().id;
                        }
                    }
                    //if you already have a target unit, then try to fight it
                    else {
                        if unit.is_unit_in_range(&target_unit.unwrap()) {
                            state
                                .attacks
                                .push(unit.start_attack(target_unit.unwrap().id));
                            //assign the units target id as this unit now
                            unit.target_id = target_unit.unwrap().id;
                            if unit.pos.0 > target_unit.unwrap().pos.0 {
                                unit.is_facing_left = true;
                            } else {
                                unit.is_facing_left = false;
                            }
                        } else {
                            unit.set_new_target_move_position(
                                &target_unit.unwrap().pos,
                                &mut state.rng,
                            );
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
                        unit.set_new_target_move_position(&new_target, &mut state.rng);
                    }
                }

                _ => {
                    // Default case
                }
            }
        }
        unit.update();
        //check if the unit is on top of a trap
        for trap in &mut state.traps {
            if distance_between(unit.foot_position(), trap.pos) < (trap.size / 2.)
                && trap.is_active()
            {
                if trap.trap_type == TrapType::Poop {
                    unit.footprint_status = FootprintStatus::Poopy;
                } else if trap.trap_type == TrapType::Acidleak {
                    let attack = Attack::new(unit.id, 1., trap.pos, trap.damage, 0., 1, Vec::new());
                    unit.take_damage(&attack, &mut state.rng);
                    unit.footprint_status = FootprintStatus::Acid;
                } else if trap.trap_type == TrapType::Landmine {
                    if let Some(closest_unit_index) =
                        closest_unit_to_position(trap.pos, &units_clone)
                    {
                        let attack = Attack::new(
                            units_clone[closest_unit_index].id,
                            1.,
                            trap.pos,
                            trap.damage,
                            8.,
                            1,
                            Vec::new(),
                        );
                        state.attacks.push(attack);
                        trap.set_inactive();
                        turbo::println!("TRAP POS {}, {}", trap.pos.0, trap.pos.1);
                    }
                }
            }
        }
    }
    //go through attacks and update, then draw
    state.attacks.retain_mut(|attack| {
        let should_keep = !attack.update(&units_clone);
        //attack.draw();

        if !should_keep {
            //deal the actual damage here
            if attack.splash_area == 0. {
                if let Some(unit_index) = state
                    .units
                    .iter()
                    .position(|u| u.id == attack.target_unit_id)
                {
                    let unit = &mut state.units[unit_index];
                    unit.take_damage(&attack, &mut state.rng);
                    if unit.health <= 0. {
                        if unit.data.has_attribute(&Attribute::ExplodeOnDeath) {
                            let mut explosion_offset = (-24., -24.);
                            if unit.flip_x() {
                                explosion_offset.0 = -24.;
                            }
                            let explosion_pos = (
                                unit.pos.0 + explosion_offset.0,
                                unit.pos.1 + explosion_offset.1,
                            );
                            let mut explosion = AnimatedSprite::new(explosion_pos, false);
                            explosion.set_anim("explosion".to_string(), 32, 14, 5, false);
                            state.explosions.push(explosion);
                        }
                    }
                }
            }
            //if it has splash area, then look for all enemy units within range
            if attack.splash_area > 0. {
                let team = find_unit_by_id(&state.units, Some(attack.target_unit_id))
                    .unwrap()
                    .team;
                for unit in &mut state.units {
                    if distance_between(attack.pos, unit.pos) <= attack.splash_area
                        && unit.state != UnitState::Dead
                        && unit.team == team
                    {
                        unit.take_damage(&attack, &mut state.rng);
                        if unit.health <= 0.0 {
                            if unit.data.has_attribute(&Attribute::ExplodeOnDeath) {
                                let mut explosion_offset = (-24., -24.);
                                if unit.flip_x() {
                                    explosion_offset.0 = -24.;
                                }
                                let explosion_pos = (
                                    unit.pos.0 + explosion_offset.0,
                                    unit.pos.1 + explosion_offset.1,
                                );
                                let mut explosion = AnimatedSprite::new(explosion_pos, false);
                                explosion.set_anim("explosion".to_string(), 32, 14, 5, false);
                                state.explosions.push(explosion);
                            }
                        }
                    }
                }
            }
            if attack.attributes.contains(&Attribute::ExplosiveAttack) {
                //create explosion
                let explosion_offset = (-24., -24.);
                let explosion_pos = (
                    attack.pos.0 + explosion_offset.0,
                    attack.pos.1 + explosion_offset.1,
                );
                let mut explosion = AnimatedSprite::new(explosion_pos, false);
                explosion.set_anim("explosion".to_string(), 32, 14, 5, false);
                state.explosions.push(explosion);
                //make a crater
                let crater_pos = (explosion_pos.0 + 16., explosion_pos.1 + 16.);
                let mut crater = AnimatedSprite::new(crater_pos, false);

                crater.set_anim("crater_01".to_string(), 16, 1, 1, true);
                crater.animator.change_tint_color(0xFFFFFF80);
                state.craters.push(crater);
            }
        }

        should_keep
    });
    //go through traps, update and draw
    for trap in &mut state.traps {
        trap.update();
        trap.draw();
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

fn lowest_health_enemy_unit(units: &Vec<Unit>, team: i32) -> Option<&Unit> {
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

fn lowest_health_closest_enemy_unit(
    units: &Vec<Unit>,
    team: i32,
    pos: (f32, f32),
) -> Option<&Unit> {
    if units.is_empty() {
        return None;
    }

    units
        .iter()
        .filter(|unit| unit.team != team && unit.health > 0.0)
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
fn calculate_points_change(is_won: bool) -> i32 {
    if is_won {
        return 10;
    }
    -10
}

fn commit_points_change(user: &mut UserStats, points_change: i32) {
    user.points += points_change;
    //can do some turbo OS stuff here
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
    let sign = if points_change > 0 { "+" } else { "-" };
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
        s_w: i32,
        num_frames: i32,
        loops_per_frame: i32,
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
        self.animator.draw(self.pos, true)
    }
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
struct Attack {
    target_unit_id: u32,
    speed: f32,
    pos: (f32, f32),
    damage: f32,
    splash_area: f32,
    size: i32,
    attributes: Vec<Attribute>,
}

impl Attack {
    //new
    fn new(
        target_unit_id: u32,
        speed: f32,
        pos: (f32, f32),
        damage: f32,
        splash_area: f32,
        size: i32,
        attributes: Vec<Attribute>,
    ) -> Self {
        Self {
            target_unit_id,
            speed,
            pos,
            damage,
            splash_area,
            size,
            attributes,
        }
    }
    fn update(&mut self, units: &Vec<Unit>) -> bool {
        let distance = 0.;

        // Get the target unit's position
        let target_unit = find_unit_by_id(units, Some(self.target_unit_id));
        if target_unit.is_some() {
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
        false
    }

    fn draw(&self) {
        // Draw a small red circle at the current position (x, y)
        circ!(
            x = self.pos.0 as i32,
            y = self.pos.1 as i32,
            d = 5 * self.size,
            color = 0xff0000ff
        ); // Diameter 5, Red color
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

fn start_match(state: &mut GameState) {
    create_units_for_all_teams(state);
    state.phase = Phase::Battle;
    set_cam!(x = 192, y = 108);
}

fn closest_enemy_unit(units: &Vec<Unit>, team: i32, pos: (f32, f32)) -> Option<&Unit> {
    units
        .iter()
        .filter(|unit| unit.team != team && unit.health > 0.0)
        .min_by(|&a, &b| {
            let dist_a = distance_between(pos, a.pos);
            let dist_b = distance_between(pos, b.pos);
            dist_a
                .partial_cmp(&dist_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

fn closest_enemy_index(unit: &Unit, units: &Vec<Unit>) -> Option<usize> {
    units
        .iter()
        .enumerate()
        .filter(|(_, other_unit)| {
            other_unit.team != unit.team && // Filter out units on the same team
            other_unit.health > 0.0 &&      // Filter out dead units
            !std::ptr::eq(unit, *other_unit) // Filter out the unit itself
        })
        .min_by(|(_, a), (_, b)| {
            let dist_a = distance_between(unit.pos, a.pos);
            let dist_b = distance_between(unit.pos, b.pos);
            dist_a
                .partial_cmp(&dist_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(index, _)| index)
}

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

fn has_some_team_won(units: &Vec<Unit>) -> Option<i32> {
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
        if *team_index == state.selected_team_index as usize {
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
    text!(
        "VS.",
        x = 150,
        y = y_start + 45,
        font = Font::L,
        color = 0xADD8E6ff
    );
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
        color = 0x696682ff
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
    ChooseTeam(i32),
    RestartGame(),
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
enum ObstacleShape {
    Square,
    Circle,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
struct Animator {
    //current animation
    cur_anim: Animation,
    anim_timer: i32,
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
        let mut frame_index = self.anim_timer / self.cur_anim.loops_per_frame; // Calculate the frame index
        frame_index = frame_index.clamp(0, self.cur_anim.num_frames - 1);
        let sx = (frame_index * self.cur_anim.s_w)
            .clamp(0, self.cur_anim.s_w * (self.cur_anim.num_frames - 1)); // Calculate the sprite X coordinate

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
        if self.cur_anim.name != new_anim.name {
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
    s_w: i32,
    num_frames: i32,
    loops_per_frame: i32,
    is_looping: bool,
}

impl Animation {
    fn total_animation_time(&self) -> i32 {
        return self.num_frames * self.loops_per_frame;
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

//POWER LEVEL AND TEAM CREATION
fn create_units_for_all_teams(state: &mut GameState) {
    //generate units
    let row_height = 16.0;
    let row_width = 20.0;
    let max_y = 200.0;
    let data_store = state
        .data_store
        .as_ref()
        .expect("Data store should be loaded");
    //shuffle the units in each team
    for team in &mut state.teams {
        shuffle(&mut state.rng, &mut team.units);
    }

    for (team_index, team) in state.teams.iter().enumerate() {
        let mut x_start = if team_index == 0 { 70.0 } else { 270.0 }; // Adjusted starting x for team 1
        let mut y_pos = 60.0;

        for (_i, unit_type) in team.units.iter().enumerate() {
            if y_pos > max_y {
                y_pos = 60.0;

                if team_index == 0 {
                    x_start -= row_width;
                } else {
                    x_start += row_width;
                }
            }
            let pos = (x_start, y_pos);
            let mut unit = Unit::new(
                unit_type.clone(),
                pos,
                team_index as i32,
                &data_store,
                state.next_id,
            );
            unit.set_starting_strategy(&mut state.rng);
            state.units.push(unit);
            state.next_id += 1;
            //let unit = Unit::new(UnitType::Axeman, (0.0, 0.0), 0, &unit_type_store);
            y_pos += row_height;
        }
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
    let power_level = unit_data.max_health
        + (unit_data.damage / (unit_data.attack_time as f32 / 60.0))
        + unit_data.speed;

    let mut final_power = power_level;

    if unit_data.range > 20.0 {
        final_power += 150.0;
    }

    if unit_data.splash_area > 0.0 {
        final_power = final_power * 3.;
    }

    final_power
}

// fn calculate_unit_power_level(data_store: &HashMap<String, UnitData>) -> HashMap<String, f32> {
//     let mut power_levels = HashMap::new();

//     // Find max values for normalization
//     let max_health = data_store
//         .values()
//         .map(|u| u.max_health)
//         .max_by(|a, b| a.partial_cmp(b).unwrap())
//         .unwrap_or(1.0);
//     let max_dps = data_store
//         .values()
//         .map(|u| u.damage / (u.attack_time as f32 / 60.0))
//         .max_by(|a, b| a.partial_cmp(b).unwrap())
//         .unwrap_or(1.0);
//     let max_speed = data_store
//         .values()
//         .map(|u| u.speed)
//         .max_by(|a, b| a.partial_cmp(b).unwrap())
//         .unwrap_or(1.0);

//     for (unit_type, unit_data) in data_store {
//         let normalized_health = (unit_data.max_health / max_health) * 50.0;
//         let dps = unit_data.damage / (unit_data.attack_time as f32 / 60.0);
//         let normalized_dps = (dps / max_dps) * 100.0;
//         let normalized_speed = (unit_data.speed / max_speed) * 10.0;

//         let mut power_level = normalized_health + normalized_dps + normalized_speed;

//         if unit_data.range > 20.0 {
//             power_level += 150.0;
//         }

//         if unit_data.splash_area > 0.0 {
//             power_level = power_level * 3.;
//         }

//         power_levels.insert(unit_type.clone(), power_level);
//     }

//     power_levels
// }

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
    unit_types: &[String], // Changed from &[&String]
    power_levels: &HashMap<String, f32>,
    target_power: f32,
    rng: &mut RNG,
) {
    let mut current_power = 0.0;
    let power1 = &power_levels[&unit_types[0]]; // Added & here
    let power2 = &power_levels[&unit_types[1]]; // Added & here

    // Generate random weights for each unit type
    let weight1 = rng.next_f32();
    let weight2 = 1.0 - weight1;

    while current_power < target_power {
        let remaining_power = target_power - current_power;

        // Use weighted random selection
        let use_first_type = rng.next_f32() < (weight1 / (weight1 + weight2));

        if use_first_type && remaining_power >= *power1 {
            // Added *
            team.units.push(unit_types[0].clone());
            current_power += power1;
        } else if !use_first_type && remaining_power >= *power2 {
            // Added *
            team.units.push(unit_types[1].clone());
            current_power += power2;
        } else {
            // If we can't add either unit without going over, try the other unit
            if !use_first_type && remaining_power >= *power1 {
                // Added *
                team.units.push(unit_types[0].clone());
                current_power += power1;
            } else if use_first_type && remaining_power >= *power2 {
                // Added *
                team.units.push(unit_types[1].clone());
                current_power += power2;
            } else {
                // If we still can't add either unit, stop adding units
                break;
            }
        }
    }

    // Ensure at least one of each unit type
    if !team.units.contains(&unit_types[0]) {
        team.units.push(unit_types[0].clone());
    }
    if !team.units.contains(&unit_types[1]) {
        team.units.push(unit_types[1].clone());
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

fn create_trap(rng: &mut RNG) -> Trap {
    //choose a random trap and a random position within some bounds
    let random_number = rng.next_in_range(0, 2);

    let trap_type = match random_number {
        0 => TrapType::Poop,
        1 => TrapType::Acidleak,
        2 => TrapType::Landmine,
        3 => TrapType::Healing,
        4 => TrapType::Spikes,
        _ => unreachable!(), // This should never happen due to the range we specified
    };
    let x_bounds = (100, 284);
    let y_bounds = (40, 176);
    let x = rng.next_in_range(x_bounds.0, x_bounds.1);
    let y = rng.next_in_range(y_bounds.0, y_bounds.1);
    let trap = Trap::new((x as f32, y as f32), trap_type);
    trap
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

    pub fn get_sprite_width(&self, unit_type: &str) -> Option<i32> {
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
            let attack_time = record.get(5).ok_or("Missing attack time")?.parse::<i32>()?;
            let splash_area = record.get(6).ok_or("Missing splash area")?.parse::<f32>()?;
            let sprite_width = record
                .get(7)
                .ok_or("Missing sprite width")?
                .parse::<i32>()?;
            let box_x = record.get(8).ok_or("Missing box_x")?.parse::<i32>()?;
            let box_y = record.get(9).ok_or("Missing box_y")?.parse::<i32>()?;
            let box_w = record.get(10).ok_or("Missing box_w")?.parse::<i32>()?;
            let box_h = record.get(11).ok_or("Missing box_h")?.parse::<i32>()?;
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
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
struct UserStats {
    points: i32,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
struct MatchData {
    timer: u32,
}