use crate::model;
use ascii_table::AsciiTable;
use humansize::{DECIMAL, format_size};
use time::format_description::well_known::Rfc3339;

const MAX_COLUMN_LEN: usize = 30;

/// Clean and shorten string for display in tables.
/// - Removes newlines and trims whitespace
/// - Truncates to `max_len` and appends ellipsis if too long
fn compact_str(s: &str, max_len: usize) -> String {
    let mut compact = s
        .replace('\n', " ") // убрать переносы строк
        .replace('\t', " ") // убрать табы
        .split_whitespace() // разбить по пробелам
        .collect::<Vec<_>>() // собрать в вектор
        .join(" ");

    if compact.len() > max_len {
        compact.truncate(max_len);
        compact.push('…');
    }

    compact
}

/// Print a slice of `WeightedQueryLog` in an ASCII table,
/// showing only the most important columns.
pub fn print_weighted_queries_table(logs: &[model::QueryLog]) {
    let mut table = AsciiTable::default();
    table.column(0).set_header("Hash");
    table.column(1).set_header("Query");
    table.column(2).set_header("Total Impact");
    table.column(3).set_header("IO Impact");
    table.column(4).set_header("CPU Impact");
    table.column(5).set_header("Memory Impact");
    table.column(6).set_header("Time Impact");

    let data: Vec<_> = logs
        .iter()
        .map(|l| {
            let hash = format!("{:#x}", l.normalized_query_hash);
            let io_impact: String = format_size(l.io_impact, DECIMAL);
            let cpu_impact: String = format_size(l.cpu_impact, DECIMAL);
            let memory_impact: String = format_size(l.memory_impact, DECIMAL);
            let time_impact: String = format_size(l.time_impact, DECIMAL);
            let total_impact: String = format_size(l.total_impact, DECIMAL);

            vec![
                hash.to_string(),
                compact_str(&l.query, MAX_COLUMN_LEN),
                total_impact,
                io_impact,
                cpu_impact,
                memory_impact,
                time_impact,
            ]
        })
        .collect();
    table.print(data);
}

/// Print a slice of `Errors` in an ASCII table.
pub fn print_errors_table(errs: &[model::Error]) {
    let mut table = AsciiTable::default();
    table.column(0).set_header("Code");
    table.column(1).set_header("Name");
    table.column(2).set_header("Count");
    table.column(3).set_header("Last Seen");
    table.column(4).set_header("Message");

    let data: Vec<_> = errs
        .iter()
        .map(|e| {
            let last_seen = e
                .last_error_time
                .format(&Rfc3339)
                .unwrap_or_else(|_| "-".into());
            vec![
                e.code.to_string(),
                e.name.to_string(),
                e.count.to_string(),
                last_seen,
                compact_str(&e.error_message, MAX_COLUMN_LEN),
            ]
        })
        .collect();
    table.print(data);
}

pub fn print_context_names_table(names: &[String]) {
    let mut table = AsciiTable::default();
    table.column(0).set_header("Name");

    let data: Vec<_> = names.iter().map(|n| vec![n]).collect();
    table.print(data);
}

pub fn print_context_current(active: Option<&str>) {
    if let Some(name) = active {
        println!("{name}");
    } else {
        println!("No active context set");
    }
}

pub fn print_context_config_path(path: &std::path::PathBuf) {
    println!("{}", path.display());
}

pub fn print_context_profile(profile: &model::PrintableContextProfile) {
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
    println!(
        "  Accept invalid certificate: {}",
        profile.accept_invalid_certificate
    );
}
