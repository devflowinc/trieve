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
  rds_storage_size_gb = 100
  rds_engine          = "postgres"
  rds_engine_version  = "14"
  rds_family          = "postgres14"
  rds_major_engine_version = "14"

  # Elasticache Configuration
  use_elasticache = true

  instance_type_redis  = "cache.t3.small"
  engine_version_redis = "6.x"
  family_redis         = "redis6.x"
  cluster_size_redis = 5

  # EKS Node Group Configuration
  instance_type_gpu    = "g4dn.xlarge"
  gpu_max_size         = 2
  gpu_min_size         = 1
  gpu_desired_capacity = 1

  instance_type_qdrant    = "t3.xlarge"
  qdrant_max_size         = 4
  qdrant_min_size         = 0
  qdrant_desired_capacity = 4

  instance_type_standard = "t3.medium"
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
