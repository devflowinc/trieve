#!/bin/bash

packages=("jq" "git" "caddy" "curl" "gawk");


# Add Caddy GPG key
if [ ! -f "/usr/share/keyrings/caddy-stable-archive-keyring.gpg" ]; then
    if ! curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg; then
        if ! wget -qO - 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg; then
            echo "Error: Failed to add Caddy GPG key." >&2
            exit 1
        fi
    fi
    else
        echo "File already exists. Skipping download."
fi


# Add Caddy repository
if ! curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | sudo tee /etc/apt/sources.list.d/caddy.list > /dev/null; then
    if ! wget -qO - 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | sudo tee /etc/apt/sources.list.d/caddy.list > /dev/null; then
        echo "Error: Failed to add Caddy repository." >&2
        exit 1
    fi
fi

# Install Docker
if ! command -v docker &> /dev/null; then
    echo "Installing Docker..."
    if ! curl -fsSL https://get.docker.com | sh; then
        if ! wget -qO - https://get.docker.com | sh; then
            echo "Error: Failed to install Docker." >&2
            exit 1
        fi
    fi
    sudo usermod -aG docker "$USER"
    newgrp docker
else
    echo "Docker is already installed. Skipping installation."
fi

# Update package lists and install required packages
echo "Updating package lists and installing required packages..."
if ! sudo apt update; then
    echo "Error: Failed to update package lists." >&2
    exit 1
fi


 # Installing packages
 for package in "${packages[@]}";  do
     if !  dpkg -s "$package" >/dev/null 2>&1;  then
         if sudo apt install "$package" -y;  then
             echo "Package installed: $package"
         else
             echo "Warning: Failed to install $package. Continuing..." >&2
         fi
     else
         echo "$package is already installed."
     fi
 done

 echo "Packages installation completed. Proceeding..."


echo "Script completed successfully."
