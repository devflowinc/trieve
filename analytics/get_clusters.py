import uuid
import anthropic
import clickhouse_connect
import numpy as np
from sklearn.cluster import KMeans
from scipy.spatial.distance import cosine
import dotenv

dotenv.load_dotenv()
anthropic_client = anthropic.Anthropic()


# Function to fetch data from ClickHouse
def fetch_dataset_vectors(client, dataset_id, limit=5000):
    query = """
        SELECT id, query, query_vector 
        FROM trieve.search_queries 
        WHERE dataset_id = '{}'
            AND created_at >= now() - INTERVAL 7 DAY
        ORDER BY rand() 
        LIMIT {}
        """.format(
        str(dataset_id[0]),
        limit,
    )

    vector_result = client.query(query)
    rows = vector_result.result_rows

    return rows


def get_datasets(client):
    query = """
        SELECT DISTINCT dataset_id
        FROM search_queries
        """

    dataset_result = client.query(query)
    rows = dataset_result.result_rows
    return rows


def kmeans_clustering(data, n_clusters=10):
    vectors = np.array([row[2] for row in data])
    kmeans = KMeans(n_clusters=n_clusters, init="k-means++")
    kmeans.fit(vectors)
    return kmeans, vectors


# Function to find the closest queries to the centroids
def find_closest_dense_queries(kmeans, vectors, data, n_points=5):
    centroids = kmeans.cluster_centers_
    topics = []

    for i, centroid in enumerate(centroids):
        distances = [cosine(centroid, vector) for vector in vectors]
        closest_indices = np.argsort(distances)[
            : n_points + 1
        ]  # include the centroid itself
        # Create a request to the ChatGPT model
        response = anthropic_client.messages.create(
            model="claude-3-haiku-20240307",  # or "gpt-4" if you have access
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

    return topics


def insert_centroids(client, dataset_id, topics):
    delete_previous_query = """
        DELETE FROM trieve.cluster_topics
        WHERE dataset_id = '{}'
        """.format(
        str(dataset_id[0])
    )
    client.query(delete_previous_query)
    for i, topic in enumerate(topics):
        query = """
            INSERT INTO trieve.cluster_topics
            (id, dataset_id, topic, created_at)
            VALUES
            ('{}', '{}', '{}', now())
            """.format(
            str(uuid.uuid4()), str(dataset_id[0]), str(topic).replace("'", "\\'")
        )
        client.query(query)


# Main script
if __name__ == "__main__":
    # Connect to ClickHouse
    client = clickhouse_connect.get_client(
        host="localhost",
        port=8123,
        username="clickhouse",
        password="password",
        database="trieve",
    )

    dataset_ids = get_datasets(client)
    for dataset_id in dataset_ids:
        # Fetch data
        data = fetch_dataset_vectors(client, dataset_id, 3000)

        # Perform spherical k-means clustering
        n_clusters = 15  # Change this to the desired number of clusters
        kmeans, vectors = kmeans_clustering(data, n_clusters)

        # Find the closest queries to the centroids
        topics = find_closest_dense_queries(kmeans, vectors, data)

        # Insert the topics into the database
        insert_centroids(client, dataset_id, topics)
