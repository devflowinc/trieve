FROM postgres:15
LABEL LICENSE="postgresql"
LABEL LICENSE_URL="https://opensource.org/license/postgresql"

COPY ./pg-multi-db.sh /docker-entrypoint-initdb.d/initdb.sh
RUN chmod +x /docker-entrypoint-initdb.d/initdb.sh