resource "google_sql_database_instance" "instance" {
  name             = "trieve-cloud"
  region           = var.region
  database_version = "POSTGRES_15"

  deletion_protection = false
  root_password    = "abcABC123!"

  settings {
    edition = "ENTERPRISE"

    disk_size = 100 # GB
    disk_autoresize = true

    tier = "db-custom-4-15360"

    database_flags {
      name  = "cloudsql.iam_authentication"
      value = "on"
    }

    backup_configuration {
      enabled = true
      point_in_time_recovery_enabled = true
    }

    ip_configuration {
      ipv4_enabled = true
    }
  }
}

resource "google_redis_instance" "my_memorystore_redis_instance" {
  name           = "cloud-redis"
  tier           = "BASIC"
  memory_size_gb = 5
  region         = var.region
  redis_version  = "REDIS_6_X"
}

# google_client_config and kubernetes provider must be explicitly specified like the following.
data "google_client_config" "default" {}

provider "kubernetes" {
  host                   = "https://${google_container_cluster.cluster.endpoint}"
  token                  = data.google_client_config.default.access_token
  cluster_ca_certificate = base64decode(google_container_cluster.cluster.master_auth.0.cluster_ca_certificate)
}

module "my-app-workload-identity" {
  source              = "terraform-google-modules/kubernetes-engine/google//modules/workload-identity"
  version             = "31.0.0"

  name                = "${var.cluster_name}-postgres-service-account"
  namespace           = "default"
  project_id          = var.project
  roles               = ["roles/iam.workloadIdentityUser", "roles/cloudsql.client", "roles/storage.admin"]
}

output k8s_pg_service_account {
  value = module.my-app-workload-identity.k8s_service_account_name
}

resource "google_sql_user" "iam_service_account_user" {
  # Note: for Postgres only, GCP requires omitting the ".gserviceaccount.com" suffix
  # from the service account email due to length limits on database usernames.
  name     = trimsuffix(module.my-app-workload-identity.gcp_service_account_email, ".gserviceaccount.com")
  instance = google_sql_database_instance.instance.name
  type     = "CLOUD_IAM_SERVICE_ACCOUNT"
}
