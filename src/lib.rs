use chrono::{DateTime, Duration, Utc};
use dependency_types::{
    dependency::Dependency, existential::check_existential_dependency,
    temporal::check_temporal_dependency,
};
use std::collections::{HashMap, HashSet};

pub mod dependency_types;
pub mod parser;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Event {
    pub case: String,
    pub activity: char,
    pub predecessor: Option<String>,
}

#[derive(Debug)]
pub struct State {
    pub partition: Option<usize>,
    pub sequences: HashSet<Event>,
}

#[derive(Debug)]
pub struct ExtendedPrefixAutomaton {
    pub states: HashMap<String, State>,
    pub transitions: Vec<(String, char, String)>,
    pub activities: HashSet<char>,
    pub root: String,
}

impl Default for ExtendedPrefixAutomaton {
    fn default() -> Self {
        Self::new()
    }
}

impl ExtendedPrefixAutomaton {
    pub fn new() -> Self {
        let root_id = "root".to_string();
        let mut states = HashMap::new();
        states.insert(
            root_id.clone(),
            State {
                partition: None,
                sequences: HashSet::new(),
            },
        );

        ExtendedPrefixAutomaton {
            states,
            transitions: Vec::new(),
            activities: HashSet::new(),
            root: root_id,
        }
    }

    pub fn build(plain_log: Vec<Vec<Event>>) -> Self {
        let mut epa = ExtendedPrefixAutomaton::new();
        let mut last_at: HashMap<String, String> = HashMap::new();

        for trace in plain_log {
            for event in trace {
                let pred_at = event.predecessor
                    .as_ref()
                    .and_then(|case| last_at.get(case))
                    .unwrap_or(&epa.root)
                    .to_string();

                let current_at = if let Some(target) = epa.transitions.iter()
                    .find(|(source, act, _)| source == &pred_at && *act == event.activity)
                    .map(|(_, _, target)| target.to_string())
                {
                    target
                } else {
                    let new_state_id = format!("s{}", epa.states.len());
                    let current_c = if pred_at == epa.root {
                        1
                    } else if epa.transitions.iter().any(|(source, _, _)| source == &pred_at) {
                        epa.states.values().filter_map(|s| s.partition).max().unwrap_or(0) + 1
                    } else {
                        epa.states[&pred_at].partition.unwrap_or(0)
                    };

                    epa.states.insert(new_state_id.clone(), State {
                        partition: Some(current_c),
                        sequences: HashSet::new(),
                    });
                    epa.transitions.push((pred_at, event.activity, new_state_id.clone()));
                    epa.activities.insert(event.activity);

                    new_state_id
                };

                epa.states.get_mut(&current_at).unwrap().sequences.insert(event.clone());
                last_at.insert(event.case.clone(), current_at);
            }
        }

        epa
    }

    pub fn variant_entropy(&self) -> f64 {
        let s = self.states.len() as f64;
        let s = if s > 1.0 { s - 1.0 } else { s };

        let partition_sizes: HashMap<usize, usize> = self.states.values()
            .filter_map(|state| state.partition)
            .fold(HashMap::new(), |mut acc, partition| {
                *acc.entry(partition).or_insert(0) += 1;
                acc
            });

        let sum_term: f64 = partition_sizes.values()
            .map(|&size| {
                let size_f64 = size as f64;
                size_f64 * size_f64.log(10.0)
            })
            .sum();

        s * s.log(10.0) - sum_term
    }

    pub fn normalized_variant_entropy(&self) -> f64 {
        let e_v = self.variant_entropy();
        let s = self.states.len() as f64;
        let s = if s > 1.0 { s - 1.0 } else { s };
        e_v / (s * s.log(10.0))
    }
}

pub fn generate_xes(text: &str) -> String {
    let mut output = String::with_capacity(text.len() * 2);
    let traces = get_traces(text);

    output.push_str("<log xes.version=\"1.0\" xes.features=\"nested-attributes\" openxes.version=\"1.0RC7\" xmlns=\"http://www.xes-standard.org/\">\n");

    for trace in traces {
        output.push_str("<trace>\n");

        let mut starting_time = DateTime::<Utc>::default();

        for event in trace {
            starting_time = starting_time
                .checked_add_signed(Duration::milliseconds(1000))
                .expect("Time overflow occurred");

            output.push_str(&format!(
                "<event>\n\
                <string key=\"concept:name\" value=\"{}\"/>\n\
                <date key=\"time:timestamp\" value=\"{}\"/>\n\
                </event>\n",
                event,
                starting_time.to_rfc3339()
            ));
        }

        output.push_str("</trace>\n");
    }

    output.push_str("</log>\n");

    output
}

pub fn generate_adj_matrix_from_traces(traces: Vec<Vec<String>>) -> (String, usize, usize, usize, usize, usize) {
    let mut activities = HashSet::new();

    traces.iter().for_each(|trace| {
        trace.iter().for_each(|activity| {
            activities.insert(activity.to_string());
        })
    });

    generate_adj_matrix_from_activities_and_traces(&activities, traces)
}

pub fn generate_adj_matrix_from_activities_and_traces(
    activities: &HashSet<String>,
    traces: Vec<Vec<String>>,
) -> (String, usize, usize, usize, usize, usize) {
    let max_dependency_width = 15;

    let mut output = String::with_capacity(activities.len() * activities.len() * 20);
    let mut full_independences = 0;
    let mut pure_existences = 0;
    let mut eventual_equivalences = 0;
    let mut direct_equivalences = 0;

    // Header
    output.push_str(&format!("{:<15}", " "));
    for activity in activities {
        output.push_str(&format!("{:<15}", activity));
    }
    output.push('\n');

    let format_dependency = |dep: &Dependency| {
        format!(
            "{:<width$}",
            format!("{}", dep),
            width = max_dependency_width
        )
    };

    for from in activities {
        output.push_str(&format!("{:<15}", from));
        for to in activities {
            if to != from {
                let converted_traces: Vec<Vec<&str>> = traces
                    .iter()
                    .map(|v| v.iter().map(|s| s.as_str()).collect())
                    .collect();
                let temporal_dependency = check_temporal_dependency(from, to, &converted_traces, 1.0);
                let existential_dependency = check_existential_dependency(from, to, &converted_traces, 1.0);
                let dependency = Dependency::new(
                    from.to_string(),
                    to.to_string(),
                    temporal_dependency.clone(),
                    existential_dependency.clone(),
                );

                if temporal_dependency.is_none() {
                    pure_existences += 1;
                    if existential_dependency.is_none() {
                        full_independences += 1;
                    }
                }

                if let Some(existential_dependency) = existential_dependency {
                    if existential_dependency.dependency_type == dependency_types::existential::DependencyType::Equivalence {
                        if let Some(temporal_dependency) = temporal_dependency {
                            match temporal_dependency.dependency_type {
                                dependency_types::temporal::DependencyType::Eventual => eventual_equivalences += 1,
                                dependency_types::temporal::DependencyType::Direct => direct_equivalences += 1,
                            }
                        }
                    }
                }

                output.push_str(&format_dependency(&dependency));
            } else {
                output.push_str(&format!("{:<15}", "TODO"));
            }
        }
        output.push('\n');
    }

    (output, full_independences, pure_existences, eventual_equivalences, direct_equivalences, activities.len())
}

pub fn get_activities_and_traces(text: &str) -> (Vec<String>, Vec<Vec<&str>>) {
    let mut activities = HashSet::new();
    let mut traces = Vec::new();

    for line in text.lines() {
        let trace: Vec<&str> = line
            .split(',')
            .filter(|&activity| !activity.trim().is_empty())
            .collect();

        if !trace.is_empty() {
            activities.extend(trace.iter().map(|&s| s.to_string()));
            traces.push(trace);
        }
    }

    (activities.into_iter().collect(), traces)
}

pub fn get_traces(text: &str) -> Vec<Vec<&str>> {
    text.lines()
        .filter_map(|line| {
            let trace: Vec<&str> = line
                .split(',')
                .filter(|&activity| !activity.trim().is_empty())
                .collect();
            if trace.is_empty() {
                None
            } else {
                Some(trace)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_activities_and_traces() {
        let traces = "
activity 3,activity 3,activity 3,activity 3,activity 3,activity 1,activity 1,activity 2,
activity 3,activity 1,activity 2,
activity 1,activity 1,activity 1,activity 1,activity 3,activity 1,activity 1,activity 2,
activity 3,activity 1,activity 1,activity 2,
";
        let (activities, traces) = get_activities_and_traces(traces);
        let expected_activities: HashSet<_> = vec!["activity 1", "activity 2", "activity 3"]
            .into_iter()
            .map(String::from)
            .collect();
        assert_eq!(expected_activities, activities.into_iter().collect());

        let expected_traces = vec![
            vec![
                "activity 3",
                "activity 3",
                "activity 3",
                "activity 3",
                "activity 3",
                "activity 1",
                "activity 1",
                "activity 2",
            ],
            vec!["activity 3", "activity 1", "activity 2"],
            vec![
                "activity 1",
                "activity 1",
                "activity 1",
                "activity 1",
                "activity 3",
                "activity 1",
                "activity 1",
                "activity 2",
            ],
            vec!["activity 3", "activity 1", "activity 1", "activity 2"],
        ];
        assert_eq!(expected_traces, traces);
    }

    #[test]
    fn test_get_traces() {
        let traces = "
activity 3,activity 3,activity 3,activity 3,activity 3,activity 1,activity 1,activity 2,
activity 3,activity 1,activity 2,
activity 1,activity 1,activity 1,activity 1,activity 3,activity 1,activity 1,activity 2,
activity 3,activity 1,activity 1,activity 2,
";
        let expected_traces = vec![
            vec![
                "activity 3",
                "activity 3",
                "activity 3",
                "activity 3",
                "activity 3",
                "activity 1",
                "activity 1",
                "activity 2",
            ],
            vec!["activity 3", "activity 1", "activity 2"],
            vec![
                "activity 1",
                "activity 1",
                "activity 1",
                "activity 1",
                "activity 3",
                "activity 1",
                "activity 1",
                "activity 2",
            ],
            vec!["activity 3", "activity 1", "activity 1", "activity 2"],
        ];
        assert_eq!(expected_traces, get_traces(traces));
    }
}
