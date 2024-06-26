import datetime
import os
import uuid
import anthropic
import clickhouse_connect
import clickhouse_connect.driver
import clickhouse_connect.driver.client
import numpy as np
from sklearn.cluster import KMeans
from scipy.spatial.distance import cosine

anthropic_client = anthropic.Anthropic()


# Function to fetch data from ClickHouse
def fetch_dataset_vectors(
    client: clickhouse_connect.driver.client.Client, dataset_id: uuid.UUID, limit=5000
):
    query = """
        SELECT id, query, top_score, query_vector 
        FROM trieve.search_queries 
        WHERE dataset_id = '{}'
            AND created_at >= now() - INTERVAL 7 DAY
        ORDER BY rand() 
        LIMIT {}
        """.format(
        str(dataset_id),
        limit,
    )

    vector_result = client.query(query)
    rows = vector_result.result_rows

    return rows


def get_datasets(client: clickhouse_connect.driver.client.Client):
    query = """
        SELECT DISTINCT dataset_id
        FROM search_queries
        """

    dataset_result = client.query(query)
    rows = dataset_result.result_rows
    return rows


def kmeans_clustering(data, n_clusters=10):
    vectors = np.array([row[3] for row in data])
    kmeans = KMeans(n_clusters=n_clusters, init="k-means++")
    kmeans.fit(vectors)
    return kmeans, vectors


# Function to find the closest queries to the centroids
def get_topics(kmeans, vectors, data, n_points=5):
    centroids = kmeans.cluster_centers_
    topics = []

    for i, centroid in enumerate(centroids):
        distances = [cosine(centroid, vector) for vector in vectors]
        closest_indices = np.argsort(distances)[
            : n_points + 1
        ]  # include the centroid itself

        for row in data:
            if row[4] == i:
                row.append(cosine(centroid, row[3]))

        # Create a request to the ChatGPT model
        response = anthropic_client.messages.create(
            model="claude-3-haiku-20240307",
            max_tokens=50,
            system="You are a data scientist. You have been tasked with clustering search queries into topics. You have just finished clustering a set of queries into a group. You have been asked to generate a 3-5 word topic name for this cluster. ONLY RETURN THE TOPIC AND NO OTHER CONTEXT OR WORDS",
            messages=[
                {
                    "role": "user",
                    "content": f"Here are some search queries from a cluster: {', '.join([data[idx][1] for idx in closest_indices])}",
                },
            ],
        )
        # Get the response text
        reply = response.content[0].text
        # Extract the topic name
        topics.append(reply)

    return data, topics


def append_cluster_membership(data, kmeans):
    labels = kmeans.labels_
    for i, row in enumerate(data):
        row = list(row)
        row.append(labels[i])
        data[i] = row
    return data


def insert_centroids(
    client: clickhouse_connect.driver.client.Client, data, dataset_id, topics
):
    cluster_ids_to_delete_query = """
        SELECT id
        FROM trieve.cluster_topics
        WHERE dataset_id = '{}'
        """.format(
        str(dataset_id[0])
    )
    cluster_ids_to_delete = [
        str(row[0]) for row in client.query(cluster_ids_to_delete_query).result_rows
    ]

    delete_previous_query = """
        DELETE FROM trieve.cluster_topics
        WHERE dataset_id = '{}'
        """.format(
        str(dataset_id[0])
    )
    client.query(delete_previous_query)
    if len(cluster_ids_to_delete) > 0:
        delete_previous_search_cluster_memberships_query = """
        DELETE FROM trieve.search_cluster_memberships
        WHERE cluster_id IN ('{}')
        """.format(
            "', '".join(cluster_ids_to_delete)
        )
        client.query(delete_previous_search_cluster_memberships_query)

    topic_ids = [uuid.uuid4() for _ in range(len(topics))]

    client.insert(
        "cluster_topics",
        [
            [
                topic_ids[i],
                dataset_id[0],
                topic,
                len([row for row in data if len(row) == 6 and row[4] == i]),
                np.mean([row[2] for row in data if len(row) == 6 and row[4] == i]),
                datetime.datetime.now(),
            ]
            for i, topic in enumerate(topics)
        ],
        column_names=[
            "id",
            "dataset_id",
            "topic",
            "density",
            "avg_score",
            "created_at",
        ],
        settings={
            "async_insert": "1",
            "wait_for_async_insert": "0",
        },
    )

    client.insert(
        "search_cluster_memberships",
        [[uuid.uuid4(), row[0], topic_ids[row[4]], float(row[5])] for row in data],
        settings={
            "async_insert": "1",
            "wait_for_async_insert": "0",
        },
    )


# Main script
if __name__ == "__main__":
    # Connect to ClickHouse
    client = clickhouse_connect.get_client(
        dsn=os.getenv("CLICKHOUSE_DSN"),
    )

    dataset_ids = get_datasets(client)
    for dataset_id in dataset_ids:
        # Fetch data
        data = fetch_dataset_vectors(client, dataset_id[0], 3000)

        # Perform spherical k-means clustering
        n_clusters = 15  # Change this to the desired number of clusters
        kmeans, vectors = kmeans_clustering(data, n_clusters)

        # Append cluster membership to the data
        data = append_cluster_membership(data, kmeans)

        # Find the closest queries to the centroids
        data, topics = get_topics(kmeans, vectors, data)

        # Insert the topics into the database
        insert_centroids(client, data, dataset_id, topics)
