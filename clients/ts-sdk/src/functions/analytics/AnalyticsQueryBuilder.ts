import { AggregationType, AnalyticsQuery, Column, Direction, FilterCondition, JoinType, TableName } from "../../types.gen";

export class AnalyticsQueryBuilder {
  private query: AnalyticsQuery;

  constructor() {
    this.query = {
      columns: [],
      table: 'events'
    };
  }

  /**
   * Add a column to select
   */
  select(name: string, options: { alias?: string; aggregation?: AggregationType; distinct?: boolean } = {}): AnalyticsQueryBuilder {
    this.query.columns.push({
      name,
      alias: options.alias,
      aggregation: options.aggregation,
      distinct: options.distinct
    });
    return this;
  }

  /**
   * Add multiple columns to select
   */
  selectMultiple(columns: Column[]): AnalyticsQueryBuilder {
    this.query.columns.push(...columns);
    return this;
  }

  /**
   * Add an expression to select
   */
  selectExpression(expression: string, alias?: string): AnalyticsQueryBuilder {
    if (!this.query.expressions) {
      this.query.expressions = [];
    }
    this.query.expressions.push({ expression, alias });
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
   * Add a join clause
   */
  join(table: TableName, onClause: string, joinType: JoinType = 'inner'): AnalyticsQueryBuilder {
    if (!this.query.joins) {
      this.query.joins = [];
    }
    this.query.joins.push({
      table,
      join_type: joinType,
      on_clause: onClause
    });
    return this;
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
      and_filter: conditions.slice(1)
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
      or_filter: conditions.slice(1)
    };
  }

  /**
   * Add a GROUP BY clause
   */
  groupBy(columns: string[], having?: string): AnalyticsQueryBuilder {
    this.query.group_by = { columns, having };
    return this;
  }

  /**
   * Add an ORDER BY clause
   */
  orderBy(columns: string[], direction: Direction = 'desc'): AnalyticsQueryBuilder {
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
}