//! Handles output formatting and printing for different data types in the CLI.
//!
//! Supports output formats: plain text (human-readable), JSON, and YAML.
use crate::model::{
    Error, OutputFormat as Format, PrintableContextProfile, QueryLog, QueryLogTotal,
};
use serde::Serialize;

mod text;

// Вспомогательная функция для сериализации и печати
fn serialize_and_print<T: Serialize + ?Sized>(data: &T, format: Format, data_description: &str) {
    match format {
        Format::Json => match serde_json::to_string_pretty(data) {
            Ok(json) => println!("{json}"),
            Err(err) => eprintln!("Failed to serialize {data_description} to JSON: {err}"),
        },
        Format::Yaml => match serde_yaml::to_string(data) {
            Ok(yaml) => println!("{yaml}"),
            Err(err) => eprintln!("Failed to serialize {data_description} to YAML: {err}"),
        },
        Format::Text => {
            // Эта ветка не должна достигаться, если функция используется правильно,
            // так как Text формат обрабатывается отдельно.
            eprintln!(
                "Error: serialize_and_print called with Text format for {data_description}. This should be handled separately."
            );
        }
    }
}

/// Prints the top heaviest queries in the selected output format.
///
/// - `queries`: A slice of query logs, typically sorted by weight.
/// - `format`: Output format (Text, JSON, or YAML).
pub fn print_top_queries(queries: &[QueryLog], format: Format) {
    match format {
        Format::Text => text::print_weighted_queries_table(queries),
        Format::Json | Format::Yaml => serialize_and_print(queries, format, "top queries"),
    }
}

/// Prints the total weights of queries aggregated over a period of time.
///
/// - `queries`: A reference to the total query statistics.
/// - `format`: Output format (Text, JSON, or YAML).
pub fn print_total_queries(queries: &QueryLogTotal, format: Format) {
    match format {
        Format::Text => text::print_total_queries_table(queries),
        Format::Json | Format::Yaml => serialize_and_print(queries, format, "total queries"),
    }
}

/// Prints the most frequent errors observed in `system.errors`.
///
/// - `errors`: A slice of aggregated errors.
/// - `format`: Output format (Text, JSON, or YAML).
pub fn print_top_errors(errors: &[Error], format: Format) {
    match format {
        Format::Text => text::print_errors_table(errors),
        Format::Json | Format::Yaml => serialize_and_print(errors, format, "top errors"),
    }
}

/// Prints a list of available context profile names.
///
/// - `names`: Slice of context profile names.
/// - `format`: Output format (Text, JSON, or YAML).
pub fn print_context_list(names: &[String], format: Format) {
    match format {
        Format::Text => text::print_context_names_table(names),
        Format::Json | Format::Yaml => {
            #[derive(Serialize)]
            struct ListWrapper<'a> {
                profiles: &'a [String],
            }
            let wrapper = ListWrapper { profiles: names };
            serialize_and_print(&wrapper, format, "context list")
        }
    }
}

/// Prints the name of the currently active context profile, or an empty message if none is set.
///
/// - `active`: Name of the active profile, or `None`.
/// - `format`: Output format (Text, JSON, or YAML).
pub fn print_context_current(active: Option<&str>, format: Format) {
    match format {
        Format::Text => text::print_context_current(active),
        Format::Json | Format::Yaml => {
            #[derive(Serialize)]
            struct CurrentWrapper<'a> {
                current: Option<&'a str>,
            }
            let wrapper = CurrentWrapper { current: active };
            serialize_and_print(&wrapper, format, "context current")
        }
    }
}

/// Prints the file path to the context configuration.
///
/// - `path`: Path to the `config.toml`.
/// - `format`: Output format (Text, JSON, or YAML).
pub fn print_context_config_path(path: &std::path::PathBuf, format: Format) {
    match format {
        Format::Text => text::print_context_config_path(path),
        Format::Json | Format::Yaml => {
            #[derive(Serialize)]
            struct ConfigPathWrapper<'a> {
                config_path: &'a str,
            }
            let wrapper = ConfigPathWrapper {
                config_path: &path.to_string_lossy(),
            };
            serialize_and_print(&wrapper, format, "context config-path")
        }
    }
}

/// Prints a detailed description of a single context profile.
///
/// - `profile`: The profile to print
/// - `format`: Output format (Text, JSON, or YAML).
pub fn print_context_profile(profile: &PrintableContextProfile, format: Format) {
    match format {
        Format::Text => text::print_context_profile(profile),
        Format::Json | Format::Yaml => serialize_and_print(&profile, format, "context profile"),
    }
}
