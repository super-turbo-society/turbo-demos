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

// const CELL_EMPTY: u8 = 0b00000000; // 0 - No edges are sticky
// const CELL_TOP: u8 = 0b00000001; // 1 - Top edge is sticky
// const CELL_RIGHT: u8 = 0b00000010; // 2 - Right edge is sticky
// const CELL_BOTTOM: u8 = 0b00000100; // 4 - Bottom edge is sticky
// const CELL_LEFT: u8 = 0b00001000; // 8 - Left edge is sticky
// const CELL_TOP_RIGHT: u8 = 0b00000011; // 3 - Top and Right edges are sticky
// const CELL_TOP_BOTTOM: u8 = 0b00000101; // 5 - Top and Bottom edges are sticky
// const CELL_TOP_LEFT: u8 = 0b00001001; // 9 - Top and Left edges are sticky
// const CELL_RIGHT_BOTTOM: u8 = 0b00000110; // 6 - Right and Bottom edges are sticky
// const CELL_RIGHT_LEFT: u8 = 0b00001010; // 10 - Right and Left edges are sticky
// const CELL_BOTTOM_LEFT: u8 = 0b00001100; // 12 - Bottom and Left edges are sticky
// const CELL_TOP_RIGHT_BOTTOM: u8 = 0b00000111; // 7 - Top, Right, and Bottom edges are sticky
// const CELL_TOP_RIGHT_LEFT: u8 = 0b00001011; // 11 - Top, Right, and Left edges are sticky
// const CELL_TOP_BOTTOM_LEFT: u8 = 0b00001101; // 13 - Top, Bottom, and Left edges are sticky
// const CELL_RIGHT_BOTTOM_LEFT: u8 = 0b00001110; // 14 - Right, Bottom, and Left edges are sticky
// const CELL_ALL: u8 = 0b00001111; // 15 - All edges are sticky

const GRID_COLUMN_WIDTH: i32 = 48;
const GRID_ROW_HEIGHT: i32 = 48;
const GRID_ROW_LOW: i32 = 110; // Position of the truck
const GRID_ROW_HIGH: i32 = 62; // Position of the plane. We can make these less magic numbery later.
const GRID_COLUMN_OFFSET: i32 = 152;


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
                }>,
                selected_index: usize,
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
    pub fn new(kind: UpgradeKind, shape: Shape) -> Self {
        Self { kind, shape }
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
                        turbo::println!("NO OVERLAP AND CAN STICK! {:?}", u.shape.offset);
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
        })
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
        })
    }
    #[rustfmt::skip]
    fn new_auto_rifle() -> Self {
        Self::new(UpgradeKind::AutoRifle, {
            let mut cells = BTreeMap::new();
            cells.insert((0, 0), Cell { edges: [false, true, false, false] });
            cells.insert((1, 0), Cell { edges: [false, false, false, false] });
            Shape::new(cells)
        })
    }
    #[rustfmt::skip]
    fn new_harpoon() -> Self {
        Self::new(UpgradeKind::Harpoon, {
            let mut cells = BTreeMap::new();
            cells.insert((0, 0), Cell { edges: [false, false, true, false] });
            cells.insert((1, 0), Cell { edges: [false, false, false, false] });
            cells.insert((2, 0), Cell { edges: [false, false, false, false] });
            Shape::new(cells)
        })
    }
    #[rustfmt::skip]
    fn new_laser_gun() -> Self {
        Self::new(UpgradeKind::LaserGun, {
            let mut cells = BTreeMap::new();
            cells.insert((0, 0), Cell { edges: [false, true, false, false] });
            cells.insert((1, 0), Cell { edges: [false, false, false, false] });
            Shape::new(cells)
        })
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
        // let (w, h) = self.size;
        // let w1 = w.saturating_sub((x + w).saturating_sub(8));
        // let h1 = h.saturating_sub((y + h).saturating_sub(8));
        // if is_active {
        //     rect!(
        //         w = w1 * 16,
        //         h = h1 * 16,
        //         x = x * 16,
        //         y = y * 16,
        //         color = if can_place {
        //             0x00ff0044u32
        //         } else {
        //             0xff000044u32
        //         }
        //     );
        // }
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
            // let mut next_upgrade = None;
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
    
                let is_empty = screen.upgrades.is_empty();
                let upgrade_shapes = screen.upgrades.iter().map(|u| u.shape.clone()).collect::<Vec<_>>();
                let is_overlapping = upgrade.shape.overlaps_any(&upgrade_shapes);
                let is_stickable = upgrade.shape.can_stick_any(&upgrade_shapes);
                // text!("empty = {}\noverlap = {}\nstick = {}", is_empty, is_overlapping, is_stickable; y = 128, font = Font::L);
                can_place_upgrade = !is_overlapping && is_stickable;
                if can_place_upgrade {
                    if gamepad(0).a.just_pressed() {
                        can_place_upgrade = false;
                        screen.upgrades.push(upgrade.clone());
                        let upgrade_shapes = screen.upgrades.iter().map(|u| u.shape.clone()).collect::<Vec<_>>();
                        screen.upgrade = Upgrade::random_placeable(&upgrade_shapes);
                       // turbo::println!("NEXT = {:?}", screen.upgrade);
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
                    // let color = if screen.grid[y][x] == 0 {
                    //     0x222222ff
                    // } else {
                    //     0xff0000ff
                    // };
                    rect!(
                        w = 14,
                        h = 14,
                        x = x * 16 + 1,
                        y = y * 16 + 1,
                        color = 0x111111ff
                    );
                }
            }
    
            let mut x = 0;
            for upgrade in &screen.upgrades {
                sprite!(
                    upgrade.kind.to_str(),
                    x = upgrade.shape.offset.0 * 16,
                    y = upgrade.shape.offset.1 * 16,
                    opacity = 1
                );
                upgrade.shape.draw(false, false);
                // set_cam!(x = x * 6, y = 216 - (6 * 8));
                // upgrade.shape.draw_mini();
                // set_cam!(x = 0, y = 0);
                x += 9;
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
        
            // Handle input for cycling through upgrades
            if gamepad(0).up.just_pressed() || gamepad(0).right.just_pressed() {
                let mut next_index = screen.selected_index;
                loop {
                    next_index = (next_index + 1) % screen.upgrades.len();
                    if screen.upgrades[next_index].kind.to_str() != "truck" {
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
                    if screen.upgrades[prev_index].kind.to_str() != "truck" {
                        break;
                    }
                }
                screen.selected_index = prev_index;
            }
        
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
               // turbo::println!("Enemy: {:?}, Column: {}, Row: {}, X: {}, Y: {}",
               // enemy.kind, column, row, GRID_COLUMN_OFFSET + column * GRID_COLUMN_WIDTH, y_position);
               // turbo::println!("Drawing sprite: {}, at X: {}, Y: {}",
                // sprite_name, GRID_COLUMN_OFFSET + column * GRID_COLUMN_WIDTH, y_position);
                sprite!(
                    sprite_name,
                    x = GRID_COLUMN_OFFSET + column * GRID_COLUMN_WIDTH,
                    y = y_position
                );
            }
        }

    }
    //Using this to move the upgrades variable into the battle screen
    if transition_to_battle {
        state.screen = Screen::Battle(BattleScreen {
            upgrades: upgrades_for_battle,
            //replace this with enemy wave data eventually
            enemies: vec![
                Enemy { kind: EnemyKind::Car, grid_position: (0, 1) },
                Enemy { kind: EnemyKind::Plane, grid_position: (1, 0) },
                Enemy { kind: EnemyKind::Car, grid_position: (2, 1) },
                Enemy { kind: EnemyKind::Car, grid_position: (3, 1) },
            ], // Initialize with some enemies
            selected_index: 1, // Initialize selected_index to 1
        });
    }

    state.save();
});
