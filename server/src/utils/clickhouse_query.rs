use clickhouse::sql::Identifier;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::fmt;
use tokio::io::AsyncBufReadExt;
use utoipa::ToSchema;

use crate::errors::ServiceError;

#[derive(Debug, Display)]
pub enum ValidationError {
    #[display(fmt = "Invalid column name: {_0}")]
    InvalidColumnName(String),

    #[display(fmt = "Invalid operator: {_0}")]
    InvalidOperator(String),

    #[display(fmt = "Invalid aggregation function: {_0}")]
    InvalidAggregationFunction(String),

    #[display(fmt = "Invalid expression: {_0}")]
    InvalidExpression(String),

    #[display(fmt = "Invalid join condition: {_0}")]
    InvalidJoinCondition(String),

    #[display(fmt = "Invalid having clause: {_0}")]
    InvalidHavingClause(String),

    #[display(fmt = "Invalid order by clause: {_0}")]
    InvalidOrderByClause(String),

    #[display(fmt = "Invalid limit: {_0}")]
    InvalidLimit(String),

    #[display(fmt = "Invalid value: {_0}")]
    InvalidValue(String),
}

fn check_for_injection(query: &str) -> Result<(), ValidationError> {
    let query = query.to_lowercase();
    let dangerous_keywords = [
        "insert",
        "update",
        "delete",
        "drop",
        "alter",
        "truncate",
        "create",
        "execute",
        "exec",
        "union",
        "into",
        "outfile",
        "load_file",
    ];

    for keyword in dangerous_keywords {
        if query.contains(&format!(" {} ", keyword))
            || query.starts_with(&format!("{} ", keyword))
            || query.ends_with(&format!(" {}", keyword))
            || query == keyword
        {
            return Err(ValidationError::InvalidValue(format!(
                "Query contains dangerous keyword: {}",
                keyword
            )));
        }
    }

    // Check for multiple statements
    if query.contains(';') {
        return Err(ValidationError::InvalidValue(
            "Query contains multiple statements".to_string(),
        ));
    }

    // Check for comment markers which could be used to bypass validation
    if query.contains("--") || query.contains("/*") || query.contains("*/") {
        return Err(ValidationError::InvalidValue(
            "Query contains comment markers".to_string(),
        ));
    }

    Ok(())
}

/// Represents the type of join between tables
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, PartialEq, Display)]
pub enum JoinType {
    #[serde(rename = "inner")]
    #[display(fmt = "INNER JOIN")]
    Inner,
    #[serde(rename = "left")]
    #[display(fmt = "LEFT JOIN")]
    Left,
    #[serde(rename = "right")]
    #[display(fmt = "RIGHT JOIN")]
    Right,
    #[serde(rename = "full")]
    #[display(fmt = "FULL JOIN")]
    Full,
    #[serde(rename = "cross")]
    #[display(fmt = "CROSS JOIN")]
    Cross,
    #[serde(rename = "anti")]
    #[display(fmt = "ANTI JOIN")]
    Anti,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, PartialEq, Display)]
pub enum FilterOperator {
    #[serde(rename = "=")]
    #[display(fmt = "=")]
    Equals,
    #[serde(rename = "!=")]
    #[display(fmt = "!=")]
    NotEquals,
    #[serde(rename = "<>")]
    #[display(fmt = "<>")]
    NotEquals2,
    #[serde(rename = ">")]
    #[display(fmt = ">")]
    GreaterThan,
    #[serde(rename = "<")]
    #[display(fmt = "<")]
    LessThan,
    #[serde(rename = ">=")]
    #[display(fmt = ">=")]
    GreaterThanOrEquals,
    #[serde(rename = "<=")]
    #[display(fmt = "<=")]
    LessThanOrEquals,
    #[serde(rename = "like")]
    #[display(fmt = "LIKE")]
    Like,
    #[serde(rename = "not like")]
    #[display(fmt = "NOT LIKE")]
    NotLike,
    #[serde(rename = "in")]
    #[display(fmt = "IN")]
    In,
    #[serde(rename = "not in")]
    #[display(fmt = "NOT IN")]
    NotIn,
    #[serde(rename = "is null")]
    #[display(fmt = "IS NULL")]
    IsNull,
    #[serde(rename = "is not null")]
    #[display(fmt = "IS NOT NULL")]
    IsNotNull,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, PartialEq, Display)]
pub enum AggregationType {
    #[serde(rename = "SUM")]
    #[display(fmt = "SUM")]
    Sum,
    #[serde(rename = "COUNT")]
    #[display(fmt = "COUNT")]
    Count,
    #[serde(rename = "AVG")]
    #[display(fmt = "AVG")]
    Avg,
    #[serde(rename = "MIN")]
    #[display(fmt = "MIN")]
    Min,
    #[serde(rename = "MAX")]
    #[display(fmt = "MAX")]
    Max,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, Display)]
pub enum TableName {
    #[serde(rename = "search_queries")]
    #[display(fmt = "search_queries")]
    SearchQueries,
    #[serde(rename = "rag_queries")]
    #[display(fmt = "rag_queries")]
    RagQueries,
    #[serde(rename = "recommendations")]
    #[display(fmt = "recommendations")]
    Recommendations,
    #[serde(rename = "events")]
    #[display(fmt = "events")]
    Events,
    #[serde(rename = "cluster_topics")]
    #[display(fmt = "cluster_topics")]
    ClusterTopics,
    #[serde(rename = "search_cluster_memberships")]
    #[display(fmt = "search_cluster_memberships")]
    SearchClusterMemberships,
    #[serde(rename = "topics")]
    #[display(fmt = "topics")]
    Topics,
    #[display(fmt = "{_0}")]
    #[serde(untagged)]
    Custom(String),
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, Display)]
pub enum Direction {
    #[serde(rename = "asc")]
    #[display(fmt = "ASC")]
    Asc,
    #[serde(rename = "desc")]
    #[display(fmt = "DESC")]
    Desc,
}

/// Represents a join condition between tables
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct JoinClause {
    pub table: TableName,
    pub join_type: Option<JoinType>,
    pub on_clause: String, // Ideally, this would be more structured to allow for better parameterization
}

impl JoinClause {
    pub fn validate(&self) -> Result<(), ValidationError> {
        check_for_injection(&self.on_clause)?;
        Ok(())
    }
}

/// Represents a SQL function or expression
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct Expression {
    pub expression: String,
    pub alias: Option<String>,
}

impl Expression {
    pub fn validate(&self) -> Result<(), ValidationError> {
        check_for_injection(&self.expression)?;
        if let Some(alias) = &self.alias {
            if !alias.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Err(ValidationError::InvalidExpression(format!(
                    "Invalid alias: {}",
                    alias
                )));
            }
        }

        Ok(())
    }
}

/// Represents a column with optional aggregation and alias
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct Column {
    pub name: String,
    pub alias: Option<String>,
    pub aggregation: Option<AggregationType>, // e.g., "SUM", "COUNT", "AVG", etc.
    pub distinct: Option<bool>,
}

impl Column {
    pub fn validate(&self) -> Result<(), ValidationError> {
        check_for_injection(&self.name)?;

        if !self
            .name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '.' || c == '*')
        {
            return Err(ValidationError::InvalidColumnName(self.name.clone()));
        }

        // Validate alias if present
        if let Some(alias) = &self.alias {
            check_for_injection(alias)?;

            if !alias.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Err(ValidationError::InvalidColumnName(format!(
                    "Invalid alias: {}",
                    alias
                )));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(untagged)]
pub enum FilterValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<FilterValue>),
}

impl fmt::Display for FilterValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FilterValue::String(s) => write!(f, "'{}'", s),
            FilterValue::Number(n) => write!(f, "{}", n),
            FilterValue::Boolean(b) => write!(f, "{}", b),
            FilterValue::Array(arr) => {
                write!(f, "[")?;
                for (i, value) in arr.iter().enumerate() {
                    write!(f, "{}", value)?;
                    if i < arr.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")
            }
        }
    }
}

/// Represents a query filter condition
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct FilterCondition {
    pub column: String,
    pub operator: FilterOperator, // "=", ">", "<", "LIKE", "IN", etc.
    pub value: FilterValue,
    pub and_filter: Option<Vec<FilterCondition>>,
    pub or_filter: Option<Vec<FilterCondition>>,
}

impl FilterCondition {
    pub fn validate(&self) -> Result<(), ValidationError> {
        check_for_injection(&self.column)?;
        if !self
            .column
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '.')
        {
            return Err(ValidationError::InvalidColumnName(self.column.clone()));
        }

        check_for_injection(&self.value.to_string())?;

        if let Some(and_conditions) = &self.and_filter {
            for condition in and_conditions {
                condition.validate()?;
            }
        }

        if let Some(or_conditions) = &self.or_filter {
            for condition in or_conditions {
                condition.validate()?;
            }
        }

        Ok(())
    }
}

/// Represents a GROUP BY clause
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct GroupBy {
    pub columns: Vec<String>,
    pub having: Option<String>,
}

impl GroupBy {
    pub fn validate(&self) -> Result<(), ValidationError> {
        for column in &self.columns {
            check_for_injection(column)?;
            if !column
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '.')
            {
                return Err(ValidationError::InvalidColumnName(column.clone()));
            }
        }

        if let Some(having) = &self.having {
            check_for_injection(having)?;
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct OrderBy {
    pub columns: Vec<String>,
    pub direction: Option<Direction>,
}

impl OrderBy {
    pub fn validate(&self) -> Result<(), ValidationError> {
        for column in &self.columns {
            check_for_injection(column)?;
            if !column
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '.')
            {
                return Err(ValidationError::InvalidColumnName(column.clone()));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct CommonTableExpression {
    pub alias: String,
    pub query: Box<ClickhouseQuery>,
}

impl CommonTableExpression {
    pub fn validate(&self) -> Result<(), ValidationError> {
        check_for_injection(&self.alias)?;
        self.query.validate()?;
        Ok(())
    }
}

/// Represents a complete ClickHouse query with parameters
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct ClickhouseQuery {
    /// Simple columns to select
    pub columns: Vec<Column>,

    /// Complex expressions to select
    pub expressions: Option<Vec<Expression>>,

    /// Main table to query from
    pub table: TableName,

    /// Tables to join with
    pub joins: Option<Vec<JoinClause>>,

    /// WHERE clause conditions
    pub filter_conditions: Option<Vec<FilterCondition>>,

    /// GROUP BY clause
    pub group_by: Option<GroupBy>,

    /// ORDER BY clause
    pub order_by: Option<OrderBy>,

    /// LIMIT clause
    pub limit: Option<u32>,

    /// OFFSET clause
    pub offset: Option<u32>,

    /// Common Table Expression (CTE) query
    pub cte_query: Option<CommonTableExpression>,
}

// No lifetime 'a needed, it will own its strings for identifiers.
pub struct SqlQuery {
    sql_part: String,
    identifier_args: Vec<String>, // Stores owned strings that will become Identifier arguments
}

impl Default for SqlQuery {
    fn default() -> Self {
        Self::new()
    }
}

impl SqlQuery {
    pub fn new() -> Self {
        Self {
            sql_part: String::new(),
            identifier_args: Vec::new(),
        }
    }

    // Add an argument that will be bound as an Identifier
    pub fn add_identifier_argument(&mut self, arg: String) -> &mut Self {
        self.sql_part.push_str(" ? "); // Placeholder for the identifier in SQL string
        self.identifier_args.push(arg); // Store the owned string
        self
    }

    pub fn push_str(&mut self, s: &str) -> &mut Self {
        self.sql_part.push_str(s);
        self
    }

    pub fn concat_query(&mut self, other: &SqlQuery) -> &mut Self {
        self.sql_part.push_str(&other.sql_part);
        self.identifier_args
            .extend(other.identifier_args.iter().cloned()); // Clone to take ownership
        self
    }

    pub fn remove_suffix(&mut self, suffix: &str) -> &mut Self {
        if self.sql_part.ends_with(suffix) {
            let new_len = self.sql_part.len() - suffix.len();
            self.sql_part.truncate(new_len);
        }
        self
    }

    // Renamed to avoid confusion with previous (String, Vec<Value>) tuple return
    pub fn into_parts(self) -> (String, Vec<String>) {
        (self.sql_part, self.identifier_args)
    }
}

impl ClickhouseQuery {
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Validate columns
        for column in &self.columns {
            column.validate()?;
        }

        // Validate expressions
        if let Some(expressions) = &self.expressions {
            for expression in expressions {
                expression.validate()?;
            }
        }

        // Validate joins
        if let Some(joins) = &self.joins {
            for join in joins {
                join.validate()?;
            }
        }

        // Validate filter conditions
        if let Some(conditions) = &self.filter_conditions {
            for condition in conditions {
                condition.validate()?;
            }
        }

        // Validate GROUP BY
        if let Some(group_by) = &self.group_by {
            group_by.validate()?;
        }

        // Validate ORDER BY
        if let Some(order_by) = &self.order_by {
            order_by.validate()?;
        }

        if let Some(limit) = &self.limit {
            if *limit == 0 {
                return Err(ValidationError::InvalidLimit(
                    "Limit cannot be 0".to_string(),
                ));
            }

            if *limit > 500 {
                return Err(ValidationError::InvalidLimit(
                    "Limit cannot be greater than 500".to_string(),
                ));
            }
        }

        if let Some(cte_query) = &self.cte_query {
            cte_query.validate()?;
        }

        Ok(())
    }

    /// Builds a parameterized SQL query string from the structure and returns the parameters
    pub fn to_parameterized_sql(&self) -> SqlQuery {
        let mut query = SqlQuery::new();

        if let Some(cte_query) = &self.cte_query {
            query.push_str("WITH ");
            query.add_identifier_argument(cte_query.alias.clone());
            query.push_str(" AS ( ");
            let cte_query = cte_query.query.to_parameterized_sql();
            query.concat_query(&cte_query);
            query.push_str(" ) ");
        }

        // SELECT clause
        query.push_str("SELECT ");

        let mut all_columns = Vec::new();

        // Add simple columns
        for column in &self.columns {
            let mut col_str = SqlQuery::new();

            if let Some(agg) = &column.aggregation {
                col_str.add_identifier_argument(agg.to_string());
                col_str.push_str("(");
                if let Some(distinct) = column.distinct {
                    if distinct {
                        col_str.push_str("DISTINCT ");
                    }
                }
                col_str.add_identifier_argument(column.name.clone());
                col_str.push_str(")");
            } else {
                col_str.add_identifier_argument(column.name.clone());
            }

            if let Some(alias) = &column.alias {
                col_str.push_str(" AS ");
                col_str.add_identifier_argument(alias.clone());
            }

            all_columns.push(col_str);
        }

        // Add expressions
        if let Some(expressions) = &self.expressions {
            for expr in expressions {
                let mut expr_str = SqlQuery::new();
                expr_str.push_str(&expr.expression);

                if let Some(alias) = &expr.alias {
                    expr_str.push_str(" AS ");
                    expr_str.add_identifier_argument(alias.clone());
                }

                all_columns.push(expr_str);
            }
        }

        // If no columns specified, use *
        if all_columns.is_empty() {
            query.push_str("*");
        } else {
            for column in all_columns {
                query.concat_query(&column);
                query.push_str(", ");
            }
            query.remove_suffix(", ");
        }

        let table_name = &self.table.to_string();
        // FROM clause - table name is an enum, so no need for parameterization
        query.push_str(" FROM ");
        query.add_identifier_argument(table_name.clone());

        // JOIN clauses
        if let Some(joins) = &self.joins {
            for join in joins {
                query.push_str(" ");
                if let Some(join_type) = &join.join_type {
                    query.push_str(&join_type.to_string());
                } else {
                    query.push_str("INNER JOIN");
                }
                query.push_str(" ");
                query.add_identifier_argument(join.table.to_string());

                // Get parameterized on clause
                query.push_str(" ON ");
                query.push_str(&join.on_clause);
            }
        }

        // WHERE clause
        if let Some(conditions) = &self.filter_conditions {
            if !conditions.is_empty() {
                query.push_str(" WHERE ");
                let conditions_sql = Self::build_parameterized_filter_conditions(conditions);
                query.concat_query(&conditions_sql);
            }
        }

        // GROUP BY clause - column names are identifiers, not values
        if let Some(group_by) = &self.group_by {
            if !group_by.columns.is_empty() {
                query.push_str(" GROUP BY ");
                for column in &group_by.columns {
                    query.add_identifier_argument(column.clone());
                    query.push_str(", ");
                }
                query.remove_suffix(", ");

                // HAVING clause with parameterization
                if let Some(having_clause) = &group_by.having {
                    query.push_str(" HAVING ");
                    query.add_identifier_argument(having_clause.clone());
                }
            }
        }

        // ORDER BY clause - column names are identifiers
        if let Some(order_by) = &self.order_by {
            if !order_by.columns.is_empty() {
                query.push_str(" ORDER BY ");
                for column in &order_by.columns {
                    query.add_identifier_argument(column.clone());
                    query.push_str(", ");
                }
                query.remove_suffix(", ");
                if let Some(direction) = &order_by.direction {
                    query.push_str(format!(" {}", direction).as_str());
                } else {
                    query.push_str(" DESC");
                }
            }
        }

        // LIMIT clause
        if let Some(limit) = self.limit {
            query.push_str(" LIMIT ");
            query.add_identifier_argument(limit.to_string());
        } else {
            query.push_str(" LIMIT 20");
        }

        // OFFSET clause
        if let Some(offset) = self.offset {
            query.push_str(" OFFSET ");
            query.add_identifier_argument(offset.to_string());
        }

        query
    }

    /// Recursively builds parameterized filter conditions
    fn build_parameterized_filter_conditions(conditions: &[FilterCondition]) -> SqlQuery {
        if conditions.is_empty() {
            return SqlQuery::new();
        }

        // If there's just one condition, no need for parentheses
        if conditions.len() == 1 {
            return Self::build_parameterized_single_filter_condition(&conditions[0]);
        }

        // Multiple conditions joined with AND
        let mut result = SqlQuery::new();
        result.push_str("(");

        for condition in conditions {
            let condition_str = Self::build_parameterized_single_filter_condition(condition);
            result.concat_query(&condition_str);
            result.push_str(" AND ");
        }
        result.remove_suffix(" AND ");
        result.push_str(")");

        result
    }

    /// Builds a parameterized single filter condition, including nested AND/OR conditions
    fn build_parameterized_single_filter_condition(condition: &FilterCondition) -> SqlQuery {
        let mut query = SqlQuery::new();

        // Add the base condition
        match condition.operator {
            FilterOperator::IsNull => {
                query.push_str(&condition.column);
                query.push_str(" IS NULL");
            }
            FilterOperator::IsNotNull => {
                query.push_str(&condition.column);
                query.push_str(" IS NOT NULL");
            }
            FilterOperator::In | FilterOperator::NotIn => {
                query.push_str(&condition.column);
                query.push_str(" ");
                query.push_str(&condition.operator.to_string());
                if let FilterValue::Array(arr) = &condition.value {
                    query.push_str(" ( ");
                    for value in arr {
                        query.push_str(&value.to_string());
                        query.push_str(", ");
                    }
                    query.remove_suffix(", ");
                    query.push_str(")");
                } else {
                    query.push_str(" ( ");
                    query.push_str(&condition.value.to_string());
                    query.push_str(" )");
                }
            }
            // For other operators (=, <>, >, <, >=, <=, LIKE, NOT LIKE)
            _ => {
                query.push_str(&condition.column);
                query.push_str(" ");
                query.push_str(&condition.operator.to_string());
                query.push_str(" ");
                query.push_str(&condition.value.to_string());
            }
        };

        // Add AND conditions if present
        if let Some(and_conditions) = &condition.and_filter {
            if !and_conditions.is_empty() {
                let and_cond = Self::build_parameterized_filter_conditions(and_conditions);
                query.push_str(" AND ");
                query.concat_query(&and_cond);
            }
        }

        // Add OR conditions if present
        if let Some(or_conditions) = &condition.or_filter {
            if !or_conditions.is_empty() {
                let or_cond = Self::build_parameterized_filter_conditions(or_conditions);
                query.push_str(" OR ");
                query.concat_query(&or_cond);
            }
        }

        if query.identifier_args.len() > 1
            && !(query.sql_part.starts_with('(') && query.sql_part.ends_with(')'))
        {
            let mut temp_sql = String::from("(");
            temp_sql.push_str(&query.sql_part);
            temp_sql.push(')');
            query.sql_part = temp_sql;
        }

        query
    }

    pub async fn execute(
        &self,
        dataset_id: &str,
        clickhouse_client: &clickhouse::Client,
    ) -> Result<serde_json::Value, ServiceError> {
        self.validate()
            .map_err(|e| ServiceError::BadRequest(e.to_string()))?;

        let mut tenant_query = self.clone();
        tenant_query
            .filter_conditions
            .as_mut()
            .unwrap_or(&mut vec![])
            .push(FilterCondition {
                column: "dataset_id".to_string(),
                operator: FilterOperator::Equals,
                value: FilterValue::String(dataset_id.to_string()),
                and_filter: None,
                or_filter: None,
            });

        let query = self.to_parameterized_sql();
        log::info!("Query: {}", query.sql_part);
        log::info!("Params: {:?}", query.identifier_args);

        let params = query
            .identifier_args
            .iter()
            .map(|s| Identifier(s))
            .collect::<Vec<_>>();

        let mut query = clickhouse_client.query(query.sql_part.as_str());

        for param in params {
            query = query.bind(param);
        }

        let mut response_lines = query
            .fetch_bytes("JSONEachRow")
            .map_err(|e| ServiceError::InternalServerError(e.to_string()))?
            .lines();
        let mut result = Vec::new();

        while let Some(line) = response_lines
            .next_line()
            .await
            .map_err(|e| ServiceError::InternalServerError(e.to_string()))?
        {
            let value: serde_json::Value = serde_json::from_str(&line)
                .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;
            result.push(value);
        }

        Ok(serde_json::Value::Array(result))
    }
}
