docker image pull trieve/server:latest
docker image tag trieve/server localhost:5001/server:latest
docker push localhost:5001/server:latest

docker image pull trieve/ingest:latest
docker image tag trieve/ingest localhost:5001/ingest:latest
docker push localhost:5001/ingest:latest

docker image pull trieve/delete_worker:latest
docker image tag trieve/delete_worker localhost:5001/delete_worker:latest
docker push localhost:5001/delete_worker:latest

docker image pull trieve/file_worker:latest
docker image tag trieve/file_worker localhost:5001/file_worker:latest
docker push localhost:5001/file_worker:latest

docker image pull trieve/analytics-site:latest
docker image tag trieve/analytics-site localhost:5001/analytics-site:latest
docker push localhost:5001/analytics-site:latest

docker image pull trieve/ingest:latest
docker image tag trieve/ingest localhost:5001/ingest:latest
docker push localhost:5001/ingest:latest

docker image pull trieve/dashboard:latest
docker image tag trieve/dashboard localhost:5001/dashboard:latest
docker push localhost:5001/dashboard:latest

docker image pull trieve/chat:latest
docker image tag trieve/chat localhost:5001/chat:latest
docker push localhost:5001/chat:latest

docker image pull trieve/search:latest
docker image tag trieve/search localhost:5001/search:latest
docker push localhost:5001/search:latest

docker image pull trieve/analytics-site:latest
docker image tag trieve/analytics-site:latest localhost:5001/analytics-site:latest
docker push localhost:5001/analytics-site:latest

docker image pull trieve/clickhouse-clustering
docker image tag trieve/clickhouse-clustering trlocalhost:5001/clickhouse-clustering
docker push localhost:5001/clickhouse-clustering
