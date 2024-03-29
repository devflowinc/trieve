FROM quay.io/keycloak/keycloak:23.0.7

WORKDIR /opt/keycloak

COPY ./themes/trieve /opt/keycloak/themes/trieve

COPY ./realm-export.json /opt/keycloak/data/import/realm-export.json

EXPOSE 8080

ENTRYPOINT ["/opt/keycloak/bin/kc.sh"]

CMD ["start-dev", "--import-realm", "--spi-theme-static-max-age=-1", "--spi-theme-cache-themes=false", "--spi-theme-cache-templates=false"]
