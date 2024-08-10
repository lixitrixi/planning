use std::hash::Hash;

/// Defines a goal that can be satisfied by a state.
///
/// Implementing this trait allows the type to be used in a plan.
/// Enums provide the best basis for implementations, as with actions.
///
/// ## Priority
/// Implementing he `priority` method allows multiple goals to be ranked by their importance.
/// It also allows goals to be prioritized dynamically based on the current state.
///
/// ## Heuristic
/// Implementing the `heuristic` method is optional, and it will default to a constant value if not implemented.
/// This method will make the search more efficient, and should not *overestimate* the actual cost.
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
    fn heuristic(&self, _state: &S) -> i32 {
        0
    }

    /// Returns a priority for this goal based on the given state.
    /// This may return a constant for each goal, or be implemented to support a dynamic priority system.
    /// Goals are sorted by priority in descending order.
    ///
    /// # Example
    /// ```
    /// # use planning::*;
    ///
    /// #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    /// struct State {
    ///     has_worked: bool,
    ///     hungry: bool,
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
    /// let goal = MyGoal::Eaten;
    /// let mut state = State { has_worked: false, hungry: false };
    /// assert_eq!(goal.priority(&state), 0);
    /// state.hungry = true;
    /// assert_eq!(goal.priority(&state), 2);
    ///
    /// ```
    fn priority(&self, _state: &S) -> i32 {
        0
    }
}
