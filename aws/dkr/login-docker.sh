#!/bin/bash
ACCOUNT_ID=$(aws sts get-caller-identity --query "Account" --output text)
_AWS_REGION=${AWS_REGION:-us-east-2}
aws ecr get-login-password --region ${_AWS_REGION} | docker login --username AWS --password-stdin ${AWS_ACCOUNT_ID}.dkr.ecr.${_AWS_REGION}.amazonaws.com
