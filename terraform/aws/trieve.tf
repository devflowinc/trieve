module "trieve" {
  source = "./trieve-aws"

  # General Configuration
  aws_region = "us-east-2"
  name = "trieve-aws-cluster"

  # VPC Configuration
  create_vpc = true

  # RDS Configuration
  use_rds             = true
  rds_instance_size   = "db.t3.small"
  rds_storage_size_gb = 20
  rds_engine          = "postgres"
  rds_engine_version  = "14"
  rds_family          = "postgres14"
  rds_major_engine_version = "14"
  rds_master_password = "makeamoresecurepasswordrighthere"

  # Elasticache Configuration
  use_elasticache = true

  instance_type_redis  = "cache.m5.2xlarge"
  cluster_size_redis = 1

  # EKS Node Group Configuration
  instance_type_gpu    = "g5.xlarge"
  gpu_max_size         = 8
  gpu_min_size         = 1
  gpu_desired_capacity = 5

  instance_type_qdrant    = "m5.xlarge"
  qdrant_max_size         = 8
  qdrant_min_size         = 0
  qdrant_desired_capacity = 8

  instance_type_standard = "c7a.xlarge"
  standard_max_size      = 3
  standard_min_size      = 0
  standard_desired_capacity = 1

  # EBS Config
  ebs_optimized = false

  # Use public IP for the cluster
  public_ip = false

  # Application Load Balancer
  install_alb_controller = true

  # Pod Identity
  pod_identity_version = "v1.2.0-eksbuild.1"
}

output "redis" {
  value = module.trieve.redis_output
}
