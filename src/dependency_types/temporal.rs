use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Eq)]
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
    println!("Checking temporal dependency for {} -> {}", from, to);
    let mut dependencies = Vec::new();

    for (i, trace) in traces.iter().enumerate() {
        println!("Checking trace {}: {:?}", i, trace);
        let trace_deps = check_trace_dependency(from, to, trace);
        println!("Trace {} dependencies: {:?}", i, trace_deps);
        dependencies.extend(trace_deps);
    }

    println!("All dependencies: {:?}", dependencies);
    let result = classify_dependencies(from, to, dependencies, threshold);
    println!("Final result: {:?}", result);
    result
}

fn check_trace_dependency(
    from: &str,
    to: &str,
    trace: &[&str],
) -> Vec<(DependencyType, Direction)> {
    let mut result = Vec::new();
    let from_indices: Vec<_> = trace
        .iter()
        .enumerate()
        .filter(|(_, &event)| event == from)
        .map(|(index, _)| index)
        .collect();
    let to_indices: Vec<_> = trace
        .iter()
        .enumerate()
        .filter(|(_, &event)| event == to)
        .map(|(index, _)| index)
        .collect();

    println!("From indices: {:?}", from_indices);
    println!("To indices: {:?}", to_indices);

    for &from_index in &from_indices {
        for &to_index in &to_indices {
            match from_index.cmp(&to_index) {
                Ordering::Less => {
                    let dep_type = if to_index - from_index == 1 {
                        DependencyType::Direct
                    } else {
                        DependencyType::Eventual
                    };
                    println!("Forward dependency: {:?} at {} -> {}", dep_type, from_index, to_index);
                    result.push((dep_type, Direction::Forward));
                }
                Ordering::Greater => {
                    let dep_type = if from_index - to_index == 1 {
                        DependencyType::Direct
                    } else {
                        DependencyType::Eventual
                    };
                    println!("Backward dependency: {:?} at {} -> {}", dep_type, from_index, to_index);
                    result.push((dep_type, Direction::Backward));
                }
                Ordering::Equal => {
                    println!("Equal indices at {}, ignoring", from_index);
                } // Ignore if they're at the same position
            }
        }
    }

    result
}

fn classify_dependencies(
    from: &str,
    to: &str,
    dependencies: Vec<(DependencyType, Direction)>,
    threshold: f64,
) -> Option<TemporalDependency> {
    if dependencies.is_empty() {
        println!("No dependencies found");
        return None;
    }

    let total_count = dependencies.len() as f64;
    let forward_count = dependencies
        .iter()
        .filter(|(_, dir)| *dir == Direction::Forward)
        .count() as f64;
    let backward_count = total_count - forward_count;

    println!("Total dependencies: {}", total_count);
    println!("Forward dependencies: {}", forward_count);
    println!("Backward dependencies: {}", backward_count);

    // Calculate the ratio of the dominant direction
    let dominant_ratio = (forward_count.max(backward_count) / total_count).max(threshold);

    if dominant_ratio >= threshold && (forward_count == 0.0 || backward_count == 0.0) {
        let direction = if forward_count > backward_count {
            println!("Classified as Forward");
            Direction::Forward
        } else {
            println!("Classified as Backward");
            Direction::Backward
        };

        let dependency_type = if dependencies
            .iter()
            .all(|(dep, _)| *dep == DependencyType::Direct)
        {
            println!("Classified as Direct");
            DependencyType::Direct
        } else {
            println!("Classified as Eventual");
            DependencyType::Eventual
        };

        Some(TemporalDependency::new(
            from,
            to,
            dependency_type,
            direction,
        ))
    } else {
        println!("No clear direction, below threshold, or mixed directions. Classified as independent");
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_with_loops_general() {
        let event_names = vec![
            vec!["A", "B", "C", "B", "D", "B"],
            vec!["A", "C", "B", "D"],
            vec!["B", "A", "C", "D", "B"],
        ];
        let activities = ["A", "B", "C", "D"];
        let mut pairs_and_deps = HashMap::new();

        pairs_and_deps.insert(
            ("A", "B"),
            check_temporal_dependency("A", "B", &event_names, 1.0),
        );
        pairs_and_deps.insert(
            ("A", "C"),
            check_temporal_dependency("A", "C", &event_names, 1.0),
        );
        pairs_and_deps.insert(
            ("B", "C"),
            check_temporal_dependency("B", "C", &event_names, 1.0),
        );
        pairs_and_deps.insert(
            ("C", "D"),
            check_temporal_dependency("C", "D", &event_names, 1.0),
        );
        // FIXME: fix us :3
        // pairs_and_deps.insert(("B", "D"), check_temporal_dependency("B", "D", &event_names, 1.0));
        // pairs_and_deps.insert(("B", "B"), check_temporal_dependency("B", "B", &event_names, 1.0));

        let expected = HashMap::from([
            (("A", "B"), None),
            (
                ("A", "C"),
                Some(TemporalDependency::new(
                    "A",
                    "C",
                    DependencyType::Eventual,
                    Direction::Forward,
                )),
            ),
            (("B", "C"), None),
            (("B", "D"), None),
            (
                ("C", "D"),
                Some(TemporalDependency::new(
                    "C",
                    "D",
                    DependencyType::Eventual,
                    Direction::Forward,
                )),
            ),
            (
                ("B", "B"),
                Some(TemporalDependency::new(
                    "B",
                    "B",
                    DependencyType::Eventual,
                    Direction::Forward,
                )),
            ),
        ]);

        pairs_and_deps.iter().for_each(|(key, value)| {
            assert_eq!(value, expected.get(key).unwrap());
        });
    }

    #[test]
    fn test_with_loop_1() {
        let trace = vec![vec!["A", "B", "C", "A", "C"]];
        let activities = ["A", "B", "C"];
        let expected = Some(TemporalDependency::new(
            "A",
            "C",
            DependencyType::Eventual,
            Direction::Forward,
        ));
        let actual = check_temporal_dependency("A", "C", &trace, 1.0);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_independence() {
        let trace = vec![vec!["A", "B", "C", "C", "A"]];
        let actual = check_temporal_dependency("A", "C", &trace, 1.0);
        assert_eq!(None, actual);
    }

    #[test]
    fn test_with_loop_2() {
        let trace = vec![vec!["A", "C", "B", "C"]];
        // should first check A and C at positions 0 and 1 (direct dep), then since there is no more A, A and C are eventual
        // so in total it should be eventual
        let actual = check_temporal_dependency("A", "C", &trace, 1.0);
        let expected = Some(TemporalDependency::new(
            "A",
            "C",
            DependencyType::Eventual,
            Direction::Forward,
        ));
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_with_loop_3() {
        let trace = vec![vec!["C", "A", "C"]];
        // should first check A and C at positions 0 and 1 (direct dep in backward direction), then since there is no more A,
        // we check A at 1 and C at 2 and we get a direct dep in forward direction
        // so in total we get an independence (since at least two of the deps have opposite directions)
        let actual = check_temporal_dependency("A", "C", &trace, 1.0);
        assert_eq!(None, actual);
    }
}
