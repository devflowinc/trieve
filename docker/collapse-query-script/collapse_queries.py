import clickhouse_connect
import os
import dotenv
import uuid
from typing import Optional
import datetime


def get_search_queries(
    client, dataset_id: uuid.UUID, limit=5000, offset=Optional[datetime.datetime]
):
    query = """
        SELECT id, query, top_score, created_at, search_type, request_params, latency, results, query_vector, is_duplicate, query_rating, dataset_id
        FROM default.search_queries 
        WHERE dataset_id = '{}' AND is_duplicate = 0 AND search_type != 'rag'
        ORDER BY created_at, length(query)
        LIMIT {}
        """.format(
        str(dataset_id), limit
    )
    if offset is not None:
        query = """
        SELECT id, query, top_score, created_at, search_type, request_params, latency, results, query_vector, is_duplicate, query_rating, dataset_id
        FROM default.search_queries 
        WHERE dataset_id = '{}'
            AND created_at >= '{}' AND search_type != 'rag'
        ORDER BY created_at, length(query)
        LIMIT {}
        """.format(
            str(dataset_id),
            datetime.datetime.utcfromtimestamp(offset).isoformat(),
            limit,
        )
    vector_result = client.query(query, query_formats={"Date*": "int"})
    return vector_result.result_rows


def get_datasets(client):
    query = "SELECT DISTINCT dataset_id FROM default.search_queries"
    dataset_result = client.query(query)
    return dataset_result.result_rows


def get_dataset_last_collapsed(client, dataset_id: uuid.UUID):
    query = """
        SELECT last_collapsed
        FROM default.last_collapsed_dataset
        WHERE dataset_id = '{}'
    """.format(
        str(dataset_id)
    )
    dataset_result = client.query(query, query_formats={"Date*": "int"})
    row = dataset_result.result_rows
    return row[0][0] if row else None


def set_dataset_last_collapsed(
    client, dataset_id: uuid.UUID, last_collapsed: datetime.datetime
):
    client.insert(
        "last_collapsed_dataset",
        [
            [
                uuid.uuid4(),
                last_collapsed,
                dataset_id,
                datetime.datetime.now(),
            ]
        ],
        column_names=[
            "id",
            "last_collapsed",
            "dataset_id",
            "created_at",
        ],
    )


def collapse_queries(rows, look_range=10, time_window=10):
    rows_to_be_deleted = []
    sorted_rows = sorted(rows, key=lambda x: x[3])  # Sort by timestamp

    for i, current_row in enumerate(sorted_rows):
        current_query = current_row[1].strip().lower()
        is_duplicate = False
        longest_query = current_query
        longest_row = current_row

        # Look behind
        start = max(0, i - look_range)
        for j in range(start, i):
            prev_row = sorted_rows[j]
            prev_query = prev_row[1].strip().lower()
            time_difference = current_row[3] - prev_row[3]

            if time_difference > time_window:
                continue

            if current_query.startswith(prev_query) or prev_query.startswith(
                current_query
            ):
                is_duplicate = True
                if len(prev_query) > len(longest_query):
                    longest_query = prev_query
                    longest_row = prev_row

        # Look ahead
        end = min(len(sorted_rows), i + look_range + 1)
        for j in range(i + 1, end):
            next_row = sorted_rows[j]
            next_query = next_row[1].strip().lower()
            time_difference = next_row[3] - current_row[3]

            if time_difference > time_window:
                break

            if current_query.startswith(next_query) or next_query.startswith(
                current_query
            ):
                is_duplicate = True
                if len(next_query) > len(longest_query):
                    longest_query = next_query
                    longest_row = next_row

        if is_duplicate:
            if longest_row != current_row:
                rows_to_be_deleted.append(current_row)
            else:
                # If current row is the longest, mark others for deletion
                for j in range(start, end):
                    if j != i:
                        other_row = sorted_rows[j]
                        other_query = other_row[1].strip().lower()
                        if current_query.startswith(
                            other_query
                        ) or other_query.startswith(current_query):
                            rows_to_be_deleted.append(other_row)

    return rows_to_be_deleted


def insert_duplicate_rows(client, rows):
    if rows:
        duplicate_rows = [list(row) for row in rows]
        for row in duplicate_rows:
            row[9] = 1  # Set is_duplicate to 1
        client.insert(
            "search_queries",
            duplicate_rows,
            column_names=[
                "id",
                "query",
                "top_score",
                "created_at",
                "search_type",
                "request_params",
                "latency",
                "results",
                "query_vector",
                "is_duplicate",
                "query_rating",
                "dataset_id",
            ],
        )


def main():
    dotenv.load_dotenv()
    client = clickhouse_connect.get_client(dsn=os.getenv("CLICKHOUSE_DSN"))

    try:
        datasets = get_datasets(client)

        for dataset in datasets:
            dataset_id = dataset[0]
            last_collapsed = get_dataset_last_collapsed(client, dataset_id)

            print("Collapsing dataset", dataset_id, "from", last_collapsed)

            num_duplicates = 0
            rows = get_search_queries(client, dataset_id, 5000, last_collapsed)

            while rows:
                last_collapsed = rows[-1][3]

                duplicates = collapse_queries(rows)
                num_duplicates += len(duplicates)

                insert_duplicate_rows(client, duplicates)

                new_rows = get_search_queries(client, dataset_id, 5000, last_collapsed)
                if new_rows and new_rows[-1][0] == rows[-1][0]:
                    break
                rows = new_rows

            if last_collapsed:
                set_dataset_last_collapsed(client, dataset_id, last_collapsed)

            print(f"Processed dataset {dataset_id}, marked {num_duplicates} duplicates")
            print()

    except Exception as e:
        print(f"Error: {e}")

    finally:
        # Optionally, you can force a merge to ensure all duplicates are removed
        client.command("OPTIMIZE TABLE default.search_queries FINAL")
        client.command("OPTIMIZE TABLE default.last_collapsed_dataset FINAL")


if __name__ == "__main__":
    main()
