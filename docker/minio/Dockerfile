FROM minio/minio:latest
LABEL LICENSE="Dual AGPLv3 & Commercial"
LABEL LICENSE_URL="https://min.io/compliance"

COPY ./docker-entrypoint.sh ./docker-entrypoint.sh

RUN chmod +x docker-entrypoint.sh

EXPOSE 9000
EXPOSE 9001

ENTRYPOINT ["/docker-entrypoint.sh"]
CMD ["minio", "server", "--console-address", ":9001", "/data"]
