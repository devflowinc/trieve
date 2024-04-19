#!/bin/bash
eksctl utils associate-iam-oidc-provider --cluster trieve --approve
exit $?
