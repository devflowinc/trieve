#!/bin/bash
mkdir -p /opt/keycloak/data/import
cp /imports/realm-export.json /opt/keycloak/data/import/realm-export.json
/opt/keycloak/bin/kc.sh start --import-realm --spi-theme-static-max-age=-1 --spi-theme-cache-themes=false --spi-theme-cache-templates=false
