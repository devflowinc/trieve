#!/bin/bash

function get_zoneid() {
   aws route53 list-hosted-zones-by-name --output json --dns-name "trieve-aws.ai." --query HostedZones[0].Id --out text
}
