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

    #[display(fmt = "Invalid identifier: {_0}")]
    InvalidIdentifier(String),
}

fn validate_identifier(name: &str) -> Result<(), ValidationError> {
    // Only allow alphanumeric, underscore, and dot for qualified names
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '.' || c == '*')
    {
        return Err(ValidationError::InvalidIdentifier(name.to_string()));
    }

    // Additional checks for dots
    if name.starts_with('.') || name.ends_with('.') || name.contains("..") {
        return Err(ValidationError::InvalidIdentifier(format!(
            "Invalid identifier format: {}",
            name
        )));
    }

    // Check for SQL injection attempts in identifiers
    let name = name.to_lowercase();
    let dangerous_patterns = ["--", "/*", "*/", ";", "'", "\"", "\\"];
    for pattern in dangerous_patterns {
        if name.contains(pattern) {
            return Err(ValidationError::InvalidIdentifier(format!(
                "Identifier contains dangerous pattern: {}",
                pattern
            )));
        }
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
    #[serde(rename = "experiments")]
    #[display(fmt = "experiments")]
    Experiments,
    #[serde(rename = "experiment_user_assignments")]
    #[display(fmt = "experiment_user_assignments")]
    ExperimentUserAssignments,
    #[display(fmt = "{_0}")]
    #[serde(untagged)]
    Custom(String),
}

impl TableName {
    fn validate(&self) -> Result<(), ValidationError> {
        if let TableName::Custom(name) = self {
            validate_identifier(name)?;
        }
        Ok(())
    }
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

/// Structured join condition instead of raw SQL
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(tag = "type")]
pub enum JoinCondition {
    #[serde(rename = "column_equals")]
    ColumnEquals {
        left_column: String,
        right_column: String,
    },
    #[serde(rename = "using")]
    Using { columns: Vec<String> },
}

impl JoinCondition {
    pub fn validate(&self) -> Result<(), ValidationError> {
        match self {
            JoinCondition::ColumnEquals {
                left_column,
                right_column,
            } => {
                validate_identifier(left_column)?;
                validate_identifier(right_column)?;
            }
            JoinCondition::Using { columns } => {
                for column in columns {
                    validate_identifier(column)?;
                }
            }
        }
        Ok(())
    }
}

/// Represents a join between tables
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct JoinClause {
    pub table: TableName,
    pub join_type: Option<JoinType>,
    pub condition: JoinCondition,
}

impl JoinClause {
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.table.validate()?;
        self.condition.validate()?;
        Ok(())
    }
}

/// Structured expression type
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(tag = "type")]
pub enum ExpressionType {
    #[serde(rename = "column")]
    Column { name: String },
    #[serde(rename = "literal")]
    Literal { value: FilterValue },
    #[serde(rename = "function")]
    Function {
        name: String,
        args: Vec<ExpressionType>,
    },
    #[serde(rename = "raw")]
    Raw {
        // For backward compatibility - validated
        sql: String,
    },
}

/// Represents a SQL expression with optional alias
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct Expression {
    pub expression: ExpressionType,
    pub alias: Option<String>,
}

impl Expression {
    pub fn validate(&self) -> Result<(), ValidationError> {
        match &self.expression {
            ExpressionType::Column { name } => validate_identifier(name)?,
            ExpressionType::Function { name, args } => {
                // Whitelist of allowed functions
                let allowed_functions = [
                    "COUNT", "SUM", "AVG", "MIN", "MAX", "COALESCE", "CAST", "DATE", "toDate",
                ];
                if !allowed_functions.contains(&name.to_uppercase().as_str()) {
                    return Err(ValidationError::InvalidExpression(format!(
                        "Function '{}' is not allowed",
                        name
                    )));
                }
                for arg in args {
                    // Recursive validation
                    Expression {
                        expression: arg.clone(),
                        alias: None,
                    }
                    .validate()?;
                }
            }
            ExpressionType::Raw { sql } => {
                // Basic validation for raw SQL
                let lower = sql.to_lowercase();
                if lower.contains("drop") || lower.contains("delete") || lower.contains("insert") {
                    return Err(ValidationError::InvalidExpression(
                        "Expression contains dangerous keywords".to_string(),
                    ));
                }
            }
            ExpressionType::Literal { .. } => {} // Literals are safe
        }

        if let Some(alias) = &self.alias {
            validate_identifier(alias)?;
        }
        Ok(())
    }
}

/// Represents a column with optional aggregation and alias
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct Column {
    pub name: String,
    pub alias: Option<String>,
    pub aggregation: Option<AggregationType>,
    pub distinct: Option<bool>,
}

impl Column {
    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_identifier(&self.name)?;

        if let Some(alias) = &self.alias {
            validate_identifier(alias)?;
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
            FilterValue::String(s) => write!(f, "'{}'", s.replace('\'', "''")),
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
    pub operator: FilterOperator,
    pub value: FilterValue,
    pub and_filter: Option<Vec<FilterCondition>>,
    pub or_filter: Option<Vec<FilterCondition>>,
}

impl FilterCondition {
    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_identifier(&self.column)?;

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

/// Structured HAVING condition
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(tag = "type")]
pub enum HavingCondition {
    #[serde(rename = "aggregate")]
    Aggregate {
        function: AggregationType,
        column: String,
        operator: FilterOperator,
        value: FilterValue,
    },
    #[serde(rename = "and")]
    And { conditions: Vec<HavingCondition> },
    #[serde(rename = "or")]
    Or { conditions: Vec<HavingCondition> },
}

impl HavingCondition {
    pub fn validate(&self) -> Result<(), ValidationError> {
        match self {
            HavingCondition::Aggregate { column, .. } => {
                validate_identifier(column)?;
            }
            HavingCondition::And { conditions } | HavingCondition::Or { conditions } => {
                for condition in conditions {
                    condition.validate()?;
                }
            }
        }
        Ok(())
    }
}

/// Represents a GROUP BY clause
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct GroupBy {
    pub columns: Vec<String>,
    pub having: Option<HavingCondition>,
}

impl GroupBy {
    pub fn validate(&self) -> Result<(), ValidationError> {
        for column in &self.columns {
            validate_identifier(column)?;
        }

        if let Some(having) = &self.having {
            having.validate()?;
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
            validate_identifier(column)?;
        }

        Ok(())
    }
}

/// Represents a complete ClickHouse query with parameters
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct SubQuery {
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
}

impl From<&SubQuery> for AnalyticsQuery {
    fn from(sub_query: &SubQuery) -> Self {
        AnalyticsQuery {
            columns: sub_query.columns.clone(),
            expressions: sub_query.expressions.clone(),
            table: sub_query.table.clone(),
            joins: sub_query.joins.clone(),
            filter_conditions: sub_query.filter_conditions.clone(),
            group_by: sub_query.group_by.clone(),
            order_by: sub_query.order_by.clone(),
            limit: sub_query.limit,
            offset: sub_query.offset,
            cte_query: None,
        }
    }
}

impl SubQuery {
    pub fn validate(&self) -> Result<(), ValidationError> {
        let clickhouse_query = AnalyticsQuery::from(self);
        clickhouse_query.validate()
    }

    pub fn to_parameterized_sql(&self) -> SqlQuery {
        let clickhouse_query = AnalyticsQuery::from(self);
        clickhouse_query.to_parameterized_sql()
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct CommonTableExpression {
    pub alias: String,
    pub query: Box<SubQuery>,
}

impl CommonTableExpression {
    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_identifier(&self.alias)?;
        self.query.validate()?;
        Ok(())
    }
}

/// Represents a complete Analytics query with parameters
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct AnalyticsQuery {
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

/// Represents a parameter that will be bound to the query
#[derive(Debug, Clone)]
pub enum QueryParameter {
    /// Regular value parameter (properly escaped by driver)
    Value(FilterValue),
    /// Identifier parameter (for table/column names)
    Identifier(String),
}

// Enhanced SqlQuery struct with proper parameterization
pub struct SqlQuery {
    sql_part: String,
    parameters: Vec<QueryParameter>,
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
            parameters: Vec::new(),
        }
    }

    // Add an identifier (table/column name)
    pub fn add_identifier(&mut self, identifier: String) -> &mut Self {
        self.sql_part.push_str(" ? ");
        self.parameters.push(QueryParameter::Identifier(identifier));
        self
    }

    // Add a value parameter
    pub fn add_value(&mut self, value: FilterValue) -> &mut Self {
        self.sql_part.push_str(" ? ");
        self.parameters.push(QueryParameter::Value(value));
        self
    }

    // Add raw SQL (only for keywords, operators, etc.)
    pub fn push_str(&mut self, s: &str) -> &mut Self {
        self.sql_part.push_str(s);
        self
    }

    pub fn concat_query(&mut self, other: &SqlQuery) -> &mut Self {
        self.sql_part.push_str(&other.sql_part);
        self.parameters.extend(other.parameters.iter().cloned());
        self
    }

    pub fn remove_suffix(&mut self, suffix: &str) -> &mut Self {
        if self.sql_part.ends_with(suffix) {
            let new_len = self.sql_part.len() - suffix.len();
            self.sql_part.truncate(new_len);
        }
        self
    }

    // Get SQL and parameters separately
    pub fn into_parts(self) -> (String, Vec<QueryParameter>) {
        (self.sql_part, self.parameters)
    }
}

impl AnalyticsQuery {
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Validate table
        self.table.validate()?;

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

            if *limit > 10000 {
                return Err(ValidationError::InvalidLimit(
                    "Limit cannot be greater than 10000".to_string(),
                ));
            }
        }

        if let Some(cte_query) = &self.cte_query {
            cte_query.validate()?;
        }

        Ok(())
    }

    /// Builds a parameterized SQL query string from the structure
    pub fn to_parameterized_sql(&self) -> SqlQuery {
        let mut query = SqlQuery::new();

        if let Some(cte_query) = &self.cte_query {
            query.push_str("WITH ");
            query.add_identifier(cte_query.alias.clone());
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
                col_str.push_str(agg.to_string().as_str());
                col_str.push_str("(");
                if let Some(distinct) = column.distinct {
                    if distinct {
                        col_str.push_str("DISTINCT ");
                    }
                }
                col_str.push_str(&column.name);
                col_str.push_str(")");
            } else {
                col_str.push_str(&column.name);
            }

            if let Some(alias) = &column.alias {
                col_str.push_str(" AS ");
                col_str.push_str(alias);
            }

            all_columns.push(col_str);
        }

        // Add expressions
        if let Some(expressions) = &self.expressions {
            for expr in expressions {
                let mut expr_str = SqlQuery::new();
                self.build_expression(&expr.expression, &mut expr_str);

                if let Some(alias) = &expr.alias {
                    expr_str.push_str(" AS ");
                    expr_str.push_str(alias);
                }

                all_columns.push(expr_str);
            }
        }

        // If no columns specified, use *
        if all_columns.is_empty() {
            query.push_str("*");
        } else {
            for (i, column) in all_columns.iter().enumerate() {
                if i > 0 {
                    query.push_str(", ");
                }
                query.concat_query(column);
            }
        }

        // FROM clause
        query.push_str(" FROM ");
        query.add_identifier(self.table.to_string());

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
                query.add_identifier(join.table.to_string());

                match &join.condition {
                    JoinCondition::ColumnEquals {
                        left_column,
                        right_column,
                    } => {
                        query.push_str(" ON ");
                        query.push_str(left_column);
                        query.push_str(" = ");
                        query.push_str(right_column);
                    }
                    JoinCondition::Using { columns } => {
                        query.push_str(" USING (");
                        for (i, column) in columns.iter().enumerate() {
                            if i > 0 {
                                query.push_str(", ");
                            }
                            query.push_str(column);
                        }
                        query.push_str(")");
                    }
                }
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

        // GROUP BY clause
        if let Some(group_by) = &self.group_by {
            if !group_by.columns.is_empty() {
                query.push_str(" GROUP BY ");
                for (i, column) in group_by.columns.iter().enumerate() {
                    if i > 0 {
                        query.push_str(", ");
                    }
                    query.push_str(column);
                }

                // HAVING clause
                if let Some(having) = &group_by.having {
                    query.push_str(" HAVING ");
                    self.build_having_condition(having, &mut query);
                }
            }
        }

        // ORDER BY clause
        if let Some(order_by) = &self.order_by {
            if !order_by.columns.is_empty() {
                query.push_str(" ORDER BY ");
                for (i, column) in order_by.columns.iter().enumerate() {
                    if i > 0 {
                        query.push_str(", ");
                    }
                    query.push_str(column);
                }
                if let Some(direction) = &order_by.direction {
                    query.push_str(format!(" {}", direction).as_str());
                } else {
                    query.push_str(" DESC");
                }
            }
        }

        // LIMIT clause - use value parameter
        if let Some(limit) = self.limit {
            query.push_str(" LIMIT ");
            query.add_value(FilterValue::Number(limit as f64));
        } else {
            query.push_str(" LIMIT 20");
        }

        // OFFSET clause - use value parameter
        if let Some(offset) = self.offset {
            query.push_str(" OFFSET ");
            query.add_value(FilterValue::Number(offset as f64));
        }

        query
    }

    #[allow(clippy::only_used_in_recursion)]
    fn build_expression(&self, expr: &ExpressionType, query: &mut SqlQuery) {
        match expr {
            ExpressionType::Column { name } => {
                query.push_str(name);
            }
            ExpressionType::Literal { value } => {
                query.parameters.push(QueryParameter::Value(value.clone()));
                query.push_str("?");
            }
            ExpressionType::Function { name, args } => {
                query.push_str(name);
                query.push_str("(");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        query.push_str(", ");
                    }
                    self.build_expression(arg, query);
                }
                query.push_str(")");
            }
            ExpressionType::Raw { sql } => {
                // For backward compatibility
                query.push_str(sql);
            }
        }
    }

    #[allow(clippy::only_used_in_recursion)]
    fn build_having_condition(&self, condition: &HavingCondition, query: &mut SqlQuery) {
        match condition {
            HavingCondition::Aggregate {
                function,
                column,
                operator,
                value,
            } => {
                query.push_str(&function.to_string());
                query.push_str("(");
                query.push_str(column);
                query.push_str(") ");
                query.push_str(&operator.to_string());
                query.push_str(" ");
                query.parameters.push(QueryParameter::Value(value.clone()));
                query.push_str("?");
            }
            HavingCondition::And { conditions } => {
                query.push_str("(");
                for (i, cond) in conditions.iter().enumerate() {
                    if i > 0 {
                        query.push_str(" AND ");
                    }
                    self.build_having_condition(cond, query);
                }
                query.push_str(")");
            }
            HavingCondition::Or { conditions } => {
                query.push_str("(");
                for (i, cond) in conditions.iter().enumerate() {
                    if i > 0 {
                        query.push_str(" OR ");
                    }
                    self.build_having_condition(cond, query);
                }
                query.push_str(")");
            }
        }
    }

    fn build_simple_condition_part(condition: &FilterCondition) -> SqlQuery {
        let mut query = SqlQuery::new();
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
                query.push_str(" (");
                if let FilterValue::Array(arr) = &condition.value {
                    for (i, value) in arr.iter().enumerate() {
                        if i > 0 {
                            query.push_str(", ");
                        }
                        query.add_value(value.clone());
                    }
                } else {
                    query.add_value(condition.value.clone());
                }
                query.push_str(")");
            }
            _ => {
                query.push_str(&condition.column);
                query.push_str(" ");
                query.push_str(&condition.operator.to_string());
                query.add_value(condition.value.clone());
            }
        };
        query
    }

    fn build_parameterized_filter_conditions(conditions: &[FilterCondition]) -> SqlQuery {
        if conditions.is_empty() {
            return SqlQuery::new();
        }

        if conditions.len() == 1 {
            return Self::build_parameterized_single_filter_condition(&conditions[0]);
        }

        let mut result = SqlQuery::new();
        result.push_str("(");

        for (i, condition) in conditions.iter().enumerate() {
            if i > 0 {
                result.push_str(" AND ");
            }
            let condition_str = Self::build_parameterized_single_filter_condition(condition);
            result.concat_query(&condition_str);
        }
        result.push_str(")");

        result
    }

    fn build_parameterized_single_filter_condition(condition: &FilterCondition) -> SqlQuery {
        let mut final_query = SqlQuery::new();

        let base_part_query = Self::build_simple_condition_part(condition);
        let mut or_group_query = SqlQuery::new();

        if let Some(or_conditions) = &condition.or_filter {
            if !or_conditions.is_empty() {
                or_group_query.push_str("(");
                or_group_query.concat_query(&base_part_query);

                for or_sub_condition in or_conditions {
                    or_group_query.push_str(" OR ");
                    let sub_cond_query =
                        Self::build_parameterized_single_filter_condition(or_sub_condition);
                    or_group_query.concat_query(&sub_cond_query);
                }
                or_group_query.push_str(")");
            } else {
                or_group_query.concat_query(&base_part_query);
            }
        } else {
            or_group_query.concat_query(&base_part_query);
        }

        final_query.concat_query(&or_group_query);

        if let Some(and_conditions) = &condition.and_filter {
            if !and_conditions.is_empty() {
                let and_block_query = Self::build_parameterized_filter_conditions(and_conditions);

                if !final_query.sql_part.is_empty() && !and_block_query.sql_part.is_empty() {
                    final_query.push_str(" AND ");
                }
                final_query.concat_query(&and_block_query);
            }
        }

        final_query
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

        let query = tenant_query.to_parameterized_sql();
        let (sql, params) = query.into_parts();

        let mut ch_query = clickhouse_client.query(&sql);

        // Set query execution limits
        ch_query = ch_query.with_option("max_execution_time", "30".to_string());
        ch_query = ch_query.with_option("readonly", "2");
        ch_query = ch_query.with_option("allow_ddl", "0");

        for param in params {
            match param {
                QueryParameter::Value(value) => match value {
                    FilterValue::String(s) => ch_query = ch_query.bind(s),
                    FilterValue::Number(n) => ch_query = ch_query.bind(n),
                    FilterValue::Boolean(b) => ch_query = ch_query.bind(b),
                    FilterValue::Array(_) => {}
                },
                QueryParameter::Identifier(id) => {
                    ch_query = ch_query.bind(Identifier(&id));
                }
            }
        }

        let mut response_lines = ch_query
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
