#!/bin/bash

bash create-identity-provider.sh &&
   bash make-service-account.sh && 
   bash make-trust-policy-file.sh &&
   bash create-role.sh &&
   bash attach-role.sh &&
   bash annotate-service-account.sh &&
   bash add-driver.sh
