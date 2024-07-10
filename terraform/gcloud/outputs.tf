output "redis_host" {
 description = "The IP address of the instance."
 value = "${google_redis_instance.my_memorystore_redis_instance.host}"
}

output "postgres_host" {
 description = "The IP address of the instance."
 value = "${google_sql_database_instance.instance.ip_address}"
}
