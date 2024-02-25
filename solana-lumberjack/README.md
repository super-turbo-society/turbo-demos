# Solana Lumberjack

A fully on-chain game where you chop wood. LFG 😤

## Development

**Start the local validator**

```sh
solana-test-validator
```

**Build the Solana program**

```sh
cd solana/lumberjack
cargo build-sbf
```

**Deploy Solana program**

```sh
cd solana/lumberjack
solana program deploy target/deploy/solana_lumberjack.so
```

**Run your game**

```sh
TURBO_SOL_SIGNER=<LOCAL_PRIVATE_KEY> turbo run -w .
```

## Walkthrough

### Understanding the Game Code

The game code is structured around several key components:

- **Constants and Instructions**: `lumberjack::constants` and `lumberjack::instructions` define game rules and blockchain interactions.
- **State Management**: `lumberjack::state::player_data::PlayerData` manages player data, including progress and achievements.
- **Serialization**: Anchor uses `borsh::BorshDeserialize` for data serialization and deserialization between game states and the blockchain.
- **Solana Integration**: `turbo::solana` provides the tools needed to interact with the Solana blockchain, including data fetching and transaction signing.

### Game Initialization

- **Configuration**: The game's resolution and Solana RPC URLs are set in the `turbo::cfg!` macro. This is where you define the game's display settings and blockchain connection points.

- **Game Loop**: The `turbo::go!` macro kicks off the game loop. It clears the screen, sets up the level, and checks for player and game data. Depending on the data availability, it either starts the game, initializes player data, or shows an error screen.

### Screen Functions

- `start_game_screen`: This function displays the game's start screen. It shows the signer's public key and waits for the player to press start.
- `chopping_screen`: When player and game data are available, this screen allows players to chop wood by pressing start, simulating game action with a simple animation.
- `error_screen`: Displays errors related to data fetching, helping in debugging.


### Data Management

- Getting Player and Game Data: `get_player_data` and `get_game_data` functions fetch data from the blockchain, using public keys derived from the signer and level seed.
- Initializing and Updating Game State: `init_player_and_game` and `chop_tree` functions send transactions to the Solana blockchain to initialize or update the game state.