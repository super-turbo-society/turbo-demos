// Define the game configuration
turbo::cfg! {r#"
    name = "Level Editor"
    version = "1.0.0"
    author = "Turbo"
    description = "A Turbo Demo to make a level editor for a platformer"
    [settings]
    resolution = [256, 144]
"#}

turbo::init! {}

turbo::go!({
    let mut state = GameState::load();
    state.save();
});
