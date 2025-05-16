use crate::model::{ContextProfile, Error, OutputFormat as Format, WeightedQueryLog};
use serde::Serialize;
use time::format_description::well_known::Rfc3339;

pub fn print_top_queries(queries: &[WeightedQueryLog], format: Format) {
    match format {
        Format::Json => print_top_json(queries),
        Format::Yaml => print_top_yaml(queries),
        Format::Text => print_top_text(queries),
    }
}

pub fn print_top_errors(errors: &[Error], format: Format) {
    match format {
        Format::Json => match serde_json::to_string_pretty(&errors) {
            Ok(json) => println!("{json}"),
            Err(err) => eprintln!("Failed to serialize to JSON: {err}"),
        },
        Format::Yaml => match serde_yaml::to_string(&errors) {
            Ok(yaml) => println!("{yaml}"),
            Err(err) => eprintln!("Failed to serialize to YAML: {err}"),
        },
        Format::Text => {
            if errors.is_empty() {
                println!("No errors found.");
                return;
            }

            println!(
                "{:<6} {:<25} {:<8} {:<20}",
                "Code", "Name", "Count", "Last Seen",
            );
            println!("{}", "-".repeat(59));

            for error in errors {
                let time_str = error
                    .last_error_time
                    .format(&Rfc3339)
                    .unwrap_or_else(|_| "-".into());
                println!(
                    "{:<6} {:<25} {:<8} {:<20}",
                    error.code, error.name, error.count, time_str,
                );
            }
        }
    }
}

/// Print a list of context profile names in the chosen format.
pub fn print_context_list(names: &[String], format: Format) {
    match format {
        Format::Text => {
            for name in names {
                println!("{name}");
            }
        }
        Format::Json => {
            #[derive(Serialize)]
            struct ListWrapper<'a> {
                profiles: &'a [String],
            }
            let wrapper = ListWrapper { profiles: names };
            match serde_json::to_string_pretty(&wrapper) {
                Ok(json) => println!("{json}"),
                Err(e) => {
                    eprintln!("Failed to serialize contexts to JSON: {e}");
                }
            }
        }
        Format::Yaml => {
            #[derive(Serialize)]
            struct ListWrapper<'a> {
                profiles: &'a [String],
            }
            let wrapper = ListWrapper { profiles: names };
            match serde_yaml::to_string(&wrapper) {
                Ok(yaml) => println!("{yaml}"),
                Err(e) => {
                    eprintln!("Failed to serialize contexts to YAML: {e}");
                }
            }
        }
    }
}

/// Print the active (or default) context name, or a message if none is set.
pub fn print_context_current(active: Option<&str>, format: Format) {
    match format {
        Format::Text => {
            if let Some(name) = active {
                println!("{name}");
            } else {
                println!("No active context set");
            }
        }
        Format::Json => {
            #[derive(Serialize)]
            struct CurrentWrapper<'a> {
                current: Option<&'a str>,
            }
            let wrapper = CurrentWrapper { current: active };
            match serde_json::to_string_pretty(&wrapper) {
                Ok(json) => println!("{json}"),
                Err(e) => {
                    eprintln!("Failed to serialize current context to JSON: {e}");
                }
            }
        }
        Format::Yaml => {
            #[derive(Serialize)]
            struct CurrentWrapper<'a> {
                current: Option<&'a str>,
            }
            let wrapper = CurrentWrapper { current: active };
            match serde_yaml::to_string(&wrapper) {
                Ok(yaml) => println!("{yaml}"),
                Err(e) => {
                    eprintln!("Failed to serialize current context to YAML: {e}");
                }
            }
        }
    }
}

pub fn print_context_profile(profile: &ContextProfile, format: Format) {
    match format {
        Format::Text => {
            println!("Profile:");
            println!("  URLs: {}", profile.urls.join(", "));
            println!("  User: {}", profile.user);
            println!(
                "  Password: {}",
                if profile.password.is_empty() {
                    "(empty)"
                } else {
                    &profile.password
                }
            );
        }
        Format::Json => {
            // Serialize the ContextProfile itself
            match serde_json::to_string_pretty(&profile) {
                Ok(json) => println!("{json}"),
                Err(e) => {
                    eprintln!("Failed to serialize profile to JSON: {e}");
                }
            }
        }
        Format::Yaml => match serde_yaml::to_string(&profile) {
            Ok(yaml) => println!("{yaml}"),
            Err(e) => {
                eprintln!("Failed to serialize profile to YAML: {e}");
            }
        },
    }
}

fn print_top_text(queries: &[WeightedQueryLog]) {
    for (i, q) in queries.iter().enumerate() {
        let weight = q.weight;
        let percent = (weight as f64 / q.total_weight as f64) * 100.0;

        println!(
            "{index}. Weight: {weight}\nWeight Percent: {percent:.2}%\nQuery:\n{query}\n",
            index = i + 1,
            weight = weight,
            percent = percent,
            query = q.query.query,
        );
    }
}

fn print_top_json(queries: &[WeightedQueryLog]) {
    match serde_json::to_string_pretty(queries) {
        Ok(json) => println!("{json}"),
        Err(err) => eprintln!("Failed to serialize queries to JSON: {err}"),
    }
}

fn print_top_yaml(queries: &[WeightedQueryLog]) {
    match serde_yaml::to_string(queries) {
        Ok(yaml) => println!("{yaml}"),
        Err(err) => eprintln!("Failed to serialize queries to YAML: {err}"),
    }
}
