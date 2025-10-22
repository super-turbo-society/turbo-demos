# Space Shooter

![Turbo game window with a spaceship, enemies, and projectiles](preview.gif)

## Description

Navigate a spaceship in a retro 2D space shooter! Move, shoot, and dodge enemies to try to get a high score.

## Getting Started

From the project directory, run the following command:

```sh
turbo run -w .
```

## Walkthrough

### Game State

Think of `GameState` as the command center of our game. It keeps track of the key variables and entities in our game:

-   `screen`: Manages our transition between the start screen and the game screen.
-   `start_tick`: Lets us track when the game started, to manage timers in the game.
-   `player`: Here's you! This struct has all the details about your spaceship, like health, position, and firepower.
-   `enemies`: These guys are out to get you. The struct tracks all the enemy ships you're up against.
-   `projectiles`: These are all the bullets and beams flying around. The struct keeps track of their positions and who shot them.

We divided our code into multiple scripts to keep it organized! `lib.rs` handles our main gameplay loop, checks for input, tracks lists of entities like enemies and projectiles, and draws graphics.

The update function in `lib.rs` runs 60 frames a second, and drives the action in the game. If you look at the code, it's mostly just calling `update()` for each of the entity types, which are `player.rs`, `enemy.rs`, and `projectile.rs`.

Let's take a look at the code inside of those scripts to see how they work, then we'll come back to review the code in `update()`.

### Player

The `Player` struct is all about you and your ship. It's got:

-   `hitbox`: We use this to track collisions between the player and enemy bullets.
-   `x, y`: Your position in the vastness of space.
-   `dx, dy`: Track your velocity in each direction.
-   `hp`: Keep this above zero, or it's game over!
-   `hit_timer`: Gives you some invincibility after you get hit.
-   `shoot_timer`: Tracks how long between shots, so the player can't create an infinite stream of bullets.
- `shooting`: This bool tracks the frame in which you start shooting, to trigger the animation.

We set up all those variables in `player.new()` and then handle our movement and shooting in `player.update()`.

The `player.update()` function first checks for keyboard input and adjusts the velocity based on that. Then it checks if the player is shooting, and creates a projectile if the player is able to shoot.

The `player.draw()` function animates the player, and determines which sprites to use based on what the player is doing in the game.

### Enemy

Enemies make the game fun and challenging. The `Enemy` struct has a lot of similarities to the `Player`.

-  `enemy_type`: We can have different types of enemies with different movements and sprites.
-  `hitbox`: Just like with our `player` we'll use this to track when bullets hit the enemy
-   `x, y`: Where they are in the game world.
-   `hit_timer`: Enemies don't get an invincibility window like the player, but we use this timer to track their animations when they get hit.
-   `destroyed`: Used to track when enemies should be removed from the game. This can happen if they run out of `hp` or if they fly off screen.
-   `hp`: Tracks health.
-   `angle`: Can be used to vary how enemies move and shoot.

The `enemy.update()` function first moves the enemy ship straight down. Next, it determines if the enemy should create a projectile. It's set to have a 1 out of 250 chance to shoot each frame - feel free to change this number around and see if the game feels more fun!

Finally, it checks if the enemy has flown out of bounds, and marks it for destruction if it has.


### Projectile

Last but not least, `Projectile` is all about the ammo flying around. It tracks:

-   `hitbox`: Same as `player` and `enemy` this is how we check for collisions.
-   `x, y`: The projectile's location.
-   `velocity`: How fast it is moving.
-   `angle`: Use this to change the direction of the projectile.
-   `projectile_owner`: Who shot it? You or the bad guys?

In `projectile.update()` we move the projectile based on its angle, then check if it's off screen and destroy it if it is.

After that, we check for collisions. The collision check takes advantage of the `intersects()` function from the Turbo `Bounds` struct that can check if two `Bounds` are touching. For enemy bullets, we check for intersection with `player.hitbox`, and for player bullets we loop through all the enemies and check for intersection with their hitboxes. When we find an intersection, we process the hit as needed.

### Game Loop

Now that we know how our entities work, let's circle back to the `update()` function in `lib.rs`. 

The first `if/else` statement is tracking which `Screen` we are in. We start the game in the `Menu` screen, which is our title screen. The code here checks to see if the player has pressed spacebar or z, and if they have, then it transitions the screen to `Game`. That's where the action happens!

Since the `Screen` enum only has two options (`Menu` and `Game`), we can use the `else` to determine that we are in the `Game` screen. In the `Game` Screen we update all our enemies, projectiles and our player. We use the rust iterator `retain_mut` to loop through the list of projectiles and enemies, and remove any that are `destroyed`. `retain_mut` is a handy way to edit a list of items based on a certain criteria.

The last thing we do in our update function is check if the player's health has dropped to 0, and if it has we reach Game Over! During Game Over, we check if the player pressed spacebar or z, and if they have we reset the game using `*self = GameState::new()`.

Next, we have `fn draw(&self)` which draws all the instances of our player, enemies and projectiles.

You'll see we also have a function here called `fn spawn_enemies(&mut self)`. This is how we make new enemies in the game. We set the `spawn_rate` to `100`, which means a new enemy appears every 100 frames. You could change this number to make the game easier or harder. We also set a maximum of `24` enemies on screen at any time.

Lastly, we have a function `draw_screen(&self)`. This function draws our title screen and our Game Over screen, but doesn't do anything when the player is playing the game.

### Wrapping Up
-----------

So there you have it! A quick tour of the main structs that make our Space Shooter game tick. Each piece plays a vital role in creating an exciting and dynamic experience. Now, it's up to you to make the game your own! Try changing the variables to make it easier or harder. Once you feel good about it, try adding a new enemy type that moves or behaves differently! Where you go from there is up to you, but we'd love to see it. 

If you have any feedback, let us know on [discord](https://discord.gg/makegamesfast). 