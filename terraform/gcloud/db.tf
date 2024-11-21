resource "google_sql_database_instance" "instance" {
  name             = "trieve-${var.cluster_name}"
  region           = var.region
  database_version = "POSTGRES_15"

  deletion_protection = true
  root_password       = var.postgres-root-password

  settings {
    edition           = "ENTERPRISE_PLUS"
    availability_type = "REGIONAL"

    disk_size       = 100 # GB
    disk_autoresize = true

    tier = "db-perf-optimized-N-2"

    insights_config {
      query_insights_enabled  = true
      query_plans_per_minute  = 5
      query_string_length     = 1024
      record_application_tags = false
      record_client_address   = false
    }

    data_cache_config {
      data_cache_enabled = true
    }

    database_flags {
      name  = "cloudsql.iam_authentication"
      value = "on"
    }

    backup_configuration {
      enabled                        = true
      point_in_time_recovery_enabled = true
    }

    ip_configuration {
      ipv4_enabled = true
    }
  }
}

resource "google_redis_instance" "my_memorystore_redis_instance" {
  name               = "${var.cluster_name}-redis"
  authorized_network = google_compute_network.vpc_network.id
  tier               = "BASIC"
  memory_size_gb     = 3
  region             = var.region
  redis_version      = "REDIS_6_X"
}

# google_client_config and kubernetes provider must be explicitly specified like the following.
data "google_client_config" "default" {}

provider "kubernetes" {
  host                   = "https://${google_container_cluster.cluster.endpoint}"
  token                  = data.google_client_config.default.access_token
  cluster_ca_certificate = base64decode(google_container_cluster.cluster.master_auth.0.cluster_ca_certificate)
}

module "my-app-workload-identity" {
  source  = "terraform-google-modules/kubernetes-engine/google//modules/workload-identity"
  version = "31.0.0"

  name       = "${var.cluster_name}-postgres-service-account"
  namespace  = "default"
  project_id = var.project
  roles      = ["roles/iam.workloadIdentityUser", "roles/cloudsql.client"]
}

output "k8s_pg_service_account" {
  value = module.my-app-workload-identity.k8s_service_account_name
}

resource "google_sql_user" "iam_service_account_user" {
  # Note: for Postgres only, GCP requires omitting the ".gserviceaccount.com" suffix
  # from the service account email due to length limits on database usernames.
  name     = trimsuffix(module.my-app-workload-identity.gcp_service_account_email, ".gserviceaccount.com")
  instance = google_sql_database_instance.instance.name
  type     = "CLOUD_IAM_SERVICE_ACCOUNT"
}
