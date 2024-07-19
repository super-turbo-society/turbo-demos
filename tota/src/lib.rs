use std::collections::BTreeMap;

use std::ops::Add;

// Define easing function types
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, BorshSerialize, BorshDeserialize)]
enum Easing {
    #[default]
    Linear,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,
    EaseInQuint,
    EaseOutQuint,
    EaseInOutQuint,
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,
    EaseInExpo,
    EaseOutExpo,
    EaseInOutExpo,
    EaseInCirc,
    EaseOutCirc,
    EaseInOutCirc,
    EaseInBack,
}

#[allow(unused)]
impl Easing {
    pub const ALL: [Self; 23] = [
        Self::Linear,
        Self::EaseInQuad,
        Self::EaseOutQuad,
        Self::EaseInOutQuad,
        Self::EaseInCubic,
        Self::EaseOutCubic,
        Self::EaseInOutCubic,
        Self::EaseInQuart,
        Self::EaseOutQuart,
        Self::EaseInOutQuart,
        Self::EaseInQuint,
        Self::EaseOutQuint,
        Self::EaseInOutQuint,
        Self::EaseInSine,
        Self::EaseOutSine,
        Self::EaseInOutSine,
        Self::EaseInExpo,
        Self::EaseOutExpo,
        Self::EaseInOutExpo,
        Self::EaseInCirc,
        Self::EaseOutCirc,
        Self::EaseInOutCirc,
        Self::EaseInBack,
    ];
    fn apply(&self, t: f64) -> f64 {
        match *self {
            Easing::Linear => t,
            Easing::EaseInQuad => t * t,
            Easing::EaseOutQuad => t * (2.0 - t),
            Easing::EaseInOutQuad => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }
            Easing::EaseInCubic => t * t * t,
            Easing::EaseOutCubic => {
                let t = t - 1.0;
                t * t * t + 1.0
            }
            Easing::EaseInOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    let t = t - 1.0;
                    (t * t * t * 4.0) + 1.0
                }
            }
            Easing::EaseInQuart => t * t * t * t,
            Easing::EaseOutQuart => {
                let t = t - 1.0;
                1.0 - t * t * t * t
            }
            Easing::EaseInOutQuart => {
                if t < 0.5 {
                    8.0 * t * t * t * t
                } else {
                    let t = t - 1.0;
                    1.0 - 8.0 * t * t * t * t
                }
            }
            Easing::EaseInQuint => t * t * t * t * t,
            Easing::EaseOutQuint => {
                let t = t - 1.0;
                t * t * t * t * t + 1.0
            }
            Easing::EaseInOutQuint => {
                if t < 0.5 {
                    16.0 * t * t * t * t * t
                } else {
                    let t = t - 1.0;
                    1.0 + 16.0 * t * t * t * t * t
                }
            }
            Easing::EaseInSine => 1.0 - (t * std::f64::consts::FRAC_PI_2).cos(),
            Easing::EaseOutSine => (t * std::f64::consts::FRAC_PI_2).sin(),
            Easing::EaseInOutSine => 0.5 * (1.0 - (std::f64::consts::PI * t).cos()),
            Easing::EaseInExpo => {
                if t == 0.0 {
                    0.0
                } else {
                    (2.0 as f64).powf(10.0 * (t - 1.0))
                }
            }
            Easing::EaseOutExpo => {
                if t == 1.0 {
                    1.0
                } else {
                    1.0 - (2.0 as f64).powf(-10.0 * t)
                }
            }
            Easing::EaseInOutExpo => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else if t < 0.5 {
                    (2.0 as f64).powf(10.0 * (2.0 * t - 1.0)) * 0.5
                } else {
                    (2.0 - (2.0 as f64).powf(-10.0 * (2.0 * t - 1.0))) * 0.5
                }
            }
            Easing::EaseInCirc => 1.0 - (1.0 - t * t).sqrt(),
            Easing::EaseOutCirc => (1.0 - (t - 1.0).powi(2)).sqrt(),
            Easing::EaseInOutCirc => {
                if t < 0.5 {
                    0.5 * (1.0 - (1.0 - 4.0 * t * t).sqrt())
                } else {
                    0.5 * ((-((2.0 * t - 2.0).powi(2) - 1.0)).sqrt() + 1.0)
                }
            }
            Easing::EaseInBack => {
                let c1 = 1.70158;
                let c3 = c1 + 1.;
                c3 * t * t * t - c1 * t * t
            }
        }
    }
}

// Define a generic Tween struct
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, BorshSerialize, BorshDeserialize)]
struct Tween<T> {
    start: T,
    end: T,
    duration: usize,
    elapsed: usize,
    easing: Easing,
    start_tick: Option<usize>,
}

#[allow(unused)]
impl<T> Tween<T>
where
    T: Copy + Default + PartialEq + Interpolate<T> + Add<Output = T>,
{
    fn new(start: T) -> Self {
        Self {
            start,
            end: start,
            duration: 0,
            elapsed: 0,
            easing: Easing::default(),
            start_tick: None,
        }
    }

    fn duration(&mut self, duration: usize) -> Self {
        self.duration = duration;
        *self
    }

    fn ease(&mut self, easing: Easing) -> Self {
        self.easing = easing;
        *self
    }

    fn set_duration(&mut self, duration: usize) {
        self.duration = duration;
    }

    fn set_ease(&mut self, easing: Easing) {
        self.easing = easing;
    }

    fn set(&mut self, new_end: T) -> Self {
        if new_end == self.end {
            return *self;
        }
        self.start = self.get();
        self.end = new_end;
        self.elapsed = 0;
        self.start_tick = Some(tick());
        *self
    }

    fn add(&mut self, delta: T) {
        self.start = self.get();
        self.end = self.end + delta;
        self.elapsed = 0;
        self.start_tick = Some(tick());
    }

    fn get(&mut self) -> T {
        if self.done() {
            return self.end;
        }
        if self.start_tick.is_none() {
            self.start_tick = Some(tick());
        }
        self.elapsed = tick() - self.start_tick.unwrap_or(0);
        let t = self.elapsed as f64 / self.duration.max(1) as f64;
        let eased_t = self.easing.apply(t);
        T::interpolate(eased_t, self.start, self.end)
    }

    fn done(&self) -> bool {
        self.duration == 0 || self.elapsed >= self.duration
    }
}

trait Interpolate<T> {
    fn interpolate(t: f64, start: T, end: T) -> T;
}

impl Interpolate<f32> for f32 {
    fn interpolate(t: f64, start: f32, end: f32) -> f32 {
        let n = start as f64 + (end as f64 - start as f64) * t;
        n as f32
    }
}

impl Interpolate<f64> for f64 {
    fn interpolate(t: f64, start: f64, end: f64) -> f64 {
        let n = start as f64 + (end as f64 - start as f64) * t;
        n
    }
}

impl Interpolate<usize> for usize {
    fn interpolate(t: f64, start: usize, end: usize) -> usize {
        let n = start as f64 + (end as f64 - start as f64) * t;
        n as usize
    }
}

impl Interpolate<isize> for isize {
    fn interpolate(t: f64, start: isize, end: isize) -> isize {
        let n = start as f64 + (end as f64 - start as f64) * t;
        n as isize
    }
}

impl Interpolate<u64> for u64 {
    fn interpolate(t: f64, start: u64, end: u64) -> u64 {
        let n = start as f64 + (end as f64 - start as f64) * t;
        n as u64
    }
}

impl Interpolate<i64> for i64 {
    fn interpolate(t: f64, start: i64, end: i64) -> i64 {
        let n = start as f64 + (end as f64 - start as f64) * t;
        n as i64
    }
}

impl Interpolate<u32> for u32 {
    fn interpolate(t: f64, start: u32, end: u32) -> u32 {
        let n = start as f64 + (end as f64 - start as f64) * t;
        n as u32
    }
}

impl Interpolate<i32> for i32 {
    fn interpolate(t: f64, start: i32, end: i32) -> i32 {
        let n = start as f64 + (end as f64 - start as f64) * t;
        n as i32
    }
}

impl Interpolate<u16> for u16 {
    fn interpolate(t: f64, start: u16, end: u16) -> u16 {
        let n = start as f64 + (end as f64 - start as f64) * t;
        n as u16
    }
}

impl Interpolate<i16> for i16 {
    fn interpolate(t: f64, start: i16, end: i16) -> i16 {
        let n = start as f64 + (end as f64 - start as f64) * t;
        n as i16
    }
}

impl Interpolate<u8> for u8 {
    fn interpolate(t: f64, start: u8, end: u8) -> u8 {
        let n = start as f64 + (end as f64 - start as f64) * t;
        n as u8
    }
}

impl Interpolate<i8> for i8 {
    fn interpolate(t: f64, start: i8, end: i8) -> i8 {
        let n = start as f64 + (end as f64 - start as f64) * t;
        n as i8
    }
}

// Define the game configuration using the turbo::cfg! macro
turbo::cfg! {r#"
    name = "Titans of the Apocalypse"
    version = "1.0.0"
    author = "Turbo"
    description = "Stack up your guns on the road to destruction!"
    [settings]
    resolution = [384, 216]
"#}


const ROW_POSITIONS: [i32; 3] = [32, 104, 152];
const COLUMN_POSITIONS: [i32; 2] = [176, 272];
const BULLET_SPEED: f32 = 6.0;
const TRUCK_BASE_OFFSET_X: i32 = 16;
const TRUCK_BASE_OFFSET_Y: i32 = 112;
//Enemy details
const ENEMY_MOVE_SPEED: f32 = 2.0;
const ENEMY_OFFSET_START: f32 = 240.0;
const TWEEN_DUR_MIN: usize = 90;
const TWEEN_RAND_ADJ: usize = 120;

const CHAR_WIDTH_L: u32 = 8;

// Define the game state initialization using the turbo::init! macro
turbo::init! {

    struct GameState {
        screen: enum Screen {
            Title(struct TitleScreen {
                elapsed: u32,
            }),
            Garage(struct GarageScreen {
                upgrade: Option<struct Upgrade {
                    kind: enum UpgradeKind {
                        MeatGrinder,
                        Truck,
                        CrookedCarburetor,
                        PsykoJuice,
                        Skull,
                        TheRipper,
                        BoomerBomb,
                        SlimeSpitter,
                        GoldfishGun,
                        CrapStack,
                        KnuckleBuster,
                        ThePersuader,
                        JailedDucks,
                        Boombox,
                        CanOfWorms,
                        SkullOfDeath,
                        Teepee,
                        EngineShield,
                    },
                    shape: struct Shape {
                        offset: (usize, usize),
                        size: (usize, usize),
                        cells: BTreeMap<(usize, usize), struct Cell {
                            edges: [bool; 4], // [top, right, bottom, left]
                        }>
                    },
                    cooldown_counter: i32,
                    cooldown_max: i32,
                    speed: i32,
                    endurance: i32,
                    brutality: i32,
                    firepower: i32,
                    hype: i32,
                    sprite_name: String,
                    is_active: bool,
                }>,
                upgrades: Vec<Upgrade>,  
                current_preset_index: usize,              
            }),
            UpgradeSelection(struct UpgradeSelectionScreen {
                upgrades: Vec<Upgrade>,
                options: Vec<Upgrade>,
                selected_index: usize,
                placing_upgrade: bool,
                existing_upgrades: Vec<Upgrade>, 
            }),
            Battle(struct BattleScreen {
                truck_tween: Tween<f32>,
                upgrades: Vec<Upgrade>,
                enemies: Vec<struct Enemy {
                    kind: enum EnemyKind {
                        Car,
                        Plane,
                    },
                    grid_position: (i32, i32),
                    max_health: i32,
                    health: i32,
                    damage: i32, //this is how much damage this enemy does when it attacks
                    position_offset: Tween<f32>, // This is the code to move the enemies into place
                }>,
                bullets: Vec<struct Bullet {
                    x: f32,
                    y: f32,
                    damage: i32,
                    is_enemy: bool,
                    path: Vec<(f32, f32)>,
                    current_path_index: usize,
                }>,
                explosions: Vec<struct Explosion {
                    x: f32,
                    y: f32,
                    timer: u32,
                }>,
                selected_index: usize,
                battle_state: enum BattleState {
                    PreCombat{
                        first_frame: usize,
                    }
                    ChooseAttack{
                        first_frame: bool,
                    },
                    AnimateAttack {
                        weapon_sprite: String,
                        weapon_position: (f32, f32),
                        target_position: (f32, f32),
                        target_enemies: Vec<usize>,
                        num_enemies_hit: usize,
                        active: bool,
                        damage: i32,
                    },
                    EnemiesAttack {
                        first_frame: bool,
                    },
                    StartingNewWave,
                    PostCombat{
                        first_frame: usize,
                    },
                },
                player_health: i32,
                waves: Vec<struct Wave{
                    enemies: Vec<Enemy>,
                }>,
                current_wave: usize,
                text_effects: Vec<struct TextEffect{
                    text: String,
                    text_color: u32,
                    background_color: u32,
                    text_x: i32,
                    text_y: i32,
                    text_duration: i32,

                }>,
            }),
            GameEnd(struct GameEndScreen{
                did_win: bool,
            }),
        },
        driver_name: String,
        saved_battle_screen: Option<BattleScreen>,
        fade_out: Tween<f32>,
    } = {
        Self {
            screen: Screen::Title(TitleScreen { elapsed: 0 }),
            //set this as "shoota" by default, but if you change the presets you have to change this to match the first preset
            driver_name: "shoota".to_string(),
            saved_battle_screen: None,
            fade_out: Tween::new(0.0).duration(20).ease(Easing::EaseInQuad),
            
        }
    }
}

enum ScreenTransition {
    ToUpgradeSelection(Vec<Upgrade>),
    BackToBattle,
    None,
}

impl GarageScreen {
    fn new() -> Self {
        let presets = car_presets();
        let upgrades = presets[0].upgrades.iter().map(|(upgrade, position)| {
            let mut upgrade = upgrade.clone();
            upgrade.shape.offset = *position;
            upgrade
        }).collect();
        
        Self {
            current_preset_index: 0,
            upgrades,
            upgrade: None,
            //current_preset_name: presets[0].name.to_string(),
        }
    }

    fn handle_input(&mut self, driver_name: &mut String) {
        let presets = car_presets();
        if gamepad(0).right.just_pressed() {
            self.current_preset_index = (self.current_preset_index + 1) % presets.len();
            self.set_upgrades(presets[self.current_preset_index].upgrades.clone());
            *driver_name = presets[self.current_preset_index].name.to_string();
        }
        if gamepad(0).left.just_pressed() {
            self.current_preset_index = if self.current_preset_index == 0 {
                presets.len() - 1
            } else {
                self.current_preset_index - 1
            };
            self.set_upgrades(presets[self.current_preset_index].upgrades.clone());
            *driver_name = presets[self.current_preset_index].name.to_string();
        }
    }

    fn set_upgrades(&mut self, new_upgrades: Vec<(Upgrade, (usize, usize))>) {
        self.upgrades = new_upgrades.into_iter().map(|(mut upgrade, position)| {
            upgrade.shape.offset = position;
            upgrade
        }).collect();
    }
}

impl UpgradeSelectionScreen {
    fn new(upgrades: Vec<Upgrade>) -> Self {
        let options = Self {
            upgrades: vec![],
            options: vec![],
            selected_index: 0,
            placing_upgrade: false,
            existing_upgrades: upgrades.clone(),
        }
        .generate_options();

        Self {
            upgrades: upgrades.clone(),
            options,
            selected_index: 0,
            placing_upgrade: false,
            existing_upgrades: upgrades,
        }
    }

    fn generate_options(&self) -> Vec<Upgrade> {
        let mut options = Vec::new();
        let mut existing_kinds = Vec::new();
        while options.len() < 3 {
            let mut new_upgrade = Upgrade::random();
            while new_upgrade.kind == UpgradeKind::Truck
                || existing_kinds.contains(&new_upgrade.kind)
                || !self.can_place_anywhere(&new_upgrade)
            {
                new_upgrade = Upgrade::random();
            }
            existing_kinds.push(new_upgrade.kind.clone());
            options.push(new_upgrade);
        }
        options
    }

    fn is_touching_below(&self, new_upgrade: &Upgrade) -> bool {
        for (pos, _) in &new_upgrade.shape.cells {
            let (new_x, new_y) = (pos.0 + new_upgrade.shape.offset.0, pos.1 + new_upgrade.shape.offset.1);

            for upgrade in &self.existing_upgrades {
                if upgrade as *const _ != new_upgrade as *const _ {
                    for (u_pos, _) in &upgrade.shape.cells {
                        let (existing_x, existing_y) = (u_pos.0 + upgrade.shape.offset.0, u_pos.1 + upgrade.shape.offset.1);

                        // Check if the new cell is directly above an existing cell
                        if new_x == existing_x && new_y + 1 == existing_y {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
    
    fn can_place_upgrade(&self, new_upgrade: &Upgrade) -> bool {
        let existing_shapes: Vec<Shape> = self.existing_upgrades.iter().map(|u| u.shape.clone()).collect::<Vec<_>>();
        let can_place = !new_upgrade.shape.overlaps_any(&existing_shapes) && self.is_touching_below(new_upgrade);
        can_place
    }

    fn can_place_upgrade_at_position(&self, new_upgrade: &Upgrade, position: (usize, usize)) -> bool {
        let mut new_upgrade = new_upgrade.clone();
        new_upgrade.shape.offset = position;

        // Check if the new shape would be out of bounds
        let (offset_x, offset_y) = position;
        let (shape_width, shape_height) = new_upgrade.shape.size;
        if offset_x + shape_width > 8 || offset_y + shape_height > 8 {
            return false;
        }

        let can_place = self.can_place_upgrade(&new_upgrade);
        can_place
    }

    fn can_place_anywhere(&self, new_upgrade: &Upgrade) -> bool {
        for x in 0..8 {
            for y in 0..8 {
                if self.can_place_upgrade_at_position(new_upgrade, (x, y)) {
                    return true;
                }
            }
        }
        false
    }

    fn handle_input(&mut self) -> ScreenTransition {
        if self.placing_upgrade {
            // Move the upgrade
            if let Some(last_upgrade) = self.upgrades.last_mut() {
                if gamepad(0).up.just_pressed() {
                    last_upgrade.shape.move_up();
                }
                if gamepad(0).down.just_pressed() {
                    last_upgrade.shape.move_down();
                }
                if gamepad(0).left.just_pressed() {
                    last_upgrade.shape.move_left();
                }
                if gamepad(0).right.just_pressed() {
                    last_upgrade.shape.move_right();
                }
            }

            // Check if the upgrade can be placed
            if gamepad(0).a.just_pressed() {
                if let Some(last_upgrade) = self.upgrades.last() {
                    if self.can_place_upgrade(last_upgrade) {
                        return ScreenTransition::BackToBattle;
                    }
                }
            }
        } else {
            if gamepad(0).right.just_pressed() {
                if self.selected_index == 0 {
                    self.selected_index = self.options.len() - 1;
                } else {
                    self.selected_index -= 1;
                }
            }
            if gamepad(0).left.just_pressed() {
                self.selected_index = (self.selected_index + 1) % self.options.len();
            }
            if gamepad(0).a.just_pressed() {
                if let Some(selected_upgrade) = self.options.get(self.selected_index) {
                    let mut new_upgrade = selected_upgrade.clone();
                    new_upgrade.shape.offset = (0, 0); // Set the position to (0, 0)
                    self.upgrades.push(new_upgrade);
                    self.placing_upgrade = true;
                }
            }
        }
        ScreenTransition::None
    }

    fn draw(&self, driver_name: &str) {
        clear!(0xeae0ddff);
        let [canvas_w, canvas_h] = canvas_size!();
        let grid_offset_x = ((canvas_w - 128) / 2) as usize; // Adjust 128 based on grid width
        let grid_offset_y = ((canvas_h - 128) / 2) as usize; // Adjust 128 based on grid height

        // Draw the grid
        sprite!("main_grid_16x16", x = grid_offset_x, y = grid_offset_y);

        for upgrade in &self.upgrades {
            if upgrade.kind == UpgradeKind::Truck {
                draw_truck(Some(upgrade.shape.offset.0 as i32 * 16 + grid_offset_x as i32), Some(upgrade.shape.offset.1 as i32 * 16 + grid_offset_y as i32), false, driver_name);
            } else {
                sprite!(
                    &upgrade.sprite_name,
                    x = upgrade.shape.offset.0 * 16 + grid_offset_x,
                    y = upgrade.shape.offset.1 * 16 + grid_offset_y,
                    opacity = 1
                );
            }
            upgrade.shape.draw(false, false, grid_offset_x as i32, grid_offset_y as i32);
        }
        if self.placing_upgrade {
            if let Some(last_upgrade) = self.upgrades.last() {
                let can_place = self.can_place_upgrade_at_position(last_upgrade, last_upgrade.shape.offset);
                let color = if can_place { 0x00ff0044u32 } else { 0xff000044u32 };
                last_upgrade.shape.draw(true, can_place, grid_offset_x as i32, grid_offset_y as i32);
            }
        }

        //find the values with the new upgrade, to draw the improved stat values in green    
        let mut upgrades_with_selected = self.upgrades.clone();
        if let Some(selected_upgrade) = self.options.get(self.selected_index) {
            upgrades_with_selected.push(selected_upgrade.clone());
        }
        draw_stats_panel(&self.upgrades, &upgrades_with_selected.to_vec());

        text!("CHOOSE AN UPGRADE", x = canvas_w / 2 - 69, y = 20, font = Font::L, color = 0x564f5bff);
        //draw arrows
        sprite!("arrow", x = 7, y = 105, rotate = 270);
        sprite!("arrow", x = 99, y = 105, rotate = 90);
        //draw upgrade
        sprite!(&self.options[self.selected_index].sprite_name, x = 30, y = 79);
        //draw frame
        sprite!("driver_frame", x = 30, y = 79);
        // Draw the new upgrade options on the left side of the screen
    }
}


impl Default for GameState {
    fn default() -> Self {
        Self {
            screen: Screen::Title(TitleScreen { elapsed: 0 }),
            driver_name: "shoota".to_string(),
            saved_battle_screen: None,
            fade_out: Tween::new(0.0).duration(20).ease(Easing::EaseInQuad),
        }
    }
}

impl BattleScreen {
    fn new(upgrades: Vec<Upgrade>) -> Self {
        let tween_dur_min = 90;
        let tween_rand_adj = 120;
        let tween = Tween::new(ENEMY_OFFSET_START).duration(tween_dur_min).ease(Easing::EaseOutQuart);
        // Initialize the waves
        let waves = vec![
            Wave {
                enemies: vec![
                    Enemy::new_car((0, 1), 4, 2),
                    Enemy::new_plane((1, 0), 2, 2),
                ],
            },
            Wave {
                enemies: vec![
                    Enemy::new_car((0, 1), 6, 3),
                    Enemy::new_car((0, 2), 4, 3),
                    Enemy::new_plane((1, 0), 2, 3),
                ],
            },
            Wave {
                enemies: vec![
                    Enemy::new_car((0, 1), 7, 3),
                    Enemy::new_car((0, 2), 5, 3),
                    Enemy::new_plane((0, 0), 3, 4),
                    Enemy::new_plane((1, 0), 3, 4),
                ],
            },
            Wave {
                enemies: vec![
                    Enemy::new_plane((0, 0), 3, 4),
                    Enemy::new_plane((1, 0), 4, 4),
                    Enemy::new_car((1, 1), 6, 3),
                    Enemy::new_car((0, 2), 6, 3),
                ],
            },
            Wave {
                enemies: vec![
                    Enemy::new_plane((0, 0), 3, 4),
                    Enemy::new_plane((1, 0), 4, 4),
                    Enemy::new_car((1, 1), 6, 3),
                    Enemy::new_car((0, 2), 7, 3),
                    Enemy::new_car((1, 2), 8, 3),
                    Enemy::new_car((0, 1), 9, 3),
                ],
            },
        ];

        Self {
            upgrades,
            enemies: waves[0].enemies.clone(), 
            bullets: vec![],
            explosions: vec![],
            selected_index: 1,
            battle_state: BattleState::StartingNewWave,
            player_health: 100,
            waves, 
            current_wave: 0, 
            text_effects : vec![], 
            truck_tween: Tween::new(0.0),
        }
    }
}


impl Upgrade {
    pub fn new(kind: UpgradeKind, shape: Shape, cooldown_max: i32, speed: i32, endurance: i32, brutality: i32, firepower: i32, hype: i32, sprite_name: String, is_active: bool) -> Self {
        Self {
            kind,
            shape,
            cooldown_counter: 0,
            cooldown_max,
            speed,
            endurance,
            brutality,
            firepower,
            hype,
            sprite_name,
            is_active,
        }
    }
    pub fn random() -> Self {
        match rand() % 15 {
            0 => Self::new_meat_grinder(),
            1 => Self::new_crooked_carburetor(),
            2 => Self::new_psyko_juice(),
            3 => Self::new_boomer_bomb(),
            4 => Self::new_the_ripper(),
            5 => Self::new_slime_spitter(),
            6 => Self::new_goldfish_gun(),
            7 => Self::new_crap_stack(),
            8 => Self::new_knuckle_buster(),
            9 => Self::new_the_persuader(),
            10 => Self::new_jailed_ducks(),
            11 => Self::new_boombox(),
            12 => Self::new_can_of_worms(),
            13 => Self::new_skull_of_death(),
            _ => Self::new_teepee(),
        }
    }

    #[rustfmt::skip]
    fn new_truck() -> Self {
        Self::new(UpgradeKind::Truck, {
            let mut cells = BTreeMap::new();
            cells.insert((4, 0), Cell { edges: [true, false, true, false] });
            cells.insert((5, 0), Cell { edges: [true, false, false, false] });
    
            cells.insert((0, 1), Cell { edges: [true, false, false, false] });
            cells.insert((1, 1), Cell { edges: [true, false, false, false] });
            cells.insert((2, 1), Cell { edges: [true, false, false, false] });
            cells.insert((3, 1), Cell { edges: [true, false, false, false] });
            cells.insert((4, 1), Cell { edges: [false, false, false, false] });
            cells.insert((5, 1), Cell { edges: [false, false, false, false] });
            cells.insert((6, 1), Cell { edges: [true, false, false, false] });
            cells.insert((7, 1), Cell { edges: [true, false, false, false] });
    
            cells.insert((0, 2), Cell { edges: [false, false, false, false] });
            cells.insert((1, 2), Cell { edges: [false, false, false, false] });
            cells.insert((2, 2), Cell { edges: [false, false, false, false] });
            cells.insert((3, 2), Cell { edges: [false, false, false, false] });
            cells.insert((4, 2), Cell { edges: [false, false, false, false] });
            cells.insert((5, 2), Cell { edges: [false, false, false, false] });
            cells.insert((6, 2), Cell { edges: [false, false, false, false] });
            cells.insert((7, 2), Cell { edges: [false, false, false, false] });
    
            Shape::new(cells)
        }, 5, 10, 0, 0, 0, 0, "truck".to_string(), false)
    }
    #[rustfmt::skip]
    fn new_meat_grinder() -> Self {
        Self::new(UpgradeKind::MeatGrinder, {
            let mut cells = BTreeMap::new();
            cells.insert((0, 0), Cell { edges: [true, true, true, true] });
            cells.insert((1, 0), Cell { edges: [true, true, true, true] });
            cells.insert((0, 1), Cell { edges: [true, true, true, true] });
            cells.insert((1, 1), Cell { edges: [true, true, true, true] });
            Shape::new(cells)
        }, 5, 0, 10, 5, 0, 2, "meat_grinder".to_string(), false)
    }
    #[rustfmt::skip]
    fn new_crooked_carburetor() -> Self {
        Self::new(UpgradeKind::CrookedCarburetor, {
            let mut cells = BTreeMap::new();
            cells.insert((1, 0), Cell { edges: [true, true, true, true] });
            cells.insert((1, 1), Cell { edges: [true, true, true, true] });
            cells.insert((0, 1), Cell { edges: [true, true, true, true] });
            cells.insert((0, 2), Cell { edges: [true, true, true, true] });
            Shape::new(cells)
        }, 0, 7, 3, 3, 0, 0, "crooked_carburetor".to_string(), false)
    }

    #[rustfmt::skip]
    fn new_psyko_juice() -> Self {
        Self::new(UpgradeKind::PsykoJuice, {
            let mut cells = BTreeMap::new();
            cells.insert((0, 0), Cell { edges: [true, true, true, true] });
            Shape::new(cells)
        }, 0, 4, 0, 0, 0, 0, "psyko_juice".to_string(), false)
    }

    #[rustfmt::skip]
    fn new_boomer_bomb() -> Self {
        Self::new(UpgradeKind::BoomerBomb, {
            let mut cells = BTreeMap::new();
            cells.insert((0, 0), Cell { edges: [false, true, false, false] });
            cells.insert((1, 0), Cell { edges: [false, false, false, false] });
            Shape::new(cells)
        }, 3, 0, 0, 0, 3, 0, "boomer_bomb".to_string(), true)
    }

    #[rustfmt::skip]
    fn new_the_ripper() -> Self {
        Self::new(UpgradeKind::TheRipper, {
            let mut cells = BTreeMap::new();
            cells.insert((0, 0), Cell { edges: [false, true, false, false] });
            cells.insert((1, 0), Cell { edges: [false, false, false, false] });
            cells.insert((2, 0), Cell { edges: [false, false, false, false] });
            Shape::new(cells)
        }, 2, 0, 0, 0, 1, 0, "the_ripper".to_string(), true)
    }
    fn new_slime_spitter() -> Self {
        Self::new(UpgradeKind::SlimeSpitter, {
            let mut cells = BTreeMap::new();
            cells.insert((0, 0), Cell { edges: [false, true, false, false] });
            cells.insert((1, 0), Cell { edges: [false, false, false, false] });
            Shape::new(cells)
        }, 3, 0, 0, 0, 3, 0, "slime_spitter".to_string(), true)
    }
    fn new_goldfish_gun() -> Self {
        Self::new(UpgradeKind::GoldfishGun, {
            let mut cells = BTreeMap::new();
            cells.insert((0, 0), Cell { edges: [false, true, false, false] });
            cells.insert((1, 0), Cell { edges: [false, false, false, false] });
            cells.insert((2, 0), Cell { edges: [false, false, false, false] });
            cells.insert((0, 1), Cell { edges: [false, false, false, false] });
            Shape::new(cells)
        }, 3, 0, 0, 0, 2, 0, "goldfish_gun".to_string(), true)   
    }
    fn new_crap_stack() -> Self {
        Self::new(UpgradeKind::CrapStack, {
            let mut cells = BTreeMap::new();
            cells.insert((0, 0), Cell { edges: [false, true, false, false] });
            Shape::new(cells)
        }, 1, 0, 0, 4, 0, 0, "crap_stack".to_string(), false)
    }
    fn new_knuckle_buster() -> Self {
        Self::new(UpgradeKind::KnuckleBuster, {
            let mut cells = BTreeMap::new();
            cells.insert((0, 0), Cell { edges: [false, true, false, false] });
            cells.insert((1, 0), Cell { edges: [false, true, false, false] });
            cells.insert((2, 0), Cell { edges: [false, true, false, false] });
            cells.insert((3, 0), Cell { edges: [false, true, false, false] });
            Shape::new(cells)
        }, 3, 0, 0, 0, 2, 0, "knuckle_buster".to_string(), true)
    }
    fn new_the_persuader() -> Self {
        Self::new(UpgradeKind::ThePersuader, {
            let mut cells = BTreeMap::new();
            cells.insert((1, 0), Cell { edges: [false, true, false, false] });
            cells.insert((2, 0), Cell { edges: [false, true, false, false] });
            cells.insert((3, 0), Cell { edges: [false, true, false, false] });
            cells.insert((0, 1), Cell { edges: [false, true, false, false] });
            cells.insert((1, 1), Cell { edges: [false, true, false, false] });
            Shape::new(cells)
        }, 2, 0, 0, 0, 2, 0, "the_persuader".to_string(), true)
    }
    fn new_jailed_ducks() -> Self {
        Self::new(UpgradeKind::JailedDucks, {
            let mut cells = BTreeMap::new();
            cells.insert((0, 0), Cell { edges: [false, true, false, false] });
            cells.insert((1, 0), Cell { edges: [false, true, false, false] });
            cells.insert((2, 0), Cell { edges: [false, true, false, false] });
            Shape::new(cells)
        }, 1, 2, 0, 10, 0, 3, "jailed_ducks".to_string(), false)
    }
    fn new_boombox() -> Self {
        Self::new(UpgradeKind::Boombox, {
            let mut cells = BTreeMap::new();
            cells.insert((0, 0), Cell { edges: [false, true, false, false] });
            cells.insert((1, 0), Cell { edges: [false, true, false, false] });
            Shape::new(cells)
        }, 1, 5, 2, 0, 0, 5, "boombox".to_string(), false)
    }
    fn new_can_of_worms() -> Self {
        Self::new(UpgradeKind::CanOfWorms, {
            let mut cells = BTreeMap::new();
            cells.insert((0, 0), Cell { edges: [false, true, false, false] });
            Shape::new(cells)
        }, 1, 0, 4, 0, 0, 0, "can_of_worms".to_string(), false)
    }
    fn new_skull_of_death() -> Self {
        Self::new(UpgradeKind::SkullOfDeath, {
            let mut cells = BTreeMap::new();
            cells.insert((0, 0), Cell { edges: [false, true, false, false] });
            Shape::new(cells)
        }, 1, 0, 0, 4, 0, 0, "skull_of_death".to_string(), false)
    }
    fn new_teepee() -> Self {
        Self::new(UpgradeKind::Teepee, {
            let mut cells = BTreeMap::new();
            cells.insert((0, 0), Cell { edges: [false, true, false, false] });
            Shape::new(cells)
        }, 1, 2, 0, 0, 0, 2, "teepee".to_string(), false)
    }
    fn new_engine_shield() -> Self {
        Self::new(UpgradeKind::EngineShield, {
            let mut cells = BTreeMap::new();
            cells.insert((0, 0), Cell { edges: [false, true, false, false] });
            cells.insert((0, 1), Cell { edges: [false, true, false, false] });
            cells.insert((1, 0), Cell { edges: [false, true, false, false] });
            cells.insert((1, 1), Cell { edges: [false, true, false, false] });
            Shape::new(cells)
        }, 1, 0, 0, 2, 2, 1, "engine_shield".to_string(), false)
    }

    fn get_weapon_path(&self, enemies: &[Enemy]) -> Vec<(f32, f32)> {
        let mut path = Vec::new();

        match self.kind {
            UpgradeKind::BoomerBomb => {
                let start_x = (self.shape.offset.0 * 16 + TRUCK_BASE_OFFSET_X as usize) as f32;
                let start_y = (self.shape.offset.1 * 16 + 32) as f32;
                let end_x = (COLUMN_POSITIONS[0] + COLUMN_POSITIONS[1]) as f32 / 2.0;
                let end_y = (ROW_POSITIONS[1] + ROW_POSITIONS[2]) as f32 / 2.0;

                let num_circles = 10; // Number of circles to draw
                for i in 0..num_circles {
                    let t = i as f32 / (num_circles - 1) as f32;
                    let x = start_x + t * (end_x - start_x);
                    // Create a parabolic effect
                    let y = start_y + t * (end_y - start_y) - (4.0 * t * (1.0 - t) * 50.0);
                    path.push((x, y));
                }
            },

            UpgradeKind::KnuckleBuster => {
                if let Some(first_enemy) = enemies.first() {
                    let start_position = (
                        self.shape.offset.0 as f32 * 16.0 + TRUCK_BASE_OFFSET_X as f32,
                        self.shape.offset.1 as f32 * 16.0 + 32.0,
                    );
                    let mid_position = (
                        start_position.0,
                        ROW_POSITIONS[first_enemy.grid_position.1 as usize] as f32,
                    );
                    let end_position = (
                        canvas_size()[0] as f32,
                        mid_position.1,
                    );
                    
                    let num_vertical_points = 3;
                    let num_horizontal_points = 6;

                    // Add vertical part of the path
                    for i in 0..=num_vertical_points {
                        let t = i as f32 / num_vertical_points as f32;
                        let x = start_position.0;
                        let y = start_position.1 * (1.0 - t) + mid_position.1 * t;
                        path.push((x, y));
                    }

                    // Add horizontal part of the path
                    for i in 0..=num_horizontal_points {
                        let t = i as f32 / num_horizontal_points as f32;
                        let x = mid_position.0 * (1.0 - t) + end_position.0 * t;
                        let y = mid_position.1;
                        path.push((x, y));
                    }
                }
            }

            _ => {
                let target_enemies = self.target_enemies_list(enemies.to_vec());
                if let Some(&first_enemy_index) = target_enemies.first() {
                    let start_x = (self.shape.offset.0 * 16 + TRUCK_BASE_OFFSET_X as usize) as f32;
                    let start_y = (self.shape.offset.1 * 16 + 32) as f32;
                    let (end_x, end_y) = calculate_target_position(enemies[first_enemy_index].grid_position);

                    let num_circles = 10; // Number of circles to draw
                    for i in 0..num_circles {
                        let t = i as f32 / (num_circles - 1) as f32;
                        let x = start_x + t * (end_x - start_x);
                        let y = start_y + t * (end_y - start_y);
                        path.push((x, y));
                    }
                }
            },
        }
        path
    }

    fn draw_weapon_path(&self, path: &[(f32, f32)]) {
        let circle_radius = 4.0;
        let circle_color : u32 = 0xff0000ff;
        for &(x, y) in path {
            circ!(x = x as i32, y = y as i32, d = circle_radius as u32, color = circle_color);
        }
    }

    fn target_enemies_list(&self, enemies: Vec<Enemy>) -> Vec<usize>{
        let mut target_enemies = Vec::new();
        match self.kind{
            //find the closest 
            UpgradeKind::BoomerBomb => {
                for (index, enemy) in enemies.iter().enumerate(){
                    if enemy.grid_position.0 == 0{
                        if enemy.grid_position.1 == 1 || enemy.grid_position.1 == 2{
                            target_enemies.push(index);
                        }
                    }
                }
            },
            //Find the closest plane, then also target any cars directly below it
            UpgradeKind::GoldfishGun=> {
                for (index, enemy) in enemies.iter().enumerate() {
                    if enemy.grid_position.0 == 0 && enemy.grid_position.1 == 0 {
                        target_enemies.push(index);
                        for (index, next_enemy) in enemies.iter().enumerate() {
                            if next_enemy.grid_position.0 ==0{
                                if next_enemy.grid_position.1 == 1 || next_enemy.grid_position.1 == 2{
                                    target_enemies.push(index);
                                    break;
                                }
                            }    
                    }
                    break;
                }
                else if enemy.grid_position.0 == 1 && enemy.grid_position.1 == 0{
                    target_enemies.push(index);
                        for (index, next_enemy) in enemies.iter().enumerate() {
                            if next_enemy.grid_position.0 == 1{
                                if next_enemy.grid_position.1 == 1 || next_enemy.grid_position.1 == 2{
                                    target_enemies.push(index)
                                }
                            }    
                        }
                    }
                }
            },
            //find the row with the most enemies, prioritizing the lowest, and add all in that row
            UpgradeKind::KnuckleBuster => {

                let mut a = 0;
                let mut b = 0;
                let mut c = 0;
                for enemy in enemies.clone() {
                    if enemy.grid_position.1 == 0 {
                        a+=1;
                    }
                    else if enemy.grid_position.1 == 1 {
                        b +=1;
                    }
                    else if enemy.grid_position.1 == 2{
                        c +=1;
                    }
                }
                for (index, enemy) in enemies.iter().enumerate() {
                    if c >= b && c >= a {
                        if enemy.grid_position.1 == 2{
                            target_enemies.push(index);
                        }
                    }
                    else if b >= a{
                        if enemy.grid_position.1 == 1 {
                            target_enemies.push(index);
                        }
                    }
                    else{
                        if enemy.grid_position.1 == 0{
                            target_enemies.push(index);
                        }
                    }
                
                }
            },
            //target 1 enemy, starting with the closest, starting from the bottom
            UpgradeKind::SlimeSpitter => {
                for (index, enemy) in enemies.iter().enumerate(){
                    if enemy.grid_position.0 == 0{
                        if enemy.grid_position.1 == 0{
                            target_enemies.push(index);
                            break;
                        }
                        else if enemy.grid_position.1 == 1{
                            target_enemies.push(index);
                            break;
                        }
                        else{
                            target_enemies.push(index);
                            break;
                        }
                    }
                    else if enemy.grid_position.0 == 1{
                        if enemy.grid_position.1 == 0{
                            target_enemies.push(index);
                            break;
                        }
                        else if enemy.grid_position.1 == 1{
                            target_enemies.push(index);
                            break;
                        }
                        else{
                            target_enemies.push(index);
                            break;
                        }

                    }
                }
            },

            //targets all air enemies
            UpgradeKind::ThePersuader => {
                for (index, enemy) in enemies.iter().enumerate(){
                if enemy.grid_position.1 == 0{
                    target_enemies.push(index);
                }
                }
            },

            //target all enemies
            UpgradeKind::TheRipper => {
                for (index, enemy) in enemies.iter().enumerate(){
                    target_enemies.push(index);
                }
            },
            _ => {}
        }
        return target_enemies;
    } 
    fn get_start_position(&self) -> f32{
        let rightmost = self.shape.cells.keys().map(|&(x, _)| x).max().unwrap_or(0) + 1;
        (self.shape.offset.0 + rightmost) as f32 * 16.0 + TRUCK_BASE_OFFSET_X as f32
    }
}


impl Shape {
    fn new(cells: BTreeMap<(usize, usize), Cell>) -> Self {
        let size = if cells.is_empty() {
            (0, 0)
        } else {
            let min_x = cells.keys().map(|&(x, _)| x).min().unwrap();
            let max_x = cells.keys().map(|&(x, _)| x).max().unwrap();
            let min_y = cells.keys().map(|&(_, y)| y).min().unwrap();
            let max_y = cells.keys().map(|&(_, y)| y).max().unwrap();
            (max_x - min_x + 1, max_y - min_y + 1)
        };
        Self {
            cells,
            offset: (0, 0),
            size,
        }
    }

    #[rustfmt::skip]
    fn semi() -> Self {
        let mut cells = BTreeMap::new();
        cells.insert((5, 0), Cell { edges: [true, false, true, false] });
        cells.insert((6, 0), Cell { edges: [true, false, false, false] });

        cells.insert((0, 1), Cell { edges: [true, false, false, false] });
        cells.insert((1, 1), Cell { edges: [true, false, false, false] });
        cells.insert((2, 1), Cell { edges: [true, false, false, false] });
        cells.insert((3, 1), Cell { edges: [true, false, false, false] });
        cells.insert((4, 1), Cell { edges: [true, false, false, false] });
        cells.insert((5, 1), Cell { edges: [false, false, false, false] });
        cells.insert((6, 1), Cell { edges: [false, false, false, false] });
        cells.insert((7, 1), Cell { edges: [true, false, false, false] });

        cells.insert((0, 2), Cell { edges: [false, false, false, false] });
        cells.insert((1, 2), Cell { edges: [false, false, false, false] });
        cells.insert((2, 2), Cell { edges: [false, false, false, false] });
        cells.insert((3, 2), Cell { edges: [false, false, false, false] });
        cells.insert((4, 2), Cell { edges: [false, false, false, false] });
        cells.insert((5, 2), Cell { edges: [false, false, false, false] });
        cells.insert((6, 2), Cell { edges: [false, false, false, false] });
        cells.insert((7, 2), Cell { edges: [false, false, false, false] });

        Self::new(cells)
    }

    fn move_up(&mut self) {
        self.offset.1 = self.offset.1.saturating_sub(1)
    }

    fn move_down(&mut self) {
        self.offset.1 = self
            .offset
            .1
            .saturating_add(1)
            .min(8_usize.saturating_sub(self.size.1))
    }

    fn move_left(&mut self) {
        self.offset.0 = self.offset.0.saturating_sub(1)
    }

    fn move_right(&mut self) {
        self.offset.0 = self
            .offset
            .0
            .saturating_add(1)
            .min(8_usize.saturating_sub(self.size.0))
    }

    fn get_cell(&self, x: usize, y: usize) -> Option<&Cell> {
        self.cells.get(&(x, y))
    }

    fn get_cell_edges(&self, x: usize, y: usize) -> Option<[bool; 4]> {
        self.get_cell(x, y).map(|cell| cell.edges)
    }

    fn overlaps(&self, other: &Shape) -> bool {
        for (&(x1, y1), _) in &self.cells {
            let global_x1 = x1 + self.offset.0;
            let global_y1 = y1 + self.offset.1;
            for (&(x2, y2), _) in &other.cells {
                let global_x2 = x2 + other.offset.0;
                let global_y2 = y2 + other.offset.1;
                // turbo::println!(
                //     "Checking overlap: self ({}, {}) -> ({}, {}), other ({}, {}) -> ({}, {})",
                //     x1, y1, global_x1, global_y1, x2, y2, global_x2, global_y2
                // );
                if global_x1 == global_x2 && global_y1 == global_y2 {
                    //turbo::println!("Overlap found: ({}, {}) with ({}, {})", global_x1, global_y1, global_x2, global_y2);
                    return true;
                }
            }
        }
        false
    }

    fn can_stick(&self, other: &Shape) -> bool {
        for (&(x1, y1), cell1) in &self.cells {
            let x1 = x1 + self.offset.0;
            let y1 = y1 + self.offset.1;

            for (&(x2, y2), cell2) in &other.cells {
                let x2 = x2 + other.offset.0;
                let y2 = y2 + other.offset.1;

                // cell1 top, cell2 bottom
                if x1 == x2 && y1 - 1 == y2 && cell1.edges[0] && cell2.edges[1] {
                    return true;
                }
                // cell1 bottom, cell2 top
                if x1 == x2 && y1 + 1 == y2 && cell1.edges[1] && cell2.edges[0] {
                    return true;
                }
                // cell1 left, cell2 right
                if x1 - 1 == x2 && y1 == y2 && cell1.edges[2] && cell2.edges[3] {
                    return true;
                }
                // cell1 right, cell2 left
                if x1 + 1 == x2 && y1 == y2 && cell1.edges[3] && cell2.edges[2] {
                    return true;
                }
            }
        }
        false
    }

    fn overlaps_any(&self, shapes: &[Shape]) -> bool {
        
        shapes.iter().any(|s| self.overlaps(s))
    }

    fn can_stick_any(&self, shapes: &[Shape]) -> bool {
        shapes.iter().any(|s| self.can_stick(s))
    }

    fn draw(&self, is_active: bool, can_place: bool, offset_x: i32, offset_y: i32) {
        let (x, y) = self.offset;
        let color = if can_place {
            0x00ff0080u32
        } else {
            0xff000080u32
        };
        for (pos, cell) in &self.cells {
            let (x, y) = (x + pos.0, y + pos.1);
            if x < 8 && y < 8 {
                let (x, y) = ((x * 16) + 1 + offset_x as usize, (y * 16) + 1 + offset_y as usize);
                let (w, h) = (16, 16);
                if is_active {
                    rect!(w = w, h = h, x = x, y = y, color = color);
                }
            }
        }
    }

    fn draw_mini(&self) {
        rect!(
            w = 8 * 6,
            h = 8 * 6,
            color = 0x00000000,
            border_color = 0xffffff33,
            border_width = 1,
        );
        for (pos, cell) in &self.cells {
            let (x, y) = (pos.0, pos.1);
            if x < 8 && y < 8 {
                let (x, y) = ((x * 6) + 1, (y * 6) + 1);
                let (w, h) = (4, 4);
                rect!(w = w, h = h, x = x, y = y, color = 0x00ff00ff);
                // top
                if cell.edges[0] {
                    rect!(w = w, h = 1, x = x, y = y, color = 0xff00ffff);
                }
                // bottom
                if cell.edges[1] {
                    rect!(w = w, h = 1, x = x, y = y + h - 1, color = 0xff00ffff);
                }
                // left
                if cell.edges[2] {
                    rect!(w = 1, h = h, x = x, y = y, color = 0xff00ffff);
                }
                // right
                if cell.edges[3] {
                    rect!(w = 1, h = h, x = x + w - 1, y = y, color = 0xff00ffff);
                }
            }
        }
    }
}

impl Enemy {
    fn new_car(grid_position: (i32, i32), max_health: i32, damage: i32) -> Self {
        let tween = Tween::new(ENEMY_OFFSET_START).duration(TWEEN_DUR_MIN).ease(Easing::EaseOutQuart);
        Self {
            kind: EnemyKind::Car,
            grid_position,
            max_health,
            health: max_health,
            damage,
            position_offset: tween.clone().duration(TWEEN_DUR_MIN + rand() as usize % TWEEN_RAND_ADJ),
        }
    }

    fn new_plane(grid_position: (i32, i32), max_health: i32, damage: i32) -> Self {
        let tween = Tween::new(ENEMY_OFFSET_START).duration(TWEEN_DUR_MIN).ease(Easing::EaseOutQuart);
        Self {
            kind: EnemyKind::Plane,
            grid_position,
            max_health,
            health: max_health,
            damage,
            position_offset: tween.clone().duration(TWEEN_DUR_MIN + rand() as usize % TWEEN_RAND_ADJ),
        }
    }
    
    fn draw(&mut self) {
        let (column, row) = self.grid_position;
        let x = COLUMN_POSITIONS[column as usize] + self.position_offset.get() as i32;
        let y = ROW_POSITIONS[row as usize];

        match self.kind {
            EnemyKind::Car => {
                // Draw enemy base
                // sprite!(
                //     "enemy_01_base",
                //     x = x,
                //     y = y,
                //     sw = 96,
                //     flip_x = true
                //  );
                sprite!(
                    "enemy_blue_car",
                    x = x,
                    y = y,
                    sw = 96,
                    //flip_x = true
                );
                // Draw enemy tires
                sprite!(
                    "enemy_blue_car_tires",
                    x = x,
                    y = y,
                    sw = 96,
                    fps = fps::FAST,
                );
                
                // Draw enemy shooter
                sprite!(
                    "enemy_gun_01",
                    x = x + 22,
                    y = y - 12,
                );
            },
            EnemyKind::Plane => {
                sprite!(
                    "enemy_03_base",
                    x = x,
                    y = y,
                    sw = 105,
                    fps = fps::FAST,
                );
            },
        }
    }

    fn draw_UI(&mut self) {
        let (column, row) = self.grid_position;
        let x = COLUMN_POSITIONS[column as usize] + self.position_offset.get() as i32;
        let y = ROW_POSITIONS[row as usize];
        let x_bar = x + 32;
        let y_bar = y - 12;
        let w_bar = 10 * self.max_health;
        let h_bar = 8;
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
        rect!(w = w_bar + 2, h = h_bar, x = x_bar - 1, 
            y = y_bar, color = 0, border_color = border_color, 
            border_width = 2, border_radius = 3);
    }
}


fn show_health(player_health: i32) {
    let full_rect_width = 40;
    let rect_height = 8;
    let x = 70;
    let y = 160;

    // Draw the full health bar background (black)
    rect!(
        w = full_rect_width,
        h = rect_height,
        x = x,
        y = y,
        color = 0x000000ff // Black color
    );

    // Draw the current health bar (red)
    let health_width = (player_health.max(0) as f32 / 100.0 * full_rect_width as f32) as i32;
    rect!(
        w = health_width,
        h = rect_height,
        x = x,
        y = y,
        color = 0xff0000ff // Red color
    );

    //border
    let border_color: u32 =  0xa69e9aff;
    rect!(w = full_rect_width + 2, h = rect_height, x = x-1, 
        y = y, color = 0, border_color = border_color, 
        border_width = 2, border_radius = 3);
}

fn draw_truck(x: Option<i32>, y: Option<i32>, should_animate: bool, driver_name: &str) {
    let x = x.unwrap_or(TRUCK_BASE_OFFSET_X); // Default x position
    let y = y.unwrap_or(TRUCK_BASE_OFFSET_Y); // Default y position
    let s_n = format!("{}_small", driver_name);
    let driver_x_offset = 76;
    sprite!("truck_base", x = x, y = y, sw = 128);
    sprite!(s_n.as_str(), x=x+driver_x_offset, y=y);
    if should_animate{
        sprite!("truck_tires", x = x, y = y, sw = 128, fps = fps::FAST);
        sprite!("truck_shadow", x=x, y=y, sw = 128, fps = fps::FAST);
    }
    else{
        sprite!("truck_tires", x = x, y = y, sw = 128);
        sprite!("truck_shadow", x=x, y=y, sw = 128);  
    }
}

// New function to draw the scrolling background
//TODO: Separate update and draw, and then stop scrolling if you are in the choosing attack phase
fn draw_background() {
    //draw the sun
    circ!(color = 0xfcf7b3ff, x=60, y=12, d=120);
    let width = canvas_size()[0];
    let t = tick() as f32;
    //draw the mountain
    sprite!("desert_bg",repeat = true, w=width, tx= -t * 0.5);
    //draw the bg dunes
    sprite!("mid_dunes",repeat = true, w=width, tx= -t*1.25, y=60);
    //draw the sides of the road
    rect!(color = 0xE1BF89ff, x = 0, y = canvas_size()[1] - 130, w = canvas_size()[0], h = 130);
    //draw the rocks in the road
    sprite!("fg_path",repeat = true, w=width, tx= -t*2.5, y=TRUCK_BASE_OFFSET_Y+9);
    //draw the foreground dunes
    sprite!("mid_dunes",repeat = true, w=width, tx= -t*4., y=190);
}


fn calculate_target_position(grid_position: (i32, i32)) -> (f32, f32) {
    let (column, row) = grid_position;
    let x = COLUMN_POSITIONS[column as usize];
    let y = ROW_POSITIONS[row as usize];
    (x as f32, y as f32)
}

fn create_enemy_bullet(bullets: &mut Vec<Bullet>, x: f32, y: f32, target_x: f32, target_y: f32, damage: i32) {
    let max_rand_x = 60.0;
    let max_rand_y = 15.0;

    let random_x = (rand() as i32 % (2 * max_rand_x as i32 + 1) - max_rand_x as i32) as f32;
    let random_y = (rand() as i32 % (2 * max_rand_y as i32 + 1) - max_rand_y as i32) as f32;

    let adjusted_target_x = target_x + random_x;
    let adjusted_target_y = target_y + random_y;

    let num_circles = 10;
    let mut path = Vec::new();
    for i in 0..num_circles {
        let t = i as f32 / (num_circles - 1) as f32;
        let x = x + t * (adjusted_target_x - x);
        let y = y + t * (adjusted_target_y - y);
        path.push((x, y));
    }

    bullets.push(Bullet::new(x, y, damage, true, path));
}

//TODO: Figure out why this is never used
// fn create_player_bullet(bullets: &mut Vec<Bullet>, x: f32, y: f32, target_x: f32, target_y: f32, damage: i32) {
//     bullets.push(Bullet::new(x, y, target_x, target_y, damage, false));
// }

fn should_draw_ui(battle_state: &BattleState) -> bool {
    matches!(
        battle_state,
        BattleState::AnimateAttack { .. }
        | BattleState::ChooseAttack { .. }
        | BattleState::EnemiesAttack { .. }
    )
}

fn draw_enemies(enemies: &mut [Enemy]) {
    for enemy in enemies.iter_mut() {
        enemy.draw();
    }
}

fn draw_enemy_ui(enemies: &mut [Enemy]) {
    for enemy in enemies.iter_mut() {
        enemy.draw_UI();
    }
}

impl Bullet {
    fn new(x: f32, y: f32, damage: i32, is_enemy: bool, path: Vec<(f32, f32)>) -> Self {
        Self {
            x,
            y,
            damage,
            is_enemy,
            path,
            current_path_index: 0,
        }
    }
    fn move_bullet(&mut self) -> bool {
        if self.current_path_index >= self.path.len() {
            return true;
        }

        let (target_x, target_y) = self.path[self.current_path_index];
        let dx = target_x - self.x;
        let dy = target_y - self.y;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance > BULLET_SPEED {
            let direction_x = dx / distance;
            let direction_y = dy / distance;
            self.x += direction_x * BULLET_SPEED;
            self.y += direction_y * BULLET_SPEED;
        } else {
            self.x = target_x;
            self.y = target_y;
            self.current_path_index += 1;
        }

        self.current_path_index >= self.path.len()
    }

    fn draw_bullet(&self) {
        let (next_x, next_y) = if self.current_path_index < self.path.len() {
            self.path[self.current_path_index]
        } else {
            (self.x, self.y)
        };
        let angle = (next_y - self.y).atan2(next_x - self.x);
        sprite!(
            "bullet",
            x = self.x,
            y = self.y,
            rotate = angle.to_degrees() + 90.0,
            scale_x = 0.175,
            scale_y = 0.175
        );
    }

    fn has_reached_target(&self) -> bool {
        self.current_path_index >= self.path.len()
    }

    fn set_target(&mut self, t_x: f32, t_y: f32) {
        if self.current_path_index < self.path.len() {
            self.path[self.current_path_index] = (t_x, t_y);
        }
    }
}

fn move_bullets(bullets: &mut Vec<Bullet>) {
    for bullet in bullets.iter_mut() {
        bullet.move_bullet();
    }
}

fn draw_bullets(bullets: &mut Vec<Bullet>){
    for bullet in bullets{
        bullet.draw_bullet();
    }
}

//called when you apply damage, 
fn create_explosion(explosions: &mut Vec<Explosion>, x: f32, y: f32) {
    explosions.push(Explosion {
        x,
        y: y - 20.0, //the exact position of the car is too low,
        timer: 0,
    });
}

//Go through the animation one time
fn advance_explosion_animation(explosions: &mut Vec<Explosion>) {
    let total_time = 20;
    let cell_width = 64;
    explosions.retain_mut(|explosion| {
        explosion.timer += 1;
        if explosion.timer <= total_time {
            sprite!("explosion_small", x = explosion.x, y = explosion.y, 
            sw = cell_width, fps = fps::FAST);
        } 
        explosion.timer <= total_time // Keep the explosion if the timer is under total time
    });
}

fn draw_portrait(spr_name: &str) {
    
    //draw arrows
    sprite!("arrow", x = 7, y = 105, rotate = 270);
    sprite!("arrow", x = 99, y = 105, rotate = 90);
    //draw portrait
    sprite!(spr_name, x=30, y=79);
    //draw frame
    sprite!("driver_frame", x=30, y=79);


}

fn draw_stats_panel(upgrades: &Vec<Upgrade>, new_upgrades: &Vec<Upgrade>,) {
    let [canvas_w, canvas_h] = canvas_size!();
    let text_x = canvas_w as i32 - 120;
    let text_y = (canvas_h as i32 / 2) - 70;

    StatBar::new("Speed", calculate_speed(upgrades)).set_is_improved(calculate_speed(new_upgrades))
        .draw_stat_bar(text_x, text_y);
    StatBar::new("Endurance", calculate_endurance(upgrades)).set_is_improved(calculate_endurance(new_upgrades))
        .draw_stat_bar(text_x, text_y+30);
    StatBar::new("Brutality", calculate_brutality(upgrades)).set_is_improved(calculate_brutality(new_upgrades))
        .draw_stat_bar(text_x, text_y+60);
    StatBar::new("Firepower", calculate_firepower(upgrades)).set_is_improved(calculate_firepower(new_upgrades))
        .draw_stat_bar(text_x, text_y+90);
    StatBar::new("Hype", calculate_hype(upgrades)).set_is_improved(calculate_hype(new_upgrades))
        .draw_stat_bar(text_x, text_y+120);

}

impl TextEffect{
    fn new(text: &str, text_color: u32, background_color: u32, text_x: i32, text_y: i32) -> Self{
        Self {
            text: text.to_string(),
            text_color,
            background_color,
            text_x,
            text_y,
            text_duration: 180,
        }
    }
    fn set_duration(&self, new_value: i32) -> Self {
        let mut next = self.clone();
        next.text_duration = new_value;
        return next;
    }

    fn draw(&self,) {
        let rect_width = self.text.len() as i32 * CHAR_WIDTH_L as i32 + 2;
        let border_color: u32 = 0xa69e9aff;
        let rect_height = 16;
        rect!(
           x = self.text_x-1,
           y = self.text_y,
           w = rect_width,
           h = rect_height,
           color = self.background_color 
        );
        text!(
            &self.text,
            x = self.text_x,
            y = self.text_y + 5,
            font = Font::L,
            color = self.text_color,
         );
         // Draw the rounded border
        rect!(w = rect_width + 2, h = rect_height, x = self.text_x-2, 
            y = self.text_y, color = 0, border_color = border_color, 
            border_width = 2, border_radius = 3);
    }

    fn update(&mut self,){
        self.text_duration -= 1;
    }
}


#[derive(Debug, Clone)]
struct StatBar{
    stat_name: String,
    stat_value: i32,
    improved_stat_value: i32,
}

impl StatBar{
    fn new(stat_name: &str, stat_value: i32) -> Self{
        Self {
            stat_name: stat_name.to_string(),
            stat_value,
            improved_stat_value: stat_value,  
        }
    }
    fn set_is_improved(&self, new_value: i32) -> Self {
        let mut next = self.clone();
        next.improved_stat_value = new_value;
        return next;
    }

    fn draw_stat_bar(&self, x: i32, y: i32) {
        let Self{
            stat_name,
            stat_value,
            improved_stat_value,
        } = self;
    
        let full_rect_width = 100;
        let rect_height = 14;
        let text_color: u32 = 0x564f5bff;
        let empty_color: u32 = 0xcbc6c1FF;
        let filled_color: u32 =  0xf8c53aff;
        let improved_color: u32 = 0x9daa3aff; 
        let border_color: u32 = 0xa69e9aff;
        let b_w = 2;
        let b_r = 3;
        let spacing = 10;
        
        // Print stat name text at position x/y
        text!(&stat_name, x = x, y = y, font = Font::L, color = text_color);
        
        // Draw the unfilled rectangle
        rect!(w = full_rect_width, h = rect_height, x = x+1, y = y + spacing, color = empty_color);

        rect!(w = improved_stat_value*2, h = rect_height, x = x+1, y = y + spacing, color = improved_color);
 
        rect!(w = stat_value*2, h = rect_height, x = x+1, y = y + spacing, color = filled_color);
        // Draw the rounded border
        rect!(w = full_rect_width + b_w, h = rect_height, x = x, y = y + spacing, color = 0, border_color = border_color, border_width = b_w, border_radius = b_r);
    }
}


struct CarPreset {
    name: &'static str,
    upgrades: Vec<(Upgrade, (usize, usize))>,
}

fn calculate_speed(upgrades: &Vec<Upgrade>) -> i32 {
    upgrades.iter().map(|u| u.speed).sum()
} 

fn calculate_endurance(upgrades: &Vec<Upgrade>) -> i32 {
    upgrades.iter().map(|u| u.endurance).sum()
}

fn calculate_brutality(upgrades: &Vec<Upgrade>) -> i32 {
    upgrades.iter().map(|u| u.brutality).sum()
}

fn calculate_firepower(upgrades: &Vec<Upgrade>) -> i32 {
    upgrades.iter().map(|u| u.firepower).sum()
}

fn calculate_hype(upgrades: &Vec<Upgrade>) -> i32 {
    upgrades.iter().map(|u| u.hype).sum()
}

fn car_presets() -> Vec<CarPreset> {
    vec![
        CarPreset {
            name: "shoota",
            upgrades: vec![
                (Upgrade::new_truck(), (0, 5)),
                (Upgrade::new_meat_grinder(), (2, 4)),
                (Upgrade::new_boomer_bomb(), (0, 5)),
                (Upgrade::new_knuckle_buster(), (0, 3)),
                (Upgrade::new_slime_spitter(), (6, 5)),
                (Upgrade::new_crooked_carburetor(), (4, 2)),
                (Upgrade::new_boombox(), (4, 1)),
            ],
        },
        CarPreset {
            name: "meatbag",
            upgrades: vec![
                (Upgrade::new_truck(), (0, 5)),
                (Upgrade::new_meat_grinder(), (0, 4)),
                (Upgrade::new_skull_of_death(), (4, 4)),
                (Upgrade::new_psyko_juice(), (5, 4)),
                (Upgrade::new_jailed_ducks(), (5, 3)),
                (Upgrade::new_goldfish_gun(), (0, 2)),
                (Upgrade::new_the_persuader(), (2, 3)),
                (Upgrade::new_the_ripper(), (2, 1)),
            ],
        },
        CarPreset {
            name: "lughead",
            upgrades: vec![
                (Upgrade::new_truck(), (0, 5)),
                (Upgrade::new_crap_stack(), (0, 5)),
                (Upgrade::new_slime_spitter(), (1, 3)),
                (Upgrade::new_boomer_bomb(), (0, 2)),
                (Upgrade::new_meat_grinder(), (2, 4)),
                (Upgrade::new_the_ripper(), (4, 4)),
                (Upgrade::new_can_of_worms(), (6, 5)),
            ],
        },
    ]
}

//utilites
fn text_pixel_width(text: &str) -> u32{
    return text.len() as u32 * CHAR_WIDTH_L;
}

fn centered_text_position(text: &str) -> u32{
    return  canvas_size()[0]/2 - text_pixel_width(text)/2;
}

fn rand_out_of_100(odds: u32) -> bool {
    let chance: u32 = (rand() % 100) as u32; // Generate a random number between 0 and 99
    chance < odds // Return true if chance is less than speed, otherwise false
}


turbo::go!({
    // Load the game state
    let mut state = GameState::load();
   
    //use next_screen to transition screens
    let mut next_screen: Option<Screen> = None;
    
    match &mut state.screen {
        Screen::Title(screen) => {
            clear!(0xfad6b8ff);
            let [canvas_w, canvas_h] = canvas_size!();
            let top = canvas_h.saturating_sub(screen.elapsed);
            //TODO: Turn these into tweens
            let foreground_start_pos = 78;
            let foreground_end_pos = 216;

            sprite!("title", y = top);
            sprite!("title_foreground", y = canvas_h.saturating_sub(((screen.elapsed as f32 / 2.7 as f32) as u32) + (canvas_h-foreground_start_pos)).max(canvas_h - foreground_end_pos));
            if top == 0 && tick() % 64 < 32 {
                text!("PRESS START", y = canvas_h - 32, x = (canvas_w / 2) - ((11 * 8) / 2), font = Font::L);
            }
            if gamepad(0).start.just_pressed() {
                if screen.elapsed < canvas_h {
                    screen.elapsed = canvas_h;
                } else {
                    // state.fade_out = Tween::new(1.0);
                    // state.fade_out.set(0.0);
                    next_screen = Some(Screen::Garage(GarageScreen::new()));
                }
            } else {
                screen.elapsed += 1;
            }
        },
        Screen::Garage(screen) => {
            clear!(0xeae0ddff);

            let [canvas_w, canvas_h] = canvas_size!();
            let grid_offset_x = ((canvas_w - 128) / 2 ) as usize; //Adjust 128 based on grid width to cetner it
            let grid_offset_y = ((canvas_h - 128) / 2 ) as usize; //Adjust 128 based on grid height
            
            screen.handle_input(&mut state.driver_name); 

            if gamepad(0).start.just_pressed() {
                next_screen = Some(Screen::Battle(BattleScreen::new(screen.upgrades.clone())));
            }
            
            //Draw the grid
            sprite!("main_grid_16x16", x=grid_offset_x, y=grid_offset_y);

            //Draw the upgraades
            for upgrade in &screen.upgrades {
                if upgrade.kind == UpgradeKind::Truck {
                    draw_truck(Some(upgrade.shape.offset.0 as i32 * 16 + grid_offset_x as i32), Some(upgrade.shape.offset.1 as i32 * 16 + grid_offset_y as i32), false, &state.driver_name);
                } else {
                    sprite!(
                        &upgrade.sprite_name,
                        x = upgrade.shape.offset.0 * 16 + grid_offset_x,
                        y = upgrade.shape.offset.1 * 16 + grid_offset_y,
                        opacity = 1
                    );
                }
            }

            draw_portrait(&state.driver_name); 
            draw_stats_panel(&screen.upgrades, &screen.upgrades);
            
            //draw central text
            let text = "CHOOSE YOUR DRIVER";
            text!(text, x = centered_text_position(text), y = 20, font = Font::L, color = 0x564f5bff);

        }

        Screen::UpgradeSelection(screen) => {
          
            match screen.handle_input() {
                ScreenTransition::BackToBattle => {
                    // Restore the saved Battle screen state and update upgrades
                    if let Some(mut battle_screen) = state.saved_battle_screen.take() {
                        battle_screen.upgrades = screen.upgrades.clone();
                        battle_screen.battle_state = BattleState::StartingNewWave;
                        next_screen = Some(Screen::Battle(battle_screen));
                    }
                },
                ScreenTransition::None => {},
                _ => {},
            }
            screen.draw(&state.driver_name);
            
        },

        Screen::Battle(screen) => {
            clear!(0xFFE0B7ff); //beige sky



            match &mut screen.battle_state {
               BattleState::PreCombat {first_frame } => {
                //reset the truck position
                screen.truck_tween = Tween::new(0.0);
                //sit and wait for 5 secoinds
                if tick() - *first_frame > 20{
                    for enemy in &mut screen.enemies{
                        if enemy.position_offset.elapsed > 0 && enemy.position_offset.elapsed < 45{
                            break;
                        }
                        if enemy.position_offset.get() == ENEMY_OFFSET_START{
                            enemy.position_offset.set(0.0);
                            break;
                        }   
                    }
                    let all_done = screen.enemies.iter().all(|enemy|{
                        enemy.position_offset.done()
                    });
                    if all_done{
                        let text = "TIME TO BATTLE!!";
                        let new_effect = TextEffect::new(
                            text,
                            0x564f5bFF,
                            0xcbc6c1FF,
                            centered_text_position(text) as i32,
                            10,
                        );
                        turbo::println!("CTP {:?}", centered_text_position(text));
                        screen.text_effects.push(new_effect);
                        screen.battle_state = BattleState::ChooseAttack { first_frame: true };   
                        }
                    }
                }
                BattleState::ChooseAttack { ref mut first_frame } => {
                    // Decrease cooldown counters
                    if *first_frame {
                        for upgrade in &mut screen.upgrades {
                            if upgrade.cooldown_counter > 0 {
                                upgrade.cooldown_counter -= 1;
                            }
                        }
                        
                        //set selected weapon to a usable weapon
                        let mut next_index = screen.selected_index;
                        loop {
                            next_index = (next_index + 1) % screen.upgrades.len();
                            if screen.upgrades[next_index].cooldown_counter == 0 && screen.upgrades[next_index].is_active{
                                break;
                            }
                        }
                        screen.selected_index = next_index;

                        *first_frame = false;
                    }

                    // Handle input for cycling through upgrades
                    if gamepad(0).up.just_pressed() || gamepad(0).right.just_pressed() {
                        //turbo::println!("PRESSED UP OR RIGHT {:?}", screen.enemies.len().to_string());

                        let mut next_index = screen.selected_index;
                        loop {
                            next_index = (next_index + 1) % screen.upgrades.len();
                            if screen.upgrades[next_index].cooldown_counter == 0 && screen.upgrades[next_index].is_active{
                                break;
                            }
                        }
                        screen.selected_index = next_index;
                    }
                    if gamepad(0).down.just_pressed() || gamepad(0).left.just_pressed() {
                        let mut prev_index = screen.selected_index;
                        loop {
                            if prev_index == 0 {
                                prev_index = screen.upgrades.len() - 1;
                            } else {
                                prev_index -= 1;
                            }
                            if screen.upgrades[prev_index].cooldown_counter == 0 && screen.upgrades[prev_index].is_active {
                                break;
                            }
                        }
                        screen.selected_index = prev_index;
                    }

                    // Handle attack selection
                    if gamepad(0).a.just_pressed() {
                        let selected_upgrade = &mut screen.upgrades[screen.selected_index];
                        //check if the weapon isn't on cooldown (theoretically should never happen bc of selection system)
                        if selected_upgrade.cooldown_counter == 0 {
                            let target_enemies = selected_upgrade.target_enemies_list(screen.enemies.clone());
                           
                           //TODO: set new sprites for each weapon to use as the bullet
                            let weapon_sprite = "bullet".to_string();

                            let target_position = if target_enemies.is_empty() {
                                let [canvas_w, _canvas_h] = canvas_size!();
                                (canvas_w as f32, selected_upgrade.shape.offset.1 as f32 * 16.0)
                            } else {
                                calculate_target_position(screen.enemies[target_enemies[0]].grid_position)
                            };

                            selected_upgrade.cooldown_counter = selected_upgrade.cooldown_max;
                            
                            screen.battle_state = BattleState::AnimateAttack {
                                weapon_sprite: weapon_sprite,
                                weapon_position: (
                                    selected_upgrade.shape.offset.0 as f32 * 16.0 + TRUCK_BASE_OFFSET_X as f32,
                                    selected_upgrade.shape.offset.1 as f32 * 16.0 + 32 as f32,
                                ),
                                target_position,
                                target_enemies,
                                num_enemies_hit: 0,
                                active: true,
                                damage: selected_upgrade.firepower,
                            };
                        }
                    }
                }

                BattleState::AnimateAttack { 
                    ref mut weapon_sprite, 
                    ref mut weapon_position,
                    ref mut target_position, 
                    ref mut target_enemies,
                    ref mut num_enemies_hit, 
                    ref mut active,
                    ref damage, 
                } => {
                    let mut new_battle_state: Option<BattleState> = None; // Temporary variable to hold the new battle state

                    if *active {
                        let selected_upgrade = &screen.upgrades[screen.selected_index];
                        let bullet_path = selected_upgrade.get_weapon_path(&screen.enemies);
                        let bullet = Bullet::new(
                            weapon_position.0,
                            weapon_position.1,
                            *damage,
                            false,
                            bullet_path,
                        );

                        screen.bullets.push(bullet);


                        *active = false;
                    }

                    move_bullets(&mut screen.bullets);

                    // Check if any bullet has reached its target
                    for bullet in screen.bullets.iter_mut() {
                        if bullet.has_reached_target() && !bullet.is_enemy {
                            if !target_enemies.is_empty() {
                                let enemy_index = target_enemies[*num_enemies_hit];
                                {
                                    let enemy = &mut screen.enemies[enemy_index];
                                    enemy.health -= bullet.damage;
                                    if rand_out_of_100(calculate_brutality(&screen.upgrades) as u32) {
                                        let text = "Brutality: Critical Hit";
                                        let new_effect = TextEffect::new(
                                            text,
                                            0x564f5bff,
                                            0xcbc6c1FF,
                                            centered_text_position(text) as i32,
                                            10,
                                        );
                                        screen.text_effects.push(new_effect);
                                        enemy.health = 0;
                                    }
                                }
                                create_explosion(&mut screen.explosions, bullet.x, bullet.y);

                                *num_enemies_hit += 1;

                                if target_enemies.len() > *num_enemies_hit {
                                    *target_position = calculate_target_position(screen.enemies[target_enemies[*num_enemies_hit]].grid_position);
                                    bullet.set_target(target_position.0,target_position.1);
                                } else {
                                    new_battle_state = Some(BattleState::EnemiesAttack { first_frame: true });
                                }
                            } else {
                                new_battle_state = Some(BattleState::EnemiesAttack { first_frame: true });
                            }
                        }
                    }
                    if let Some(state) = new_battle_state {
                        screen.enemies.retain(|e| e.health > 0);
                        screen.battle_state = state;
                    }
                },
                    

                BattleState::EnemiesAttack { ref mut first_frame } => {
                    //if all enemies are dead, transition to postcombat phase
                    if screen.enemies.is_empty() {
                        screen.battle_state = BattleState::PostCombat { first_frame: tick() };
                    } 
                    else {
                        if *first_frame {
                            //Apply Speed Effect here - if it is accurate, this will skip the enemy shooting phase
                            if !rand_out_of_100(calculate_speed(&screen.upgrades) as u32){
                                // Set the truck position for enemies to shoot at
                                let (truck_x, truck_y) = (50.0+TRUCK_BASE_OFFSET_X as f32, TRUCK_BASE_OFFSET_Y as f32);
                                
                                // Create bullets for each enemy
                                for enemy in &screen.enemies {
                                    let (enemy_x, enemy_y) = calculate_target_position(enemy.grid_position);
                                    //TODO: Add a delay to the bullets function, so we can create them all at once, but slowly 'release' them based on the delay
                                    //Roll endurance here and if it is 0, then we can apply endurance effects without passing screen values across everything
                                    let mut dmg = enemy.damage;
                                    if (rand_out_of_100(calculate_endurance(&screen.upgrades) as u32)){
                                        turbo::println!("ENDURANCE ACTIVE!");
                                        
                                        //create an endurance pop up - 
                                        //TODO: move this to when the damage is applied or just change how it works
                                        let text = "Endurance: Damage Blocked";
                                        let new_effect = TextEffect::new(
                                            text,
                                            0x564f5bff,
                                            0xcbc6c1FF,
                                            centered_text_position(text) as i32,
                                            10,
                                        );
                                        screen.text_effects.push(new_effect);

                                        dmg = 0
                                    }
                                    create_enemy_bullet(&mut screen.bullets, enemy_x, enemy_y, truck_x, truck_y, dmg);
                                }
                            }
                            else{
                                //apply speed effect here
                                let new_effect = TextEffect::new(
                                    "Speed Bonus: Shoot Again",
                                    0x564f5bff,
                                    0xcbc6c1FF,
                                    160,
                                    10,

                                );
                                screen.text_effects.push(new_effect);
                            }
                            *first_frame = false;
                        }

                        move_bullets(&mut screen.bullets);
                        
                        screen.bullets.retain(|bullet| {
                            if bullet.has_reached_target() {
                                if bullet.is_enemy {
                                    screen.player_health -= bullet.damage;
                                    create_explosion(&mut screen.explosions, bullet.x, bullet.y);
                                }
                                false // Remove the bullet
                            } else {
                                true // Keep the bullet
                            }
                        });

                        if screen.bullets.is_empty() {
                            if screen.player_health <= 0 {
                                next_screen = Some(Screen::GameEnd(GameEndScreen { did_win: false }));
                            } else {
                                screen.battle_state = BattleState::ChooseAttack { first_frame: true };
                            }
                        }
                    }
                }, 

                BattleState::StartingNewWave => {
                    //do all the cleanup here, e.g. make anything blank that needs to be blank
                    screen.bullets.clear();
                    screen.explosions.clear();
                    screen.text_effects.clear();
                    for upgrade in &mut screen.upgrades{
                        upgrade.cooldown_counter = 0;
                    }
                    //probably a better way to deal with this...
                    screen.selected_index = 1;
                    //include any wave transition stuff in here later, for now just transition to choose attack
                    screen.battle_state = BattleState::PreCombat  { first_frame: tick() };

                },   
                BattleState::PostCombat {first_frame } => {
                    if tick() == *first_frame+1{
                        //set a tween for a truck offset position
                        screen.truck_tween = Tween::new(0.0).duration(60).set(400.0).ease(Easing::EaseInBack);
                    }
                    //transition to upgrade selection
                    if screen.truck_tween.done(){
                        if screen.current_wave + 1 < screen.waves.len() {
                            screen.current_wave += 1;
                            screen.enemies = screen.waves[screen.current_wave].enemies.clone();
                            //give back 20 health (cap at 100)
                            screen.player_health = (screen.player_health + 20).min(100);
                            state.saved_battle_screen = Some(screen.clone()); // Save current Battle screen state
                            //this will also set us up to add some wiggle around the truck later on
                            next_screen = Some(Screen::UpgradeSelection(UpgradeSelectionScreen::new(screen.upgrades.clone())));
                            
                        }
                        else {
                            next_screen = Some(Screen::GameEnd(GameEndScreen { did_win: true }));
                        }
                        //screen.truck_tween.set(0.0);
                    }
                }
            }          
            
                //////////BATTLE STATE DRAWING CODE//////

                draw_background();
                
                // Draw upgrades
                let truck_pos = TRUCK_BASE_OFFSET_X + (screen.truck_tween.get() as i32);
                for (index, upgrade) in screen.upgrades.iter().enumerate() {
                    let is_selected = index == screen.selected_index;
                    if upgrade.kind == UpgradeKind::Truck {
                        draw_truck(Some(truck_pos), None, true, &state.driver_name);
                    } else {
                        sprite!(
                            &upgrade.sprite_name,
                            x = (upgrade.shape.offset.0 * 16) + truck_pos as usize,
                            y = (upgrade.shape.offset.1 * 16) + 32,
                        );
                    }
                    if should_draw_ui(&screen.battle_state){
                        upgrade.shape.draw(is_selected, true, TRUCK_BASE_OFFSET_X, 32);
                    }
                }

                // Draw enemies
                draw_enemies(&mut screen.enemies);
                
                // Determine the target enemies based on the selected weapon
                // Would be good to get this out of being 'every frame' eventually
                let selected_upgrade = &screen.upgrades[screen.selected_index];
                let target_enemies = selected_upgrade.target_enemies_list(screen.enemies.clone());
                let path = selected_upgrade.get_weapon_path(&screen.enemies);
                selected_upgrade.draw_weapon_path(&path);

                // // Highlight target enemies - this will change when we have a new highlight system
                // for &enemy_index in &target_enemies {
                //     let enemy = &screen.enemies[enemy_index];
                //     let (column, row) = enemy.grid_position;
                //     let y_position = ROW_POSITIONS[row as usize];
                //     rect!(
                //         w = 96,
                //         h = 50,
                //         x = COLUMN_POSITIONS[column as usize],
                //         y = y_position,
                //         color = 0xff0000aa // More solid red rectangle with higher opacity
                //     );
                // }

                // Highlight upgrades that have positive cooldown (e.g. turn red bc you can't use them)
                if should_draw_ui(&screen.battle_state){
                    for upgrade in &screen.upgrades {
                        if upgrade.cooldown_counter > 0 {
                            rect!(
                                w = upgrade.shape.size.0 as i32 * 16,
                                h = upgrade.shape.size.1 as i32 * 16,
                                x = upgrade.shape.offset.0 as i32 * 16 + TRUCK_BASE_OFFSET_X,
                                y = upgrade.shape.offset.1 as i32 * 16 + 32,
                                color = 0xff0000aa // More solid red rectangle with higher opacity
                            );
                        }
                    }
                }

                draw_bullets(&mut screen.bullets);
            
                // Advance explosion animations
                if !screen.explosions.is_empty() {
                    advance_explosion_animation(&mut screen.explosions);
                }

                // Show player health
                if should_draw_ui(&screen.battle_state){
                    show_health(screen.player_health);
                    draw_enemy_ui(&mut screen.enemies);
                }
                
                screen.text_effects.retain_mut(|text_effect| {
                    text_effect.update();
                    if text_effect.text_duration < 0 {
                        false // Remove it from the array
                    } else {
                        text_effect.draw();
                        true // Keep it in the array
                    }
                });
            },
        Screen::GameEnd(screen) => {
            clear!(0x000000ff); // Black background
            let [canvas_w, canvas_h] = canvas_size!();
            let text_width = 8 * 8; // Approximate width for text (8 characters, each 8 pixels wide)
            let message = if screen.did_win {
                "You Win"
            } else {
                "You Lose"
            };
            text!(
                message, 
                x = (canvas_w / 2) - (text_width / 2), 
                y = (canvas_h / 2) - 10, 
                font = Font::L, 
                color = 0xffffffff // White text
            );
        },
    }
    // let o = state.fade_out.get();
    // //turbo::println!("tween val {:?}", o);
    // rect!(x = 0, y=0, w=canvas_size()[0], h = canvas_size()[1], color = black_with_opacity(o));
    //rect!(x=0, y=0, w=100, h=100, color = 0x00ff0080u32);
    //change screens whenever next_screen is different from screen    
    if let Some(screen) = next_screen {
        //turbo::println!("IN THE LAST SCREEN FUNCTION");
        state.screen = screen;
    }

    state.save();
});


fn black_with_opacity(opacity: f32) -> u32 {
    // Convert the opacity to an 8-bit value (0-255)
    let alpha = (opacity * 255.0).round() as u32;

    // Combine the alpha value with the black color (0x000000)
    // Format: 0x000000AA
    0x00000000 | (alpha & 0xFF)
}