use crate::model;
use time::OffsetDateTime;
use time::macros::format_description;

#[derive(Debug, Clone)]
pub struct QueryLogFilter {
    pub from: Option<OffsetDateTime>,
    pub to: Option<OffsetDateTime>,
}

pub enum QueryParam {
    DateTime(OffsetDateTime),
}

impl QueryParam {
    pub fn to_sql_string(&self) -> Result<String, time::error::Format> {
        match self {
            QueryParam::DateTime(t) => {
                let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

                Ok(t.format(&format)?)
            }
        }
    }
}

impl QueryLogFilter {
    /// Собирает SQL-фрагменты WHERE и возвращает (условие, параметры)
    pub fn build_where(self) -> (String, Vec<QueryParam>) {
        let mut clauses = Vec::new();
        let mut params = Vec::new();

        if let Some(from) = self.from {
            clauses.push("event_time >= ?");
            params.push(QueryParam::DateTime(from));
        }
        if let Some(to) = self.to {
            clauses.push("event_time < ?");
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

impl From<model::Filter> for QueryLogFilter {
    fn from(args: model::Filter) -> Self {
        Self {
            from: args.from,
            to: args.to,
        }
    }
}
