use std::string;

turbo::cfg! {r#"
    name = "Pixel Wars"
    version = "1.0.0"
    author = "Turbo"
    description = "Epic Fantasy Battles of All Time"
    [settings]
    resolution = [256, 144]
"#}

turbo::init! {
    struct GameState {
        phase: Phase,
        units: Vec<Unit>,
        teams: Vec<Team>,
    } = {
        let units = Vec::new();

        // Push 5 Tank units into the units vector
        // for i in 0..5 {
        //     units.push(Unit::new(UnitType::Tank, (20., 30. * i as f32), 0));
        // }
        // for i in 0..10 {
        //     units.push(Unit::new(UnitType::Speedy, (200., 15. * i as f32), 1));
        // }
        let mut teams = Vec::new();
        teams.push(Team::new("Battle Bois".to_string()));
        teams.push(Team::new("Pixel Peeps".to_string()));
        Self {
            phase: Phase::PreBattle,
            units,
            teams,
        }
    }
}

turbo::go!({
    let mut state = GameState::load();
    if state.phase == Phase::PreBattle{
        //handle input
        let gp = gamepad(0);
        if gp.up.just_pressed(){
            state.teams[0].add_unit(UnitType::Tank);
        }
        if gp.down.just_pressed(){
            state.teams[0].remove_unit(UnitType::Tank);
        }
        if gp.right.just_pressed(){
            state.teams[1].add_unit(UnitType::Speedy);
        }
        if gp.left.just_pressed(){
            state.teams[1].remove_unit(UnitType::Speedy);
        }
        if gp.start.just_pressed(){
            //generate units
            for (team_index, team) in state.teams.iter().enumerate() {
                let x_start = if team_index == 0 { 20.0 } else { 200.0 };
                let mut y_pos = 20.0;
            
                for (i, unit_type) in team.units.iter().enumerate() {
                    let pos = (x_start, y_pos);
                    state.units.push(Unit::new(*unit_type, pos, team_index as i32));
                    y_pos += 20.0; // Increment y position for the next unit
                }
            }
            //go to game state
            state.phase = Phase::Battle;
        }
        //Draw text saying which team has which units
        let pos_0 = 20;
        let pos_1 = 200;
        let y_start = 20;
        let y_spacing = 20;

        // Draw info for Team 0 (Left side)
        let team_0 = &state.teams[0];
        let mut y_pos = y_start;
        let name_text_0 = format!("{}:", team_0.name);
        text!(name_text_0.as_str(), x = pos_0, y = y_pos);
        y_pos += y_spacing;

        for unit_type in [UnitType::Tank, UnitType::Speedy, UnitType::DPS].iter() {
            let num_units = team_0.num_unit(*unit_type);
            let unit_text = format!("[{}] {:?}", num_units, unit_type);
            text!(unit_text.as_str(), x = pos_0, y = y_pos);
            y_pos += y_spacing;
        }

        // Draw info for Team 1 (Right side)
        let team_1 = &state.teams[1];
        y_pos = y_start;
        let name_text_1 = format!("{}:", team_1.name);
        text!(name_text_1.as_str(), x = pos_1, y = y_pos);
        y_pos += y_spacing;

        for unit_type in [UnitType::Tank, UnitType::Speedy, UnitType::DPS].iter() {
            let num_units = team_1.num_unit(*unit_type);
            let unit_text = format!("[{}] {:?}", num_units, unit_type);
            text!(unit_text.as_str(), x = pos_1, y = y_pos);
            y_pos += y_spacing;
        }
    }
    
    if state.phase == Phase::Battle{
        let units_clone = state.units.clone();
        let mut damage_map = Vec::new();
        //go through each unit, see what it wants to do, and handle all actions from here
        for unit in &mut state.units {
            //check if unit is moving or not
            if unit.state == UnitState::Idle {
                if let Some(index) = closest_enemy_index(&unit, &units_clone) {
                    if unit.distance_to(&units_clone[index]) < unit.range {
                        damage_map.push((index, unit.damage));
                        unit.start_attack();
                    } else {
                        if unit.state == UnitState::Idle {
                            unit.move_toward_enemy(units_clone[index]);
                        }
                    }
                }
            }
            unit.update();
            unit.draw();
        }
        for d in damage_map {
            state.units[d.0].take_damage(d.1);
        }
        //check for game over
        if all_units_on_either_team_dead(&state.units) {
            text!("GAME OVER", x = cam!().0,);
        }
    }

    state.save();
});

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Copy)]
enum Phase {
    PreBattle,
    Battle,
    WrapUp,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Copy)]
struct Unit {
    unit_type: UnitType,
    team: i32,
    damage: f32,
    range: f32,
    max_health: f32,
    health: f32,
    speed: f32,
    pos: (f32, f32),
    state: UnitState,
    move_tween_x: Tween<f32>,
    move_tween_y: Tween<f32>,
    attack_time: i32,
    attack_timer: i32,
}

impl Unit {
    fn new(unit_type: UnitType, pos: (f32, f32), team: i32) -> Self {
        // Initialize default values
        let (damage, max_health, speed, range, attack_time) = match unit_type {
            UnitType::Tank => (30.0, 200.0, 2.5, 16.0, 10),
            UnitType::Speedy => (8.0, 80.0, 10.0, 25.0, 5),
            UnitType::DPS => (15.0, 100.0, 5.0, 10.0, 7),
        };

        Self {
            unit_type,
            team,
            damage,
            max_health,
            health: max_health,
            speed,
            range,
            pos,
            state: UnitState::Idle,
            move_tween_x: Tween::new(0.),
            move_tween_y: Tween::new(0.),
            attack_time,
            attack_timer: 0,
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
        //if moving or attacking, update tween and check if tween is done
        //if idle do nothing
    }

    fn draw(&self) {
        if self.state != UnitState::Dead {
            match self.unit_type {
                UnitType::Tank => {
                    let mut color: usize = 0x0000ffff;
                    if self.state == UnitState::Attacking {
                        color = 0xff0000ff;
                    }
                    rect!(
                        x = self.pos.0,
                        y = self.pos.1,
                        w = 12,
                        h = 12,
                        color = color
                    );
                }
                UnitType::Speedy => {
                    let mut color: usize = 0x00ff00ff;
                    if self.state == UnitState::Attacking {
                        color = 0xffa500ff;
                    }
                    circ!(x = self.pos.0, y = self.pos.1, d = 4, color = color);
                }
                UnitType::DPS => {}
            }
            self.draw_health_bar();
        }
    }

    fn draw_health_bar(&self) {
        let x = self.pos.0;
        let y = self.pos.1;
        let x_bar = x;
        let y_bar = y - 5.;
        let w_bar = 0.1 * self.max_health;
        let h_bar = 5;
        let border_color: u32 = 0xa69e9aff;
        let main_color: u32 = 0xff0000ff;
        let back_color: u32 = 0x000000ff;
        let mut health_width = (self.health as f32 / self.max_health as f32 * w_bar as f32) as i32;
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
            border_width = 1,
            border_radius = 2
        )
    }

    fn move_toward_enemy(&mut self, enemy: Unit) {
        //set tween position to be x units toward the enemy
        self.new_target_tween_position(enemy.pos);
    }

    fn new_target_tween_position(&mut self, target: (f32, f32)) {
        // Calculate the direction vector from self.pos to target
        let dir_x = target.0 - self.pos.0;
        let dir_y = target.1 - self.pos.1;

        // Calculate the length (magnitude) of the direction vector
        let length = (dir_x * dir_x + dir_y * dir_y).sqrt();

        // Normalize the direction vector
        let norm_dir_x = dir_x / length;
        let norm_dir_y = dir_y / length;

        let new_x = self.pos.0 + norm_dir_x * self.speed + (rand() % 5) as f32;
        let new_y = self.pos.1 + norm_dir_y * self.speed;
        self.move_tween_x = Tween::new(self.pos.0).set(new_x).duration(20);
        self.move_tween_y = Tween::new(self.pos.1).set(new_y).duration(20);
        self.state = UnitState::Moving;
    }

    fn take_damage(&mut self, damage: f32) {
        self.health -= damage;
    }

    fn start_attack(&mut self) {
        self.attack_timer = self.attack_time;
        self.state = UnitState::Attacking;
        //do whatever visual changes here
    }

    fn distance_to(&self, other: &Unit) -> f32 {
        let dx = self.pos.0 - other.pos.0;
        let dy = self.pos.1 - other.pos.1;
        (dx * dx + dy * dy).sqrt()
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

fn all_units_on_either_team_dead(units: &Vec<Unit>) -> bool {
    let all_team_1_dead = units
        .iter()
        .filter(|unit| unit.team == 0)
        .all(|unit| unit.state == UnitState::Dead);
    let all_team_2_dead = units
        .iter()
        .filter(|unit| unit.team == 1)
        .all(|unit| unit.state == UnitState::Dead);

    all_team_1_dead || all_team_2_dead
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Copy)]
enum UnitType {
    Tank,
    Speedy,
    DPS,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Copy)]
enum UnitState {
    Moving,
    Attacking,
    Idle,
    Dead,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
struct Team {
    name: String,
    units: Vec<UnitType>,
}

impl Team {
    fn new(name: String) -> Self {
        Self {
            name,
            units: Vec::new(),
        }
    }

    fn add_unit(&mut self, unit: UnitType) {
        self.units.push(unit);
    }

    fn num_unit(&self, unit_type: UnitType) -> i32 {
        // Return the number of units of a specific UnitType in self.units
        self.units.iter().filter(|&unit| *unit == unit_type).count() as i32
    }

    fn remove_unit(&mut self, unit_type: UnitType) -> bool {
        // Remove the last unit of the specified UnitType, only if there is at least one
        if let Some(pos) = self.units.iter().rposition(|&unit| unit == unit_type) {
            self.units.remove(pos);
            true
        } else {
            false
        }
    }
}
