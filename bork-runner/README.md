# Bork Runner

![screenshot](./screenshot.png)

## Description

Control an energetic dog on a borking adventure. Dodge enemies, unleash powerful borks, and collect powerups to increase your score. Embrace the challenge and see how far you can run in this delightful game.

## Getting Started

```sh
turbo-cli run -w .
```

## Key Code Snippets

### Game Configuration

Define game metadata and settings such as name, version, author, description, and resolution.

```rust
turbo::cfg! {r#"
    name = "Bork Runner"
    version = "1.0.0"
    author = "Turbo"
    description = "Infinite runner as a dog with borks and a bat."
    [settings]
    resolution = [256, 144]
"#} 
``` 

## Game State Initialization

Initialize the game state with relevant parameters.
```rust
turbo::init! {
    struct GameState {
        // ... (abbreviated for brevity)
    } = {
        Self::new()
    }
}
```

## Bork Generation

In the game, when the player presses a button, the dog shoots out something called a "Bork." This Bork travels across the screen, and if it hits an enemy,

```rust 
if state.last_game_over == 0 && state.is_ready {
    // Bork!!!
    if gp.start.just_released() {
        if state.tick - state.last_bork >= state.bork_rate && state.energy > 0 {
            state.borks.push(Bork::new(state.dog_x, state.dog_y));
            state.last_bork = state.tick;
            state.energy -= 1;
        }
    }

    // ... (other controls and game logic)
}
```



