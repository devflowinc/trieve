terraform {
  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "=3.0.0"
    }
  }
}

# variable "cluster_name" {
#     default = "test-cluster"
# }
#
# variable "location" {
#     default = "West US2"
# }
#
# ###############################################################
# # K8s configuration
# ###############################################################
# # Set ARM_CLIENT_ID, ARM_CLIENT_SECRET, ARM_SUBSCRIPTION_ID, ARM_TENANT_ID
# provider "azurerm" {
#   features {}
# }
#
# resource "azurerm_resource_group" "trieve-resources" {
#   name     = "trieve-resources"
#   location = var.location
# }
#
# resource "azurerm_kubernetes_cluster" "trieve-aks" {
#   name                = var.cluster_name
#   location            = azurerm_resource_group.trieve-resources.location
#   resource_group_name = azurerm_resource_group.trieve-resources.name
#   dns_prefix          = "treiveaks1"
#   kubernetes_version  = "1.28.8"
#
#   identity {
#     type = "SystemAssigned"
#   }
#
#   default_node_pool {
#     name       = "default"
#     node_count = 1
#     vm_size    = "Standard_DS2_v2"
#   }
#
#   tags = {
#     Environment = "Production"
#   }
# }
#
# resource "azurerm_kubernetes_cluster_node_pool" "general-compute" {
#   name                  = "general"
#   kubernetes_cluster_id = azurerm_kubernetes_cluster.trieve-aks.id
#   vm_size               = "Standard_F16s_v2"
#   node_count            = 2
#
#   tags = {
#     Environment = "Production"
#   }
# }
#
# resource "azurerm_kubernetes_cluster_node_pool" "gpu-compute" {
#   name                  = "gpu"
#   kubernetes_cluster_id = azurerm_kubernetes_cluster.trieve-aks.id
#   vm_size               = "Standard_NC4as_T4_v3"
#   node_count            = 1
#
#   tags = {
#     Environment = "Production"
#   }
# }

provider "azurerm" {
  features {}
}

variable "resource_group_name" {
  description = "The name of the resource group"
  default     = "aksrgvk"
}

variable "location" {
  description = "The Azure region to deploy the resources"
  default     = "australiasoutheast"
}

variable "cluster_name" {
  description = "The name of the AKS cluster"
  default     = "akscluster0132"
}

variable "node_count" {
  description = "The number of nodes in the AKS cluster"
  default     = 1
}

resource "azurerm_resource_group" "aks" {
  name     = var.resource_group_name
  location = var.location
}

resource "azurerm_kubernetes_cluster" "aks" {
  name                = var.cluster_name
  location            = azurerm_resource_group.aks.location
  resource_group_name = azurerm_resource_group.aks.name
  dns_prefix          = "${var.cluster_name}-dns"

  default_node_pool {
    name       = "default"
    node_count = var.node_count
    vm_size    = "Standard_DS2_v2"
  }

  identity {
    type = "SystemAssigned"
  }
}
