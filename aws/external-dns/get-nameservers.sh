#!/bin/bash
ZONE_ID=$(aws route53 list-hosted-zones-by-name --output json \
	  --dns-name "example.com." --query HostedZones[0].Id --out text)

aws route53 list-resource-record-sets --output text \
	 --hosted-zone-id $ZONE_ID --query \
	  "ResourceRecordSets[?Type == 'NS'].ResourceRecords[*].Value | []" | tr '\t' '\n'
