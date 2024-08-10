use planning::*;

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
