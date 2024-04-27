#!/bin/bash
eksctl utils associate-iam-oidc-provider --cluster trieve-02 --approve
exit $?
