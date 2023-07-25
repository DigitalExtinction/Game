use bevy::{ecs::schedule::run_enter_schedule, prelude::*};
pub use paste;

pub trait DeStateTransition {
    /// This method is almost equal to Bevy's [`App::add_state`]. The only
    /// difference is that the state transition is added to an associated
    /// state. See [`StateWithSet`].
    fn add_state_with_set<S: States + StateWithSet>(&mut self) -> &mut Self;

    fn add_child_state<P: StateWithSet, S: States + StateWithSet>(&mut self) -> &mut Self;
}

pub trait StateWithSet {
    type Set: SystemSet;

    fn state_set() -> Self::Set;
}

impl DeStateTransition for App {
    fn add_state_with_set<S: States + StateWithSet>(&mut self) -> &mut Self {
        self.init_resource::<State<S>>()
            .init_resource::<NextState<S>>()
            .add_systems(
                StateTransition,
                (
                    run_enter_schedule::<S>.run_if(run_once()),
                    apply_state_transition::<S>.in_set(S::state_set()),
                )
                    .chain(),
            );

        self
    }

    fn add_child_state<P: StateWithSet, S: States + StateWithSet>(&mut self) -> &mut Self {
        self.add_state_with_set::<S>();
        self.configure_sets(StateTransition, (P::state_set(), S::state_set()).chain());
        self
    }
}

/// Creates a Bevy state and a Bevy plugin.
///
/// The child state is bound to a given parent state with this syntax:
/// `ParentState::ParentVariant -> ChildState`.
///
/// Transitions of the child state are scheduled after transitions of the
/// parent state.
///
/// Variant `None` is automatically added to the child state. Additional
/// variants are configurable with `variants` argument.
///
/// `None` variant of the child state is entered when the given variant of the
/// parent state is exited. The `enter` system is called on enter of the parent
/// variant and the `exit` system is called on exit of the parent variant.
#[macro_export]
macro_rules! nested_state {
    (
        $parent:ident::$parent_variant:ident -> $name:ident,
        doc = $doc:expr,
        $(enter = $enter:ident,)?
        $(exit = $exit:ident,)?
        variants = {
            $($variant:ident),* $(,)?
        }
    ) => {
        use $crate::transition::{DeStateTransition, paste::paste, StateWithSet};

        paste! {
            #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, SystemSet)]
            pub struct [<$name Plugin>];

            impl Plugin for [<$name Plugin>] {
                fn build(&self, app: &mut App) {
                    app.add_child_state::<$parent, $name>()
                        .add_systems(OnExit($parent::$parent_variant), go_to_none)
                        $(.add_systems(OnEnter($parent::$parent_variant), $enter))?
                        $(.add_systems(OnExit($parent::$parent_variant), $exit))?;
                }
            }


            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, States)]
            #[doc = $doc]
            pub enum $name {
                #[default]
                None,
                $($variant),*
            }

            impl StateWithSet for $name {
                type Set = [<$name Set>];

                fn state_set() -> Self::Set {
                    [<$name Set>]
                }
            }

            #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, SystemSet)]
            pub struct [<$name Set>];

            fn go_to_none(mut next_state: ResMut<NextState<$name>>) {
                next_state.set($name::None);
            }
        }
    };
}
