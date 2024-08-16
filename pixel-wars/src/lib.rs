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
        }
    }
}

turbo::go!({
    let mut state = GameState::load();
    if state.phase == Phase::PreBattle {
        //handle input
        let gp = gamepad(0);
        if gp.up.just_pressed() {
            state.teams[0].add_unit(UnitType::Tank);
        }
        if gp.down.just_pressed() {
            state.teams[0].remove_unit(UnitType::Tank);
        }
        if gp.right.just_pressed() {
            state.teams[1].add_unit(UnitType::Speedy);
        }
        if gp.left.just_pressed() {
            state.teams[1].remove_unit(UnitType::Speedy);
        }
        if gp.start.just_pressed() {
            //generate units
            let row_height = 20.0;
            let row_width = 20.0;
            let max_y = 180.0; // Adjust this value as needed

            for (team_index, team) in state.teams.iter().enumerate() {
                let mut x_start = if team_index == 0 { 20.0 } else { 380.0 }; // Adjusted starting x for team 1
                let mut y_pos = 20.0;

                for (i, unit_type) in team.units.iter().enumerate() {
                    if y_pos > max_y {
                        // Reset y_pos and adjust x_start
                        y_pos = 20.0;
                        if team_index == 0 {
                            x_start += row_width;
                        } else {
                            x_start -= row_width;
                        }
                    }
                    let pos = (x_start, y_pos);
                    state
                        .units
                        .push(Unit::new(*unit_type, pos, team_index as i32));
                    y_pos += row_height;
                }
            }
            //go to game state
            state.phase = Phase::Battle;
        }
        draw_team_info_and_buttons(&mut state);
        while let Some(event) = state.event_queue.pop() {
            match event {
                GameEvent::AddUnitToTeam(team_index, unit_type) => {
                    state.teams[team_index].add_unit(unit_type);
                }
                GameEvent::ChangeSelectedUnit(unit_type) => {
                    // Logic to change the selected unit type
                }
                GameEvent::RemoveUnitFromTeam(team_index, unit_type) => {
                    state.teams[team_index].remove_unit(unit_type);
                }
            }
        }
    }

    if state.phase == Phase::Battle {
        let units_clone = state.units.clone();
        //let mut damage_map = Vec::new();

        //go through each unit, see what it wants to do, and handle all actions from here
        for unit in &mut state.units {
            //check if unit is moving or not
            if unit.state == UnitState::Idle {
                if let Some(index) = closest_enemy_index(&unit, &units_clone) {
                    if unit.distance_to(&units_clone[index]) < unit.range {
                        state.attacks.push(unit.start_attack(index));
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
        //go through attacks and update, then draw
        state.attacks.retain_mut(|attack| {
            let should_keep = !attack.update(&units_clone);
            attack.draw();

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
    splash_area: f32,
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
        let (damage, max_health, speed, range, attack_time, splash_area) = match unit_type {
            UnitType::Tank => (30.0, 200.0, 5.0, 16.0, 20, 0.),
            UnitType::Speedy => (8.0, 80.0, 15.0, 8.0, 40, 0.),
            UnitType::DPS => (50.0, 50.0, 5.0, 120.0, 120, 0.),
            UnitType::AOE => (80.0, 100.0, 5.0, 80.0, 40, 20.),
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
            splash_area,
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
                UnitType::DPS => {
                    let mut color: usize = 0x6A0DADFF;
                    if self.state == UnitState::Attacking {
                        color = 0x9370DBFF;
                    }
                    circ!(x = self.pos.0, y = self.pos.1, d = 6, color = color);
                }
                UnitType::AOE => {
                    let mut color: usize = 0xD3D3D3FF;
                    if self.state == UnitState::Attacking {
                        color = 0xA9A9A9FF;
                    }
                    rect!(
                        x = self.pos.0,
                        y = self.pos.1,
                        w = 16,
                        h = 16,
                        color = color
                    );
                }
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

        let rand_x = (rand() % 5) as f32 * norm_dir_x.signum();
        let rand_y = (rand() % 5) as f32 * norm_dir_y.signum();

        let new_x = self.pos.0 + norm_dir_x * self.speed + rand_x;
        let new_y = self.pos.1 + norm_dir_y * self.speed + rand_y;
        self.move_tween_x = Tween::new(self.pos.0).set(new_x).duration(20);
        self.move_tween_y = Tween::new(self.pos.1).set(new_y).duration(20);
        self.state = UnitState::Moving;
    }

    fn take_damage(&mut self, damage: f32) {
        self.health -= damage;
    }

    fn start_attack(&mut self, target_index: usize) -> Attack {
        self.attack_timer = self.attack_time;
        self.state = UnitState::Attacking;
        //create the actual attack
        let mut size = 1;
        if self.unit_type == UnitType::AOE {
            size = 2;
        }
        Attack::new(
            target_index,
            2.,
            self.pos,
            self.damage,
            self.splash_area,
            size,
        )
    }

    fn distance_to(&self, other: &Unit) -> f32 {
        let dx = self.pos.0 - other.pos.0;
        let dy = self.pos.1 - other.pos.1;
        (dx * dx + dy * dy).sqrt()
    }
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Copy)]
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

fn draw_team_info_and_buttons(state: &mut GameState) {
    let pos_0 = 20;
    let pos_1 = 200;
    let y_start = 20;
    let y_spacing = 20;
    let button_width = 20;
    let button_height = 10;

    // Draw info and buttons for Team 0 (Left side)
    let team_0 = &mut state.teams[0].clone();
    let mut y_pos = y_start;
    let name_text_0 = format!("{}:", team_0.name);
    text!(name_text_0.as_str(), x = pos_0, y = y_pos);
    y_pos += y_spacing;

    for unit_type in [
        UnitType::Tank,
        UnitType::Speedy,
        UnitType::DPS,
        UnitType::AOE,
    ]
    .iter()
    {
        let num_units = team_0.num_unit(*unit_type);
        let unit_text = format!("[{}] {:?}", num_units, unit_type);
        text!(unit_text.as_str(), x = pos_0, y = y_pos);

        // Plus Button
        let plus_button = Button::new(
            String::from("+"),
            (pos_0 as f32 + 100.0, y_pos as f32),
            (button_width as f32, button_height as f32),
            GameEvent::AddUnitToTeam(0, *unit_type),
        );
        plus_button.draw();
        plus_button.handle_click(state);

        // Minus Button
        let minus_button = Button::new(
            String::from("-"),
            (pos_0 as f32 + 130.0, y_pos as f32),
            (button_width as f32, button_height as f32),
            GameEvent::RemoveUnitFromTeam(0, *unit_type),
        );
        minus_button.draw();
        minus_button.handle_click(state);

        y_pos += y_spacing;
    }

    // Draw info and buttons for Team 1 (Right side)
    let team_1 = &mut state.teams[1].clone();
    y_pos = y_start;
    let name_text_1 = format!("{}:", team_1.name);
    text!(name_text_1.as_str(), x = pos_1, y = y_pos);
    y_pos += y_spacing;

    for unit_type in [
        UnitType::Tank,
        UnitType::Speedy,
        UnitType::DPS,
        UnitType::AOE,
    ]
    .iter()
    {
        let num_units = team_1.num_unit(*unit_type);
        let unit_text = format!("[{}] {:?}", num_units, unit_type);
        text!(unit_text.as_str(), x = pos_1, y = y_pos);

        // Plus Button
        let plus_button = Button::new(
            String::from("+"),
            (pos_1 as f32 + 100.0, y_pos as f32),
            (button_width as f32, button_height as f32),
            GameEvent::AddUnitToTeam(1, *unit_type),
        );
        plus_button.draw();
        plus_button.handle_click(state);

        // Minus Button
        let minus_button = Button::new(
            String::from("-"),
            (pos_1 as f32 + 130.0, y_pos as f32),
            (button_width as f32, button_height as f32),
            GameEvent::RemoveUnitFromTeam(1, *unit_type),
        );
        minus_button.draw();
        minus_button.handle_click(state);

        y_pos += y_spacing;
    }
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Copy)]
enum UnitType {
    Tank,
    Speedy,
    DPS,
    AOE,
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

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
enum GameEvent {
    AddUnitToTeam(usize, UnitType),
    RemoveUnitFromTeam(usize, UnitType),
    ChangeSelectedUnit(UnitType),
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
