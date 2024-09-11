use csv::{Reader, ReaderBuilder};
use std::cmp::{max, Ordering};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{format, Display};

const UNIT_DATA_CSV: &[u8] = include_bytes!("../resources/unit-data.csv");
const DAMAGE_EFFECT_TIME: u32 = 12;
const DAMAGE_TINT_COLOR: usize = 0xb9451dff;
const COLOR_WHITE: usize = 0xffffffff;
const UNIT_ANIM_SPEED: i32 = 8;
const MAX_Y_ATTACK_DISTANCE: f32 = 10.;

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
        teams: Vec<Team>,
        unit_previews: Vec<UnitPreview>,
        attacks: Vec<Attack>,
        event_queue: Vec<GameEvent>,
        rng: RNG,
        data_store: Option<UnitDataStore>,
        traps: Vec<Trap>,
        auto_assign_teams: bool,
        selected_team_index: i32,
        explosions: Vec<AnimatedSprite>,
    } = {
        let teams = Vec::new();
        Self {
            phase: Phase::PreBattle,
            units: Vec::new(),
            teams,
            attacks: Vec::new(),
            event_queue: Vec::new(),
            traps: Vec::new(),
            unit_previews: Vec::new(),
            explosions: Vec::new(),
            //replace this number with a program number later
            rng: RNG::new(12345),
            data_store: None,
            auto_assign_teams: true,
            selected_team_index: 0,
        }
    }
}

turbo::go!({
    let mut state = GameState::load();
    clear!(0x8f8cacff);
    if state.phase == Phase::PreBattle {
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
            // for i in 0..10{
            //     let a = state.rng.next_in_range(0, 10);
            //     turbo::println!("A: {}", a);
            // }
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

                let (team1, team2) = generate_balanced_teams(&data_store, &mut state.rng);
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
        let gp = gamepad(0);
        if gp.start.just_pressed() {
            //generate units
            create_units_for_all_teams(&mut state);
            state.phase = Phase::Battle;
        }

        if state.auto_assign_teams {
            //draw each unit based on the teams
            draw_assigned_team_info(&mut state);
            for u in &mut state.unit_previews {
                u.update();
                u.draw();
            }
        }

        if !state.auto_assign_teams {
            draw_team_info_and_buttons(&mut state);
        }
    } else if state.phase == Phase::Battle {
        //using this for some comparisons, but might be able to remove it eventually
        let units_clone = state.units.clone();

        //go through each unit, see what it wants to do, and handle all actions from here
        for unit in &mut state.units {
            //check if unit is moving or not
            if unit.state == UnitState::Idle {
                if let Some(index) = closest_enemy_index(&unit, &units_clone) {
                    if unit.is_unit_in_range(&units_clone[index]) {
                        state.attacks.push(unit.start_attack(index));
                        if unit.pos.0 > units_clone[index].pos.0{
                            unit.is_facing_left = true;
                        }
                        else{
                            unit.is_facing_left = false;
                        }
                    } else {
                        if unit.state == UnitState::Idle {
                            unit.new_target_tween_position(&units_clone[index].pos, &mut state.rng);
                        }
                    }
                }
            }
            unit.update();
            //check for traps
            for trap in &state.traps {
                if distance_between(unit.pos, trap.pos) < (trap.size / 2.) && trap.is_active() {
                    unit.take_damage(trap.damage);
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
                    state.units[attack.target_unit_index].take_damage(attack.damage);
                }
                //if it has splash area, then look for all enemy units within range
                if attack.splash_area > 0. {
                    for unit in &mut state.units {
                        if distance_between(attack.pos, unit.pos) <= attack.splash_area && unit.state != UnitState::Dead {
                            unit.take_damage(attack.damage);
                        }
                    }
                }
                if attack.is_explosive{
                     //create explosion
                     let explosion_offset = (-16., -16.);
                     let explosion_pos = (attack.pos.0 + explosion_offset.0, attack.pos.1 + explosion_offset.1);
                     let mut explosion = AnimatedSprite::new(explosion_pos, false);
                     explosion.set_anim("explosion".to_string(), 32, 15, 5, false);
                     state.explosions.push(explosion);
                }
            }

            should_keep
        });
        //go through traps, update and draw
        for trap in &mut state.traps {
            trap.update();
            trap.draw();
        }

        //check for game over
        let mut winning_team = has_some_team_won(&state.units);

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
        state.explosions.retain_mut(|explosion| {
            explosion.update();
            !explosion.animator.is_done()
        });
        for explosion in &mut state.explosions{
            explosion.draw();
        }
        //draw text box
        if winning_team.is_some() {
            let index: usize = winning_team.take().unwrap_or(-1) as usize;
            let mut text = "You Chose Incorrectly!";
            if index == state.selected_team_index as usize {
                text = "You Chose Correctly!";
            }
            //let text = format!("{} Win!", state.teams[index].name);
            //text!(text.as_str(), x = cam!().0,);
            draw_text_box(
                text.to_string(),
                (20., 150.),
                (120., 20.),
                0x333333ff,
                0x87CEFAff,
            );
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
                    unit.state = UnitState::Idle;
                }
            }
        }
        //Draw team health bars
        // Initialize variables to store health totals for each team
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

        // Draw health bar for team 0
        draw_team_health_bar(
            team0_base_health,
            team0_current_health,
            (20.0, 20.0),
            &state.teams[0].name,
            0xc4f129ff,
        );

        // Draw health bar for team 1
        draw_team_health_bar(
            team1_base_health,
            team1_current_health,
            (250.0, 20.0),
            &state.teams[1].name,
            0xa69e9aff,
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
                create_units_for_all_teams(&mut state);
                state.phase = Phase::Battle;
            }
            GameEvent::RestartGame() => {
                state = GameState::default();
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

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
enum Phase {
    PreBattle,
    Battle,
    WrapUp,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
struct Unit {
    unit_type: String,
    data: UnitData,
    team: i32,
    health: f32,
    pos: (f32, f32),
    state: UnitState,
    move_tween_x: Tween<f32>,
    move_tween_y: Tween<f32>,
    target_pos: (f32, f32),
    attack_timer: i32,
    animator: Animator,
    damage_effect_timer: u32,
    blood_splatter: Option<AnimatedSprite>,
    is_facing_left: bool,
}

impl Unit {
    fn new(unit_type: String, pos: (f32, f32), team: i32, store: &UnitDataStore) -> Self {
        // Initialize default values
        let data = store.get_unit_data(&unit_type).unwrap_or_else(|| {
            panic!("Unit type not found in the data store");
        });
        Self {
            data: data.clone(),
            unit_type,
            team,

            health: data.max_health,
            pos,
            state: UnitState::Idle,
            move_tween_x: Tween::new(0.),
            move_tween_y: Tween::new(0.),
            attack_timer: 0,
            damage_effect_timer: 0,
            blood_splatter: None,
            is_facing_left: false,
            target_pos: (0.,0.),
            //placeholder, gets overwritten when they are drawn, but I can't figure out how to do it more logically than this
            animator: Animator::new(Animation {
                name: "placeholder".to_string(),
                s_w: data.sprite_width,
                num_frames: 4,
                loops_per_frame: UNIT_ANIM_SPEED,
                is_looping: true,
            }),
        }
    }
    fn update(&mut self) {
        if self.state == UnitState::Moving {
            self.pos.0 = self.move_tween_x.get();
            self.pos.1 = self.move_tween_y.get();
            if self.move_tween_x.done() {
                self.state = UnitState::Idle;
            }
        }
        if self.state == UnitState::Attacking {
            self.attack_timer -= 1;
            if self.attack_timer <= 0 {
                self.state = UnitState::Idle;
            }
        }
        if self.health <= 0. {
            self.state = UnitState::Dead;
        }
    }

    fn draw(&mut self) {
        let mut new_anim = Animation {
            name: self.unit_type.to_lowercase(),
            s_w: self.data.sprite_width,
            num_frames: 4,
            loops_per_frame: UNIT_ANIM_SPEED,
            is_looping: true,
        };
        if self.state == UnitState::Moving {
            new_anim.name += "_walk";
            self.animator.set_cur_anim(new_anim);
        } else if self.state == UnitState::Dead {
            new_anim.name += "_death";
            new_anim.is_looping = false;
            self.animator.set_cur_anim(new_anim);
            self.animator.next_anim = None;
        } else if self.state == UnitState::Attacking {
            //only set this once, when the attack starts.
            //That way when attack ends, they will idle (could change to reload or something later)
            if self.attack_timer == self.data.attack_time - 1 {
                new_anim.name += "_attack";
                new_anim.is_looping = false;
                self.animator.set_cur_anim(new_anim);
                let next_anim = Animation {
                    name: self.unit_type.to_lowercase() + "_idle",
                    s_w: self.data.sprite_width,
                    num_frames: 4,
                    loops_per_frame: UNIT_ANIM_SPEED,
                    is_looping: true,
                };
                self.animator.set_next_anim(Some(next_anim));
            }
        } else if self.state == UnitState::Idle {
            self.animator.cur_anim.is_looping = false;
            let next_anim = Animation {
                name: self.unit_type.to_lowercase() + "_idle",
                s_w: self.data.sprite_width,
                num_frames: 4,
                loops_per_frame: UNIT_ANIM_SPEED,
                is_looping: true,
            };
            self.animator.set_next_anim(Some(next_anim));
        }
        if self.damage_effect_timer > 0 {
            self.animator.change_tint_color(DAMAGE_TINT_COLOR);
            self.damage_effect_timer -= 1;
        } else {
            self.animator.change_tint_color(COLOR_WHITE);
        }
        self.animator.update();
        self.animator.draw(self.draw_position(), self.flip_x());
        if let Some(ref mut splatter) = self.blood_splatter {
            splatter.update();
            if splatter.animator.is_done() {
                log!("Splatter Done");
                self.blood_splatter = None;
            } else {
                splatter.draw();
                log("Splatter Drawing");
            }
        }
        //TESTING FOR center position
        // circ!(x=self.pos.0, y=self.pos.1, d = 2, color = 0x000000ff);
        // sprite!("blood_16px_01", x=self.pos.0, y=self.pos.1);

        //TURN THIS ON TO SHOW HEALTH BARS
        // if self.state == UnitState::Dead {
        //     self.draw_health_bar();
        // }
    }

    fn draw_health_bar(&self) {
        let d_p = self.draw_position();
        let x = d_p.0;
        let y = d_p.1;
        let x_bar = x;
        let y_bar = y - 2.;
        let w_bar = 0.06 * self.data.max_health;
        let h_bar = 2;
        let mut main_color: u32 = 0xc4f129ff;
        if self.team == 1 {
            main_color = 0xa69e9aff;
        }
        let back_color: u32 = 0xb9451dff;
        let mut health_width =
            (self.health as f32 / self.data.max_health as f32 * w_bar as f32) as i32;
        health_width = health_width.max(0);

        // Draw health bar background
        rect!(
            w = w_bar,
            h = h_bar,
            x = x_bar,
            y = y_bar,
            color = back_color
        );

        // Draw current health bar
        rect!(
            w = health_width,
            h = h_bar,
            x = x_bar,
            y = y_bar,
            color = main_color
        );

        // // Draw health bar border
        // rect!(
        //     w = w_bar + 2.,
        //     h = h_bar,
        //     x = x_bar - 1.,
        //     y = y_bar,
        //     color = 0,
        //     border_color = border_color,
        //     border_width = 1,
        //     border_radius = 2
        // )
    }

    fn new_target_tween_position(&mut self, target: &(f32, f32), rng: &mut RNG) {
        // Calculate the direction vector from self.pos to target
        let mut adj_target = *target;
        // if self.data.range > 40.{
        //     //adjust this so your target X is more like where you will attack from (target - range)
        //     if adj_target.0 > self.pos.0{
        //         adj_target.0 -= self.data.range - 10.;
        //     }
        //     else{
        //         adj_target.0 += self.data.range - 10.;
        //     }
        // }
        let dir_x = adj_target.0 - self.pos.0;
        let dir_y = adj_target.1 - self.pos.1;

        if dir_x > 0.{
            self.is_facing_left = false;
        }
        else if dir_x < 0.{
            self.is_facing_left = true;
        }
        // Calculate the length (magnitude) of the direction vector
        let length = (dir_x * dir_x + dir_y * dir_y).sqrt();

        // Normalize the direction vector
        let norm_dir_x = dir_x / length;
        let norm_dir_y = dir_y / length;

        let rand_x = rng.next_in_range(0, 5) as f32 * norm_dir_x.signum();
        //turbo::println!("rand_x: {}", rand_x);

        let rand_y = rng.next_in_range(0, 5) as f32 * norm_dir_y.signum();
        //turbo::println!("rand_y: {}", rand_x);

        let new_x = self.pos.0 + norm_dir_x * self.data.speed + rand_x;
        let new_y = self.pos.1 + norm_dir_y * self.data.speed + rand_y;
        self.move_tween_x = Tween::new(self.pos.0).set(new_x).duration(20);
        self.move_tween_y = Tween::new(self.pos.1).set(new_y).duration(20);
        self.state = UnitState::Moving;
    }

    //Not using this for now - but if we need some more control over movement we can
    fn new_target_position(&mut self, target:&(f32, f32), rng: &mut RNG){
        //Move toward the target xunits + some randomness
         // Calculate the direction vector from self.pos to target
         let dir_x = target.0 - self.pos.0;
         let dir_y = target.1 - self.pos.1;
 
         // Calculate the length (magnitude) of the direction vector
         let length = (dir_x * dir_x + dir_y * dir_y).sqrt();
 
         // Normalize the direction vector
         let norm_dir_x = dir_x / length;
         let norm_dir_y = dir_y / length;
 
         let rand_x = rng.next_f32() * norm_dir_x.signum() * 10.;
         //turbo::println!("rand_x: {}", rand_x);
 
         let rand_y = rng.next_f32() * norm_dir_y.signum() * 10.;
         //turbo::println!("rand_y: {}", rand_x);
        
         let new_x = self.pos.0 + norm_dir_x * (self.data.speed / 50.) + rand_x;
         let new_y = self.pos.1 + norm_dir_y * (self.data.speed / 50.) + rand_y;
         self.target_pos = (new_x, new_y);
         self.state = UnitState::Moving;

    }

    fn take_damage(&mut self, damage: f32) {
        self.health -= damage;
        self.health = self.health.max(0.);
        self.damage_effect_timer = DAMAGE_EFFECT_TIME;
        if self.blood_splatter.is_none() {
            //make the splatter position the top-middle of the sprite
            let mut splat_pos = self.pos;
            //TODO: Figure out something better to do with these numbers, they do sort of just work for now
            if self.flip_x() {
                splat_pos.0 -= 8.;
            } else {
                splat_pos.0 -= 12.;
            }
            splat_pos.1 -= 12.;
            let mut new_splatter = AnimatedSprite::new(splat_pos, self.flip_x());
            let num = rand() % 8 + 1;
            let name = format!("blood_16px_0{}", num);
            new_splatter.set_anim(name, self.data.sprite_width, 4, UNIT_ANIM_SPEED, false);
            self.blood_splatter = Some(new_splatter);
        }
    }

    fn start_attack(&mut self, target_index: usize) -> Attack {
        self.attack_timer = self.data.attack_time;
        self.state = UnitState::Attacking;
        //create the actual attack
        let size = 1;
        let mut attack = Attack::new(
            target_index,
            2.,
            self.pos,
            self.data.damage,
            self.data.splash_area,
            size,
        );
        if self.unit_type == "bazooka"{
            attack.is_explosive = true;
        }
        attack
    }

    fn distance_to(&self, pos: &(f32, f32)) -> f32 {
        let dx = self.pos.0 - pos.0;
        let dy = self.pos.1 - pos.1;
        (dx * dx + dy * dy).sqrt()
    }

    fn is_unit_in_range(&self, other: &Unit) -> bool{
        let other_pos = other.pos;
        let dx = (self.pos.0 - other_pos.0).abs();
        let dy = (self.pos.1 - other_pos.1).abs();
        if dx < self.data.range && dy < MAX_Y_ATTACK_DISTANCE{
            return true;
        }
        false
    }

    fn draw_position(&self) -> (f32, f32) {
        //TODO: I think this might need some work - we probably need to define an 'anchor' point
        //in the csv. I am trying to 'guess' about how far the body is from where the sprite is drawing
        //and since theres a lot of empty space on some sprites, when you flip_x you get a lot of empty space.
        let mut d_x = -8.;
        if self.flip_x() {
            d_x = 8. - self.data.sprite_width as f32;
        }
        return (self.pos.0 + d_x, self.pos.1 - 8.);
    }

    fn flip_x(&self) -> bool {
        //self.team == 1
        self.is_facing_left
    }
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
struct UnitData {
    unit_type: String,
    damage: f32,
    max_health: f32,
    speed: f32,
    range: f32,
    attack_time: i32,
    splash_area: f32,
    sprite_width: i32,
}

//TODO: Make this generic as an AnimatedSprite, and update accordingly
//then use it for explosions
//should also have a flip_x that gets set when it is created and use that for the animation
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
    target_unit_index: usize,
    speed: f32,
    pos: (f32, f32),
    damage: f32,
    splash_area: f32,
    size: i32,
    is_explosive: bool,
}

impl Attack {
    //new
    fn new(
        target_unit_index: usize,
        speed: f32,
        pos: (f32, f32),
        damage: f32,
        splash_area: f32,
        size: i32,
    ) -> Self {
        Self {
            target_unit_index,
            speed,
            pos,
            damage,
            splash_area,
            size,
            is_explosive: false,
        }
    }
    fn update(&mut self, units: &Vec<Unit>) -> bool {
        // Get the target unit's position
        let target_position = units[self.target_unit_index].pos;

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
        distance <= self.speed
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
    team_color: u32,
) {
    //normalized health bar width
    //colors
    //sizes
    //draw text
    let x = pos.0;
    let y = pos.1;
    let x_bar = x;
    let y_bar = y;
    let w_bar = 120.;
    let h_bar = 15;
    let main_color: u32 = team_color;
    let back_color: u32 = 0xb9451dff;
    let border_color: u32 = 0x000000ff;
    let mut health_width = (current_health / total_base_health * w_bar) as i32;
    health_width = health_width.max(0);

    // Draw health bar background
    rect!(
        w = w_bar,
        h = h_bar,
        x = x_bar,
        y = y_bar,
        color = back_color
    );

    // Draw current health bar
    rect!(
        w = health_width,
        h = h_bar,
        x = x_bar,
        y = y_bar,
        color = main_color
    );

    // Draw health bar border
    rect!(
        w = w_bar + 2.,
        h = h_bar,
        x = x_bar - 1.,
        y = y_bar,
        color = 0,
        border_color = border_color,
        border_width = 3,
        border_radius = 2
    );

    //put team name in white over the bar
    text!(team_name, x = x_bar, y = y_bar - 10., font = Font::L);
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
enum UnitState {
    Moving,
    Attacking,
    Idle,
    Dead,
}
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
struct UnitPreview {
    //unit type as a string
    unit_type: String,
    //animator
    animator: Animator,
    s_w: i32,
    pos: (f32, f32),
    flip_x: bool,
    state: UnitState,
}

impl UnitPreview {
    fn new(unit_type: String, s_w: i32, pos: (f32, f32), flip_x: bool) -> Self {
        Self {
            unit_type, //placeholder, gets overwritten when they are drawn, but I can't figure out how to do it more logically than this
            animator: Animator::new(Animation {
                name: "placeholder".to_string(),
                s_w: s_w,
                num_frames: 0,
                loops_per_frame: 0,
                is_looping: true,
            }),
            s_w,
            pos,
            flip_x,
            state: UnitState::Idle,
        }
    }
    //add walk to animator, then if its done, add the other one
    fn update(&mut self) {
        self.animator.update();
        let mut new_anim = Animation {
            name: self.unit_type.to_lowercase(),
            s_w: self.s_w,
            num_frames: 4,
            loops_per_frame: UNIT_ANIM_SPEED,
            is_looping: false,
        };
        if self.state == UnitState::Idle {
            self.state = UnitState::Moving;
            new_anim.name += "_walk";
            self.animator.set_cur_anim(new_anim);
        } else if self.animator.is_done() {
            if self.state == UnitState::Moving {
                self.state = UnitState::Attacking;
                new_anim.name += "_attack";
                self.animator.set_cur_anim(new_anim);
            } else if self.state == UnitState::Attacking {
                self.state = UnitState::Moving;
                new_anim.name += "_walk";
                self.animator.set_cur_anim(new_anim);
            }
        }
    }

    fn draw(&self) {
        self.animator.draw(self.draw_pos(), self.flip_x);
    }

    fn draw_pos(&self) -> (f32, f32) {
        if self.flip_x {
            return (self.pos.0 - (self.s_w as f32 - 16.), self.pos.1);
        }
        self.pos
    }
    //draw from animator
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
struct Team {
    name: String,
    units: Vec<String>,
    data: UnitDataStore,
}

impl Team {
    fn new(name: String, data: UnitDataStore) -> Self {
        Self {
            name,
            units: Vec::new(),
            data,
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
            tint_color: COLOR_WHITE,
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
        frame_index = frame_index.clamp(0, self.cur_anim.num_frames-1);
        let sx = (frame_index * self.cur_anim.s_w)
            .clamp(0, self.cur_anim.s_w * (self.cur_anim.num_frames - 1)); // Calculate the sprite X coordinate
        let mut x_adj = 0.;
        // if sx > 32 {
        //     x_adj = -self.cur_anim.s_w as f32;
        // }
        //log!("FI: {}", frame_index);
        if frame_index >= 3 {//* self.cur_anim.s_w {
            x_adj = (-self.cur_anim.s_w * (frame_index - 2)) as f32;
        }

        sprite!(
            name,
            x = pos.0 + x_adj,
            y = pos.1,
            sx = sx,
            flip_x = flip_x,
            sw = self.cur_anim.s_w,
            color = self.tint_color,
        );
        // sprite!(
        //     "flameboi_attack copy",
        //     x = pos.0 + x_adj,
        //     y = pos.1,
        //     sx =sx,
        //     flip_x = flip_x,
        //     sw = 32,
        //     w = 32,
        // );
        // let mut y = 10;
        // if flip_x{y=20};
        // text!("sx: {}", sx; x=10, y=y);
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
struct StatusEffect {
    status: Status,
    timer: i32,
}

impl StatusEffect {
    //new
    //update - run timer
    //draw - draw sprite based on name, at position
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
enum Status {
    Poison,
    Healing,
    Freeze,
    Burn,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
struct Trap {
    size: f32,
    pos: (f32, f32),
    damage: f32,
    on_dur: i32,
    off_dur: i32,
    timer: i32,
}

impl Trap {
    // New trap with 4 parameters, timer always starts at 0
    fn new(size: f32, pos: (f32, f32), damage: f32, on_dur: i32, off_dur: i32) -> Self {
        Trap {
            size,
            pos,
            damage,
            on_dur,
            off_dur,
            timer: 0,
        }
    }

    // Update function: add 1 to timer, if timer is greater than off_dur+on_dur reset it to 0
    fn update(&mut self) {
        self.timer += 1;
        if self.timer > self.off_dur + self.on_dur {
            self.timer = 0;
        }
    }

    fn draw(&self) {
        if self.timer <= self.on_dur {
            circ!(
                x = self.draw_pos().0,
                y = self.draw_pos().1,
                d = self.size,
                color = 0xFF0000ff
            );
        }
    }

    fn draw_pos(&self) -> (f32, f32) {
        (self.pos.0 - self.size / 2., self.pos.1 - self.size / 2.)
    }
    // Helper function to check if the trap is currently active
    fn is_active(&self) -> bool {
        self.timer <= self.on_dur
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
            border_radius = 2,
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
        let mut y_pos = 50.0;

        for (i, unit_type) in team.units.iter().enumerate() {
            if y_pos > max_y {
                y_pos = 50.0;

                if team_index == 0 {
                    x_start -= row_width;
                } else {
                    x_start += row_width;
                }
            }
            let pos = (x_start, y_pos);
            state.units.push(Unit::new(
                unit_type.clone(),
                pos,
                team_index as i32,
                &data_store,
            ));
            //let unit = Unit::new(UnitType::Axeman, (0.0, 0.0), 0, &unit_type_store);
            y_pos += row_height;
        }
    }
    //add a random trap for now
    //state.traps.push(Trap::new(48., (160., 75.), 1., 120, 120));
    //state.traps.push(Trap::new(48., (200., 120.), 1., 120, 120));
    //go to Battle Phase
}
fn calculate_unit_power_level(data_store: &HashMap<String, UnitData>) -> HashMap<String, f32> {
    let mut power_levels = HashMap::new();

    // Find max values for normalization
    let max_health = data_store
        .values()
        .map(|u| u.max_health)
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(1.0);
    let max_dps = data_store
        .values()
        .map(|u| u.damage / (u.attack_time as f32 / 60.0))
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(1.0);
    let max_speed = data_store
        .values()
        .map(|u| u.speed)
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(1.0);

    for (unit_type, unit_data) in data_store {
        let normalized_health = (unit_data.max_health / max_health) * 50.0;
        let dps = unit_data.damage / (unit_data.attack_time as f32 / 60.0);
        let normalized_dps = (dps / max_dps) * 100.0;
        let normalized_speed = (unit_data.speed / max_speed) * 10.0;

        let mut power_level = normalized_health + normalized_dps + normalized_speed;

        if unit_data.range > 20.0 {
            power_level += 150.0;
        }

        if unit_data.splash_area > 0.0 {
            power_level += 100.0;
        }

        power_levels.insert(unit_type.clone(), power_level);
    }

    power_levels
}

fn generate_balanced_teams(data: &UnitDataStore, rng: &mut RNG) -> (Team, Team) {
    let power_levels = calculate_unit_power_level(&data.data);
    let average_power: f32 = power_levels.values().sum::<f32>() / power_levels.len() as f32;
    let target_team_power = average_power * 25.0;

    let mut unit_types: Vec<&String> = power_levels.keys().collect();
    unit_types.sort();

    // Select four different unit types
    let mut selected_types = Vec::new();
    while selected_types.len() < 4 {
        let index = rng.next_in_range(0, unit_types.len() as u32 - 1) as usize;
        turbo::println!("Index: {}", index);
        let unit_type = unit_types[index];
        if !selected_types.contains(&unit_type) {
            selected_types.push(unit_type);
        }
    }

    let mut team1 = Team::new("Pixel Peeps".to_string(), data.clone());
    let mut team2 = Team::new("Battle Bois".to_string(), data.clone());

    create_team(
        &mut team1,
        &selected_types[0..2],
        &power_levels,
        target_team_power,
        rng,
    );
    create_team(
        &mut team2,
        &selected_types[2..4],
        &power_levels,
        target_team_power,
        rng,
    );

    (team1, team2)
}

fn create_team(
    team: &mut Team,
    unit_types: &[&String],
    power_levels: &HashMap<String, f32>,
    target_power: f32,
    rng: &mut RNG,
) {
    let mut current_power = 0.0;
    let power1 = power_levels[unit_types[0]];
    let power2 = power_levels[unit_types[1]];

    // Generate random weights for each unit type
    let weight1 = rng.next_f32();
    let weight2 = 1.0 - weight1;

    while current_power < target_power {
        let remaining_power = target_power - current_power;

        // Use weighted random selection
        let use_first_type = rng.next_f32() < (weight1 / (weight1 + weight2));

        if use_first_type && remaining_power >= power1 {
            team.units.push(unit_types[0].clone());
            current_power += power1;
        } else if !use_first_type && remaining_power >= power2 {
            team.units.push(unit_types[1].clone());
            current_power += power2;
        } else {
            // If we can't add either unit without going over, try the other unit
            if !use_first_type && remaining_power >= power1 {
                team.units.push(unit_types[0].clone());
                current_power += power1;
            } else if use_first_type && remaining_power >= power2 {
                team.units.push(unit_types[1].clone());
                current_power += power2;
            } else {
                // If we still can't add either unit, stop adding units
                break;
            }
        }
    }

    // Ensure at least one of each unit type
    if !team.units.contains(unit_types[0]) {
        team.units.push(unit_types[0].clone());
    }
    if !team.units.contains(unit_types[1]) {
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
    let mut y_start = 50.;
    let mut x = 120.;
    if is_facing_left {
        x += 60.;
    }
    for (unit_type, _count) in team_summary {
        let unit_type = unit_type.to_lowercase();
        let s_w = data_store.get_sprite_width(&unit_type).unwrap();
        let u_p = UnitPreview::new(unit_type, s_w, (x, y_start), is_facing_left);
        unit_previews.push(u_p);
        y_start += 30.;
    }
    unit_previews
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

            let unit_data = UnitData {
                unit_type,
                damage,
                max_health,
                speed,
                range,
                attack_time,
                splash_area,
                sprite_width,
            };
            store.add_unit_data(unit_data);
        }

        Ok(store)
    }
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
struct RNG {
    seed: u32,
}

impl RNG {
    // Create a new MyRNG with a seed
    fn new(seed: u32) -> Self {
        RNG { seed }
    }

    // Generate the next random number
    fn next(&mut self) -> u32 {
        // Constants for the LCG (these are just examples; you can use different values)
        let a: u32 = 1664525;
        let c: u32 = 1013904223;
        let m: u32 = u32::MAX;

        // Update the seed and produce a new number
        self.seed = (a.wrapping_mul(self.seed).wrapping_add(c)) % m;
        self.seed
    }

    // Generate a random number within a specific range [min, max]
    fn next_in_range(&mut self, min: u32, max: u32) -> u32 {
        let range = max - min + 1;
        let mut number = (self.next() % range) + min;

        // Make sure you use an odd number
        if range % 2 == 0 {
            number += 1;
        }

        number % range + min
    }
    // generates an f32 between 0 and 1
    fn next_f32(&mut self) -> f32 {
        // Generate a random u32 and convert it to a float between 0 and 1
        self.next() as f32 / u32::MAX as f32
    }
}