use crate::{Action, Goal};
use pathfinding::prelude::astar;
use std::hash::{Hash, Hasher};

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
/// # use planning::*;
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
/// let initial_state = State { is_correct: false };
/// let mut actions = vec![];
/// let goal = IsCorrect;
///
/// let result = plan(&initial_state, &actions, &goal);
/// assert_eq!(result, None);
///
/// actions.push(MakeCorrect);
///
/// let (path, cost) = plan(&initial_state, &actions, &goal).unwrap();
/// assert_eq!(path, vec![MakeCorrect]);
/// assert_eq!(cost, 1);
///
/// let initial_state = State { is_correct: true };
///
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_empty() {
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
        let mut actions = vec![];
        let goal = IsCorrect;

        let result = plan(&initial_state, &actions, &goal);
        assert_eq!(result, None);

        actions.push(MakeCorrect);

        let (path, cost) = plan(&initial_state, &actions, &goal).unwrap();
        assert_eq!(path, vec![MakeCorrect]);
        assert_eq!(cost, 1);

        let initial_state = State { is_correct: true };

        let (path, cost) = plan(&initial_state, &actions, &goal).unwrap();
        assert_eq!(path, vec![]);
        assert_eq!(cost, 0);
    }

    #[test]
    fn plan_one() {
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
        assert_eq!(path, vec![MakeCorrect]);
        assert_eq!(cost, 1);

        let initial_state = State { is_correct: true };

        let (path, cost) = plan(&initial_state, &actions, &goal).unwrap();
        assert_eq!(path, vec![]);
        assert_eq!(cost, 0);
    }
}
