#[derive(Debug, Clone)]
pub struct TemporalDependency {
    pub from: String,
    pub to: String,
    pub dependency_type: DependencyType,
    pub direction: Direction,
}

impl TemporalDependency {
    pub fn new(
        from: &str,
        to: &str,
        dependency_type: DependencyType,
        direction: Direction,
    ) -> TemporalDependency {
        TemporalDependency {
            from: from.to_string(),
            to: to.to_string(),
            dependency_type,
            direction,
        }
    }
}

impl std::fmt::Display for TemporalDependency {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.direction {
            Direction::Forward => write!(f, "≺{}", self.dependency_type),
            Direction::Backward => write!(f, "≻{}", self.dependency_type),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Direction {
    Forward,
    Backward,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencyType {
    Direct,
    Eventual,
}

impl std::fmt::Display for DependencyType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DependencyType::Direct => write!(f, "d"),
            DependencyType::Eventual => write!(f, ""),
        }
    }
}

pub fn check_temporal_dependency(
    from: &str,
    to: &str,
    traces: &[Vec<&str>],
    threshold: f64,
) -> Option<TemporalDependency> {
    assert!(
        (0.0..=1.0).contains(&threshold),
        "Threshold must be between 0 and 1"
    );

    if has_direct_dependency(from, to, &traces, threshold)
        || has_direct_dependency(to, from, &traces, threshold)
    {
        return Some(TemporalDependency {
            from: from.to_string(),
            to: to.to_string(),
            dependency_type: DependencyType::Direct,
            direction: if has_direct_dependency(from, to, &traces, threshold) {
                Direction::Forward
            } else {
                Direction::Backward
            },
        });
    }

    if has_eventual_dependency(from, to, &traces, threshold)
        || has_eventual_dependency(to, from, &traces, threshold)
    {
        return Some(TemporalDependency {
            from: from.to_string(),
            to: to.to_string(),
            dependency_type: DependencyType::Eventual,
            direction: if has_eventual_dependency(from, to, &traces, threshold) {
                Direction::Forward
            } else {
                Direction::Backward
            },
        });
    }

    None
}

/// Checks if there is a direct dependency between two events.
///
/// The condition simply specifies that the "from" event is
/// directly followed by the "to" event. (index + 1)
///
/// # Parameters
/// - `from`: The name of the starting event.
/// - `to`: The name of the ending event.
/// - `event_names`: A vector of vectors containing event names.
/// - `noise_threshold`: A threshold value to filter out noise.
///
/// # Returns
/// - `bool`: Returns `true` if there is a direct dependency, otherwise `false`.
fn has_direct_dependency(
    from: &str,
    to: &str,
    event_names: &[Vec<&str>],
    noise_threshold: f64,
) -> bool {
    determine_dependency(
        from,
        to,
        event_names,
        |from_index, to_index| from_index + 1 == to_index,
        noise_threshold,
    )
}

/// Checks if there is an eventual dependency between two events.
///
/// The condition simply specifies that the "from" event is
/// before the "to" event.
///
/// # Parameters
/// - `from`: The name of the starting event.
/// - `to`: The name of the ending event.
/// - `event_names`: A vector of vectors containing event names.
/// - `noise_threshold`: A threshold value to filter out noise.
///
/// # Returns
/// - `bool`: Returns `true` if there is an eventual dependency, otherwise `false`.
fn has_eventual_dependency(
    from: &str,
    to: &str,
    event_names: &[Vec<&str>],
    noise_threshold: f64,
) -> bool {
    determine_dependency(
        from,
        to,
        event_names,
        |from_index, to_index| from_index < to_index,
        noise_threshold,
    )
}

/// Helper function for determining if there is a dependency between two events based on a passed condition.
/// Determines if there is a dependency between two events based on their positions in traces.
///
/// # Parameters
/// * `from`- The starting event name to look for in the traces.
/// * `to`- The ending event name to look for in the traces.
/// * `event_names`- A vector of traces, where each trace is a vector of event names.
/// * `condition`- A closure that takes the indices of `from` and `to` in a trace and returns a boolean indicating if the dependency condition is met.
/// * `threshold`- A threshold value to determine if the dependency is significant.
///
/// # Returns
/// - `true` if the ratio of traces meeting the condition to the total number of traces is greater than or equal to the threshold.
/// - `false` otherwise.
///
/// # Example
/// ```
/// let event_names = vec![
///     vec!["start", "middle", "end"],
///     vec!["start", "end"],
///     vec!["middle", "start", "end"],
/// ];
// let condition = |from_index, to_index| from_index < to_index;
/// // let result = determine_dependency("start", "end", &event_names, condition, 0.5);
/// ```
fn determine_dependency<F>(
    from: &str,
    to: &str,
    event_names: &[Vec<&str>],
    condition: F,
    threshold: f64,
) -> bool
where
    F: Fn(usize, usize) -> bool,
{
    let traces_with_from_and_to: Vec<&Vec<&str>> = event_names
        .iter()
        .filter(|trace| trace.contains(&from) && trace.contains(&to))
        .collect();

    if traces_with_from_and_to.is_empty() {
        return false;
    }

    let mut count = 0;
    let total = traces_with_from_and_to.len();

    for trace in traces_with_from_and_to.iter() {
        let from_index = trace.iter().position(|&activity| activity == from);
        let to_index = trace.iter().position(|&activity| activity == to);

        if let (Some(from_index), Some(to_index)) = (from_index, to_index) {
            if condition(from_index, to_index) {
                count += 1;
            }
        }
    }

    (count as f64) / (total as f64) >= threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_direct_dependency_1() {
        let event_names = vec![
            vec!["A", "B", "C", "D"],
            vec!["A", "C", "B", "D"],
            vec!["A", "E", "D"],
        ];
        let activities = ["A", "B", "C", "D", "E"];
        let pairs = [("A", "E"), ("E", "D")];
        activities.iter().for_each(|from| {
            activities.iter().for_each(|to| {
                if from != to {
                    if pairs.contains(&(from, to)) {
                        assert!(has_direct_dependency(from, to, &event_names, 1.0));
                    } else {
                        assert!(!has_direct_dependency(from, to, &event_names, 1.0));
                    }
                }
            });
        });
    }

    #[test]
    fn test_has_direct_dependency_1_with_noise() {
        let event_names = vec![
            vec!["A", "B", "C", "D"],
            vec!["A", "C", "B", "D"],
            vec!["A", "E", "D"],
            vec!["A", "F", "D"],
            vec!["H", "A", "E"],
            vec!["A", "E", "F"],
            vec!["A", "F", "E"],
            vec!["A", "E", "G"],
        ];
        let activities = ["A", "B", "C", "D", "E", "F", "G", "H"];
        let pairs = [("A", "E"), ("E", "D"), ("E", "G"), ("F", "D"), ("H", "A")];
        activities.iter().for_each(|from| {
            activities.iter().for_each(|to| {
                if from != to {
                    if pairs.contains(&(from, to)) {
                        println!("({}, {})", from, to);
                        assert!(has_direct_dependency(from, to, &event_names, 0.8));
                    } else {
                        println!("FALSE: ({}, {})", from, to);
                        assert!(!has_direct_dependency(from, to, &event_names, 0.8));
                    }
                }
            });
        });
    }

    #[test]
    fn test_has_eventual_dependency_1() {
        let event_names = vec![
            vec!["A", "B", "C", "D"],
            vec!["A", "C", "B", "D"],
            vec!["A", "E", "D"],
        ];
        let activities = ["A", "B", "C", "D", "E"];
        let pairs = vec![
            ("A", "B"),
            ("A", "C"),
            ("A", "D"),
            ("A", "E"),
            ("B", "D"),
            ("C", "D"),
            ("E", "D"),
        ];
        activities.iter().for_each(|from| {
            activities.iter().for_each(|to| {
                if from != to {
                    if pairs.contains(&(from, to)) {
                        assert!(has_eventual_dependency(from, to, &event_names, 1.0));
                    } else {
                        assert!(!has_eventual_dependency(from, to, &event_names, 1.0));
                    }
                }
            });
        });
    }

    #[test]
    fn test_has_direct_dependency_2() {
        let event_names = vec![
            vec!["A", "C", "E", "G"],
            vec!["A", "E", "C", "G"],
            vec!["B", "D", "F", "G"],
            vec!["B", "F", "D", "G"],
        ];
        let activities = ["A", "B", "C", "D", "E", "F", "G"];
        activities.iter().for_each(|from| {
            activities.iter().for_each(|to| {
                if from != to {
                    assert!(!has_direct_dependency(from, to, &event_names, 1.0));
                }
            });
        });
    }

    #[test]
    fn test_has_eventual_dependency_2() {
        let event_names = vec![
            vec!["A", "C", "E", "G"],
            vec!["A", "E", "C", "G"],
            vec!["B", "D", "F", "G"],
            vec!["B", "F", "D", "G"],
        ];
        let activities = ["A", "B", "C", "D", "E", "F", "G"];
        let pairs = vec![
            ("A", "C"),
            ("A", "E"),
            ("A", "G"),
            ("B", "D"),
            ("B", "F"),
            ("B", "G"),
            ("C", "G"),
            ("D", "G"),
            ("E", "G"),
            ("F", "G"),
        ];
        activities.iter().for_each(|from| {
            activities.iter().for_each(|to| {
                if from != to {
                    if pairs.contains(&(from, to)) {
                        assert!(has_eventual_dependency(from, to, &event_names, 1.0));
                    } else {
                        assert!(!has_eventual_dependency(from, to, &event_names, 1.0));
                    }
                }
            });
        });
    }
}
