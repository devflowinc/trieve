source ./functions.sh
oidc_id=$(get_oidc)
region=us-east-2
account_id=$(aws sts get-caller-identity --query "Account" --output text)
cat >load-balancer-role-trust-policy.json <<EOF
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Effect": "Allow",
            "Principal": {
                "Federated": "arn:aws:iam::$account_id:oidc-provider/oidc.eks.$region.amazonaws.com/id/$oidc_id"
            },
            "Action": "sts:AssumeRoleWithWebIdentity",
            "Condition": {
                "StringEquals": {
                    "oidc.eks.$region.amazonaws.com/id/$oidc_id:aud": "sts.amazonaws.com",
                    "oidc.eks.$region.amazonaws.com/id/$oidc_id:sub": "system:serviceaccount:kube-system:aws-load-balancer-controller"
                }
            }
        }
    ]
}
EOF

exit $?
