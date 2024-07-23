import clickhouse_connect.driver
import clickhouse_connect
import os
import dotenv
import uuid
from typing import Optional
import datetime


def get_search_queries(
    client: clickhouse_connect.driver.client.Client,
    dataset_id: uuid.UUID,
    limit=5000,
    offset=Optional[uuid.UUID],
):
    query = """
        SELECT id, query, top_score, created_at
        FROM default.search_queries 
        WHERE dataset_id = '{}' AND is_duplicate = 0
        ORDER BY created_at, length(query)
        LIMIT {}
        """.format(
        str(dataset_id),
        limit,
    )
    if offset is not None:
        query = """
        SELECT id, query, top_score, created_at
        FROM default.search_queries 
        WHERE dataset_id = '{}'
            AND created_at >= '{}' AND is_duplicate = 0
        ORDER BY created_at, length(query)
        LIMIT {}
        """.format(
            str(dataset_id),
            str(offset),
            limit,
        )
    vector_result = client.query(query)
    rows = vector_result.result_rows
    return rows


def get_datasets(client: clickhouse_connect.driver.client.Client):
    query = """
        SELECT DISTINCT dataset_id
        FROM default.search_queries
        """
    dataset_result = client.query(query)
    rows = dataset_result.result_rows
    return rows


def get_dataset_last_collapsed(
    client: clickhouse_connect.driver.client.Client, dataset_id: uuid.UUID
):
    query = """
        SELECT last_collapsed
        FROM default.last_collapsed_dataset
        WHERE dataset_id = '{}'
    """.format(
        str(dataset_id)
    )
    dataset_result = client.query(query, query_formats={"DateTime": "int"})
    row = dataset_result.result_rows
    if len(row) == 1:
        return datetime.datetime.fromtimestamp(row[0][0])
    return None


def delete_dataset_last_collapsed(
    client: clickhouse_connect.driver.client.Client,
    dataset_id: uuid.UUID,
):
    query = """
        DELETE FROM default.last_collapsed_dataset
        WHERE dataset_id = '{}'
    """.format(
        str(dataset_id)
    )
    client.command(query)


def set_dataset_last_collapsed(
    client: clickhouse_connect.driver.client.Client,
    dataset_id: uuid.UUID,
    last_collapsed: datetime.datetime,
):
    delete_dataset_last_collapsed(client, dataset_id)

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


def collapse_queries(rows):
    rows_to_be_deleted = []
    cur_row = None
    for row in rows:
        if cur_row is None:
            cur_row = row
        elif row[1].startswith(cur_row[1]):
            # Check if the current row's timestamp is within 10 seconds of the previous row
            time_difference = (row[3] - cur_row[3]).total_seconds()
            if time_difference <= 10:
                rows_to_be_deleted.append(cur_row)
                cur_row = row
            else:
                cur_row = row
        else:
            cur_row = row
    return rows_to_be_deleted


def delete_queries(client: clickhouse_connect.driver.client.Client, rows):
    for row in rows:
        query = """
        ALTER TABLE default.search_queries
        UPDATE is_duplicate = 1
        WHERE id = '{}'
        """.format(
            str(row[0])
        )
        client.command(query)


def main():
    dotenv.load_dotenv()

    client = clickhouse_connect.get_client(dsn=os.getenv("CLICKHOUSE_DSN"))

    try:

        datasets = get_datasets(client)

        last_collapsed: Optional[str] = None

        for dataset in datasets:

            dataset_id = dataset[0]

            last_collapsed = get_dataset_last_collapsed(client, dataset_id)

            print("Collapsing dataset ", dataset_id, "from ", last_collapsed)

            num_deleted = 0

            rows = get_search_queries(client, dataset_id, 5000, last_collapsed)

            while len(rows) > 0:
                # offset is timestamp
                last_collapsed = rows[-1][3]

                to_be_deleted = collapse_queries(rows)

                num_deleted += len(to_be_deleted)

                delete_queries(client, to_be_deleted)

                new_rows = get_search_queries(client, dataset_id, 5000, last_collapsed)
                if len(new_rows) > 0 and new_rows[-1][0] == rows[-1][0]:
                    break

            if last_collapsed is not None:
                set_dataset_last_collapsed(client, dataset_id, last_collapsed)

            print(f"Processed dataset {dataset_id}, deleted {num_deleted} rows")
            print()

    except Exception as e:
        print(f"Error: {e}")


main()
