import {
  AggregationType,
  AnalyticsQuery,
  Direction,
  FilterCondition,
  JoinType,
  TableName,
  FilterOperator,
  JoinCondition,
  ExpressionType,
  Expression,
  HavingCondition,
  JoinClause,
  GroupBy,
  Column,
  FilterValue,
} from "../../types.gen";

export class AnalyticsQueryBuilder {
  private query: AnalyticsQuery;

  constructor() {
    this.query = {
      columns: [],
      table: "events",
    };
  }

  /**
   * Add a column to select
   */
  select(
    name: string,
    options: {
      alias?: string;
      aggregation?: AggregationType;
      distinct?: boolean;
    } = {},
  ): AnalyticsQueryBuilder {
    this.query.columns.push({
      name,
      alias: options.alias,
      aggregation: options.aggregation,
      distinct: options.distinct,
    });
    return this;
  }

  /**
   * Add a structured expression to select
   */
  selectExpression(
    expression: ExpressionType,
    alias?: string,
  ): AnalyticsQueryBuilder {
    if (!this.query.expressions) {
      this.query.expressions = [];
    }

    this.query.expressions.push({
      expression,
      alias,
    });
    return this;
  }

  /**
   * Set the table to query from
   */
  from(table: TableName): AnalyticsQueryBuilder {
    this.query.table = table;
    return this;
  }

  /**
   * Add a structured join clause
   */
  join(
    table: TableName,
    condition: JoinCondition,
    options: {
      type?: JoinType;
    } = {
      type: "inner",
    },
  ): AnalyticsQueryBuilder {
    if (!this.query.joins) {
      this.query.joins = [];
    }

    this.query.joins.push({
      table,
      join_type: options.type,
      condition,
    });
    return this;
  }

  /**
   * Add a join with column equality condition
   */
  joinOn(
    table: TableName,
    leftColumn: string,
    rightColumn: string,
    type: JoinType = "inner",
  ): AnalyticsQueryBuilder {
    return this.join(
      table,
      {
        type: "column_equals",
        left_column: leftColumn,
        right_column: rightColumn,
      },
      { type },
    );
  }

  /**
   * Add a join with USING clause
   */
  joinUsing(
    table: TableName,
    columns: string[],
    type: JoinType = "inner",
  ): AnalyticsQueryBuilder {
    return this.join(
      table,
      {
        type: "using",
        columns,
      },
      { type },
    );
  }

  /**
   * Add a complex filter condition
   */
  where(condition: FilterCondition): AnalyticsQueryBuilder {
    if (!this.query.filter_conditions) {
      this.query.filter_conditions = [];
    }
    this.query.filter_conditions.push(condition);
    return this;
  }

  /**
   * Create a nested 'AND' filter condition
   */
  static and(conditions: FilterCondition[]): FilterCondition {
    return {
      column: conditions[0].column,
      operator: conditions[0].operator,
      value: conditions[0].value,
      and_filter: conditions.slice(1),
    };
  }

  /**
   * Create a nested 'OR' filter condition
   */
  static or(conditions: FilterCondition[]): FilterCondition {
    return {
      column: conditions[0].column,
      operator: conditions[0].operator,
      value: conditions[0].value,
      or_filter: conditions.slice(1),
    };
  }

  /**
   * Add a GROUP BY clause with optional structured HAVING condition
   */
  groupBy(
    columns: string[],
    having?: HavingCondition | string,
  ): AnalyticsQueryBuilder {
    if (typeof having === "string") {
      throw new Error(
        "String HAVING clauses are no longer supported. Use structured having conditions.",
      );
    }

    this.query.group_by = {
      columns,
      having,
    };
    return this;
  }

  /**
   * Helper to create an aggregate HAVING condition
   */
  static having(
    func: AggregationType,
    column: string,
    operator: FilterOperator,
    value: FilterValue,
  ): HavingCondition {
    return {
      type: "aggregate",
      function: func,
      column,
      operator,
      value,
    };
  }

  /**
   * Helper to create an AND HAVING condition
   */
  static havingAnd(...conditions: HavingCondition[]): HavingCondition {
    return {
      type: "and",
      conditions,
    };
  }

  /**
   * Helper to create an OR HAVING condition
   */
  static havingOr(...conditions: HavingCondition[]): HavingCondition {
    return {
      type: "or",
      conditions,
    };
  }

  /**
   * Add an ORDER BY clause
   */
  orderBy(
    columns: string[],
    direction: Direction = "desc",
  ): AnalyticsQueryBuilder {
    this.query.order_by = { columns, direction };
    return this;
  }

  /**
   * Set a LIMIT clause
   */
  limit(limit: number): AnalyticsQueryBuilder {
    this.query.limit = limit;
    return this;
  }

  /**
   * Set an OFFSET clause
   */
  offset(offset: number): AnalyticsQueryBuilder {
    this.query.offset = offset;
    return this;
  }

  /**
   * Add a Common Table Expression (CTE)
   */
  withCte(alias: string, query: AnalyticsQuery): AnalyticsQueryBuilder {
    this.query.cte_query = { alias, query };
    return this;
  }

  /**
   * Build the final query object
   */
  build(): AnalyticsQuery {
    return this.query;
  }

  /**
   * Helper to create a function expression
   */
  static func(name: string, ...args: ExpressionType[]): ExpressionType {
    return {
      type: "function",
      name,
      args,
    };
  }

  /**
   * Helper to create a column expression
   */
  static col(name: string): ExpressionType {
    return {
      type: "column",
      name,
    };
  }

  /**
   * Helper to create a literal expression
   */
  static lit(value: FilterValue): ExpressionType {
    return {
      type: "literal",
      value,
    };
  }
}

// Export helper types
export type {
  FilterCondition,
  JoinType,
  TableName,
  AggregationType,
  Direction,
  FilterOperator,
  FilterValue,
  JoinCondition,
  ExpressionType,
  Expression,
  HavingCondition,
  JoinClause,
  GroupBy,
  Column,
};
