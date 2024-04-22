#!/bin/bash
aws iam create-policy --policy-name "AllowExternalDNSUpdates" --policy-document file://policy.json
exit 0
