use crate::model;
use std::time::Duration;
use time::OffsetDateTime;
use time::macros::format_description;

#[derive(Debug, Clone)]
pub struct QueryLogFilter {
    pub from: Option<OffsetDateTime>,
    pub to: Option<OffsetDateTime>,
    pub last: Option<Duration>,

    pub users: Vec<String>,
    pub databases: Vec<String>,
    pub tables: Vec<String>,

    pub min_query_duration: Option<std::time::Duration>,
    pub min_read_rows: Option<u64>,
    pub min_read_data: Option<bytesize::ByteSize>,
}

#[derive(Debug, Clone)]
pub enum QueryParam {
    DateTime(OffsetDateTime),
    UInt64(u64),
    Int32(i32),
    String(String),
}

impl QueryParam {
    pub fn to_sql_string(&self) -> Result<String, time::error::Format> {
        match self {
            QueryParam::DateTime(t) => {
                let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

                Ok(t.format(&format)?)
            }
            QueryParam::UInt64(v) => Ok(format!("{}", v)),
            QueryParam::Int32(v) => Ok(format!("{}", v)),
            QueryParam::String(v) => Ok(v.clone()),
        }
    }
}

impl QueryLogFilter {
    /// Собирает SQL-фрагменты WHERE и возвращает (условие, параметры)
    pub fn build_where(&self) -> (String, Vec<QueryParam>) {
        let mut clauses: Vec<String> = Vec::new();
        let mut params = Vec::new();

        if let Some(from) = self.from {
            clauses.push("event_time >= toDateTime(?, 'UTC')".to_owned());
            params.push(QueryParam::DateTime(from));
        }
        if let Some(last) = self.last {
            let now = OffsetDateTime::now_utc();
            let threshold = now - last;
            clauses.push("event_time >= toDateTime(?, 'UTC')".to_owned());
            params.push(QueryParam::DateTime(threshold));
        }
        if let Some(to) = self.to {
            clauses.push("event_time < toDateTime(?, 'UTC')".to_owned());
            params.push(QueryParam::DateTime(to));
        }

        if !self.users.is_empty() {
            let placeholders = vec!["?"; self.users.len()].join(", ");
            clauses.push(format!("user IN ({placeholders})"));
            self.users.iter().for_each(|user| {
                params.push(QueryParam::String(user.clone()));
            });
        }
        if let Some(min_read_rows) = self.min_read_rows {
            clauses.push("read_rows >= ?".to_owned());
            params.push(QueryParam::UInt64(min_read_rows));
        }
        if let Some(min_read_data) = self.min_read_data {
            let min_read_bytes = min_read_data.as_u64();
            clauses.push("read_bytes >= ?".to_owned());
            params.push(QueryParam::UInt64(min_read_bytes));
        }
        if let Some(min_query_duration) = self.min_query_duration {
            let min_query_duration = min_query_duration.as_millis() as u64;
            clauses.push("query_duration_ms >= ?".to_owned());
            params.push(QueryParam::UInt64(min_query_duration));
        }

        if !self.tables.is_empty() {
            let placeholders = vec!["?"; self.tables.len()].join(", ");
            clauses.push(format!("hasAny(query_log.tables, [{placeholders}])"));
            self.tables.iter().for_each(|table| {
                params.push(QueryParam::String(table.clone()));
            });
        }
        if !self.databases.is_empty() {
            let placeholders = vec!["?"; self.databases.len()].join(", ");
            clauses.push(format!("hasAny(query_log.databases, [{placeholders}])"));
            self.databases.iter().for_each(|database| {
                params.push(QueryParam::String(database.clone()));
            });
        }

        let where_clause = if clauses.is_empty() {
            String::new()
        } else {
            format!("AND {}", clauses.join(" AND "))
        };

        (where_clause, params)
    }
}

impl From<model::QueriesFilter> for QueryLogFilter {
    fn from(filter: model::QueriesFilter) -> Self {
        Self {
            from: filter.from,
            to: filter.to,
            last: filter.last,
            users: filter.users,
            tables: filter.tables,
            databases: filter.databases,
            min_query_duration: filter.min_query_duration,
            min_read_rows: filter.min_read_rows,
            min_read_data: filter.min_read_data,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ErrorFilter {
    pub last: Option<Duration>,
    pub min_count: Option<usize>,
    pub code: Vec<i32>,
}

impl From<model::ErrorsFilter> for ErrorFilter {
    fn from(args: model::ErrorsFilter) -> Self {
        Self {
            last: args.last,
            min_count: args.min_count,
            code: args.code,
        }
    }
}

impl ErrorFilter {
    /// Собирает SQL-фрагменты WHERE и возвращает (условие, параметры)
    pub fn build_where(&self) -> (String, Vec<QueryParam>) {
        let mut clauses: Vec<String> = Vec::new();
        let mut params = Vec::new();

        if !self.code.is_empty() {
            let placeholders = vec!["?"; self.code.len()].join(", ");
            clauses.push(format!("code IN ({})", placeholders));
            self.code.iter().for_each(|code| {
                params.push(QueryParam::Int32(*code));
            });
        }

        let where_clause = if clauses.is_empty() {
            String::new()
        } else {
            format!("AND {}", clauses.join(" AND "))
        };

        (where_clause, params)
    }

    /// Собирает SQL-фрагменты HAVING и возвращает (условие, параметры)
    pub fn build_having(&self) -> (String, Vec<QueryParam>) {
        let mut clauses = Vec::new();
        let mut params = Vec::new();

        if let Some(min_count) = self.min_count {
            clauses.push("count >= ?");
            params.push(QueryParam::UInt64(min_count as u64));
        }

        if let Some(last) = self.last {
            let now = OffsetDateTime::now_utc();
            let threshold = now - last;
            clauses.push("last_error_time >= toDateTime(?, 'UTC')");
            params.push(QueryParam::DateTime(threshold));
        }

        let having_clause = if clauses.is_empty() {
            String::new()
        } else {
            format!("AND {}", clauses.join(" AND "))
        };

        (having_clause, params)
    }
}
