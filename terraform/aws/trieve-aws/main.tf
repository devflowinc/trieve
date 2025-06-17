terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  region = var.aws_region
}

# VPC Module
module "vpc" {
  source  = "terraform-aws-modules/vpc/aws"
  version = "5.16.0"

  name = "${var.name}-vpc"
  cidr = "10.0.0.0/16"

  azs             = ["${var.aws_region}a", "${var.aws_region}b", "${var.aws_region}c"]
  private_subnets = ["10.0.1.0/24", "10.0.2.0/24", "10.0.3.0/24"]
  public_subnets  = ["10.0.101.0/24", "10.0.102.0/24", "10.0.103.0/24"]

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
  version = "~> 20.0"

  cluster_name    = var.name
  cluster_version = "1.32"

  vpc_id     = module.vpc.vpc_id
  subnet_ids = module.vpc.private_subnets

  # Add-ons
  cluster_addons = {
    coredns = {}
    kube-proxy = {}
    vpc-cni = {}
    aws-ebs-csi-driver = {
      service_account_role_arn = module.ebs_csi_irsa.iam_role_arn
    }
    eks-pod-identity-agent = {}
  }

  cluster_endpoint_public_access = true
  enable_cluster_creator_admin_permissions = true
  enable_irsa = true

  eks_managed_node_groups = {
    standard = {
      min_size     = var.standard_min_size
      max_size     = var.standard_max_size
      desired_size = var.standard_desired_capacity

      instance_types = [var.instance_type_standard]
      capacity_type  = "ON_DEMAND"
      ami_type       = "AL2_x86_64"
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

      labels = {
        "eks-node" = "gpu"
      }

    }
  }
}

# Add this after your EKS module
resource "aws_security_group_rule" "node_to_node_all" {
  description              = "Allow nodes to communicate with each other on all ports"
  type                     = "ingress"
  from_port                = 0
  to_port                  = 65535
  protocol                 = "-1"
  source_security_group_id = module.eks.node_security_group_id
  security_group_id        = module.eks.node_security_group_id
}

resource "aws_security_group_rule" "vpc_cidr_ingress" {
  description       = "Allow all traffic from VPC CIDR"
  type              = "ingress"
  from_port         = 0
  to_port           = 65535
  protocol          = "-1"
  cidr_blocks       = [module.vpc.vpc_cidr_block]
  security_group_id = module.eks.node_security_group_id
}

module "ebs_csi_irsa" {
  source  = "terraform-aws-modules/iam/aws//modules/iam-role-for-service-accounts-eks"
  version = "~> 5.0"

  role_name = "${var.name}-ebs-csi-irsa"

  # Bind the role to the cluster’s OIDC provider and the CSI controller SA
  oidc_providers = {
    main = {
      provider_arn               = module.eks.oidc_provider_arn
      namespace_service_accounts = ["kube-system:ebs-csi-controller-sa"]
    }
  }

  attach_ebs_csi_policy = true
}
