import datetime
import os
import uuid
import anthropic
import clickhouse_connect
import clickhouse_connect.driver
import clickhouse_connect.driver.client
import numpy as np
from sklearn.cluster import HDBSCAN
from scipy.spatial.distance import cosine
import dotenv

dotenv.load_dotenv()
anthropic_client = anthropic.Anthropic(api_key=os.getenv("ANTHROPIC_API_KEY"))


# Function to fetch data from ClickHouse
def fetch_dataset_vectors(
    client: clickhouse_connect.driver.client.Client, dataset_id: uuid.UUID, limit=5000
):
    query = """
        SELECT id, query, top_score, query_vector 
        FROM default.search_queries 
        WHERE dataset_id = '{}'
            AND created_at >= now() - INTERVAL 7 DAY AND is_duplicate = 0
        ORDER BY rand() 
        LIMIT {}
        """.format(
        str(dataset_id),
        limit,
    )

    vector_result = client.query(query)
    rows = vector_result.result_rows

    return rows


def get_clusters(hdbscan, data):
    labels = hdbscan.labels_
    probabilties = hdbscan.probabilities_
    clusters = {}
    for i in range(len(labels)):
        label = labels[i]
        if label < 0:
            continue
        else:
            if label not in clusters:
                clusters[label] = []
            clusters[label].append((data[i], probabilties[i], i))
    return clusters


def get_datasets(client: clickhouse_connect.driver.client.Client):
    query = """
        SELECT DISTINCT dataset_id
        FROM default.search_queries
        """

    dataset_result = client.query(query)
    rows = dataset_result.result_rows
    return rows


def hdbscan_clustering(data):
    vectors = np.array([row[3] for row in data])
    hdb = HDBSCAN(min_cluster_size=30, min_samples=None)
    hdb.fit(vectors)
    return hdb


def get_topics(hdbscan, clusters, data, top_n=5):
    topics = {}
    for label, queries_and_index in clusters.items():
        # Get the top_n queries with the highest probabilites for this cluster
        top_queries = [
            q[0][1] for q in sorted(queries_and_index, key=lambda x: x[1])[:top_n]
        ]

        # Create a request to the ChatGPT model
        response = anthropic_client.messages.create(
            model="claude-3-haiku-20240307",
            max_tokens=50,
            system="You are a data scientist. You have been tasked with clustering search queries into topics. You have just finished clustering a set of queries into a group. You have been asked to generate a 3-5 word topic name for this cluster. ONLY RETURN THE TOPIC AND NO OTHER CONTEXT OR WORDS",
            messages=[
                {
                    "role": "user",
                    "content": f"Here are some search queries from a cluster: {', '.join(top_queries)}",
                },
            ],
        )
        # Get the response text
        reply = response.content[0].text
        # Extract the topic name
        topics[label] = reply

    return topics


def insert_centroids(
    client: clickhouse_connect.driver.client.Client, data, dataset_id, topics, clusters
):
    for t in topics:
        topics[t] = (topics[t], uuid.uuid4())

    client.insert(
        "cluster_topics",
        [
            [
                topic_and_topic_id[1],
                dataset_id[0],
                topic_and_topic_id[0],
                len(clusters[label]),
                np.mean([p[0][2] for p in clusters[label]]),
                datetime.datetime.now(),
            ]
            for label, topic_and_topic_id in topics.items()
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

    membership_rows = []
    for label, queries_and_index in clusters.items():
        cluster_id = topics[label][1]
        for row in queries_and_index:
            search_id = row[0][0]
            prob = row[1]
            membership_rows.append([uuid.uuid4(), search_id, cluster_id, prob])

    client.insert(
        "search_cluster_memberships",
        membership_rows,
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
        try:
            # Fetch data
            data = fetch_dataset_vectors(client, dataset_id[0], 3000)

            if len(data) < 30:
                print(f"Skipping dataset {dataset_id[0]} due to insufficient data")
                continue

            # Perform spherical k-means clustering
            hdbscan = hdbscan_clustering(data)

            clusters = get_clusters(hdbscan, data)

            # Find the closest queries to the centroids
            topics = get_topics(hdbscan, clusters, data)

            # Insert the topics into the database
            insert_centroids(client, data, dataset_id, topics, clusters)

            print(f"Finished clustering for {dataset_id[0]}")
        except Exception as e:
            print(f"ERROR: {e}")
            continue
