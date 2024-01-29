import psycopg2
import time

def terminate_connections(db_url: str):
    connection = get_db_connection(db_url)
    try:
        # Create a cursor
        cursor = connection.cursor()

        # Execute the query to terminate connections
        cursor.execute(
            f"SELECT pg_terminate_backend(pg_stat_activity.pid) FROM pg_stat_activity WHERE pg_stat_activity.datname = '{db_url.split('/')[-1]}' AND pid <> pg_backend_pid();"
        )

        # Commit the changes
        connection.commit()

        print("Connections terminated successfully.")

    except Exception as e:
        print(f"Error terminating connections: {e}")

    finally:
        # Close the cursor and connection
        if connection:
            connection.close()

def get_db_connection(db_url: str):
    attempt_num = 1
    while True:
        try:
            print("Attempting to connect to database...")
            connection = psycopg2.connect(db_url)
            return connection
        except Exception as e:
            print(f"Error connecting to database {db_url} on attempt {attempt_num}: {e}")
            time.sleep(1)
            attempt_num += 1
