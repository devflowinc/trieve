variable "cluster_name" {
  default = "test-cluster"
  type = string
}

variable "project" {
  type = string
}

variable "region" {
  default = "us-central1"
  type = string
}

variable "zone" {
  default = "us-central1-a"
  type = string
}

variable "postgres-root-password" {
    type = string
  
}

variable "qdrant-machine-type" {
    default = "e2-standard-4"
    type = string
}

variable "standard-machine-type" {
    default = "e2-highcpu-4"
    type = string
}

variable "gpu-machine-type" {
    default = "g2-standard-4"
    type = string
  
}
