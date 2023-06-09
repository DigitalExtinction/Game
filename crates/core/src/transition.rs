use bevy::{
    ecs::schedule::{run_enter_schedule, FreeSystemSet},
    prelude::*,
};

pub trait DeStateTransition {
    /// This method is almost equal to Bevy's [`App::add_state`]. The only
    /// difference is that the state transition is added to an associated
    /// state. See [`StateWithSet`].
    fn add_state_with_set<S: States + StateWithSet>(&mut self) -> &mut Self;
}

pub trait StateWithSet {
    type Set: FreeSystemSet;

    fn state_set() -> Self::Set;
}

impl DeStateTransition for App {
    fn add_state_with_set<S: States + StateWithSet>(&mut self) -> &mut Self {
        self.init_resource::<State<S>>();
        self.init_resource::<NextState<S>>();

        let mut schedules = self.world.resource_mut::<Schedules>();

        let Some(default_schedule) = schedules.get_mut(&*self.default_schedule_label) else {
            let schedule_label = &self.default_schedule_label;
            panic!("Default schedule {schedule_label:?} does not exist.")
        };

        default_schedule.add_systems(
            (
                run_enter_schedule::<S>.run_if(run_once()),
                apply_state_transition::<S>.in_set(S::state_set()),
            )
                .chain()
                .in_base_set(CoreSet::StateTransitions),
        );

        for variant in S::variants() {
            default_schedule.configure_set(
                OnUpdate(variant.clone())
                    .in_base_set(CoreSet::Update)
                    .run_if(in_state(variant)),
            );
        }

        // These are different for loops to avoid conflicting access to self
        for variant in S::variants() {
            self.add_schedule(OnEnter(variant.clone()), Schedule::new());
            self.add_schedule(OnExit(variant), Schedule::new());
        }

        self
    }
}
