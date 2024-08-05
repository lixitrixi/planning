# Planning

A Rust library allowing the planning of minimal sequences of actions to achieve a given goal state.

This crate is based on the work on Goal-Oriented Action-Planner (GOAP) by Jeff Orkin. It takes inspiration from tynril's [rgoap](https://github.com/tynril/rgoap) library, but offers dynamic goal priority and action costs as well as arbitrary state types. 

## Usage

Add the crate as a dependency with `cargo add planning` or by adding the following lines to your `Cargo.toml`:
```toml
[dependencies]
planning = "0.1"
```

Use the library like so:

```rust
use planning::*;
use std::hash::Hash;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct State {
    num_flowers: u16,
    hungry: bool,
    picnic_set: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum MyAction {
    PickFlower,
    SetPicnic,
    Eat,
}

impl Action<State> for MyAction {
    fn is_applicable(&self, state: &State) -> bool {
        match self {
            MyAction::PickFlower => state.num_flowers < 5,
            MyAction::SetPicnic => !state.picnic_set,
            MyAction::Eat => state.hungry && state.picnic_set,
        }
    }

    fn apply_mut(&self, state: &mut State) {
        match self {
            MyAction::PickFlower => state.num_flowers += 1,
            MyAction::SetPicnic => state.picnic_set = true,
            MyAction::Eat => state.hungry = false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum MyGoal {
    BouquetMade,
    Eaten,
}

impl Goal<State> for MyGoal {
    fn is_satisfied(&self, state: &State) -> bool {
        match self {
            MyGoal::BouquetMade => state.num_flowers >= 5,
            MyGoal::Eaten => !state.hungry,
        }
    }

    fn priority(&self, state: &State) -> i32 {
        match self {
            MyGoal::BouquetMade => 1,
            MyGoal::Eaten => if state.hungry { 2 } else { 0 },
        }
    }
}

let mut agent = Agent::new(
    State { num_flowers: 0, hungry: true, picnic_set: false },
    vec![MyAction::PickFlower, MyAction::SetPicnic, MyAction::Eat],
    vec![MyGoal::BouquetMade, MyGoal::Eaten],
);

let (goal, plan, _) = agent.plan_dynamic().unwrap();
assert_eq!(goal, &MyGoal::Eaten);
assert_eq!(plan, vec![MyAction::SetPicnic, MyAction::Eat]);

agent.state.hungry = false;
let (goal, plan, _) = agent.plan_dynamic().unwrap();
assert_eq!(goal, &MyGoal::BouquetMade);
assert_eq!(plan, vec![MyAction::PickFlower; 5]);
```

## Features

If the `bevy` feature is enabled, the `Agent` type will implement Bevy's `Component` type. This allows it to be used as part of a game engine for unit AI behavior:

```toml
[dependencies]
planning = { version = "0.1", features = ["bevy"] }
```
