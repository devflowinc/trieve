docker image pull trieve/server:latest
docker image tag trieve/server localhost:5001/server:5
docker push localhost:5001/server:5

docker image pull trieve/ingest:latest
docker image tag trieve/ingest localhost:5001/ingest:5
docker push localhost:5001/ingest:5

docker image pull trieve/delete_worker:latest
docker image tag trieve/delete_worker localhost:5001/delete_worker:5
docker push localhost:5001/delete_worker:5

docker image pull trieve/file_worker:latest
docker image tag trieve/file_worker localhost:5001/file_worker:5
docker push localhost:5001/file_worker:5

docker image pull trieve/analytics-site:latest
docker image tag trieve/analytics-site localhost:5001/analytics-site:5
docker push localhost:5001/analytics-site:5

docker image pull trieve/ingest:latest
docker image tag trieve/ingest localhost:5001/ingest:5
docker push localhost:5001/ingest:5

docker image pull trieve/dashboard:latest
docker image tag trieve/dashboard localhost:5001/dashboard:5
docker push localhost:5001/dashboard:5

docker image pull trieve/chat:latest
docker image tag trieve/chat localhost:5001/chat:5
docker push localhost:5001/chat:5

docker image pull trieve/search:latest
docker image tag trieve/search localhost:5001/search:5
docker push localhost:5001/search:5

docker image pull trieve/analytics-site:latest
docker image tag trieve/analytics-site:latest localhost:5001/analytics-site:5
docker push localhost:5001/analytics-site:5

docker image pull trieve/clickhouse-clustering
docker image tag trieve/clickhouse-clustering localhost:5001/clickhouse-clustering
docker push localhost:5001/clickhouse-clustering
