use std::collections::BTreeMap;
use std::collections::HashSet;

// Define the game configuration using the turbo::cfg! macro
turbo::cfg! {r#"
    name = "Titans of the Apocalypse"
    version = "1.0.0"
    author = "Turbo"
    description = "Place shapes on a grid!"
    [settings]
    resolution = [384, 216]
"#}

const GRID_COLUMN_WIDTH: i32 = 48;
const GRID_ROW_HEIGHT: i32 = 48;
const GRID_ROW_LOW: i32 = 110; // Position of the truck
const GRID_ROW_HIGH: i32 = 62; // Position of the plane. We can make these less magic numbery later.
const GRID_COLUMN_OFFSET: i32 = 152;
const BULLET_SPEED: f32 = 4.0;

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
                    cooldown_max: i32
                }>,
                upgrades: Vec<Upgrade>,                
            }),
            Battle(struct BattleScreen {
                upgrades: Vec<Upgrade>,
                enemies: Vec<struct Enemy {
                    kind: enum EnemyKind {
                        Car,
                        Plane,
                    },
                    grid_position: (i32, i32),
                    health: i32, // Added health for enemies
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
                    EnemiesAttack,
                }                
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
        let upgrades = vec![{
            let mut truck = Upgrade::new_truck();
            truck.shape.offset = (0, 5);
            truck
        }];
        let shapes = upgrades.iter().map(|u| u.shape.clone()).collect::<Vec<_>>();
        Self {
            upgrade: Upgrade::random_placeable(&shapes),
            upgrades,
        }
    }
}

impl Upgrade {
    pub fn new(kind: UpgradeKind, shape: Shape, cooldown_max: i32) -> Self {
        Self { kind, shape, cooldown_counter: 0, cooldown_max }
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
        }, 5)
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
        }, 5)
    }
    #[rustfmt::skip]
    fn new_auto_rifle() -> Self {
        Self::new(UpgradeKind::AutoRifle, {
            let mut cells = BTreeMap::new();
            cells.insert((0, 0), Cell { edges: [false, true, false, false] });
            cells.insert((1, 0), Cell { edges: [false, false, false, false] });
            Shape::new(cells)
        }, 3)
    }
    #[rustfmt::skip]
    fn new_harpoon() -> Self {
        Self::new(UpgradeKind::Harpoon, {
            let mut cells = BTreeMap::new();
            cells.insert((0, 0), Cell { edges: [false, false, true, false] });
            cells.insert((1, 0), Cell { edges: [false, false, false, false] });
            cells.insert((2, 0), Cell { edges: [false, false, false, false] });
            Shape::new(cells)
        }, 4)
    }
    #[rustfmt::skip]
    fn new_laser_gun() -> Self {
        Self::new(UpgradeKind::LaserGun, {
            let mut cells = BTreeMap::new();
            cells.insert((0, 0), Cell { edges: [false, true, false, false] });
            cells.insert((1, 0), Cell { edges: [false, false, false, false] });
            Shape::new(cells)
        }, 4)
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

    fn draw(&self, is_active: bool, can_place: bool) {
        let (x, y) = self.offset;
        let color = if can_place {
            0x00ff0044u32
        } else {
            0xff000044u32
        };
        for (pos, cell) in &self.cells {
            let (x, y) = (x + pos.0, y + pos.1);
            if x < 8 && y < 8 {
                let (x, y) = ((x * 16) + 1, (y * 16) + 1);
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

fn calculate_target_position(grid_position: (i32, i32)) -> (f32, f32) {
    let (column, row) = grid_position;
    let x = column as f32 * GRID_COLUMN_WIDTH as f32 + GRID_COLUMN_OFFSET as f32;
    let y = row as f32 * GRID_ROW_HEIGHT as f32 + if row == 0 { GRID_ROW_HIGH as f32 } else { GRID_ROW_LOW as f32 } - (GRID_ROW_HEIGHT as f32 / 2.0);
    (x, y)
}

// Implement the game loop using the turbo::go! macro
turbo::go!({
    // Load the game state
    let mut state = GameState::load();
    //temp vars to get around 'borrowing' issue that I don't totally understand
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
            if let Some(upgrade) = &mut screen.upgrade {
                // Handle user input for shape movement
                if gamepad(0).up.just_pressed() {
                    upgrade.shape.move_up()
                }
                if gamepad(0).down.just_pressed() {
                    upgrade.shape.move_down()
                }
                if gamepad(0).left.just_pressed() {
                    upgrade.shape.move_left()
                }
                if gamepad(0).right.just_pressed() {
                    upgrade.shape.move_right()
                }

                let _is_empty = screen.upgrades.is_empty();
                let upgrade_shapes = screen.upgrades.iter().map(|u| u.shape.clone()).collect::<Vec<_>>();
                let is_overlapping = upgrade.shape.overlaps_any(&upgrade_shapes);
                let is_stickable = upgrade.shape.can_stick_any(&upgrade_shapes);
                can_place_upgrade = !is_overlapping && is_stickable;
                if can_place_upgrade {
                    if gamepad(0).a.just_pressed() {
                        can_place_upgrade = false;
                        screen.upgrades.push(upgrade.clone());
                        let upgrade_shapes = screen.upgrades.iter().map(|u| u.shape.clone()).collect::<Vec<_>>();
                        screen.upgrade = Upgrade::random_placeable(&upgrade_shapes);
                    }
                }
            }

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
                        x = x * 16 + 1,
                        y = y * 16 + 1,
                        color = 0x111111ff
                    );
                }
            }

            let mut _x = 0;
            for upgrade in &screen.upgrades {
                sprite!(
                    upgrade.kind.to_str(),
                    x = upgrade.shape.offset.0 * 16,
                    y = upgrade.shape.offset.1 * 16,
                    opacity = 1
                );
                upgrade.shape.draw(false, false);
                _x += 9;
            }
            // Draw the current shape
            if let Some(upgrade) = &screen.upgrade {
                sprite!(
                    upgrade.kind.to_str(),
                    x = upgrade.shape.offset.0 * 16,
                    y = upgrade.shape.offset.1 * 16,
                );
                upgrade.shape.draw(true, can_place_upgrade);
            }
        }
        Screen::Battle(screen) => {
            clear!(0xffa500ff); // Orange background

            // Draw road sprite
            sprite!(
                "road",
                x = 0,
                y = 110
            );

            // Draw upgrades and enemies
            for (index, upgrade) in screen.upgrades.iter().enumerate() {
                let is_selected = index == screen.selected_index;
                sprite!(
                    upgrade.kind.to_str(),
                    x = upgrade.shape.offset.0 * 16,
                    y = upgrade.shape.offset.1 * 16,
                    opacity = 1
                );
                upgrade.shape.draw(is_selected, true); // Draw with green rectangle if selected
            }

            // Draw enemies
            for enemy in &screen.enemies {
                let (column, row) = enemy.grid_position;
                let sprite_name = match enemy.kind {
                    EnemyKind::Car => "car_enemy",
                    EnemyKind::Plane => "plane_enemy",
                };
                let y_position = match row {
                    0 => GRID_ROW_HIGH,
                    1 => GRID_ROW_LOW,
                    _ => 0, // Default case, should not happen
                };
                sprite!(
                    sprite_name,
                    x = GRID_COLUMN_OFFSET + column * GRID_COLUMN_WIDTH,
                    y = y_position
                );
            }

            // Match the whole battle_state with &mut
            match &mut screen.battle_state {
                BattleState::ChooseAttack {ref mut first_frame} => {
                
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
                
                    // Highlight upgrades with cooldown
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
                    if gamepad(0).start.just_pressed() {
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
                
                            selected_upgrade.cooldown_counter = selected_upgrade.cooldown_max;
                
                            screen.battle_state = BattleState::AnimateAttack {
                                weapon_sprite: selected_upgrade.kind.to_str().to_string(),
                                weapon_position: (
                                    selected_upgrade.shape.offset.0 as f32 * 16.0,
                                    selected_upgrade.shape.offset.1 as f32 * 16.0
                                ),
                                target_position: calculate_target_position(screen.enemies[target_enemies[0]].grid_position),
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
                            rotate = angle.to_degrees() + 90.0,
                            scale_x = 0.175,
                            scale_y = 0.175
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
                               // if target_enemy_health <= 0 {
                                //    screen.enemies.retain(|e| e.health > 0);
                               // }
                
                                *num_enemies_hit += 1;
                
                                if target_enemies.len() > *num_enemies_hit {
                                    *target_position = calculate_target_position(screen.enemies[target_enemies[*num_enemies_hit]].grid_position);
                                } else {
                                    //remove any enemies with 0 health or less
                                    screen.enemies.retain(|e| e.health > 0);
                                    new_battle_state = Some(BattleState::EnemiesAttack);
                                    *active = false;
                                }
                            } else {
                                new_battle_state = Some(BattleState::EnemiesAttack);
                                *active = false;
                            }
                        }
                    }
                
                    if let Some(new_state) = new_battle_state {
                        screen.battle_state = new_state;
                    }
                }
                
                         

                BattleState::EnemiesAttack => {
                    // Placeholder for enemies attack
                    screen.battle_state = BattleState::ChooseAttack {first_frame: true};
                },
            }
        }
    }

    // Using this to move the upgrades variable into the battle screen
    if transition_to_battle {
        state.screen = Screen::Battle(BattleScreen {
            upgrades: upgrades_for_battle,
            // Replace this with enemy wave data eventually
            enemies: vec![
                Enemy { kind: EnemyKind::Car, grid_position: (0, 1), health: 3 },
                Enemy { kind: EnemyKind::Plane, grid_position: (1, 0), health: 2 },
                Enemy { kind: EnemyKind::Car, grid_position: (2, 1), health: 3 },
                Enemy { kind: EnemyKind::Car, grid_position: (3, 1), health: 3 },
            ], // Initialize with some enemies
            selected_index: 1, // Initialize selected_index to 1
            battle_state: BattleState::ChooseAttack {first_frame: true}, // Initialize battle state
        });
    }

    state.save();
});
