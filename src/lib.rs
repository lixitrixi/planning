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

    /// Returns a heuristic estimate of the total cost to satisfy the goal from the given state.
    /// The default implementation returns a constant, and implementing this will make the search more efficient.
    ///
    /// The heuristic should not *overestimate* the actual cost, or else resulting plans may be incorrect.
    fn heuristic(&self, state: &S) -> i32 {
        0
    }

    /// Returns a priority for this goal based on the given state.
    /// This may return a constant for each goal, or be implemented support a dynamic priority system.
    fn priority(&self, state: &S) -> i32 {
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

/// Returns a sequence of actions to reach the goal while minimizing cost, if possible.
///
/// # Example
/// ```
/// # use dyn_goap::*;
///
/// #[derive(PartialEq, Eq, Hash, Clone)]
/// struct State {
///     is_correct: bool,
/// }
///
/// #[derive(PartialEq, Eq, Hash, Clone, Debug)]
/// struct MakeCorrect;
///
/// impl Action<State> for MakeCorrect {
///     fn is_applicable(&self, state: &State) -> bool {
///         !state.is_correct
///     }
///
///     fn apply_mut(&self, state: &mut State) {
///         state.is_correct = true;
///     }
/// }
///
/// #[derive(PartialEq, Eq, Hash, Clone)]
/// struct IsCorrect;
///
/// impl Goal<State> for IsCorrect {
///     fn is_satisfied(&self, state: &State) -> bool {
///         state.is_correct
///     }
/// }
///
/// let actions = vec![MakeCorrect];
/// let goal = IsCorrect;
///
/// let initial_state = State { is_correct: false };
/// let (path, cost) = plan(&initial_state, &actions, &goal).unwrap();
/// assert_eq!(path, vec![MakeCorrect]);
/// assert_eq!(cost, 1);
///
/// let initial_state = State { is_correct: true };
/// let (path, cost) = plan(&initial_state, &actions, &goal).unwrap();
/// assert_eq!(path, vec![]);
/// assert_eq!(cost, 0);
/// ```
pub fn plan<S, A, G>(initial_state: &S, actions: &Vec<A>, goal: &G) -> Option<(Vec<A>, i32)>
where
    S: Clone + Hash + Eq,
    A: Action<S> + Eq + Clone + Hash,
    G: Goal<S> + Clone,
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

#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Agent<S, A, G>
where
    S: Clone + Hash + Eq,
    A: Action<S> + Eq + Clone + Hash,
    G: Goal<S> + Clone,
{
    pub state: S,
    pub actions: Vec<A>,
    pub goals: Vec<G>,
}

impl<S, A, G> Agent<S, A, G>
where
    S: Clone + Hash + Eq,
    A: Action<S> + Eq + Clone + Hash,
    G: Goal<S> + Clone,
{
    /// Creates a new agent with the given initial state, possible actions, and goals.
    ///
    /// The goals are sorted by priority based on the initial state.
    /// This implementation uses a stable sorting algorithm, so goals with the same priority will remain in the same order.
    pub fn new(state: S, actions: Vec<A>, goals: Vec<G>) -> Self {
        let mut goals = goals;
        goals.sort_by(|a, b| b.priority(&state).cmp(&a.priority(&state))); // sort in reverse order
        Self {
            state,
            actions,
            goals,
        }
    }

    /// Returns the plan and total cost for the first goal that can be satisfied.
    ///
    /// This method does not sort the goals by priority before searching.
    /// This implementation uses a stable sorting algorithm, so goals with the same priority will remain in the same order.
    ///
    /// **If your goals return different priorities based on state, use `plan_dynamic` instead.**
    pub fn plan_constant(&self) -> Option<(&G, Vec<A>, i32)> {
        self.goals.iter().find_map(|goal| {
            plan(&self.state, &self.actions, goal).map(|(path, cost)| (goal, path, cost))
        })
    }

    /// Returns the plan and total cost for the first goal that can be satisfied.
    ///
    /// This method sorts the goals by priority based on the current state before searching.
    /// This implementation uses a stable sorting algorithm, so goals with the same priority will remain in the same order.
    ///
    /// **If your goals return the same priority regardless of state, use `plan_constant` instead.**
    ///
    /// # Example
    /// ```
    /// # use dyn_goap::*;
    ///
    /// #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    /// struct State {
    ///     has_worked: bool,
    ///     hungry: bool,
    /// }
    ///
    /// #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    /// enum MyAction {
    ///     Work,
    ///     Eat,
    /// }
    ///
    /// impl Action<State> for MyAction {
    ///     fn is_applicable(&self, state: &State) -> bool {
    ///         true // both are always possible
    ///     }
    ///
    ///     fn apply_mut(&self, state: &mut State) {
    ///         match self {
    ///             MyAction::Work => state.has_worked = true,
    ///             MyAction::Eat => state.hungry = false,
    ///         }
    ///     }
    /// }
    ///
    /// #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    /// enum MyGoal {
    ///     Worked,
    ///     Eaten,
    /// }
    ///
    /// impl Goal<State> for MyGoal {
    ///     fn is_satisfied(&self, state: &State) -> bool {
    ///         match self {
    ///             MyGoal::Worked => state.has_worked,
    ///             MyGoal::Eaten => !state.hungry,
    ///         }
    ///     }
    ///
    ///     fn priority(&self, state: &State) -> i32 {
    ///         match self {
    ///             MyGoal::Worked => 1,
    ///             MyGoal::Eaten => if state.hungry { 2 } else { 0 },
    ///         }
    ///     }
    /// }
    ///
    /// let mut agent = Agent::new(
    ///    State { has_worked: false, hungry: false },
    ///    vec![MyAction::Work, MyAction::Eat],
    ///    vec![MyGoal::Worked, MyGoal::Eaten],
    /// );
    ///
    /// let (goal, _, _) = agent.plan_dynamic().unwrap();
    /// assert_eq!(goal, &MyGoal::Worked);
    ///
    /// agent.state.hungry = true; // agent will now prioritize eating
    /// let (goal, _, _) = agent.plan_dynamic().unwrap();
    /// assert_eq!(goal, &MyGoal::Eaten);
    /// ```
    pub fn plan_dynamic(&mut self) -> Option<(&G, Vec<A>, i32)> {
        self.goals
            .sort_by(|a, b| b.priority(&self.state).cmp(&a.priority(&self.state)));
        self.plan_constant()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
