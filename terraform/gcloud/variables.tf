variable "cluster_name" {
  default = "test-cluster"
  type    = string
}

variable "project" {
  type = string
}

variable "region" {
  default = "us-central1"
  type    = string
}

variable "zone" {
  default = "us-central1-a"
  type    = string
}

variable "standard-machine-type" {
  default = "n2-standard-4"
  type    = string
}

variable "gpu-machine-type" {
  default = "g2-standard-4"
  type    = string
}

variable "gpu-accelerator-type" {
  default = "nvidia-l4"
  type    = string
}
