# Search Query Collapse Script

This script optimizes search query analytics by collapsing similar queries in a ClickHouse database. It addresses the issue of storing redundant partial queries (e.g., "a", "ap", "app", "apple") which can skew analytics results, while also considering the timing of these queries.

## Purpose

The main purpose of this script is to:
1. Identify and remove partial queries that are prefixes of longer, more complete queries, but only if they occur within a 10-second window of each other.
2. Process queries across multiple datasets stored in ClickHouse.
3. Keep track of the last processed timestamp for each dataset to allow for incremental updates.

## How it works

1. The script connects to the ClickHouse database using the provided DSN.
2. It retrieves a list of all datasets from the `search_queries` table.
3. For each dataset:
   - It fetches the timestamp of the last collapse operation from the `last_collapsed_dataset` table.
   - It retrieves search queries in batches of 5000, starting from the last collapsed timestamp.
   - The `collapse_queries` function identifies queries that are prefixes of longer queries and occur within 10 seconds of each other.
   - Identified partial queries are deleted from the database.
   - The process continues until all queries in the dataset are processed or no new queries are found.
   - The last processed timestamp is updated in the `last_collapsed_dataset` table.

## Main Functions

- `get_search_queries`: Retrieves search queries for a specific dataset, converting timestamps to datetime objects.
- `get_datasets`: Gets a list of all dataset IDs.
- `get_dataset_last_collapsed`: Retrieves the timestamp of the last collapse operation for a dataset.
- `set_dataset_last_collapsed`: Updates the last collapse timestamp for a dataset.
- `collapse_queries`: Identifies partial queries that should be removed, considering a 10-second time window.
- `delete_queries`: Removes identified partial queries from the database.

## Query Collapse Logic

The script now only collapses queries that meet the following criteria:
1. A query is a prefix of a subsequent query.
2. The subsequent query occurs within 10 seconds of the prefix query.

