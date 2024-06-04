terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

variable "ssh_pub_key_file" {
  type    = string
  default = "~/.ssh/id_ed25519.pub"
}

variable "region" {
  type = string
  default = "us-west-1"
}

variable "cluster-name" {
  type = string
  default = "test"
}

###############################################################
# VPC configuration
###############################################################
# Set AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY
provider "aws" {
  region = var.region
}

module "vpc" {
  source = "terraform-aws-modules/vpc/aws"
  name   = "${var.cluster-name}-trieve-vpc"

  enable_nat_gateway   = true
  enable_dns_hostnames = true

  cidr            = "10.0.0.0/16"
  azs             = ["us-west-1a", "us-west-1b"]
  private_subnets = ["10.0.1.0/24", "10.0.2.0/24", "10.0.3.0/24"]
  public_subnets  = ["10.0.101.0/24", "10.0.102.0/24", "10.0.103.0/24"]

  private_subnet_tags = {
    "kubernetes.io/cluster/trieve" : "shared"
    "kubernetes.io/role/internal-elb" = "1"
  }

  public_subnet_tags = {
    "kubernetes.io/role/elb" : 1
  }
}

###############################################################
# EKS configuration
###############################################################
module "eks" {
  source  = "terraform-aws-modules/eks/aws"
  version = "~> 19.0"

  vpc_id     = module.vpc.vpc_id
  subnet_ids = module.vpc.private_subnets

  cluster_name                   = "${var.cluster-name}-cluster"
  cluster_version                = "1.29"
  cluster_endpoint_public_access = true

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

  eks_managed_node_groups = {

    trieve-general = {
      name = "${var.cluster-name}-general"

      instance_types = ["c6a.xlarge"]

      desired_size = 2
      min_size     = 1
      max_size     = 2

      # Needed by the aws-ebs-csi-driver
      iam_role_additional_policies = {
        AmazonEBSCSIDriverPolicy = "arn:aws:iam::aws:policy/service-role/AmazonEBSCSIDriverPolicy"
      }
    }

    trieve-highmem = {
      name = "${var.cluster-name}-highmem"

      instance_types = ["r6a.xlarge"]

      desired_size = 4
      min_size     = 1
      max_size     = 4

      taints = [
        {
          key    = "qdrant-node"
          effect = "NO_SCHEDULE"
        }
      ]

      # Needed by the aws-ebs-csi-driver
      iam_role_additional_policies = {
        AmazonEBSCSIDriverPolicy = "arn:aws:iam::aws:policy/service-role/AmazonEBSCSIDriverPolicy"
      }
    }
  }
}

###############################################################
# K8s configuration
###############################################################
# If you are having trouble with the exec command, you can try the token technique.
# Put token  = data.aws_eks_cluster_auth.cluster_auth.token in place of exec
# data "aws_eks_cluster_auth" "cluster_auth" {
#   name = "trieve"
# }
#

data "aws_eks_cluster" "cluster" {
  depends_on = [module.eks]
  name       = "${var.cluster-name}-cluster"
}

data "aws_iam_openid_connect_provider" "oidc_provider" {
  url = data.aws_eks_cluster.cluster.identity.0.oidc.0.issuer
}

provider "kubernetes" {
  host                   = module.eks.cluster_endpoint
  cluster_ca_certificate = base64decode(module.eks.cluster_certificate_authority_data)
  #token                  = data.aws_eks_cluster_auth.cluster_auth.token
  exec {
    api_version = "client.authentication.k8s.io/v1beta1"
    args        = ["eks", "get-token", "--cluster-name", module.eks.cluster_name]
    command     = "aws"
  }
}

provider "helm" {
  kubernetes {
    host                   = module.eks.cluster_endpoint
    cluster_ca_certificate = base64decode(module.eks.cluster_certificate_authority_data)
    exec {
      api_version = "client.authentication.k8s.io/v1beta1"
      args        = ["eks", "get-token", "--cluster-name", module.eks.cluster_name]
      command     = "aws"
    }
  }
}

module "alb_controller" {
  source                           = "./alb"
  cluster_name                     = var.cluster-name
  cluster_identity_oidc_issuer     = data.aws_eks_cluster.cluster.identity.0.oidc.0.issuer
  cluster_identity_oidc_issuer_arn = data.aws_iam_openid_connect_provider.oidc_provider.arn
  aws_region                       = var.region
}


# resource "helm_release" "metrics" {
#   name      = "metrics-server"
#   namespace = "kube-system"
#
#   chart = "https://github.com/kubernetes-sigs/metrics-server/releases/latest/download/components.yaml"
# }
