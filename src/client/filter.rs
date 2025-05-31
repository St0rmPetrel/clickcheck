use crate::model;
use std::time::Duration;
use time::OffsetDateTime;
use time::macros::format_description;

#[derive(Debug, Clone)]
pub struct QueryLogFilter {
    pub from: Option<OffsetDateTime>,
    pub to: Option<OffsetDateTime>,
    pub last: Option<Duration>,
}

#[derive(Debug, Clone)]
pub enum QueryParam {
    DateTime(OffsetDateTime),
    UInt64(u64),
    Int32(i32),
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
        }
    }
}

impl QueryLogFilter {
    /// Собирает SQL-фрагменты WHERE и возвращает (условие, параметры)
    pub fn build_where(&self) -> (String, Vec<QueryParam>) {
        let mut clauses = Vec::new();
        let mut params = Vec::new();

        if let Some(from) = self.from {
            clauses.push("event_time >= toDateTime(?, 'UTC')");
            params.push(QueryParam::DateTime(from));
        }
        if let Some(last) = self.last {
            let now = OffsetDateTime::now_utc();
            let threshold = now - last;
            clauses.push("event_time >= toDateTime(?, 'UTC')");
            params.push(QueryParam::DateTime(threshold));
        }
        if let Some(to) = self.to {
            clauses.push("event_time < toDateTime(?, 'UTC')");
            params.push(QueryParam::DateTime(to));
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
    fn from(args: model::QueriesFilter) -> Self {
        Self {
            from: args.from,
            to: args.to,
            last: args.last,
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
