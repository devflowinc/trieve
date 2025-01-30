variable "vpc_id" {
  type    = string
  default = ""
}

variable "create_vpc" {
  type    = bool
  default = true
}

variable "aws_region" {
  type    = string
  default = "us-west-2"
}

variable "use_rds" {
  type    = bool
  default = true
}

variable "rds_instance_size" {
  type    = string
  default = "db.t3.medium"
}

variable "rds_storage_size_gb" {
  type    = number
  default = 100
}

variable "rds_engine" { default = "postgres" }

variable "rds_engine_version" { default = "14" }

variable "rds_family" { default = "postgres14" }

variable "rds_major_engine_version" { default = "14" }

variable "use_elasticache" {
  type    = bool
  default = true
}

variable "name" {
  type    = string
  default = "eks-nodes"
}

variable "instance_type_gpu" {
  type    = string
  default = "g4dn.xlarge"
}

variable "gpu_max_size" {
  type    = number
  default = 2
}

variable "gpu_min_size" {
  type    = number
  default = 1
}

variable "gpu_desired_capacity" {
  type    = number
  default = 1
}

variable "instance_type_qdrant" {
  type    = string
  default = "r5.8xlarge"
}

variable "qdrant_max_size" {
  type    = number
  default = 4
}

variable "qdrant_min_size" {
  type    = number
  default = 4
}

variable "qdrant_desired_capacity" {
  type    = number
  default = 4
}

variable "instance_type_standard" {
  type    = string
  default = "t3.2xlarge"
}

variable "instance_type_redis" { default = "cache.t3.medium" }

variable "engine_version_redis" { default = "6.x" }

variable "family_redis" { default = "redis6.x" }

variable "cluster_size_redis" { default = 1 }

variable "standard_max_size" {
  type    = number
  default = 3
}

variable "standard_min_size" {
  type    = number
  default = 1
}

variable "standard_desired_capacity" {
  type    = number
  default = 2
}

variable "ebs_optimized" {
  type    = bool
  default = false
}

variable "public_ip" {
  type    = bool
  default = false
}

variable "install_alb_controller" {
  type        = bool
  default     = true
  description = "Whether to install the AWS Load Balancer Controller"
}

variable "pod_identity_version" { default = "v1.2.0-eksbuild.1" }

variable "rds_master_password" {
  type = string
}
