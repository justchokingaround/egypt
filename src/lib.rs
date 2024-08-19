use chrono::{DateTime, Duration, Utc};
use dependency_types::{
    dependency::Dependency, existential::check_existential_dependency,
    temporal::check_temporal_dependency,
};
use std::collections::HashSet;

pub mod dependency_types;
pub mod parser;

pub fn generate_xes(text: &str) -> String {
    let mut output = String::with_capacity(text.len() * 2); // Estimate capacity
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

/// # Returns: (output, full_independences, pure_existences)
pub fn generate_adj_matrix_from_traces(traces: Vec<Vec<String>>) -> (String, usize, usize) {
    let mut activities = HashSet::new();

    traces.iter().for_each(|trace| {
        trace.iter().for_each(|activity| {
            activities.insert(activity.to_string());
        })
    });

    generate_adj_matrix_from_activities_and_traces(&activities, traces)
}


/// # Returns: (output, full_independences, pure_existences)
pub fn generate_adj_matrix_from_activities_and_traces(
    activities: &HashSet<String>,
    traces: Vec<Vec<String>>,
) -> (String, usize, usize) {
    let max_dependency_width = 15;

    let mut output = String::with_capacity(activities.len() * activities.len() * 20);
    let mut full_independences = 0;
    let mut pure_existences = 0;

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
                output.push_str(&format_dependency(&dependency));
            } else {
                output.push_str(&format!("{:<15}", "TODO"));
            }
        }
        output.push('\n');
    }

    (output, full_independences, pure_existences)
}

pub fn generate_adj_matrix(text: &str) -> String {
    let (activities, traces) = get_activities_and_traces(text);
    let max_dependency_width = 10;

    let mut output = String::with_capacity(activities.len() * activities.len() * 20);

    // Header
    output.push_str(&format!("{:<15}", " "));
    for activity in &activities {
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

    for from in &activities {
        output.push_str(&format!("{:<15}", from));
        for to in &activities {
            if to != from {
                let temporal_dependency = check_temporal_dependency(from, to, &traces, 1.0);
                let existential_dependency = check_existential_dependency(from, to, &traces, 1.0);
                let dependency = Dependency::new(
                    from.to_string(),
                    to.to_string(),
                    temporal_dependency,
                    existential_dependency,
                );

                output.push_str(&format!("{:<15}", format_dependency(&dependency)));
            } else {
                output.push_str(&format!("{:<15}", "todo"));
            }
        }
        output.push('\n');
    }

    output
}

fn get_activities_and_traces(text: &str) -> (Vec<String>, Vec<Vec<&str>>) {
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

fn get_traces(text: &str) -> Vec<Vec<&str>> {
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
    use std::collections::HashSet;

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
