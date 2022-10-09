use super::bounds::Bounds;

/// Iterator of fixed point parameters corresponding to potential optimal
/// permissible velocities.
pub(super) struct ParameterIterator {
    index: usize,
    inside: i32,
    bounds: Bounds,
    transitions: Vec<Transition>,
}

impl ParameterIterator {
    /// # Arguments
    ///
    /// * `inside` - number of regions which contain point corresponding to
    ///   parameter 0.
    ///
    /// * `bounds` - minimum & maximum parameter bounds. Parameters outside of
    ///   this range are not yielded.
    ///
    /// * `transitions` - unsorted vector of transitions. It could contain
    ///   multiple transitions with the same parameter.
    pub(super) fn new(inside: i32, bounds: Bounds, mut transitions: Vec<Transition>) -> Self {
        transitions.sort_unstable_by_key(|t| t.parameter);

        let mut normalized: Vec<Transition> = Vec::with_capacity(transitions.len());
        for transition in transitions {
            if transition.parameter() > bounds.max() {
                break;
            }

            match normalized
                .last_mut()
                .filter(|last| last.parameter() == transition.parameter())
            {
                Some(last) => last.update(transition),
                None => normalized.push(transition),
            }
        }

        Self {
            index: 0,
            inside,
            bounds,
            transitions: normalized,
        }
    }
}

impl Iterator for ParameterIterator {
    type Item = i32;

    fn next(&mut self) -> Option<i32> {
        loop {
            if self.index >= self.transitions.len() {
                break None;
            }
            let transition = self.transitions[self.index];
            debug_assert!(transition.parameter() <= self.bounds.max());
            self.index += 1;

            let inside_before = self.inside > 0;
            self.inside += transition.delta();
            if inside_before && self.inside > 0 {
                continue;
            }

            if transition.parameter >= self.bounds.min() {
                break Some(transition.parameter());
            }
        }
    }
}

#[derive(Copy, Clone, Default)]
pub(super) struct Transition {
    parameter: i32,
    delta: i32,
}

impl Transition {
    /// Returns a new region in/out transition.
    ///
    /// A transition corresponds to either a projection of desired velocity
    /// onto an edge line (having `delta` equal to 0) or to an intersection of
    /// two region edges. It corresponds to a single point in 2D space.
    ///
    /// # Arguments
    ///
    /// * `parameter` - displacement of the transition from its edge start point.
    ///   It is a fixed point multiple of the edge direction.
    ///
    /// * `delta` - number of regions which are entered (positive number) or
    ///   exited (negative number) at the point of the transition.
    pub(super) fn new(parameter: i32, delta: i32) -> Self {
        Self { parameter, delta }
    }

    pub(super) fn delta(self) -> i32 {
        self.delta
    }

    fn parameter(self) -> i32 {
        self.parameter
    }

    /// Sets delta of `self` to the sum of `self` and `other`.
    fn update(&mut self, other: Self) {
        debug_assert!(other.parameter == self.parameter);
        self.delta += other.delta;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_iterator() {
        let mut parameters = ParameterIterator::new(
            2,
            Bounds::new(42, 69),
            vec![
                Transition::new(22, -4),  // inside 1
                Transition::new(1, -2),   // inside 1
                Transition::new(0, 1),    // inside 3
                Transition::new(1, -1),   // inside 0
                Transition::new(45, -1),  // inside 0
                Transition::new(8, 5),    // inside 5
                Transition::new(100, -2), // inside 0
                Transition::new(55, 2),   // inside 2
            ],
        );

        assert_eq!(parameters.next(), Some(45));
        assert_eq!(parameters.next(), Some(55));
        assert_eq!(parameters.next(), None);
        assert_eq!(parameters.next(), None);
    }

    #[test]
    fn test_transition() {
        let mut transition_a = Transition::new(42, 69);
        transition_a.update(Transition::new(42, -20));
        assert_eq!(transition_a.parameter(), 42);
        assert_eq!(transition_a.delta(), 49);
    }
}
