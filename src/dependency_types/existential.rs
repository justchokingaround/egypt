#[derive(Debug, Clone)]
pub struct ExistentialDependency {
    pub from: String,
    pub to: String,
    pub dependency_type: DependencyType,
    pub direction: Direction,
}

impl ExistentialDependency {
    pub fn new(
        from: &str,
        to: &str,
        dependency_type: DependencyType,
        direction: Direction,
    ) -> Self {
        ExistentialDependency {
            from: from.to_string(),
            to: to.to_string(),
            dependency_type,
            direction,
        }
    }
}

impl std::fmt::Display for ExistentialDependency {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.dependency_type == DependencyType::Implication {
            match &self.direction {
                Direction::Forward => write!(f, "=>"),
                Direction::Backward => write!(f, "<="),
                Direction::Both => panic!("Invalid direction for Implication"),
            }
        } else {
            write!(f, "{}", self.dependency_type)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Direction {
    Forward,
    Backward,
    Both,
}

#[allow(dead_code)]
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum DependencyType {
    Implication,
    Equivalence,
    NegatedEquivalence,
    Nand,
    Or,
}

impl std::fmt::Display for DependencyType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DependencyType::Implication => write!(f, "⇒"),
            DependencyType::Equivalence => write!(f, "⇔"),
            DependencyType::NegatedEquivalence => write!(f, "⇎"),
            DependencyType::Nand => write!(f, "⊼"),
            DependencyType::Or => write!(f, "∨"),
        }
    }
}

// TODO: NAND and OR dependencies
/// Checks for an existential dependency between two activities within a set of traces.
///
/// This function analyzes the given traces to determine if there is an existential dependency
/// between the `from` and `to` activities based on the specified threshold. It considers
/// implications, equivalences, and negated equivalences to identify the type and direction
/// of the dependency (in that order).
///
/// # Arguments
///
/// * `from` - The name of the starting activity.
/// * `to` - The name of the target activity.
/// * `traces` - A vector of `Trace` objects representing the sequence of events.
/// * `threshold` - A threshold value to determine if the dependency is significant.
///
/// # Returns
///
/// An `Option` containing an `ExistentialDependency` if a dependency is found, otherwise `None`.
pub fn check_existential_dependency(
    from: &str,
    to: &str,
    traces: &[Vec<&str>],
    threshold: f64,
) -> Option<ExistentialDependency> {
    assert!(
        (0.0..=1.0).contains(&threshold),
        "Threshold must be between 0 and 1"
    );

    let implication = has_implication(from, to, traces, threshold);

    if implication || has_implication(to, from, traces, threshold) {
        return Some(ExistentialDependency {
            from: from.to_string(),
            to: to.to_string(),
            dependency_type: if implication && has_implication(to, from, traces, threshold) {
                DependencyType::Equivalence
            } else {
                DependencyType::Implication
            },
            direction: if implication {
                Direction::Forward
            } else {
                Direction::Backward
            },
        });
    }

    let negated_equivalence = negated_equivalence(from, to, traces, threshold);

    if negated_equivalence {
        return Some(ExistentialDependency {
            from: from.to_string(),
            to: to.to_string(),
            dependency_type: DependencyType::NegatedEquivalence,
            direction: Direction::Forward,
        });
    }

    None
}

/// Checks if there is an implication relationship between two events within a set of event traces.
///
/// # Parameters
/// - `from`: The event that implies the occurrence of another event.
/// - `to`: The event that is implied by the occurrence of the `from` event.
/// - `event_names`: A vector of vectors, where each inner vector represents a sequence of event names (a trace).
/// - `threshold`: A threshold value between 0 and 1 that determines the minimum proportion of valid traces required to confirm the implication.
///
/// # Returns
/// - `true` if the proportion of valid traces is greater than or equal to the threshold, indicating that the implication holds.
/// - `false` otherwise.
fn has_implication(from: &str, to: &str, event_names: &[Vec<&str>], threshold: f64) -> bool {
    let total_traces = event_names.len();
    let valid_traces = event_names
        .iter()
        .filter(|trace| {
            if trace.contains(&from) {
                trace.contains(&to)
            } else {
                true
            }
        })
        .count();
    valid_traces as f64 / total_traces as f64 >= threshold
}

fn negated_equivalence(from: &str, to: &str, event_names: &[Vec<&str>], threshold: f64) -> bool {
    let total_traces = event_names.len();
    let valid_traces = event_names
        .iter()
        .filter(|trace| {
            if trace.contains(&from) {
                !trace.contains(&to)
            } else {
                true
            }
        })
        .count();
    valid_traces as f64 / total_traces as f64 >= threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_implication() {
        let event_names = vec![
            vec!["A", "B", "C", "D"],
            vec!["A", "C", "B", "D"],
            vec!["A", "E", "D"],
            vec!["A", "D"],
        ];
        let activities = ["A", "B", "C", "D", "E"];
        let pairs = vec![
            ("A", "D"),
            ("B", "A"),
            ("B", "C"),
            ("B", "D"),
            ("C", "A"),
            ("C", "B"),
            ("C", "D"),
            ("D", "A"),
            ("E", "A"),
            ("E", "D"),
        ];
        activities.iter().for_each(|from| {
            activities.iter().for_each(|to| {
                if from != to {
                    if pairs.contains(&(from, to)) {
                        assert!(has_implication(from, to, &event_names, 1.0));
                    } else {
                        assert!(!has_implication(from, to, &event_names, 1.0));
                    }
                }
            });
        });
    }

    #[test]
    fn test_has_implication_with_noise() {
        let event_names = vec![
            vec!["A", "B", "C", "D"],
            vec!["A", "C", "B", "D"],
            vec!["A", "E", "D"],
            vec!["A", "D"],
            vec!["A", "C"], // Noise: D is missing
        ];
        assert!(has_implication("A", "D", &event_names, 0.8));
        assert!(!has_implication("A", "D", &event_names, 1.0));
    }
    // TODO: add more tests
}
