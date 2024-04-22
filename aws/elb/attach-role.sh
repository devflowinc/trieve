source ./functions.sh
oidc_id=$(get_oidc)
region=us-east-2
account_id=$(aws sts get-caller-identity --query "Account" --output text)
aws iam attach-role-policy --policy-arn arn:aws:iam::$account_id:policy/AWSLoadBalancerControllerIAMPolicy --role-name AmazonEKSLoadBalancerControllerRole
exit $?
