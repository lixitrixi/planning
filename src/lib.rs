mod util;
pub use util::*;

use std::hash::{Hash, Hasher};

use pathfinding::prelude::astar;

pub trait Action<S>
where
    S: Clone + Hash + Eq,
{
    /// Returns true if the action can be applied to the given state.
    fn is_applicable(&self, state: &S) -> bool;

    /// Applies the action to the given state in-place.
    /// This is used by the default implementation of `apply`, which may be implemented directly instead.
    fn apply_mut(&self, state: &mut S);

    /// Applies the action to the given state and returns the new state.
    fn apply(&self, state: &S) -> S {
        let mut new_state = state.clone();
        self.apply_mut(&mut new_state);
        new_state
    }

    /// Returns the cost of applying the action to the given state.
    fn cost(&self, _state: &S) -> i32 {
        1
    }
}

pub trait Goal<S>
where
    S: Clone + Hash + Eq,
{
    /// Returns true if the goal is satisfied in the given state.
    fn is_satisfied(&self, state: &S) -> bool;

    /// Returns a heuristic estimate of the cost to reach the goal from the given state.
    /// The default implementation returns a constant, and implementing this may make the search more efficient.
    ///
    /// When implementing, the heuristic should not *overestimate* the cost.
    fn heuristic(&self, state: &S) -> i32 {
        0
    }
}

#[derive(PartialEq, Eq, Clone)]
struct PlanNode<'a, S, A>
where
    S: Clone + Hash + Eq,
    A: Action<S> + Eq + Clone + Hash,
{
    pub state: S,
    pub action: Option<&'a A>,
}

impl<'a, S, A> PlanNode<'a, S, A>
where
    S: Clone + Hash + Eq,
    A: Action<S> + Eq + Clone + Hash,
{
    /// Returns the next node after applying the given action.
    fn child(&self, action: &'a A) -> PlanNode<'a, S, A> {
        PlanNode {
            state: action.apply(&self.state),
            action: Some(action),
        }
    }

    /// Returns all possible next nodes using the given actions.
    pub fn children(&self, actions: &'a Vec<A>) -> Vec<(PlanNode<'a, S, A>, i32)> {
        actions
            .iter()
            .filter(|action| action.is_applicable(&self.state))
            .map(|action| (self.child(action), action.cost(&self.state)))
            .collect()
    }
}

impl<'a, S, A> Hash for PlanNode<'a, S, A>
where
    S: Clone + Hash + Eq,
    A: Action<S> + Eq + Clone + Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.action.map(|action| action.hash(state));
        self.state.hash(state);
    }
}

/// Return a sequence of actions to reach the goal while minimising cost. Returns None if no plan is found.
pub fn plan<S, A, G>(initial_state: &S, actions: &Vec<A>, goal: &G) -> Option<(Vec<A>, i32)>
where
    S: Clone + Hash + Eq,
    A: Action<S> + Eq + Clone + Hash,
    G: Goal<S>,
{
    let initial = PlanNode {
        state: initial_state.clone(),
        action: None,
    };
    astar(
        &initial,
        |node| node.children(&actions),
        |node| goal.heuristic(&node.state),
        |node| goal.is_satisfied(&node.state),
    )
    .map(|(path, cost)| {
        (
            path.iter()
                .filter_map(|node| node.action)
                .cloned()
                .collect(),
            cost,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_single_step() {
        #[derive(PartialEq, Eq, Hash, Clone)]
        struct State {
            is_correct: bool,
        }

        #[derive(PartialEq, Eq, Hash, Clone, Debug)]
        struct MakeCorrect;

        impl Action<State> for MakeCorrect {
            fn is_applicable(&self, state: &State) -> bool {
                !state.is_correct
            }

            fn apply_mut(&self, state: &mut State) {
                state.is_correct = true;
            }
        }

        #[derive(PartialEq, Eq, Hash, Clone)]
        struct IsCorrect;

        impl Goal<State> for IsCorrect {
            fn is_satisfied(&self, state: &State) -> bool {
                state.is_correct
            }
        }

        let initial_state = State { is_correct: false };
        let actions = vec![MakeCorrect];
        let goal = IsCorrect;

        let (path, cost) = plan(&initial_state, &actions, &goal).unwrap();
        assert_eq!(path.len(), 1);
        assert_eq!(cost, 1);
        assert_eq!(path[0], MakeCorrect);
    }

    #[test]
    fn plan_complex() {
        type Pos = (i32, i32);

        fn manhattan_distance(a: Pos, b: Pos) -> i32 {
            (a.0 - b.0).abs() + (a.1 - b.1).abs()
        }

        #[derive(PartialEq, Eq, Hash, Clone)]
        struct MyState {
            has_wood: bool,
            has_axe: bool,
            house_built: bool,
            position: Pos,
            nearest_tree: Pos,
            nearest_axe: Pos,
        }

        #[derive(PartialEq, Eq, Hash, Clone, Debug)]
        enum MyAction {
            ChopTree,
            GrabAxe,
            BuildHouse,
            GoToTree,
            GoToAxe,
            GoHome,
        }

        impl Action<MyState> for MyAction {
            fn is_applicable(&self, state: &MyState) -> bool {
                match self {
                    MyAction::ChopTree => state.has_axe && state.position == state.nearest_tree,
                    MyAction::GrabAxe => !state.has_axe && state.position == state.nearest_axe,
                    MyAction::BuildHouse => state.has_wood && state.position == (0, 0),
                    MyAction::GoToTree => state.position != state.nearest_tree,
                    MyAction::GoToAxe => state.position != state.nearest_axe,
                    MyAction::GoHome => state.position != (0, 0),
                }
            }

            fn apply_mut(&self, state: &mut MyState) {
                match self {
                    MyAction::ChopTree => state.has_wood = true,
                    MyAction::GrabAxe => state.has_axe = true,
                    MyAction::BuildHouse => state.house_built = true,
                    MyAction::GoToTree => state.position = state.nearest_tree,
                    MyAction::GoToAxe => state.position = state.nearest_axe,
                    MyAction::GoHome => state.position = (0, 0),
                }
            }

            fn cost(&self, state: &MyState) -> i32 {
                match self {
                    MyAction::GoToTree => manhattan_distance(state.position, state.nearest_tree),
                    MyAction::GoToAxe => manhattan_distance(state.position, state.nearest_axe),
                    MyAction::GoHome => manhattan_distance(state.position, (0, 0)),
                    _ => 1,
                }
            }
        }

        #[derive(PartialEq, Eq, Hash, Clone)]
        struct MyGoal;

        impl Goal<MyState> for MyGoal {
            fn is_satisfied(&self, state: &MyState) -> bool {
                state.house_built
            }

            fn heuristic(&self, state: &MyState) -> i32 {
                let mut result = 0;
                if !state.has_axe {
                    result += manhattan_distance(state.position, state.nearest_axe);
                }
                if !state.has_wood {
                    result += manhattan_distance(state.nearest_axe, state.nearest_tree);
                }
                if !state.house_built {
                    result += manhattan_distance(state.nearest_tree, (0, 0));
                }
                result
            }
        }

        let initial_state = MyState {
            has_wood: false,
            has_axe: false,
            house_built: false,
            position: (0, 0),
            nearest_tree: (1, 1),
            nearest_axe: (2, 2),
        };

        let actions = vec![
            MyAction::ChopTree,
            MyAction::GrabAxe,
            MyAction::BuildHouse,
            MyAction::GoToTree,
            MyAction::GoToAxe,
            MyAction::GoHome,
        ];

        let goal = MyGoal;

        let (path, cost) = plan(&initial_state, &actions, &goal).unwrap();
        assert_eq!(
            path,
            vec![
                MyAction::GoToAxe,
                MyAction::GrabAxe,
                MyAction::GoToTree,
                MyAction::ChopTree,
                MyAction::GoHome,
                MyAction::BuildHouse,
            ]
        );
        assert_eq!(cost, 11);
    }
}
