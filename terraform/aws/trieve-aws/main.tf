terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "5.67.0"
    }
  }
}

provider "aws" {
  region = var.aws_region
}

data "aws_vpc" "existing" {
  count = var.create_vpc ? 0 : 1

  filter {
    name   = "vpc-id"
    values = [var.vpc_id]
  }
}

locals {
  vpc = var.create_vpc ? module.vpc.vpc_id : var.vpc_id
}

# VPC Module
module "vpc" {
  source  = "terraform-aws-modules/vpc/aws"
  version = "5.16.0"

  create_vpc = var.create_vpc

  name = "${var.name}-vpc"
  cidr = "10.0.0.0/16"

  azs             = ["${var.aws_region}a", "${var.aws_region}b", "${var.aws_region}c"]
  private_subnets = ["10.0.1.0/24", "10.0.2.0/24", "10.0.3.0/24"]
  public_subnets  = ["10.0.101.0/24", "10.0.102.0/24", "10.0.103.0/24"]

  create_database_subnet_group           = true
  create_database_subnet_route_table     = true
  create_database_internet_gateway_route = true

  public_subnet_tags = {
    "kubernetes.io/role/elb" = "1"
  }

  private_subnet_tags = {
    "kubernetes.io/role/internal-elb" = "1"
  }

  enable_nat_gateway = true
  single_nat_gateway = true
}

# EKS Module
module "eks" {
  source  = "terraform-aws-modules/eks/aws"
  version = "~> 19.0"

  cluster_name    = var.name
  cluster_version = "1.28"

  vpc_id     = module.vpc.vpc_id
  subnet_ids = module.vpc.private_subnets

  eks_managed_node_groups = {
    standard = {
      min_size     = var.standard_min_size
      max_size     = var.standard_max_size
      desired_size = var.standard_desired_capacity

      instance_types = [var.instance_type_standard]
      capacity_type  = "ON_DEMAND"
      ami_type       = "AL2_x86_64"
    }

    qdrant = {
      min_size     = var.qdrant_min_size
      max_size     = var.qdrant_max_size
      desired_size = var.qdrant_desired_capacity

      instance_types = [var.instance_type_qdrant]
      capacity_type  = "ON_DEMAND"
      ami_type       = "AL2_x86_64"

      taints = [
        {
          key    = "qdrant-node"
          value  = "present"
          effect = "NO_SCHEDULE"
        }
      ]
    }

    gpu = {
      min_size     = var.gpu_min_size
      max_size     = var.gpu_max_size
      desired_size = var.gpu_desired_capacity

      instance_types = [var.instance_type_gpu]
      capacity_type  = "ON_DEMAND"
      ami_type       = "BOTTLEROCKET_x86_64_NVIDIA"

      taints = [
        {
          key    = "nvidia.com/gpu"
          value  = "present"
          effect = "NO_SCHEDULE"
        }
      ]
    }
  }
  cluster_endpoint_public_access = true

  # Add-ons
  cluster_addons = {
    coredns = {
      most_recent = true
    }
    kube-proxy = {
      most_recent = true
    }
    vpc-cni = {
      most_recent = true
    }
    aws-ebs-csi-driver = {
      most_recent = true
    }
  }
}

resource "aws_db_subnet_group" "database" {
  name       = "${var.name}-db-subnet-group"
  subnet_ids = concat(module.vpc.public_subnets, module.vpc.private_subnets)

  tags = {
    Name = "${var.name}-db-subnet-group"
  }
}

resource "aws_security_group" "postgres" {
  name   = "postgres"
  vpc_id = module.vpc.vpc_id

  ingress {
    from_port   = 5432
    to_port     = 5432
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    from_port   = 5432
    to_port     = 5432
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

# RDS Module
module "db" {
  source  = "terraform-aws-modules/rds/aws"
  version = "6.10.0"

  count = var.use_rds ? 1 : 0

  identifier = "${var.name}-rds"

  engine               = "postgres"
  engine_version       = "14"
  family               = "postgres14"
  major_engine_version = "14"
  instance_class       = var.rds_instance_size

  allocated_storage = var.rds_storage_size_gb

  db_name  = "trieve"
  username = "trieve"
  port     = 5432

  multi_az             = false
  db_subnet_group_name = aws_db_subnet_group.database.name
  vpc_security_group_ids = [aws_security_group.postgres.id]

  maintenance_window = "Mon:00:00-Mon:03:00"
  backup_window = "03:00-06:00"

  manage_master_user_password = false
  password = var.rds_master_password

  # Disable backups to create DB faster
  backup_retention_period = 0

  tags = {
    Name = "${var.name}-rds"
  }
}

resource "aws_security_group" "redis" {
  name   = "redis"
  vpc_id = module.vpc.vpc_id

  ingress {
    from_port   = 6379
    to_port     = 6379
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

}

resource "aws_elasticache_subnet_group" "redis_subnet_group" {
  name       = "${var.name}-redis-security-group"
  subnet_ids = module.vpc.private_subnets
}

resource "aws_elasticache_cluster" "cache_cluster" {
  cluster_id           = "${var.name}-redis-cluster"
  engine               = "redis"
  node_type            = var.instance_type_redis
  num_cache_nodes      = var.cluster_size_redis
  parameter_group_name = "default.redis7"
  engine_version       = "7.1"
  security_group_ids   = [aws_security_group.redis.id]
  subnet_group_name    = aws_elasticache_subnet_group.redis_subnet_group.name
}

output "redis_output" {
  value = aws_elasticache_cluster.cache_cluster.cache_nodes[0].address
}
