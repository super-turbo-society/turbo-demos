use std::collections::BTreeMap;

// Define the game configuration using the turbo::cfg! macro
turbo::cfg! {r#"
    name = "Titans of the Apocalypse"
    version = "1.0.0"
    author = "Turbo"
    description = "Place shapes on a grid!"
    [settings]
    resolution = [384, 216]
"#}

const GRID_COLUMN_WIDTH: i32 = 96;
const GRID_ROW_HEIGHT: i32 = 72;
const GRID_ROW_LOW: i32 = 110; 
const GRID_ROW_HIGH: i32 = 36; 
const GRID_COLUMN_OFFSET: i32 = 152;
const BULLET_SPEED: f32 = 5.0;
//Enemy details
const ENEMY_MOVE_SPEED: f32 = 2.0;
const ENEMY_OFFSET_START: f32 = 200.0;

//tween mod
mod tween {
    use std::collections::HashMap;
    use turbo::prelude::tick;

    #[derive(Debug, Copy, Clone)]
    pub struct Tween {
        pub start: Option<f32>,
        pub curr: f32,
        pub end: f32,
        pub start_tick: usize,
        pub duration: f32,
        pub elapsed: f32,
        pub easing_fn: fn(f32) -> f32,
    }
    impl Tween {
        pub fn done(&self) -> bool {
            self.start == None
        }
        pub fn duration(&mut self, duration: f32) {
            self.duration = duration;
        }
        pub fn set(&mut self, end: f32) {
            if end == self.end {
                return;
            }
            if let Some(curr) = self.start {
                self.start = Some(curr);
            }
            self.end = end;
            self.start_tick = tick();
            self.elapsed = 0.;
        }
        pub fn get(&mut self, current_value: f32) -> f32 {
            if current_value == self.end {
                self.start = None;
                return current_value;
            }
            if self.start.is_none() {
                self.start = Some(current_value);
                self.curr = current_value;
                self.elapsed = 0.;
                self.start_tick = tick();
            }
            self.elapsed = (tick() - self.start_tick) as f32;

            let start = self.start.unwrap(); // Safe unwrap
            let t = if self.duration > 0.0 {
                self.elapsed / self.duration
            } else {
                1.0
            }
            .clamp(0.0, 1.0); // Ensure t is in the range [0, 1]

            let eased_t = (self.easing_fn)(t);

            self.curr = start + (self.end - start) * eased_t;

            self.curr
        }
    }

    // Macro to initialize and update tween groups
    #[macro_export]
    macro_rules! tween {
        ($key:expr, $easing_fn:expr, $duration:expr, $count:expr) => {
            unsafe {
                use std::collections::HashMap;
                use std::mem::MaybeUninit;

                use crate::tween::Tween;
                if crate::tween::TWEENS.is_none() {
                    crate::tween::TWEENS = Some(HashMap::new());
                }
                let groups = crate::tween::TWEENS.as_mut().unwrap();
                let tweens = [Tween {
                    start: None,
                    curr: 0.,
                    end: 0.,
                    start_tick: 0,
                    duration: 0.,
                    elapsed: 0.,
                    easing_fn: easing::linear,
                }; 1024];
                let group_entry = groups.entry($key.to_string()).or_insert(tweens);
                let group = &mut *group_entry;

                let mut ret: [std::mem::MaybeUninit<&mut Tween>; $count] =
                    std::mem::MaybeUninit::uninit().assume_init();

                for i in 0..$count {
                    // Safe mutable reference handling
                    group[i].duration = $duration;
                    group[i].easing_fn = $easing_fn;
                    if group[i].duration == 0. {
                        group[i].duration = 8.; // default duration
                    }
                    let elem = &mut group[i] as *mut Tween;
                    ret[i] = MaybeUninit::new(&mut *elem);
                }

                std::mem::transmute::<_, [&mut Tween; $count]>(ret)
            }
        };
    }

    pub static mut TWEENS: Option<HashMap<String, [Tween; 1024]>> = None;
}

mod easing {
    // Define the easing functions
    pub fn linear(t: f32) -> f32 {
        t
    }

    pub fn ease_in_quad(t: f32) -> f32 {
        t * t
    }

    pub fn ease_out_quad(t: f32) -> f32 {
        t * (2.0 - t)
    }
    pub fn ease_in_out_circ(x: f32) -> f32 {
        if x < 0.5 {
            (1.0 - (1.0 - (2.0 * x).powi(2)).sqrt()) / 2.0
        } else {
            ((1.0 - (-2.0 * x + 2.0).powi(2)).sqrt() + 1.0) / 2.0
        }
    }

    pub fn ease_out_back(x: f32) -> f32 {
        const C1: f32 = 1.70158;
        const C3: f32 = C1 + 1.0;
    
        1.0 + C3 * (x - 1.0).powf(3.0) + C1 * (x - 1.0).powf(2.0)
    }
}


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
                        AutoRifle,
                        Harpoon,
                        LaserGun,
                        SkullBox,
                        Truck,
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
                }>,
                upgrades: Vec<Upgrade>,  
                current_preset_index: usize,              
            }),
            Battle(struct BattleScreen {
                upgrades: Vec<Upgrade>,
                enemies: Vec<struct Enemy {
                    kind: enum EnemyKind {
                        Car,
                        Plane,
                    },
                    grid_position: (i32, i32),
                    health: i32,
                    damage: i32, //this is how much damage this enemy does when it attacks
                    position_offset: f32, // This is the code to move the enemies into place
                }>,
                bullets: Vec<struct Bullet {
                    x: f32,
                    y: f32,
                    target_x: f32,
                    target_y: f32,
                    damage: i32, //this comes from the enemy that shoots it
                }>,
                explosions: Vec<struct Explosion {
                    x: f32,
                    y: f32,
                    timer: u32,
                }>,
                selected_index: usize,
                battle_state: enum BattleState {
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
                        weapon_kind: UpgradeKind,
                    },
                    EnemiesAttack {
                        first_frame: bool,
                    },
                    StartingNewWave,
                    End,
                },
                bg_objects: Vec<struct ScrollingObject {
                    scroll_pos: i32,
                    sprite_name: String,
                    scroll_speed: i32,
                    bg_width: i32,
                    y: i32,
                }>,
                player_health: i32,
                waves: Vec<struct Wave{
                    enemies: Vec<Enemy>,
                }>,
                current_wave: usize,
            }),
        },
    } = {
        Self {
            screen: Screen::Title(TitleScreen { elapsed: 0 }),
        }
    }
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
        }
    }

    fn handle_input(&mut self) {
        let presets = car_presets();
        if gamepad(0).right.just_pressed() {
            self.current_preset_index = (self.current_preset_index + 1) % presets.len();
            self.set_upgrades(presets[self.current_preset_index].upgrades.clone());
        }
        if gamepad(0).left.just_pressed() {
            self.current_preset_index = if self.current_preset_index == 0 {
                presets.len() - 1
            } else {
                self.current_preset_index - 1
            };
            self.set_upgrades(presets[self.current_preset_index].upgrades.clone());
        }
    }

    fn set_upgrades(&mut self, new_upgrades: Vec<(Upgrade, (usize, usize))>) {
        self.upgrades = new_upgrades.into_iter().map(|(mut upgrade, position)| {
            upgrade.shape.offset = position;
            upgrade
        }).collect();
    }
}


impl Default for GameState {
    fn default() -> Self {
        Self {
            screen: Screen::Title(TitleScreen { elapsed: 0 }),
        }
    }
}

impl BattleScreen {
    fn new(upgrades: Vec<Upgrade>) -> Self {
        // Initialize the waves
        let waves = vec![
        Wave {
            enemies: vec![
                Enemy { kind: EnemyKind::Car, grid_position: (0, 1), health: 3, damage: 3, position_offset: ENEMY_OFFSET_START },
                Enemy { kind: EnemyKind::Plane, grid_position: (0, 0), health: 2, damage: 2, position_offset: ENEMY_OFFSET_START },
                Enemy { kind: EnemyKind::Car, grid_position: (1, 1), health: 3, damage: 3, position_offset: ENEMY_OFFSET_START },
                Enemy { kind: EnemyKind::Car, grid_position: (2, 1), health: 3, damage: 3, position_offset: ENEMY_OFFSET_START },
            ],
        },
        Wave {
            enemies: vec![
                Enemy { kind: EnemyKind::Plane, grid_position: (0, 0), health: 2, damage: 2, position_offset: ENEMY_OFFSET_START },
                Enemy { kind: EnemyKind::Plane, grid_position: (1, 0), health: 2, damage: 2, position_offset: ENEMY_OFFSET_START },
                Enemy { kind: EnemyKind::Plane, grid_position: (2, 0), health: 2, damage: 2, position_offset: ENEMY_OFFSET_START },
                Enemy { kind: EnemyKind::Car, grid_position: (2, 1), health: 3, damage: 3, position_offset: ENEMY_OFFSET_START },
            ],
        },
    ];

        Self {
            upgrades,
            enemies: waves[0].enemies.clone(), // Start with the first wave
            bullets: vec![],
            explosions: vec![],
            selected_index: 1,
            battle_state: BattleState::StartingNewWave,
            bg_objects: vec![
                ScrollingObject::new("desert_bg".to_string(), 0, 256, 0),
                ScrollingObject::new("mid_dunes".to_string(), 1, 256, 60),
                ScrollingObject::new("fg_path".to_string(), 2, 256, 85),
                ScrollingObject::new("mid_dunes".to_string(), 3, 256, 152),
                ScrollingObject::new("mid_dunes".to_string(), 4, 256, 175),
            ],
            player_health: 100,
            waves, // Store the waves
            current_wave: 0, // Start with the first wave
        }
    }
}


impl Upgrade {
    pub fn new(kind: UpgradeKind, shape: Shape, cooldown_max: i32, speed: i32, endurance: i32, brutality: i32, firepower: i32, hype: i32) -> Self {
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
        }
    }
    pub fn random() -> Self {
        match rand() % 4 {
            0 => Self::new_auto_rifle(),
            1 => Self::new_harpoon(),
            2 => Self::new_laser_gun(),
            _ => Self::new_skull_box(),
        }
    }
    pub fn random_placeable(shapes: &[Shape]) -> Option<Self> {
        let upgrades = [
            Self::new_auto_rifle(),
            Self::new_harpoon(),
            Self::new_laser_gun(),
            Self::new_skull_box(),
        ];
        let len = upgrades.len();
        let n = rand() as usize;
        for i in 0..len {
            let u = &mut upgrades[(n + i) % len].clone();
            let (w, h) = u.shape.size;
            let max_x = (8_usize).saturating_sub(w);
            let max_y = (8_usize).saturating_sub(h);
            for i in 0..=max_x {
                for j in 0..=max_y {
                    u.shape.offset = (i, j);
                    if !u.shape.overlaps_any(&shapes) && u.shape.can_stick_any(&shapes) {
                        //turbo::println!("NO OVERLAP AND CAN STICK! {:?}", u.shape.offset);
                        return Some(u.clone());
                    }
                }
            }
        }
        None
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
        }, 5, 10, 0, 0, 0, 0)
    }
    #[rustfmt::skip]
    fn new_skull_box() -> Self {
        Self::new(UpgradeKind::SkullBox, {
            let mut cells = BTreeMap::new();
            cells.insert((0, 0), Cell { edges: [true, true, true, true] });
            cells.insert((1, 0), Cell { edges: [true, true, true, true] });
            cells.insert((0, 1), Cell { edges: [true, true, true, true] });
            cells.insert((1, 1), Cell { edges: [true, true, true, true] });
            Shape::new(cells)
        }, 5, 0, 3, 1, 0, 2)
    }
    #[rustfmt::skip]
    fn new_auto_rifle() -> Self {
        Self::new(UpgradeKind::AutoRifle, {
            let mut cells = BTreeMap::new();
            cells.insert((0, 0), Cell { edges: [false, true, false, false] });
            cells.insert((1, 0), Cell { edges: [false, false, false, false] });
            Shape::new(cells)
        }, 3, 0, 0, 1, 2, 1)
    }
    #[rustfmt::skip]
    fn new_harpoon() -> Self {
        Self::new(UpgradeKind::Harpoon, {
            let mut cells = BTreeMap::new();
            cells.insert((0, 0), Cell { edges: [false, false, true, false] });
            cells.insert((1, 0), Cell { edges: [false, false, false, false] });
            cells.insert((2, 0), Cell { edges: [false, false, false, false] });
            Shape::new(cells)
        }, 4, 0, 0, 3, 5, 3)
    }
    #[rustfmt::skip]
    fn new_laser_gun() -> Self {
        Self::new(UpgradeKind::LaserGun, {
            let mut cells = BTreeMap::new();
            cells.insert((0, 0), Cell { edges: [false, true, false, false] });
            cells.insert((1, 0), Cell { edges: [false, false, false, false] });
            Shape::new(cells)
        }, 4, 0, 0, 2, 3, 2)
    }
}

impl UpgradeKind {
    fn to_str<'a>(&self) -> &'a str {
        match self {
            Self::AutoRifle => "auto_rifle",
            Self::Harpoon => "harpoon",
            Self::LaserGun => "laser_gun",
            Self::SkullBox => "skull_box",
            Self::Truck => "truck",
        }
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

    #[rustfmt::skip]
    fn new_weird_slash() -> Self {
        let mut cells = BTreeMap::new();
        cells.insert((0, 0), Cell { edges: [true, false, false, true] });
        cells.insert((1, 1), Cell { edges: [false, true, true, false] });
        cells.insert((2, 2), Cell { edges: [true, false, true, false] });
        cells.insert((3, 3), Cell { edges: [false, true, false, true] });
        Self::new(cells)
    }

    #[rustfmt::skip]
    fn new_square_2x2() -> Self {
        let mut cells = BTreeMap::new();
        cells.insert((0, 0), Cell { edges: [true, true, true, true] });
        cells.insert((1, 0), Cell { edges: [true, true, true, true] });
        cells.insert((0, 1), Cell { edges: [true, true, true, true] });
        cells.insert((1, 1), Cell { edges: [true, true, true, true] });
        Self::new(cells)
    }

    #[rustfmt::skip]
    fn new_s_thing() -> Self {
        let mut cells = BTreeMap::new();
        cells.insert((0, 0), Cell { edges: [true, true, true, true] });
        cells.insert((0, 1), Cell { edges: [true, true, true, true] });
        cells.insert((1, 1), Cell { edges: [true, true, true, true] });
        cells.insert((1, 2), Cell { edges: [true, true, true, true] });
        Self::new(cells)
    }

    #[rustfmt::skip]
    fn new_l_guy() -> Self {
        let mut cells = BTreeMap::new();
        cells.insert((0, 0), Cell { edges: [true, true, true, true] });
        cells.insert((0, 1), Cell { edges: [true, true, true, true] });
        cells.insert((0, 2), Cell { edges: [true, true, true, true] });
        cells.insert((0, 3), Cell { edges: [true, true, true, true] });
        cells.insert((1, 3), Cell { edges: [true, true, true, true] });
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
                if global_x1 == global_x2 && global_y1 == global_y2 {
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
            0x00ff0044u32
        } else {
            0xff000044u32
        };
        for (pos, cell) in &self.cells {
            let (x, y) = (x + pos.0, y + pos.1);
            if x < 8 && y < 8 {
                let (x, y) = ((x * 16) + 1 + offset_x as usize, (y * 16) + 1 + offset_y as usize);
                let (w, h) = (14, 14);
                if is_active {
                    rect!(w = w, h = h, x = x, y = y, color = color);
                }
                // top
                if cell.edges[0] {
                    rect!(w = w, h = 1, x = x, y = y, color = 0xff00ff66);
                }
                // bottom
                if cell.edges[1] {
                    rect!(w = w, h = 1, x = x, y = y + h - 1, color = 0xff00ff66);
                }
                // left
                if cell.edges[2] {
                    rect!(w = 1, h = h, x = x, y = y, color = 0xff00ff66);
                }
                // right
                if cell.edges[3] {
                    rect!(w = 1, h = h, x = x + w - 1, y = y, color = 0xff00ff66);
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

impl ScrollingObject {
    // Constructor for ScrollingObject
    fn new(sprite_name: String, scroll_speed: i32, bg_width: i32, y: i32) -> Self {
        Self {
            scroll_pos: 0,
            sprite_name,
            scroll_speed,
            bg_width,
            y,
        }
    }

    // Update the scroll position
    fn update(&mut self) {
        self.scroll_pos -= self.scroll_speed;
        if self.scroll_pos <= -self.bg_width {
            self.scroll_pos += self.bg_width;
        }
    }

    // Draw the scrolling background
    fn draw(&self) {
        sprite!(&self.sprite_name, x = self.scroll_pos, y = self.y);
        sprite!(&self.sprite_name, x = self.scroll_pos + self.bg_width, y = self.y);
        sprite!(&self.sprite_name, x = self.scroll_pos + 2 * self.bg_width, y = self.y);
    }
}

fn scroll_bg_object(objects: &mut [ScrollingObject], index: usize) {
    if let Some(object) = objects.get_mut(index) {
        object.update();
        object.draw();
    }
}

fn show_health(player_health: i32) {
    let full_rect_width = 40;
    let rect_height = 8;
    let x = 70;
    let y = 130;

    // Draw the full health bar background (black)
    rect!(
        w = full_rect_width,
        h = rect_height,
        x = x,
        y = y,
        color = 0x000000ff // Black color
    );

    // Draw the current health bar (red)
    let health_width = (player_health as f32 / 100.0 * full_rect_width as f32) as i32;
    rect!(
        w = health_width,
        h = rect_height,
        x = x,
        y = y,
        color = 0xff0000ff // Red color
    );
}

fn draw_truck(x: i32, y: i32) {
    let x = 0;
    let y = 80;
    sprite!("truck_base", x = x, y = y, sw = 128);
    sprite!("suzee", x=76, y=y);
    sprite!("truck_tires", x = x, y = y, sw = 128, fps = fps::FAST);
    sprite!("truck_shadow", x=x, y=y, sw = 128, fps = fps::FAST);
    
}

// New function to draw the scrolling background
fn draw_background(objects: &mut [ScrollingObject]) {
    //draw the sun
    circ!(color = 0xfcf7b3ff, x=60, y=12, d=120);
    //Scroll mountain bg (it's actually static though, just implemented this way for now)
    scroll_bg_object(objects, 0);
    //draw rolling middle bg
    scroll_bg_object(objects, 1);
    //draw a rect for the rest of the road
    rect!(color = 0xE1BF89ff, x = 0, y = canvas_size()[1] - 130, w = canvas_size()[0], h = 130);
    //draw scrolling road in middle
    scroll_bg_object(objects, 2);
    //draw the dunes at the bottom
    scroll_bg_object(objects,3 );
    scroll_bg_object(objects,4);
}


fn calculate_target_position(grid_position: (i32, i32)) -> (f32, f32) {
    let (column, row) = grid_position;
    let x = column as f32 * GRID_COLUMN_WIDTH as f32 + GRID_COLUMN_OFFSET as f32;
    let y = row as f32 * GRID_ROW_HEIGHT as f32 + GRID_ROW_HIGH as f32;
    (x, y)
}

fn create_enemy_bullet(bullets: &mut Vec<Bullet>, x: f32, y: f32, target_x: f32, target_y: f32, damage: i32) {
    let max_rand_x = 30.0;
    let max_rand_y = 15.0;

    // Generate random values between -max_rand_x and max_rand_x, and -max_rand_y and max_rand_y
    let random_x = (rand() as i32 % (2 * max_rand_x as i32 + 1) - max_rand_x as i32) as f32;
    let random_y = (rand() as i32 % (2 * max_rand_y as i32 + 1) - max_rand_y as i32) as f32;

    // Print the random values for debugging
    turbo::println!("random_x: {}", random_x);
    turbo::println!("random_y: {}", random_y);

    // Add randomness to the target position
    let adjusted_target_x = target_x + random_x;
    let adjusted_target_y = target_y + random_y;

    bullets.push(Bullet {
        x,
        y,
        target_x: adjusted_target_x,
        target_y: adjusted_target_y,
        damage,
    });
}

fn draw_enemies(enemies: &mut [Enemy]) {
    // Iterate over enemies and set their positions using tweens
    for (i, enemy) in enemies.iter_mut().enumerate() {
        let (column, row) = enemy.grid_position;
        let x = GRID_COLUMN_OFFSET + column * GRID_COLUMN_WIDTH;
        //if i == 0 {turbo::println!("End X {:?}", end_x_position);}
        let y = GRID_ROW_HIGH + (row * GRID_ROW_HEIGHT);

        match enemy.kind {
            EnemyKind::Car => {
                // Draw enemy driver
                sprite!(
                    "lughead",
                    x = x + 40, // Adjust this offset as needed
                    y = y + 0,  // Adjust this offset as needed
                    flip_x = true
                );

                // Draw enemy base
                sprite!(
                    "enemy_01_base",
                    x = x,
                    y = y,
                    sw = 95.0
                );

                // Draw enemy tires
                sprite!(
                    "enemy_01_tires",
                    x = x,
                    y = y,
                    sw = 95,
                    fps = fps::FAST,
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
}

fn move_bullets(bullets: &mut Vec<Bullet>, explosions: &mut Vec<Explosion>, target_x: f32, target_y: f32, player_health: &mut i32) {
    bullets.retain_mut(|bullet| {
        //bullet.position.x = tween(start, end, elapsed_time_as_a_percentage_of_1, easing type)
        let dx = bullet.target_x - bullet.x;
        let dy = bullet.target_y - bullet.y;
        let distance = (dx * dx + dy * dy).sqrt();
        if distance > 1.0 {
            let direction_x = dx / distance;
            let direction_y = dy / distance;
            bullet.x += direction_x * BULLET_SPEED;
            bullet.y += direction_y * BULLET_SPEED;
        } else {
            bullet.x = bullet.target_x;
            bullet.y = bullet.target_y;
        }

        let angle = dy.atan2(dx);
        sprite!(
            "bullet",
            x = bullet.x,
            y = bullet.y,
            rotate = angle.to_degrees() + 90.0,
            scale_x = 0.175,
            scale_y = 0.175
        );

        if (bullet.x - bullet.target_x).abs() < BULLET_SPEED && (bullet.y - bullet.target_y).abs() < BULLET_SPEED {
            *player_health -= bullet.damage;
            create_explosion(explosions, bullet.x, bullet.y); // Create explosion
            false // Remove bullet because it hit the player
        } else {
            true // Keep bullet
        }
    });
}

//called when you apply damage, 
fn create_explosion(explosions: &mut Vec<Explosion>, x: f32, y: f32) {
    explosions.push(Explosion {
        x,
        y: y - 20.0, //the exact position of the car is too low,
        timer: 0,
    });
}

//cycle through explosion animation
//could make this calculated so it easier to change.
fn advance_explosion_animation(explosions: &mut Vec<Explosion>) {
    explosions.retain_mut(|explosion| {
        explosion.timer += 1;
        if explosion.timer <= 5 {
            sprite!("explosion_frame_1", x = explosion.x, y = explosion.y);
        } else if explosion.timer <= 10 {
            sprite!("explosion_frame_2", x = explosion.x, y = explosion.y);
        } else if explosion.timer <= 15 {
            sprite!("explosion_frame_3", x = explosion.x, y = explosion.y);
        }
        explosion.timer <= 20 // Keep the explosion if the timer is 20 or less
    });
}

fn draw_portrait() {
    let [canvas_w, canvas_h] = canvas_size!();
    let text_x = 20;
    let text_y = ((canvas_h / 2) - 20) as i32; // Adjust y position for the first line
    text!("Player", x = text_x, y = text_y, font = Font::L, color = 0x000000ff); // First line
    text!("Portrait", x = text_x, y = text_y + 20, font = Font::L, color = 0x000000ff); // Second line, 20 pixels below the first
}

fn draw_stats_panel(upgrades: &Vec<Upgrade>) {
    let [canvas_w, canvas_h] = canvas_size!();
    let text_x = canvas_w as i32 - 120;
    let text_y = (canvas_h as i32 / 2) - 70; // Adjust y position for the first stat bar

    draw_stat_bar("Speed", calculate_speed(upgrades), text_x, text_y);
    draw_stat_bar("Endurance", calculate_endurance(upgrades), text_x, text_y + 30); // 40 pixels below the first stat bar
    draw_stat_bar("Brutality", calculate_brutality(upgrades), text_x, text_y + 60); // 40 pixels below the second stat bar
    draw_stat_bar("Firepower", calculate_firepower(upgrades), text_x, text_y + 90); // 40 pixels below the third stat bar
    draw_stat_bar("Hype", calculate_hype(upgrades), text_x, text_y + 120); // 40 pixels below the fourth stat bar
}

fn draw_stat_bar(stat_name: &str, stat_value: i32, x: i32, y: i32) {
    let full_rect_width = 50;
    let rect_height = 10;
    
    // Print stat name text at position x/y
    text!(stat_name, x = x, y = y, font = Font::L, color = 0x000000ff);

    // Draw the background rectangle
    rect!(w = full_rect_width, h = rect_height, x = x, y = y + 10, color = 0x808080ff); // Gray color

    // Draw the stat value rectangle
    rect!(w = stat_value, h = 10, x = x, y = y + 10, color = 0xffff00ff); // Yellow color
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
            name: "Suzee",
            upgrades: vec![
                (Upgrade::new_truck(), (0, 5)),
                (Upgrade::new_skull_box(), (2, 4)),
                (Upgrade::new_auto_rifle(), (0, 5)),
                (Upgrade::new_harpoon(), (1, 3)),
                (Upgrade::new_laser_gun(), (5, 4)),
                (Upgrade::new_auto_rifle(), (6, 5)),
            ],
        },
        CarPreset {
            name: "Meatbag",
            upgrades: vec![
                (Upgrade::new_truck(), (0, 5)),
                (Upgrade::new_skull_box(), (0, 4)),
                (Upgrade::new_skull_box(), (4, 3)),
                (Upgrade::new_harpoon(), (4, 2)),
                (Upgrade::new_laser_gun(), (6, 4)),
                (Upgrade::new_auto_rifle(), (0, 3)),
                (Upgrade::new_auto_rifle(), (2, 5)),
            ],
        },
    ]
}

turbo::go!({
    // Load the game state
    let mut state = GameState::load();
    // temp vars to get around 'borrowing' issue that I don't totally understand
    let mut transition_to_battle = false;
    let mut upgrades_for_battle = vec![];

    match &mut state.screen {
        Screen::Title(screen) => {
            clear!(0xfad6b8ff);
            let [canvas_w, canvas_h] = canvas_size!();
            let top = canvas_h.saturating_sub(screen.elapsed);
            sprite!("title", y = top);
            sprite!("title_foreground", y = canvas_h.saturating_sub((screen.elapsed as f32 / 2.7 as f32) as u32).max(canvas_h - 78));
            if top == 0 && tick() % 64 < 32 {
                text!("PRESS START", y = canvas_h - 32, x = (canvas_w / 2) - ((11 * 8) / 2), font = Font::L);
            }
            if gamepad(0).start.just_pressed() {
                if screen.elapsed < canvas_h {
                    screen.elapsed = canvas_h;
                } else {
                    state.screen = Screen::Garage(GarageScreen::new());
                }
            } else {
                screen.elapsed += 1;
            }
        },
        Screen::Garage(screen) => {
            clear!(0xffffffff);
            let mut can_place_upgrade = false;

            let [canvas_w, canvas_h] = canvas_size!();
            let grid_offset_x = ((canvas_w - 128) / 2 ) as usize; // Adjust 128 based on grid width
            let grid_offset_y = ((canvas_h - 128) / 2 ) as usize; // Adjust 128 based on grid height

            //COMMENTING THIS OUT FOR NOW - THIS IS THE CODE TO MOVE UPGRADES.
            //WE"LL WANT THIS BACK WHEN WE HAVE A MID-WAVE UPGRADE SYSTEM.
            // if let Some(upgrade) = &mut screen.upgrade {
            //     // Handle user input for shape movement
            //     if gamepad(0).up.just_pressed() {
            //         upgrade.shape.move_up()
            //     }
            //     if gamepad(0).down.just_pressed() {
            //         upgrade.shape.move_down()
            //     }
            //     if gamepad(0).left.just_pressed() {
            //         upgrade.shape.move_left()
            //     }
            //     if gamepad(0).right.just_pressed() {
            //         upgrade.shape.move_right()
            //     }

            //     let _is_empty = screen.upgrades.is_empty();
            //     let upgrade_shapes = screen.upgrades.iter().map(|u| u.shape.clone()).collect::<Vec<_>>();
            //     let is_overlapping = upgrade.shape.overlaps_any(&upgrade_shapes);
            //     let is_stickable = upgrade.shape.can_stick_any(&upgrade_shapes);
            //     can_place_upgrade = !is_overlapping && is_stickable;
            //     if can_place_upgrade {
            //         if gamepad(0).a.just_pressed() {
            //             can_place_upgrade = false;
            //             screen.upgrades.push(upgrade.clone());
            //             let upgrade_shapes = screen.upgrades.iter().map(|u| u.shape.clone()).collect::<Vec<_>>();
            //             screen.upgrade = Upgrade::random_placeable(&upgrade_shapes);
            //         }
            //     }
            // }
            
            screen.handle_input(); 

            if gamepad(0).start.just_pressed() && screen.upgrade.is_none() {
                transition_to_battle = true;
                upgrades_for_battle = screen.upgrades.clone();
            }
            // Draw the grid
            for y in 0..8 {
                for x in 0..8 {
                    rect!(
                        w = 14,
                        h = 14,
                        x = x * 16 + 1 + grid_offset_x,
                        y = y * 16 + 1 + grid_offset_y,
                        color = 0x111111ff
                    );
                }
            }

            let mut _x = 0;
            for upgrade in &screen.upgrades {
                sprite!(
                    upgrade.kind.to_str(),
                    x = upgrade.shape.offset.0 * 16 + grid_offset_x,
                    y = upgrade.shape.offset.1 * 16 + grid_offset_y,
                    opacity = 1
                );
                upgrade.shape.draw(false, false, grid_offset_x as i32, grid_offset_y as i32);
                _x += 9;
            }
            // Draw the current shape
            if let Some(upgrade) = &screen.upgrade {
                sprite!(
                    upgrade.kind.to_str(),
                    x = upgrade.shape.offset.0 * 16 + grid_offset_x,
                    y = upgrade.shape.offset.1 * 16 + grid_offset_y,
                );
                upgrade.shape.draw(true, can_place_upgrade, grid_offset_x as i32, grid_offset_y as i32);
            }

            //draw player portrait
            draw_portrait(); 
            //draw the stats panel
            draw_stats_panel(&screen.upgrades);

        }
        Screen::Battle(screen) => {
            clear!(0xFFE0B7ff); //beige sky

            draw_background(&mut screen.bg_objects);

            // Show player health
            show_health(screen.player_health);
            
            // Draw upgrades and enemies
            for (index, upgrade) in screen.upgrades.iter().enumerate() {
                let is_selected = index == screen.selected_index;
                if upgrade.kind == UpgradeKind::Truck {
                    draw_truck((upgrade.shape.offset.0 * 16) as i32, (upgrade.shape.offset.1 * 16) as i32);
                } else {
                    sprite!(
                        upgrade.kind.to_str(),
                        x = upgrade.shape.offset.0 * 16,
                        y = upgrade.shape.offset.1 * 16,
                        opacity = 1
                    );
                }
                upgrade.shape.draw(is_selected, true, 0, 0); // Draw with green rectangle if selected
            }

             // Draw enemies
            draw_enemies(&mut screen.enemies);

            // Match the whole battle_state with &mut
            match &mut screen.battle_state {
                BattleState::ChooseAttack { ref mut first_frame } => {
                    // Decrease cooldown counters
                    if *first_frame {
                        for upgrade in &mut screen.upgrades {
                            if upgrade.cooldown_counter > 0 {
                                upgrade.cooldown_counter -= 1;
                            }
                        }
                        *first_frame = false;
                    }

                    // Handle input for cycling through upgrades
                    if gamepad(0).up.just_pressed() || gamepad(0).right.just_pressed() {
                        turbo::println!("PRESSED UP OR RIGHT {:?}", screen.enemies.len().to_string());

                        let mut next_index = screen.selected_index;
                        loop {
                            next_index = (next_index + 1) % screen.upgrades.len();
                            if screen.upgrades[next_index].cooldown_counter == 0 && screen.upgrades[next_index].kind != UpgradeKind::Truck && screen.upgrades[next_index].kind != UpgradeKind::SkullBox {
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
                            if screen.upgrades[prev_index].cooldown_counter == 0 && screen.upgrades[prev_index].kind != UpgradeKind::Truck && screen.upgrades[prev_index].kind != UpgradeKind::SkullBox {
                                break;
                            }
                        }
                        screen.selected_index = prev_index;
                    }

                    // Determine the target enemies based on the selected weapon
                    // Would be good to get this out of being 'every frame' eventually
                    let selected_upgrade = &screen.upgrades[screen.selected_index];
                    let mut target_enemies = vec![];

                    match selected_upgrade.kind {
                        UpgradeKind::AutoRifle => {
                            if let Some((index, _)) = screen.enemies.iter().enumerate().min_by_key(|(_, enemy)| enemy.grid_position.0) {
                                target_enemies.push(index);
                            }
                        },
                        UpgradeKind::Harpoon => {
                            for (index, enemy) in screen.enemies.iter().enumerate() {
                                if enemy.grid_position.1 == 1 {
                                    target_enemies.push(index);
                                }
                            }
                        },
                        UpgradeKind::LaserGun => {
                            for (index, enemy) in screen.enemies.iter().enumerate() {
                                if enemy.grid_position.1 == 0 {
                                    target_enemies.push(index);
                                }
                            }
                        },
                        _ => {}
                    }

                    // Highlight target enemies
                    for &enemy_index in &target_enemies {
                        let enemy = &screen.enemies[enemy_index];
                        let (column, row) = enemy.grid_position;
                        let y_position = match row {
                            0 => GRID_ROW_HIGH,
                            1 => GRID_ROW_LOW,
                            _ => 0, // Default case, should not happen
                        };
                        rect!(
                            w = GRID_COLUMN_WIDTH,
                            h = GRID_ROW_HEIGHT,
                            x = GRID_COLUMN_OFFSET + column * GRID_COLUMN_WIDTH,
                            y = y_position,
                            color = 0xff0000aa // More solid red rectangle with higher opacity
                        );
                    }

                    // Highlight upgrades that have positive cooldown (e.g. turn red bc you can't use them)
                    for upgrade in &screen.upgrades {
                        if upgrade.cooldown_counter > 0 {
                            rect!(
                                w = upgrade.shape.size.0 as i32 * 16,
                                h = upgrade.shape.size.1 as i32 * 16,
                                x = upgrade.shape.offset.0 as i32 * 16,
                                y = upgrade.shape.offset.1 as i32 * 16,
                                color = 0xff0000aa // More solid red rectangle with higher opacity
                            );
                        }
                    }

                    // Handle attack selection
                    if gamepad(0).a.just_pressed() {
                        let selected_upgrade = &mut screen.upgrades[screen.selected_index];
                        if selected_upgrade.cooldown_counter == 0 {
                            let mut target_enemies = vec![];

                            match selected_upgrade.kind {
                                UpgradeKind::AutoRifle => {
                                    if let Some((index, _)) = screen.enemies.iter().enumerate().min_by_key(|(_, enemy)| enemy.grid_position.0) {
                                        target_enemies.push(index);
                                    }
                                },
                                UpgradeKind::Harpoon => {
                                    for (index, enemy) in screen.enemies.iter().enumerate() {
                                        if enemy.grid_position.1 == 1 {
                                            target_enemies.push(index);
                                        }
                                    }
                                },
                                UpgradeKind::LaserGun => {
                                    for (index, enemy) in screen.enemies.iter().enumerate() {
                                        if enemy.grid_position.1 == 0 {
                                            target_enemies.push(index);
                                        }
                                    }
                                },
                                _ => {}
                            }
                            //have the harpoons shoot a different sprite
                            let weapon_sprite = match selected_upgrade.kind {
                                UpgradeKind::Harpoon => "harpoon_bullet".to_string(),
                                //change this later if we get a new sprite for the laser
                                UpgradeKind::LaserGun => "bullet".to_string(),
                                _ => "bullet".to_string(),
                            };

                            let target_position = if target_enemies.is_empty() {
                                let [canvas_w, _canvas_h] = canvas_size!();
                                (canvas_w as f32, selected_upgrade.shape.offset.1 as f32 * 16.0)
                            } else {
                                calculate_target_position(screen.enemies[target_enemies[0]].grid_position)
                            };

                            selected_upgrade.cooldown_counter = selected_upgrade.cooldown_max;

                            screen.battle_state = BattleState::AnimateAttack {
                                weapon_sprite: selected_upgrade.kind.to_str().to_string(),
                                weapon_position: (
                                    selected_upgrade.shape.offset.0 as f32 * 16.0,
                                    selected_upgrade.shape.offset.1 as f32 * 16.0
                                ),
                                target_position,
                                target_enemies,
                                num_enemies_hit: 0,
                                active: true,
                                weapon_kind: selected_upgrade.kind.clone(),
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
                    ref weapon_kind 
                } => {
                    let mut new_battle_state = None; // Temporary variable to hold the new battle state

                    if *active {
                        let (wx, wy) = weapon_position;
                        let (tx, ty) = target_position;
                        let dx = *tx - *wx;
                        let dy = *ty - *wy;
                        let distance = (dx * dx + dy * dy).sqrt();
                        if distance > 1.0 {
                            let direction_x = dx / distance;
                            let direction_y = dy / distance;
                            *wx += direction_x * BULLET_SPEED;
                            *wy += direction_y * BULLET_SPEED;
                        } else {
                            *wx = *tx;
                            *wy = *ty;
                        }

                        let angle = dy.atan2(dx);

                        sprite!(
                            weapon_sprite,
                            x = *wx,
                            y = *wy,
                            rotate = angle.to_degrees(),
                            scale_x = 0.25,
                            scale_y = 0.25
                        );

                        if (*wx - *tx).abs() < BULLET_SPEED && (*wy - *ty).abs() < BULLET_SPEED {
                            if !target_enemies.is_empty() {
                                let enemy_index = target_enemies[*num_enemies_hit];
                                let target_enemy_health;
                                {
                                    let enemy = &mut screen.enemies[enemy_index];
                                    enemy.health -= 1;
                                    target_enemy_health = enemy.health;
                                }
                                create_explosion(&mut screen.explosions, *tx, *ty);

                                *num_enemies_hit += 1;

                                if target_enemies.len() > *num_enemies_hit {
                                    *target_position = calculate_target_position(screen.enemies[target_enemies[*num_enemies_hit]].grid_position);
                                } else {
                                    new_battle_state = Some(BattleState::EnemiesAttack { first_frame: true });
                                    *active = false;
                                }
                            } else {

                                new_battle_state = Some(BattleState::EnemiesAttack { first_frame: true });
                                *active = false;
                            }
                        }
                    }

                    if let Some(new_state) = new_battle_state {
                        //remove any enemies with 0 health
                        screen.enemies.retain(|e| e.health > 0);
                        //turbo::println!("enemy length {:?}", screen.enemies.len().to_string());
                        //transition to new state
                        screen.battle_state = new_state;
                    }
                }

                BattleState::EnemiesAttack { ref mut first_frame } => {
                    if screen.enemies.is_empty() {
                        //if we have more waves, then transition to new wave
                        if screen.current_wave + 1 < screen.waves.len() {
                        screen.current_wave += 1;
                        screen.enemies = screen.waves[screen.current_wave].enemies.clone();
                        //create a tween
                        
                        //set the value on every tween as end position
                        
                        //
                        screen.battle_state = BattleState::StartingNewWave
                        }
                        else {
                            screen.battle_state = BattleState::End;
                        }
                    } 
                    else {
                        if *first_frame {
                            // Set the truck position
                            let (truck_x, truck_y) = (50.0, 75.0);
                            
                            // Create bullets for each enemy
                            for enemy in &screen.enemies {
                                let (enemy_x, enemy_y) = calculate_target_position(enemy.grid_position);
                                //TODO: Add a delay to the bullets, so we can create them all at once, but slowly 'release' them based on the delay
                                create_enemy_bullet(&mut screen.bullets, enemy_x, enemy_y, truck_x, truck_y, enemy.damage);
                            }
                            
                            *first_frame = false;
                        }
                
                        // Move bullets
                        move_bullets(&mut screen.bullets, &mut screen.explosions, 50.0, 150.0, &mut screen.player_health);
                        
                        if screen.bullets.is_empty() {
                            if screen.player_health <= 0 {
                                screen.battle_state = BattleState::End;
                            } else {
                                screen.battle_state = BattleState::ChooseAttack { first_frame: true };
                            }
                        }
                    }
                }, 

                BattleState::StartingNewWave => {
                    // Draw enemies and move them into position
                    //include any wave transition stuff in here later, for now just transition to choose attack
                    screen.battle_state = BattleState::ChooseAttack { first_frame: true };

                },             
             
                BattleState::End => {
                    clear!(0x000000ff); // Black background
                    let [canvas_w, canvas_h] = canvas_size!();
                    let text_width = 8 * 8; // Approximate width for text (8 characters, each 8 pixels wide)
                    let message = if screen.player_health <= 0 {
                        "You Lose"
                    } else {
                        "You Win"
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

            // Advance explosion animations
            if !screen.explosions.is_empty() {
                advance_explosion_animation(&mut screen.explosions);
            }
        }
    }

    // Using this to move the upgrades variable into the battle screen
    if transition_to_battle {
        state.screen = Screen::Battle(BattleScreen::new(upgrades_for_battle));
    }

    state.save();
});
