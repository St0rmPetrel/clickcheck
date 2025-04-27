use crate::model::{OutputFormat as Format, WeightedQueryLog};

pub fn print_top(queries: &[WeightedQueryLog], format: Format) {
    match format {
        Format::Json => print_top_json(queries),
        Format::Yaml => print_top_yaml(queries),
        Format::Text => print_top_text(queries),
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
