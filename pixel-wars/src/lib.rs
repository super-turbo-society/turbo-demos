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
    } = {
        let mut units = Vec::new();

        // Push 5 Tank units into the units vector
        for i in 0..5 {
            units.push(Unit::new(UnitType::Tank, (20., 30. * i as f32), 0));
        }
        for i in 0..10 {
            units.push(Unit::new(UnitType::Speedy, (200., 15. * i as f32), 1));
        }
        Self {
            phase: Phase::Battle,
            units,
        }
    }
}

turbo::go!({
    let mut state = GameState::load();
    let units_clone = state.units.clone();
    let mut damage_map = Vec::new();
    //go through each unit, see what it wants to do, and handle all actions from here
    for unit in &mut state.units {
        //check if unit is moving or not
        if unit.state == UnitState::Idle {
            //TODO: move enemy index out of here, just find closest living unit as a global fn
            if let Some(index) = unit.closest_enemy_index(&units_clone) {
                if unit.distance_to(&units_clone[index]) < unit.range {
                    damage_map.push((index, 1.));
                } else {
                    unit.move_toward_enemy(units_clone[index]);
                }
            }
        }
        unit.update();
        unit.draw();
    }
    for d in damage_map {
        state.units[d.0].take_damage(d.1);
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
}

impl Unit {
    fn new(unit_type: UnitType, pos: (f32, f32), team: i32) -> Self {
        // Initialize default values
        let (damage, max_health, speed, range) = match unit_type {
            UnitType::Tank => (20.0, 200.0, 2.5, 16.0),
            UnitType::Speedy => (10.0, 80.0, 10.0, 25.0),
            UnitType::DPS => (15.0, 100.0, 5.0, 10.0),
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
        }
    }
    fn update(&mut self) {
        if self.state == UnitState::Moving {
            self.pos.0 = self.move_tween_x.get();
            self.pos.1 = self.move_tween_y.get();
        }
        if self.move_tween_x.done() {
            self.state = UnitState::Idle;
        }
        if self.health <= 0.{
            self.state = UnitState::Dead;
        }
        //if moving or attacking, update tween and check if tween is done
        //if idle do nothing
    }

    fn draw(&self) {
        if self.state != UnitState::Dead{
            match self.unit_type {
                UnitType::Tank => {
                    rect!(
                        x = self.pos.0,
                        y = self.pos.1,
                        w = 12,
                        h = 12,
                        color = 0x0000ffff
                    );
                }
                UnitType::Speedy => {
                    circ!(x = self.pos.0, y = self.pos.1, d = 4, color = 0x00ff00ff);
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

    fn closest_enemy_index(&self, units: &Vec<Unit>) -> Option<usize> {
        units
            .iter()
            .enumerate()
            .filter(|(_, unit)| unit.team != self.team)
            .min_by(|(_, a), (_, b)| {
                let dist_a = self.distance_to(a);
                let dist_b = self.distance_to(b);
                dist_a
                    .partial_cmp(&dist_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(index, _)| index)
    }

    fn take_damage(&mut self, damage: f32) {
        self.health -= damage;
    }

    fn distance_to(&self, other: &Unit) -> f32 {
        let dx = self.pos.0 - other.pos.0;
        let dy = self.pos.1 - other.pos.1;
        (dx * dx + dy * dy).sqrt()
    }
}

fn distance_to(a: (f32, f32), b: (f32, f32)) -> f32 {
    let dx = a.0 - b.0;
    let dy = a.1 - b.1;
    (dx * dx + dy * dy).sqrt()
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
