# Trieve Self-Hosting Scripts

This repository contains two bash scripts for setting up and configuring a self-hosted instance of Trieve. Below are the most important functions and features of these scripts.

## setup-self-hosting.sh

This script prepares the system for hosting Trieve by installing necessary dependencies and setting up the environment.

### Key Features:

1. **Package Installation**: Installs required packages such as jq, git, caddy, curl, and gawk.
2. **Caddy Setup**: Adds the Caddy GPG key and repository for secure web server installation.
3. **Docker Installation**: Checks for Docker and installs it if not present.
4. **Error Handling**: Provides informative error messages and exit codes for troubleshooting.

## configure-self-hosting.sh

This script configures the Trieve instance based on the system's capabilities and user input.

### Key Functions:

1. **check_gpu_and_resources()**: 
   - Detects GPU architecture and available system resources.
   - Determines the appropriate Docker Compose configuration to use based on GPU and system capabilities.

2. **check_cpu_and_ram()**: 
   - Checks CPU cores and RAM to decide whether to run CPU-based embeddings.

3. **Caddyfile Generation**: 
   - Creates a Caddyfile for reverse proxy configuration based on the specified domain.

4. **Environment File Configuration**: 
   - Modifies the .env file with appropriate values for the self-hosted instance.

5. **Docker Compose File Modification**: 
   - Updates the docker-compose.yml file with necessary changes for self-hosting.

6. **Keycloak Realm Configuration**: 
   - Modifies the realm-export.json file for Keycloak authentication setup.

### Additional Features:

- **Domain Configuration**: Prompts for and uses a custom domain for the Trieve instance.
- **DNS Record Display**: Shows required DNS A records for proper domain configuration.
- **Service Management**: Ensures the Caddy service is enabled, started, and reloaded as needed.

## Usage

1. Run `setup-self-hosting.sh` to install dependencies and prepare the system.
2. Set the `DOMAIN` environment variable and run `configure-self-hosting.sh` to set up your Trieve instance.

Example:
```bash
./setup-self-hosting.sh
DOMAIN=yourdomain.com ./configure-self-hosting.sh
```

Make sure to follow any on-screen prompts and instructions during the configuration process.

## Note

These scripts are designed to streamline the self-hosting process for Trieve. Always review the scripts and understand their actions before running them on your system. Ensure you have the necessary permissions and backups before proceeding with the installation and configuration.
