use crate::{plan, Action, Goal};
use std::hash::Hash;

/// A stateful agent capable of choosing from multiple goals based on priority.
///
/// Given a current state, a list of possible actions, and a list of goals,
/// an agent can plan a sequence of actions to achieve the most appropriate goal based on different metrics.
///
/// # Example
/// ```
/// # use planning::*;
///
/// #[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// struct State {
///     num_flowers: u16,
///     hungry: bool,
///     picnic_set: bool,
/// }
///
/// #[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// enum MyAction {
///     PickFlower,
///     SetPicnic,
///     Eat,
/// }
///
/// impl Action<State> for MyAction {
///     fn is_applicable(&self, state: &State) -> bool {
///         match self {
///             MyAction::PickFlower => state.num_flowers < 5,
///             MyAction::SetPicnic => !state.picnic_set,
///             MyAction::Eat => state.hungry && state.picnic_set,
///         }
///     }
///
///     fn apply_mut(&self, state: &mut State) {
///         match self {
///             MyAction::PickFlower => state.num_flowers += 1,
///             MyAction::SetPicnic => state.picnic_set = true,
///             MyAction::Eat => state.hungry = false,
///         }
///     }
/// }
///
/// #[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// enum MyGoal {
///     BouquetMade,
///     Eaten,
/// }
///
/// impl Goal<State> for MyGoal {
///     fn is_satisfied(&self, state: &State) -> bool {
///         match self {
///             MyGoal::BouquetMade => state.num_flowers >= 5,
///             MyGoal::Eaten => !state.hungry,
///         }
///     }
///
///     fn priority(&self, state: &State) -> i32 {
///         match self {
///             MyGoal::BouquetMade => 1,
///             MyGoal::Eaten => if state.hungry { 2 } else { 0 },
///         }
///     }
/// }
///
/// let mut agent = Agent::new(
///     State { num_flowers: 0, hungry: true, picnic_set: false },
///     vec![MyAction::PickFlower, MyAction::SetPicnic, MyAction::Eat],
///     vec![MyGoal::BouquetMade, MyGoal::Eaten],
/// );
///
/// let (goal, plan, _) = agent.plan_dynamic().unwrap();
/// assert_eq!(goal, &MyGoal::Eaten);
/// assert_eq!(plan, vec![MyAction::SetPicnic, MyAction::Eat]);
///
/// agent.state.hungry = false;
/// let (goal, plan, _) = agent.plan_dynamic().unwrap();
/// assert_eq!(goal, &MyGoal::BouquetMade);
/// assert_eq!(plan, vec![MyAction::PickFlower; 5]);
/// ```
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    /// On initialization, the goals are sorted in descending order by priority, based on the given state.
    pub fn new(state: S, actions: Vec<A>, goals: Vec<G>) -> Self {
        let mut new = Self {
            state,
            actions,
            goals,
        };
        new.sort_goals();
        new
    }

    // Sort in descending order of priority
    fn sort_goals(&mut self) {
        self.goals
            .sort_by(|a, b| b.priority(&self.state).cmp(&a.priority(&self.state)));
    }

    /// Returns the plan and total cost for the first goal that can be satisfied.
    ///
    /// This method **does not** sort the goals by priority before searching.
    ///**If your goals return dynamic priorities based on the current state, use `plan_dynamic` instead.**
    pub fn plan_constant(&self) -> Option<(&G, Vec<A>, i32)> {
        self.goals.iter().find_map(|goal| {
            plan(&self.state, &self.actions, goal).map(|(path, cost)| (goal, path, cost))
        })
    }

    /// Returns the plan and total cost for the first goal that can be satisfied.
    ///
    /// This method sorts the goals by priority based on the current state before searching.
    ///
    /// **If your goals return the same priority regardless of the agent's current state,
    /// use `plan_constant` instead.**
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
        self.sort_goals();
        self.plan_constant()
    }

    /// Calculates the best plan for each of the agent's goals and returns all possible plans.
    ///
    /// Returned plans are in arbitrary order.
    pub fn plan_all(&self) -> Vec<(&G, Vec<A>, i32)> {
        self.goals
            .iter()
            .filter_map(|goal| {
                plan(&self.state, &self.actions, goal).map(|(path, cost)| (goal, path, cost))
            })
            .collect()
    }

    /// Calculates all possible goals and returns the plan with the highest profit.
    ///
    /// Profit is defined as the difference between the goal's priority and the total cost of the plan.
    ///
    /// # Example
    /// ```
    /// # use planning::*;
    /// #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    /// struct State {
    ///     banana_sold: bool,
    ///     apple_sold: bool,
    /// }
    ///
    /// #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    /// enum Sell {
    ///     SellApple,  // Action costs 4, goal priority is 5 (higher priority)
    ///     SellBanana, // Action costs 1, goal priority is 4 (more profitable)
    /// }
    ///
    /// impl Action<State> for Sell {
    ///     fn is_applicable(&self, state: &State) -> bool {
    ///         !(state.apple_sold || state.banana_sold)
    ///     }
    ///
    ///     fn apply_mut(&self, state: &mut State) {
    ///         match self {
    ///             Sell::SellApple => state.apple_sold = true,
    ///             Sell::SellBanana => state.banana_sold = true,
    ///         }
    ///     }
    ///
    ///     fn cost(&self, _state: &State) -> i32 {
    ///         match self {
    ///             Sell::SellApple => 4,
    ///             Sell::SellBanana => 1,
    ///         }
    ///     }
    /// }
    ///
    /// impl Goal<State> for Sell {
    ///     fn is_satisfied(&self, state: &State) -> bool {
    ///         match self {
    ///             Sell::SellApple => state.apple_sold,
    ///             Sell::SellBanana => state.banana_sold,
    ///         }
    ///     }
    ///
    ///     fn priority(&self, _state: &State) -> i32 {
    ///         match self {
    ///             Sell::SellApple => 5,
    ///             Sell::SellBanana => 4,
    ///         }
    ///     }
    /// }
    ///
    /// let mut agent = Agent::new(
    ///     State {
    ///         banana_sold: false,
    ///         apple_sold: false,
    ///     },
    ///     vec![Sell::SellApple, Sell::SellBanana],
    ///     vec![Sell::SellApple, Sell::SellBanana],
    /// );
    ///
    /// let (goal, plan, _) = agent.plan_dynamic().unwrap();
    /// assert_eq!(plan, vec![Sell::SellApple]);
    /// assert_eq!(goal, &Sell::SellApple); // Higher priority
    ///
    /// let (goal, plan, _) = agent.plan_profit().unwrap();
    /// assert_eq!(plan, vec![Sell::SellBanana]);
    /// assert_eq!(goal, &Sell::SellBanana); // More profitable
    /// ```
    pub fn plan_profit(&self) -> Option<(&G, Vec<A>, i32)> {
        self.plan_all()
            .into_iter()
            .max_by_key(|(goal, _, cost)| goal.priority(&self.state) - cost)
    }
}
