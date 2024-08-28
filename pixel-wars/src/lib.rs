use csv::{ReaderBuilder, Reader};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;

const UNIT_DATA_CSV: &[u8] = include_bytes!("../resources/unit-data.csv");

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
        attacks: Vec<Attack>,
        event_queue: Vec<GameEvent>,
        rng: RNG,
        data_store: Option<UnitDataStore>,
    } = {
        let mut teams = Vec::new();
        teams.push(Team::new("Battle Bois".to_string()));
        teams.push(Team::new("Pixel Peeps".to_string()));
        Self {
            phase: Phase::PreBattle,
            units: Vec::new(),
            teams,
            attacks: Vec::new(),
            event_queue: Vec::new(),
            //replace this number with a program number later
            rng: RNG::new(12345),
            data_store: None,
        }
    }
}

turbo::go!({
    let mut state = GameState::load();
    if state.phase == Phase::PreBattle {
        //initialize the data store if it is blank
        if state.data_store.is_none() {
            match UnitDataStore::load_from_csv(UNIT_DATA_CSV) {
                Ok(loaded_store) => {
                    state.data_store = Some(loaded_store);
                    log("DATA LOADED");
                },
                Err(e) => {
                    eprintln!("Failed to load UnitDataStore: {}", e);
                    state.data_store = Some(UnitDataStore::new());
                }
            }
        }
        //handle input
        let gp = gamepad(0);
        if gp.start.just_pressed() {
            //generate units
            let row_height = 20.0;
            let row_width = 20.0;
            let max_y = 180.0;
            let data_store = state.data_store.as_ref().expect("Data store should be loaded");
            //shuffle the units in each team
            for team in &mut state.teams {
                shuffle(&mut state.rng, &mut team.units);
            }

            for (team_index, team) in state.teams.iter().enumerate() {
                let mut x_start = if team_index == 0 { 70.0 } else { 270.0 }; // Adjusted starting x for team 1
                let mut y_pos = 20.0;
                
                for (i, unit_type) in team.units.iter().enumerate() {
                    if y_pos > max_y {
                        y_pos = 20.0;

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
            //go to Battle Phase
            state.phase = Phase::Battle;
        }
        draw_team_info_and_buttons(&mut state);
        while let Some(event) = state.event_queue.pop() {
            match event {
                GameEvent::AddUnitToTeam(team_index, unit_type) => {
                    state.teams[team_index].add_unit(unit_type);
                }
                GameEvent::RemoveUnitFromTeam(team_index, unit_type) => {
                    state.teams[team_index].remove_unit(unit_type);
                }
            }
        }
    }

    if state.phase == Phase::Battle {
        clear!(0x8f8cacff);
        let units_clone = state.units.clone();
        //let mut damage_map = Vec::new();

        //go through each unit, see what it wants to do, and handle all actions from here
        for unit in &mut state.units {
            //check if unit is moving or not
            if unit.state == UnitState::Idle {
                if let Some(index) = closest_enemy_index(&unit, &units_clone) {
                    if unit.distance_to(&units_clone[index]) < unit.data.range {
                        state.attacks.push(unit.start_attack(index));
                    } else {
                        if unit.state == UnitState::Idle {
                            unit.new_target_tween_position(
                                units_clone[index].clone().pos,
                                &mut state.rng,
                            );
                        }
                    }
                }
            }
            unit.update();
            unit.draw();
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
                        if distance_between(attack.pos, unit.pos) <= attack.splash_area {
                            unit.take_damage(attack.damage);
                        }
                    }
                }
            }

            should_keep
        });
        // for d in damage_map {
        //     state.units[d.0].take_damage(d.1);
        // }
        //check for game over
        let mut winning_team = has_some_team_won(&state.units);
        if winning_team.is_some() {
            let index: usize = winning_team.take().unwrap_or(-1) as usize;
            let text = format!("{} Win!", state.teams[index].name);
            text!(text.as_str(), x = cam!().0,);
            for unit in &mut state.units {
                if unit.state != UnitState::Dead {
                    unit.state = UnitState::Idle;
                }
            }
        }
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
    attack_timer: i32,
    animator: Animator,
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
            //placeholder, gets overwritten when they are drawn, but I can't figure out how to do it more logically than this
            animator: Animator::new(Animation {
                name: "axeman_walk".to_string(),
                s_w: data.sprite_width,
                num_frames: 4,
                loops_per_frame: 10,
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
            loops_per_frame: 10,
            is_looping: true,
        };
        let mut flip_x = false;
        if self.team == 1 {
            flip_x = true;
        }
        if self.state == UnitState::Moving {
            new_anim.name += "_walk";
            self.animator.set_cur_anim(new_anim);
        } else if self.state == UnitState::Dead {
            new_anim.name += "_death";
            new_anim.is_looping = false;
            self.animator.set_cur_anim(new_anim);
        } else if self.state == UnitState::Attacking {
            new_anim.name += "_attack";
            new_anim.is_looping = false;
            self.animator.set_cur_anim(new_anim);
        } else if self.state == UnitState::Idle {
            new_anim.name += "_idle";
            self.animator.set_cur_anim(new_anim);
        }
        self.animator.draw(self.pos, flip_x);
        self.animator.update();
        if self.state != UnitState::Dead {
            self.draw_health_bar();
        }
    }

    fn draw_health_bar(&self) {
        let x = self.pos.0;
        let y = self.pos.1;
        let x_bar = x;
        let y_bar = y - 2.;
        let w_bar = 0.25 * self.data.max_health;
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

    // fn move_toward_enemy(&mut self, enemy: Unit) {
    //     //set tween position to be x units toward the enemy
    //     self.new_target_tween_position(enemy.pos);
    // }

    fn new_target_tween_position(&mut self, target: (f32, f32), rng: &mut RNG) {
        // Calculate the direction vector from self.pos to target
        let dir_x = target.0 - self.pos.0;
        let dir_y = target.1 - self.pos.1;

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

    fn take_damage(&mut self, damage: f32) {
        self.health -= damage;
    }

    fn start_attack(&mut self, target_index: usize) -> Attack {
        self.attack_timer = self.data.attack_time;
        self.state = UnitState::Attacking;
        //create the actual attack
        let size = 1;
        Attack::new(
            target_index,
            2.,
            self.pos,
            self.data.damage,
            self.data.splash_area,
            size,
        )
    }

    fn distance_to(&self, other: &Unit) -> f32 {
        let dx = self.pos.0 - other.pos.0;
        let dy = self.pos.1 - other.pos.1;
        (dx * dx + dy * dy).sqrt()
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

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
struct Attack {
    target_unit_index: usize,
    speed: f32,
    pos: (f32, f32),
    damage: f32,
    splash_area: f32,
    size: i32,
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

fn draw_team_info_and_buttons(state: &mut GameState) {
    let pos_0 = 20;
    let pos_1 = 200;
    let y_start = 20;
    let y_spacing = 20;
    let button_width = 20;
    let button_height = 10;

    let data_store = state.data_store.as_ref().expect("Data store should be loaded");
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
            let unit_type_capitalized = unit_type.chars().next().unwrap().to_uppercase().collect::<String>() + &unit_type[1..];
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


// //to create a new unit, add it here, then 
// #[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Eq, Hash, Copy, PartialOrd)]
// enum UnitType {
//     Axeman,
//     Blade,
//     Hunter,
//     Pyro,
//     BigPound,
// }

// impl UnitType {
//     fn to_lowercase_string(&self) -> String {
//         format!("{:?}", self).to_lowercase()
//     }
// }

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
enum UnitState {
    Moving,
    Attacking,
    Idle,
    Dead,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
struct Team {
    name: String,
    units: Vec<String>,
}

impl Team {
    fn new(name: String) -> Self {
        Self {
            name,
            units: Vec::new(),
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
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
enum GameEvent {
    AddUnitToTeam(usize, String),
    RemoveUnitFromTeam(usize, String),
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
}

impl Animator {
    fn new(cur_anim: Animation) -> Self {
        Animator {
            cur_anim,
            anim_timer: 0,
            next_anim: None,
        }
    }

    fn update(&mut self) {
        self.anim_timer += 1;
        if self.anim_timer > self.cur_anim.total_animation_time() {
            if self.cur_anim.is_looping {
                self.anim_timer = 0;
            } else if let Some(next_anim) = self.next_anim.take() {
                self.cur_anim = next_anim;
                self.anim_timer = 0;
            }
        }
    }

    fn draw(&self, pos: (f32, f32), flip_x: bool) {
        let name = self.cur_anim.name.as_str();
        let frame_index = (self.anim_timer / self.cur_anim.loops_per_frame); // Calculate the frame index
        let sx = (frame_index * self.cur_anim.s_w)
            .clamp(0, self.cur_anim.s_w * (self.cur_anim.num_frames - 1)); // Calculate the sprite X coordinate
        //patch for turbo bug, to be removed later, when bug is fixed
        let mut x_adj = 0.;
        if sx > 32 {
            x_adj = -self.cur_anim.s_w as f32;
        }
        if sx > 64 {
            x_adj = 2. * -self.cur_anim.s_w as f32;
        }
        sprite!(
            name,
            x = pos.0 + x_adj,
            y = pos.1,
            sx = sx,
            flip_x = flip_x,
            sw = 16,
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
struct Obstacle {
    size: i32,
    shape: ObstacleShape,
}

impl Obstacle {
    //Create an obstalce with a certain shape
    //draw obstacle
    //obstacle contains point function -> bool
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
            color = 0x808080ff
        ); // Example button background
        text!(
            self.label.as_str(),
            x = (self.position.0) as i32,
            y = (self.position.1) as i32
        ); // Example button label
    }
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
}