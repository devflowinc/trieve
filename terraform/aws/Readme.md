## Getting Kubernetes Installed

1. Create an [AWS Account](https://portal.aws.amazon.com/billing/signup#/start/email).
2. Create an IAM User with the Administrator policy. Generate access keys and grant it console access. See bottom for notes.
3. Fork this repo and set it up with [spacelift.io](https://spacelift.io/) or equivalent.
4. Set [AWS_ACCESS_KEY_ID](https://registry.terraform.io/providers/hashicorp/aws/latest/docs) and [AWS_SECRET_ACCESS_KEY](https://registry.terraform.io/providers/hashicorp/aws/latest/docs)
5. Run and apply the Terraform (takes 20-30 minutes).
