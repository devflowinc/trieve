# Search Query Collapse Script

This script is designed to optimize search query analytics by collapsing similar queries in a ClickHouse database. It addresses the issue of storing redundant partial queries (e.g., "a", "ap", "app", "apple") which can skew analytics results.

## Purpose

The main purpose of this script is to:
1. Identify and remove partial queries that are prefixes of longer, more complete queries.
2. Process queries across multiple datasets stored in ClickHouse.
3. Keep track of the last processed timestamp for each dataset to allow for incremental updates.

## How it works

1. The script connects to the ClickHouse database using the provided DSN.
2. It retrieves a list of all datasets from the `search_queries` table.
3. For each dataset:
   - It fetches the timestamp of the last collapse operation from the `last_collapsed_dataset` table.
   - It retrieves search queries in batches of 5000, starting from the last collapsed timestamp.
   - The `collapse_queries` function identifies queries that are prefixes of longer queries.
   - Identified partial queries are deleted from the database.
   - The process continues until all queries in the dataset are processed or no new queries are found.
   - The last processed timestamp is updated in the `last_collapsed_dataset` table.

## Main Functions

- `get_search_queries`: Retrieves search queries for a specific dataset.
- `get_datasets`: Gets a list of all dataset IDs.
- `get_dataset_last_collapsed`: Retrieves the timestamp of the last collapse operation for a dataset.
- `set_dataset_last_collapsed`: Updates the last collapse timestamp for a dataset.
- `collapse_queries`: Identifies partial queries that should be removed.
- `delete_queries`: Removes identified partial queries from the database.

The script will process all datasets, collapse queries, and provide output on the number of deleted rows for each dataset.
