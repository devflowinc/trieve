module "trieve" {
  source = "./trieve-aws"

  # General Configuration
  aws_region = "us-east-1"
  name = "trieve-aws-cluster"

  # EKS Node Group Configuration
  instance_type_gpu    = "g6.xlarge"
  gpu_max_size         = 8
  gpu_min_size         = 1
  gpu_desired_capacity = 5

  instance_type_standard = "c7a.xlarge"
  standard_max_size      = 3
  standard_min_size      = 0
  standard_desired_capacity = 1
  # Application Load Balancer
  install_alb_controller = true
}
