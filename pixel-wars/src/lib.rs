mod backend;
mod deckbuilder;
mod rng;
mod trap;
mod unit;

use backend::*;
use csv::{Reader, ReaderBuilder};
use deckbuilder::*;
use os::server;
use rng::*;
use std::cmp::{max, Ordering};
use std::collections::HashMap;
use std::fmt::{format, Display};
use std::str::FromStr;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use trap::*;
use unit::*;

const UNIT_DATA_CSV: &[u8] = include_bytes!("../resources/unit-data.csv");
const DAMAGE_EFFECT_TIME: u32 = 12;
//avg number of units to balance each generated team around
const TEAM_POWER_MULTIPLIER: f32 = 25.0;
const TEAM_SELECTION_TIME: u32 = 3600;
const BATTLE_COUNTDOWN_TIME: u32 = 240;
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

const UNIT_ANIM_SPEED: i32 = 8;
const MAX_Y_ATTACK_DISTANCE: f32 = 10.;
const FOOTPRINT_LIFETIME: u32 = 240;
const MAP_BOUNDS: (f32, f32, f32, f32) = (10.0, 340.0, 0.0, 200.0);

//colors
const POO_BROWN: usize = 0x654321FF;
const ACID_GREEN: usize = 0x32CD32FF;
const WHITE: usize = 0xffffffff;
const DAMAGE_TINT_RED: usize = 0xb9451dff;
const OFF_BLACK: u32 = 0x1A1A1AFF;
const DARK_GRAY: u32 = 0x808080FF;
const LIGHT_GRAY: u32 = 0xA6A6A6FF;
const SHADOW_COLOR: usize = 0x696682ff;
const _HOVER_GRAY: u32 = 0xCCCCCCFF;

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
        dbphase: DBPhase,
        shop: Vec<UnitPack>,
        round: u32,
        num_picks: u32,
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
        //test variables
        auto_assign_teams: bool,
        user: UserStats,
        last_winning_team: Option<Team>,
        team_selection_timer: u32,
        team_generation_requested: bool,
        previous_battle: Option<Battle>,
        battle_countdown_timer: u32,
        battle_simulation_requested: bool,
    } = {
        Self {
            dbphase: DBPhase::Title,
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
            user: UserStats{points: 100},
            last_winning_team: None,
            team_selection_timer: TEAM_SELECTION_TIME,//TODO: This should come from TURBO OS
            team_generation_requested: false,
            previous_battle: None,
            battle_countdown_timer: BATTLE_COUNTDOWN_TIME,
            battle_simulation_requested: false,
        }
    }
}

turbo::go!({
    let mut state = GameState::load();
    dbgo(&mut state);

    state.save();
});

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

//Local simulation - not using anymore
fn _simulate_battle(state: &mut GameState) {
    // Store initial state
    let initial_state = state.clone();
    // for u in &mut initial_state.units {
    //     u.display = None;
    // }
    // turbo::println!("BYTES: {}", initial_state.units.try_to_vec().unwrap().len());
    // Run simulation with fresh RNG
    state.rng = RNG::new(state.rng.seed);
    let mut i = 1;
    let winning_team_index = loop {
        step_through_battle(
            &mut state.units,
            &mut state.attacks,
            &mut state.traps,
            &mut state.explosions, //look into a callback to replace this
            &mut state.craters,    //look into a callback to replace this
            &mut state.rng,
            &Vec::new(),
        );
        i += 1;
        if i > 10000 {
            log!("Simulation is taking too long!!");
            panic!("ITS TAKING TOO LONG");
        }
        if let Some(winner_idx) = has_some_team_won(&state.units) {
            break winner_idx;
        }
    };

    // Create simulation result
    let simulation_result = SimulationResult {
        living_units: all_living_units(&state.units),
        seed: state.rng.seed,
        winning_team: Some(winning_team_index),
    };

    // Handle points
    if state.selected_team_index.is_some() {
        let is_won = state.selected_team_index == Some(winning_team_index);
        //let points_change = calculate_points_change(is_won);
        commit_points_change(&mut state.user, is_won);
    }
    //let updated_user = state.user.clone();
    // Reset state
    *state = initial_state;
    state.simulation_result = Some(simulation_result);
    //state.user = updated_user;

    state.last_winning_team = Some(state.teams[winning_team_index as usize].clone());

    // Update win streak
    if let Some(winning_team) = &mut state.last_winning_team {
        winning_team.win_streak += 1;
    }

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
    artifacts: &Vec<Artifact>,
) {
    let units_clone = units.clone();
    //=== MOVEMENT AND ATTACKING ===
    //go through each unit, see what it wants to do, and handle all actions from here
    for unit in &mut *units {
        if unit.state == UnitState::Idle {
            match unit.attack_strategy {
                AttackStrategy::AttackClosest => {
                    //find closest enemy
                    if let Some(index) = closest_enemy_index(&unit, &units_clone) {
                        if unit.is_unit_in_range(&units_clone[index]) {
                            //TODO: modify attacks based on artifact list for this team
                            //Add artifact list to the team and set to vec::new
                            //if it isn't none, then call modify damage
                            let mut attack = unit.start_attack(units_clone[index].id);
                            let attack =
                                modify_damage_from_artifacts(attack, &units_clone, artifacts);

                            attacks.push(unit.start_attack(units_clone[index].id));
                            if unit.pos.0 > units_clone[index].pos.0 {
                                if let Some(display) = unit.display.as_mut() {
                                    display.is_facing_left = true;
                                }
                            } else {
                                if let Some(display) = unit.display.as_mut() {
                                    display.is_facing_left = false;
                                }
                            }
                        } else {
                            unit.set_new_target_move_position(&units_clone[index].pos, rng);
                        }
                        unit.target_id = units_clone[index].id;
                    }
                }
                AttackStrategy::TargetLowestHealth => {
                    //check if target id is dead or none
                    let mut target_unit = find_unit_by_id(&units_clone, Some(unit.target_id));
                    if target_unit.is_some() && target_unit.unwrap().health > 0. {
                        if unit.is_unit_in_range(&target_unit.unwrap()) {
                            attacks.push(unit.start_attack(target_unit.unwrap().id));
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
                        target_unit =
                            lowest_health_closest_enemy_unit(&units_clone, unit.team, unit.pos);
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
                        target_unit =
                            lowest_health_ranged_enemy_unit(&units_clone, unit.team, unit.pos);
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
                        //if there is no ranged unit on the enemy team, then just go for lowest health.
                    } else {
                        unit.attack_strategy = AttackStrategy::TargetLowestHealth;
                    }
                }
                AttackStrategy::SeekTarget => {
                    //set target unit to closest enemy one time
                    let mut target_unit = find_unit_by_id(&units_clone, Some(unit.target_id));
                    if target_unit.is_none() || target_unit.unwrap().health == 0. {
                        target_unit = closest_enemy_unit(&units_clone, unit.team, unit.pos);
                        if target_unit.is_some() {
                            unit.target_id = target_unit.unwrap().id;
                        } else {
                            unit.attack_strategy = AttackStrategy::AttackClosest;
                        }
                    }
                    //if you already have a target unit, then try to fight it
                    else {
                        if unit.is_unit_in_range(&target_unit.unwrap()) {
                            attacks.push(unit.start_attack(target_unit.unwrap().id));
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

                _ => {
                    panic!("Unexpected Attack Strategy!!");
                }
            }
        }
        unit.update();
        //check if the unit is on top of a trap
        for trap in &mut *traps {
            if distance_between(unit.foot_position(), trap.pos) < (trap.size / 2.)
                && trap.is_active()
            {
                if trap.trap_type == TrapType::Poop {
                    if let Some(display) = unit.display.as_mut() {
                        display.footprint_status = FootprintStatus::Poopy;
                    }
                } else if trap.trap_type == TrapType::Acidleak {
                    let attack = Attack::new(unit.id, 1., trap.pos, trap.damage, 0., 1, Vec::new());
                    unit.take_attack(&attack, rng);
                    if let Some(display) = unit.display.as_mut() {
                        display.footprint_status = FootprintStatus::Acid;
                    }
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
                        attacks.push(attack);
                        trap.set_inactive();
                        turbo::println!("TRAP POS {}, {}", trap.pos.0, trap.pos.1);
                    }
                }
            }
        }
    }

    if let Some(_winning_team) = has_some_team_won(units) {
        //clear all attacks if there's a winner so we don't kill someone after the simulation ended.
        attacks.clear();
    } else {
        //go through attacks and update, then draw
        attacks.retain_mut(|attack| {
            let should_keep = !attack.update(&units_clone);
            //attack.draw();

            if !should_keep {
                //deal the actual damage here
                if attack.splash_area == 0. {
                    if let Some(unit_index) =
                        units.iter().position(|u| u.id == attack.target_unit_id)
                    {
                        let unit = &mut units[unit_index];
                        unit.take_attack(&attack, rng);
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
                                explosions.push(explosion);
                            }
                        }
                    }
                }
                //if it has splash area, then look for all enemy units within range
                if attack.splash_area > 0. {
                    let team = find_unit_by_id(units, Some(attack.target_unit_id))
                        .unwrap()
                        .team;
                    for unit in &mut *units {
                        if distance_between(attack.pos, unit.pos) <= attack.splash_area
                            && unit.state != UnitState::Dead
                            && unit.team == team
                        {
                            unit.take_attack(&attack, rng);
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
                                    explosions.push(explosion);
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
                    explosions.push(explosion);
                    //make a crater
                    let crater_pos = (explosion_pos.0 + 16., explosion_pos.1 + 16.);
                    let mut crater = AnimatedSprite::new(crater_pos, false);

                    crater.set_anim("crater_01".to_string(), 16, 1, 1, true);
                    crater.animator.change_tint_color(0xFFFFFF80);
                    craters.push(crater);
                }
            }

            should_keep
        });
    }
    //go through traps, update and draw
    for trap in traps {
        trap.update();
        trap.draw();
    }
}

fn modify_damage_from_artifacts(
    mut attack: Attack,
    units: &Vec<Unit>,
    artifacts: &Vec<Artifact>,
) -> Attack {
    // Go through each artifact
    for artifact in artifacts {
        match artifact.artifact_kind {
            ArtifactKind::StrenghtOfTheFallen => {
                // Count dead friendly units
                let dead_count = units
                    .iter()
                    .filter(|u| u.health <= 0. && u.team == 0)
                    .count();

                if let ArtifactConfig::DeadUnitDamageBoost { percent_per_unit } = artifact.config {
                    // Increase damage by config percentage for each dead unit
                    let damage_multiplier = 1.0 + (dead_count as f32 * percent_per_unit / 100.0);
                    //turbo::println!("Unboosted Damage: {}", attack.damage);
                    attack.damage *= damage_multiplier;
                    //turbo::println!("Boosted Damage: {}", attack.damage);
                }
            }

            ArtifactKind::SnipersFocus => {
                if attack.attributes.contains(&Attribute::Ranged) {
                    if let Some(target_unit) = find_unit_by_id(units, Some(attack.target_unit_id)) {
                        if let ArtifactConfig::DistanceDamageBoost { percent_per_pixel } =
                            artifact.config
                        {
                            // Calculate distance between attack position and target
                            let dx = (target_unit.pos.0 - attack.pos.0) as f32;
                            let dy = (target_unit.pos.1 - attack.pos.1) as f32;
                            let distance = (dx * dx + dy * dy).sqrt();

                            // Increase damage based on distance
                            let damage_multiplier = 1.0 + (distance * percent_per_pixel / 100.0);
                            attack.damage *= damage_multiplier;
                            turbo::println!("Boosted: {}%", (damage_multiplier - 1.0));
                        }
                    }
                }
            }

            // For FlameWard or any other kinds, no damage modification needed
            _ => {}
        }
    }

    attack
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

fn lowest_health_ranged_enemy_unit(units: &Vec<Unit>, team: i32, pos: (f32, f32)) -> Option<&Unit> {
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
    os::client::exec("pixel_wars", "commit_points", &[is_win as u8]);
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

// fn start_match(state: &mut GameState) {
//     state.units = create_units_for_all_teams(
//         &mut state.teams,
//         &mut state.rng,
//         state.data_store.as_ref().unwrap(),
//     );

//     state.phase = Phase::PreBattle;
//     set_cam!(x = 192, y = 108);
// }

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
    let text = time / 60;
    let text = format!("BATTLE STARTS IN: {}", text);
    power_text!(
        text.as_str(),
        x = 0,
        y = 80,
        font = Font::L,
        center_width = 384
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

        let mut unit = Unit::new(
            unit_type.clone(),
            (x, y),
            team_index as i32,
            data_store,
            *id,
        );
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
    winning_team: Option<i32>,
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
       let mut center_width: Option<i32> = None;  // Width of area to center within

       $($crate::paste::paste!{ [< $key >] = power_text!(@coerce $key, $val); })*


       let char_width = match font {
        Font::S => 4,
        Font::M => 5,
        Font::L => 8,
        Font::XL => 16,
     };
     let char_height = match font {
        Font::S => 4,
        Font::M => 7,
        Font::L => 8,
        Font::XL => 15,
     };
       // Handle centering if specified
       if let Some(width) = center_width {

           let text_width = (char_width * $text.len() as i32);
           x += (width - text_width) / 2;
       }
        // Handle drop shadow if specified
        if let Some(shadow_color) = drop_shadow {
            text!($text,
                x = x - 2,
                y = y + 1,
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
   (@coerce center_width, $val:expr) => { Some($val as i32) };
}
