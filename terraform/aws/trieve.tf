module "trieve" {
  source = "./trieve-aws"

  # General Configuration
  aws_region = "us-west-2"
  name = "trieve-aws-cluster"

  # EKS Node Group Configuration
  instance_type_gpu    = "g6.xlarge"
  gpu_max_size         = 8
  gpu_min_size         = 1
  gpu_desired_capacity = 5
  use_gpu_taints       = true

  instance_type_standard = "c7a.2xlarge"
  standard_max_size      = 2
  standard_min_size      = 0
  standard_desired_capacity = 2
}
