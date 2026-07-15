# Roguelike

A 2D [rogue like](https://en.wikipedia.org/wiki/Roguelike) game about dungeon crawling in procedurally generated levels with turn-based gameplay.

Progress through the tomb whilst facing various monsters with a multitude of items at your disposal, difficulty increases steadily as you work your way through the dungeon.

## Installation and Run Options
* Download the [roguelike](https://github.com/cmodii/roguelike/releases/tag/v1.0.0) binary from the releases tab directly

* Or build and run with the [rustup](https://rustup.rs/) tool chain
```sh
# Clone the repository
$ git clone https://github.com/cmodii/roguelike

# Access the directory
$ cd roguelike

# Build the project and run it
$ cargo run
```

## Configuration
You can configure spawn-rates of items/monsters and their stats in their respective files: [inventory.rs](src/inventory.rs) & [monsters.rs](src/monsters.rs).

* For stats, modify either the ``Equipment`` or ``Fighter`` structs inside the ``generate_X()`` functions, for example:
```rust
    .fighter(Fighter { // change the values inside the large match blocks
        base_max_hp: 10,
        hp: 10, // health
        base_defense: 2, // defense
        base_power: 4, // power
        xp: 100, // xp earned from defeating the monster
        on_death: DeathCallback::Monster
})
```


* Spawn-rates rely on a [weight-based distribution](https://rust-rspec.github.io/rspec/rand/distributions/struct.WeightedChoice.html), the weights of [items](https://github.com/cmodii/roguelike/blob/f3bdcf1f166a4db6022e6334acb6d160674ee74d/src/inventory.rs#L24) and [monsters](https://github.com/cmodii/roguelike/blob/f3bdcf1f166a4db6022e6334acb6d160674ee74d/src/monsters.rs#L12) can be modified inside the ``Transition {level: i32, weight: i32}`` struct they're defined with. `level` value indicates at what level does the `weight` rate take effect. e.g:
```rust
    ("orc", &[
        Transition {level: 3, value: 1.5}, // 15% of spawning during levels 1-4
        Transition {level: 5, value: 3.0}, // 30% of spawning during levels 5-6
        Transition {level: 7, value: 6.0} // 60% of spawning during levels >=7
    ]), 
```


* Create your very own monster/item by using the ``ObjectBuilder`` and following the same format in the ``generate_x()`` functions, the following set of components must accompany each entity:

    | Entity  | Components |
    | ------------- |:-------------:|
    | Monster      | `Fighter`, `Ai`     |
    | One-time item      | `Item`     |
    | Equipment      | `Item`, `Equipment`    |

> Note that you'll have to create your own custom logic for one-item items if it's not within bounds of [pre-existing ones](https://github.com/cmodii/roguelike/blob/f3bdcf1f166a4db6022e6334acb6d160674ee74d/src/inventory.rs#L209).

## Credits
This project was made possible with the following open source packages:
* [Tomas Sedovic's port](https://tomassedovic.github.io/roguelike-tutorial/)
* [tcod-rs](https://github.com/tomassedovic/tcod-rs)
* [Serde](https://crates.io/crates/serde)
