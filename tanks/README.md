# Tanks

![screenshot](./screenshot.png)

 ## Description

The Tanks Game is a two-player tank battle where players control colorful tanks and engage in strategic combat. The goal is to eliminate the opponent by firing missiles while avoiding collisions with obstacles on the battlefield.

## Getting Started

To run the Tanks Game, follow these steps:

Make sure you have the Turbo library installed.

Clone the repository and navigate to the project directory.

Run the following command:

```sh
turbo-cli run -w .
```

Use keyboard controls to maneuver your tank, aim, and fire missiles.

## Walkthrough

Game Configuration

```rs
turbo::cfg! {r#"
    name = "Pancake Cat"
    version = "1.0.0"
    author = "Turbo"
    description = "Catch falling pancakes!"
    [settings]
    resolution = [256, 144]
"#} 
```
The game configuration sets up basic information about the game, such as its name, version, and author.

Game State Initialization

```rs
turbo::init! {
    struct GameState {
        winner: Option<enum Winner {
            P1,
            P2,
            Draw,
        }>,
        tanks: Vec<struct Tank { /* ... */ }>,
        blocks: Vec<struct Block { /* ... */ }>,
    } = { /* Initial state setup */ }
}
```
The game state is initialized, including details about the winner, tanks, and blocks on the screen.


## Game Loop

The game loop is the core of your game, handling user input, updating the game state, and rendering. A typical Turbo game loop follows the following pattern:

```rs
turbo::go! {
   turbo::go! {
    let mut state = GameState::load();
    let mut tanks = state.tanks.iter_mut();
    let mut tank1 = tanks.next().unwrap();
    let mut tank2 = tanks.next().unwrap();

    // Draw elements
    rect!(w = 256, h = 144, color = 0x222222ff);
    draw_blocks(&state.blocks);
    draw_tank(&tank1);
    draw_tank(&tank2);

    // Update tank positions, rotations, and firing
    update_tank(&gamepad(0), &mut tank1, &state.blocks);
    update_tank(&gamepad(1), &mut tank2, &state.blocks);

    // Check for missile collisions and determine the winner
    let tank1_got_hit = did_hit_missile(tank1, &tank2.missiles);
    let tank2_got_hit = did_hit_missile(tank2, &tank1.missiles);
    state.winner = determine_winner(tank1_got_hit, tank2_got_hit);

    // Display winner message or continue the game
    display_winner_message(&state);

    // Save the game state
    state.save();
}
}
```

The main game loop is responsible for loading the game state, updating and drawing game elements, checking for collisions, determining the winner, displaying the winner message, and saving the updated state.

Functions
- `did_hit_missile`: Checks if a tank was hit by a missile.
- `update_tank`: Updates tank movement, rotation, and firing.
- `draw_tank`:` Draws tanks and their missiles.
- `draw_blocks`: Draws blocks on the screen.
- `determine_winner`: Determines the winner based on missile collisions.
- `display_winner_message`: Displays a message indicating the winner.




