variable "aws_region" {
  type    = string
  default = "us-west-2"
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

variable "instance_type_standard" {
  type    = string
  default = "t3.2xlarge"
}

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
