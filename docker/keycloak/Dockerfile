FROM quay.io/keycloak/keycloak:23.0.7

WORKDIR /opt/keycloak

COPY ./themes/trieve /opt/keycloak/themes/trieve

COPY ./realm-export.json /imports/realm-export.json
COPY ./docker-entrypoint.sh /docker-entrypoint.sh

EXPOSE 8080

USER 0
ENTRYPOINT ["/bin/bash"]
CMD ["/docker-entrypoint.sh"]
