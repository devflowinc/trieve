terraform {
  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "5.30.0"
    }
    google-beta = {
      source  = "hashicorp/google-beta"
      version = "~> 5.0"
    }
  }
}

###############################################################
# Set up the Networking Components
###############################################################
# Set GOOGLE_CREDENTIALS
provider "google" {
  region  = var.region
  project = var.project
}

provider "google-beta" {
  region  = var.region
  zone    = var.zone
  project = var.project
}

resource "google_compute_network" "vpc_network" {
  name                    = "trieve-vpc-network-${var.cluster_name}"
  auto_create_subnetworks = false
}

resource "google_compute_subnetwork" "vpc_subnet" {
  name          = "trieve-network-${var.cluster_name}"
  ip_cidr_range = "10.3.0.0/16"
  region        = var.region
  network       = google_compute_network.vpc_network.id

  log_config {
    aggregation_interval = "INTERVAL_5_SEC"
    flow_sampling        = 1
    metadata             = "INCLUDE_ALL_METADATA"
    filter_expr          = true
    metadata_fields      = []
  }
}

###############################################################
# K8s configuration
###############################################################
resource "google_container_cluster" "cluster" {
  name       = var.cluster_name
  location   = var.zone
  network    = google_compute_network.vpc_network.name
  subnetwork = google_compute_subnetwork.vpc_subnet.name

  # We can't create a cluster with no node pool defined, but we want to only use
  # separately managed node pools. So we create the smallest possible default
  # node pool and immediately delete it.
  remove_default_node_pool = true
  initial_node_count       = 1

  deletion_protection = false

  vertical_pod_autoscaling {
    enabled = true
  }

  workload_identity_config {
    workload_pool = "${var.project}.svc.id.goog"
  }
}

resource "google_container_node_pool" "standard_nodes" {
  name     = "standard-compute-${var.cluster_name}"
  location = var.zone
  cluster  = google_container_cluster.cluster.name

  node_count = 1

  autoscaling {
    min_node_count = 0
    max_node_count = 3
  }

  node_config {
    preemptible  = false
    machine_type = var.standard-machine-type

    resource_labels = {
      goog-gke-node-pool-provisioning-model = "on-demand"
    }

    kubelet_config {
      cpu_cfs_quota      = false
      pod_pids_limit     = 0
      cpu_manager_policy = ""
    }
  }
}

resource "google_container_node_pool" "gpu_nodes" {
  name       = "gpu-compute-${var.cluster_name}"
  location   = var.zone
  cluster    = google_container_cluster.cluster.name
  node_count = 1

  autoscaling {
    min_node_count = 0
    max_node_count = 5
  }

  node_config {
    preemptible  = false
    machine_type = var.gpu-machine-type

    resource_labels = {
      goog-gke-accelerator-type             = "nvidia-tesla-t4"
      goog-gke-node-pool-provisioning-model = "on-demand"
    }


    kubelet_config {
      cpu_cfs_quota      = false
      pod_pids_limit     = 0
      cpu_manager_policy = ""
    }

    gcfs_config {
      enabled = true
    }

    gvnic {
      enabled = true
    }

    guest_accelerator {
      type  = var.gpu-accelerator-type
      count = 1
      gpu_driver_installation_config {
        gpu_driver_version = "LATEST"
      }

      gpu_sharing_config {
        gpu_sharing_strategy       = "TIME_SHARING"
        max_shared_clients_per_gpu = 10
      }
    }

    workload_metadata_config {
      mode = "GCE_METADATA"
    }

    taint {
      effect = "NO_SCHEDULE"
      key    = "nvidia.com/gpu"
      value  = "present"
    }

    tags = ["gke-my-project-id-region", "gke-my-project-id-region-gpu-time-sharing"]
  }
}
