FROM quay.io/keycloak/keycloak:23.0.7

WORKDIR /opt/keycloak

COPY ./themes/arguflow /opt/keycloak/themes/arguflow

COPY ./realm-export.json /opt/keycloak/data/import/realm-export.json

ENTRYPOINT ["/opt/keycloak/bin/kc.sh"]
<<<<<<< HEAD:docker/keycloak/Dockerfile
CMD ["start-dev", "--import-realm", "--spi-theme-static-max-age=-1", "--spi-theme-cache-themes=false", "--spi-theme-cache-templates=false"]
=======
CMD ["start-dev", "--import-realm", "--spi-theme-static-max-age=-1", "--spi-theme-cache-themes=false", "--spi-theme-cache-templates=false"]
>>>>>>> main:keycloak/Dockerfile
