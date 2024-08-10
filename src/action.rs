use std::hash::Hash;

/// Defines a state transition with pre-conditions and an optional cost.
///
/// Implementing this trait allows the type to be used in a plan.
/// Enums provide the best basis for implementations as they can be matched easily from a returned sequence.
///
/// ## Applicability
/// The `is_applicable` method should return true if the action can be applied to the given state.
/// This is used to filter out actions while searching for a plan.
///
/// ## Cost
/// Implementing the `cost` method is optional, and it will default to a constant value if not implemented.
/// This method is useful when actions are not equally difficult, such as waiting or pathfinding.
/// When choosing a plan, the algorithm will choose the sequence with the lowest total cost.
///
/// # Example
/// ```
/// # use planning::*;
///
/// #[derive(PartialEq, Eq, Hash, Clone, Debug)]
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
/// assert_eq!(MakeCorrect.is_applicable(&State { is_correct: false }), true);
/// assert_eq!(MakeCorrect.is_applicable(&State { is_correct: true }), false);
///
/// let mut state = State { is_correct: false };
/// MakeCorrect.apply_mut(&mut state);
/// assert_eq!(state, State { is_correct: true });
///```
pub trait Action<S>
where
    S: Clone + Hash + Eq,
{
    /// Returns true if the action can be applied to the given state.
    fn is_applicable(&self, state: &S) -> bool;

    /// Applies the action to the given state in-place.
    ///
    /// This is used by the default implementation of `apply`, which may be implemented directly instead.
    fn apply_mut(&self, state: &mut S);

    /// Applies the action to the given state and returns the new state.
    ///
    /// This method returns a copy of the given state with the action applied.
    fn apply(&self, state: &S) -> S {
        let mut new_state = state.clone();
        self.apply_mut(&mut new_state);
        new_state
    }

    /// Returns the cost of applying the action to the given state.
    ///
    /// Implementing this method is optional, and it will default to a constant value if not implemented.
    /// This method is useful for more complex plans which include actions like pathfinding, waiting, etc.
    /// When choosing a plan, the algorithm will choose the path with the lowest total cost.
    fn cost(&self, _state: &S) -> i32 {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_apply() {
        #[derive(PartialEq, Eq, Hash, Clone, Debug)]
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

        let state = State { is_correct: false };
        assert_eq!(MakeCorrect.apply(&state), State { is_correct: true });
    }
}
