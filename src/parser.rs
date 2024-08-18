use process_mining::{import_xes_file, XESImportOptions};
use process_mining::event_log::import_xes::XESParseError;
use process_mining::event_log::AttributeValue;
use std::collections::HashSet;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
struct Event {
    activity: String,
    date: DateTime<Utc>,
}

impl Event {
    fn new(activity: String, date: DateTime<Utc>) -> Event {
        Event { activity, date }
    }
}

pub fn parse_into_traces(path: &str) -> Result<Vec<Vec<String>>, XESParseError> {
    let event_log = import_xes_file(path, XESImportOptions::default())?;
    let traces = event_log.traces;

    let mut result = Vec::new();

    for trace in traces {
        let mut events: Vec<Event> = Vec::new();

        for event in trace.events {
            let mut name = None;
            let mut date = None;

            for attribute in &event.attributes {
                match attribute.key.as_str() {
                    "concept:name" => {
                        if let AttributeValue::String(value) = &attribute.value {
                            name = Some(value.clone());
                        }
                    }
                    "time:timestamp" => {
                        if let AttributeValue::Date(value) = &attribute.value {
                            date = Some(*value);
                        }
                    }
                    _ => {}
                }
            }

            if let (Some(name), Some(date)) = (name, date) {
                events.push(Event::new(name, date));
            }
        }

        events.sort_by(|a, b| a.date.cmp(&b.date)); // just in case the events are not sorted already

        let activity_list: Vec<String> = events.into_iter().map(|event| event.activity).collect();
        result.push(activity_list);
    }

    Ok(result)
}

pub fn get_activities(path: &str) -> Option<HashSet<String>> {
    let event_log = import_xes_file(path, XESImportOptions::default()).ok()?;
    let traces = event_log.traces;
    let mut activities = HashSet::new();

    traces.into_iter().for_each(|t| {
        t.events.iter().for_each(|e| {
            e.attributes.iter().for_each(|a| {
                if a.key == "concept:name" {
                    if let AttributeValue::String(ref s) = a.value {
                        activities.insert(s.clone());
                    }
                }
            })
        });
    });
    Some(activities)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_activities() {
        let activities = get_activities("./sample-data/exercise2.xes").unwrap();
        assert_eq!(activities.len(), 5);
        let actual_activities = ["A", "B", "C", "D", "E", "F"];
        actual_activities.into_iter()
            .for_each(|a| assert!(activities.contains(a)));
    }

    #[test]
    fn test_parse_into_traces() {
        let traces = parse_into_traces("./sample-data/exercise2.xes").unwrap();
        assert_eq!(traces.len(), 2);
        assert_eq!(traces[0].len(), 3);
        assert_eq!(traces[1].len(), 3);

        assert_eq!(traces[0], ["B", "C", "E"]);
        assert_eq!(traces[1], ["A", "C", "D"]);
    }
}
