# trieve_py_client.OrganizationApi

All URIs are relative to *https://api.trieve.ai*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_organization**](OrganizationApi.md#create_organization) | **POST** /api/organization | Create Organization
[**create_organization_api_key**](OrganizationApi.md#create_organization_api_key) | **POST** /api/organization/api_key | Create Organization Api Key
[**delete_organization**](OrganizationApi.md#delete_organization) | **DELETE** /api/organization/{organization_id} | Delete Organization
[**delete_organization_api_key**](OrganizationApi.md#delete_organization_api_key) | **DELETE** /api/organization/api_key/{api_key_id} | Delete Organization Api Key
[**get_organization**](OrganizationApi.md#get_organization) | **GET** /api/organization/{organization_id} | Get Organization
[**get_organization_api_keys**](OrganizationApi.md#get_organization_api_keys) | **GET** /api/organization/api_key | Get Organization Api Keys
[**get_organization_usage**](OrganizationApi.md#get_organization_usage) | **GET** /api/organization/usage/{organization_id} | Get Organization Usage
[**get_organization_users**](OrganizationApi.md#get_organization_users) | **GET** /api/organization/users/{organization_id} | Get Organization Users
[**update_all_org_dataset_configs**](OrganizationApi.md#update_all_org_dataset_configs) | **POST** /api/organization/update_dataset_configs | Update All Dataset Configurations
[**update_organization**](OrganizationApi.md#update_organization) | **PUT** /api/organization | Update Organization


# **create_organization**
> Organization create_organization(create_organization_req_payload)

Create Organization

Create a new organization. The auth'ed user who creates the organization will be the default owner of the organization.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.create_organization_req_payload import CreateOrganizationReqPayload
from trieve_py_client.models.organization import Organization
from trieve_py_client.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to https://api.trieve.ai
# See configuration.py for a list of all supported configuration parameters.
configuration = trieve_py_client.Configuration(
    host = "https://api.trieve.ai"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: ApiKey
configuration.api_key['ApiKey'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['ApiKey'] = 'Bearer'

# Enter a context with an instance of the API client
with trieve_py_client.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = trieve_py_client.OrganizationApi(api_client)
    create_organization_req_payload = trieve_py_client.CreateOrganizationReqPayload() # CreateOrganizationReqPayload | The organization data that you want to create

    try:
        # Create Organization
        api_response = api_instance.create_organization(create_organization_req_payload)
        print("The response of OrganizationApi->create_organization:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling OrganizationApi->create_organization: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **create_organization_req_payload** | [**CreateOrganizationReqPayload**](CreateOrganizationReqPayload.md)| The organization data that you want to create | 

### Return type

[**Organization**](Organization.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Created organization object |  -  |
**400** | Service error relating to creating the organization |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **create_organization_api_key**
> CreateApiKeyResponse create_organization_api_key(tr_organization, create_api_key_req_payload)

Create Organization Api Key

Create a new api key for the organization. Successful response will contain the newly created api key.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.create_api_key_req_payload import CreateApiKeyReqPayload
from trieve_py_client.models.create_api_key_response import CreateApiKeyResponse
from trieve_py_client.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to https://api.trieve.ai
# See configuration.py for a list of all supported configuration parameters.
configuration = trieve_py_client.Configuration(
    host = "https://api.trieve.ai"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: ApiKey
configuration.api_key['ApiKey'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['ApiKey'] = 'Bearer'

# Enter a context with an instance of the API client
with trieve_py_client.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = trieve_py_client.OrganizationApi(api_client)
    tr_organization = 'tr_organization_example' # str | The organization id to use for the request.
    create_api_key_req_payload = trieve_py_client.CreateApiKeyReqPayload() # CreateApiKeyReqPayload | JSON request payload to create a new organization api key

    try:
        # Create Organization Api Key
        api_response = api_instance.create_organization_api_key(tr_organization, create_api_key_req_payload)
        print("The response of OrganizationApi->create_organization_api_key:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling OrganizationApi->create_organization_api_key: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_organization** | **str**| The organization id to use for the request. | 
 **create_api_key_req_payload** | [**CreateApiKeyReqPayload**](CreateApiKeyReqPayload.md)| JSON request payload to create a new organization api key | 

### Return type

[**CreateApiKeyResponse**](CreateApiKeyResponse.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | JSON body representing the api_key for the organization |  -  |
**400** | Service error relating to creating api_key for the organization |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **delete_organization**
> delete_organization(tr_organization, organization_id)

Delete Organization

Delete an organization by its id. The auth'ed user must be an owner of the organization to delete it.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to https://api.trieve.ai
# See configuration.py for a list of all supported configuration parameters.
configuration = trieve_py_client.Configuration(
    host = "https://api.trieve.ai"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: ApiKey
configuration.api_key['ApiKey'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['ApiKey'] = 'Bearer'

# Enter a context with an instance of the API client
with trieve_py_client.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = trieve_py_client.OrganizationApi(api_client)
    tr_organization = 'tr_organization_example' # str | The organization id to use for the request
    organization_id = 'organization_id_example' # str | The id of the organization you want to fetch.

    try:
        # Delete Organization
        api_instance.delete_organization(tr_organization, organization_id)
    except Exception as e:
        print("Exception when calling OrganizationApi->delete_organization: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_organization** | **str**| The organization id to use for the request | 
 **organization_id** | **str**| The id of the organization you want to fetch. | 

### Return type

void (empty response body)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**204** | Confirmation that the organization was deleted |  -  |
**400** | Service error relating to deleting the organization by id |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **delete_organization_api_key**
> delete_organization_api_key(api_key_id, tr_organization)

Delete Organization Api Key

Delete an api key for the auth'ed organization.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to https://api.trieve.ai
# See configuration.py for a list of all supported configuration parameters.
configuration = trieve_py_client.Configuration(
    host = "https://api.trieve.ai"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: ApiKey
configuration.api_key['ApiKey'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['ApiKey'] = 'Bearer'

# Enter a context with an instance of the API client
with trieve_py_client.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = trieve_py_client.OrganizationApi(api_client)
    api_key_id = 'api_key_id_example' # str | The id of the api key to delete
    tr_organization = 'tr_organization_example' # str | The organization id to use for the request.

    try:
        # Delete Organization Api Key
        api_instance.delete_organization_api_key(api_key_id, tr_organization)
    except Exception as e:
        print("Exception when calling OrganizationApi->delete_organization_api_key: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **api_key_id** | **str**| The id of the api key to delete | 
 **tr_organization** | **str**| The organization id to use for the request. | 

### Return type

void (empty response body)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**204** | Confirmation that the api key was deleted |  -  |
**400** | Service error relating to creating api_key for the organization |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_organization**
> OrganizationWithSubAndPlan get_organization(tr_organization, organization_id)

Get Organization

Fetch the details of an organization by its id. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.organization_with_sub_and_plan import OrganizationWithSubAndPlan
from trieve_py_client.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to https://api.trieve.ai
# See configuration.py for a list of all supported configuration parameters.
configuration = trieve_py_client.Configuration(
    host = "https://api.trieve.ai"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: ApiKey
configuration.api_key['ApiKey'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['ApiKey'] = 'Bearer'

# Enter a context with an instance of the API client
with trieve_py_client.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = trieve_py_client.OrganizationApi(api_client)
    tr_organization = 'tr_organization_example' # str | The organization id to use for the request
    organization_id = 'organization_id_example' # str | The id of the organization you want to fetch.

    try:
        # Get Organization
        api_response = api_instance.get_organization(tr_organization, organization_id)
        print("The response of OrganizationApi->get_organization:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling OrganizationApi->get_organization: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_organization** | **str**| The organization id to use for the request | 
 **organization_id** | **str**| The id of the organization you want to fetch. | 

### Return type

[**OrganizationWithSubAndPlan**](OrganizationWithSubAndPlan.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Organization with the id that was requested |  -  |
**400** | Service error relating to finding the organization by id |  -  |
**404** | Organization not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_organization_api_keys**
> List[ApiKeyRespBody] get_organization_api_keys(tr_organization)

Get Organization Api Keys

Get the api keys which belong to the organization. The actual api key values are not returned, only the ids, names, and creation dates.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.api_key_resp_body import ApiKeyRespBody
from trieve_py_client.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to https://api.trieve.ai
# See configuration.py for a list of all supported configuration parameters.
configuration = trieve_py_client.Configuration(
    host = "https://api.trieve.ai"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: ApiKey
configuration.api_key['ApiKey'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['ApiKey'] = 'Bearer'

# Enter a context with an instance of the API client
with trieve_py_client.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = trieve_py_client.OrganizationApi(api_client)
    tr_organization = 'tr_organization_example' # str | The organization id to use for the request.

    try:
        # Get Organization Api Keys
        api_response = api_instance.get_organization_api_keys(tr_organization)
        print("The response of OrganizationApi->get_organization_api_keys:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling OrganizationApi->get_organization_api_keys: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_organization** | **str**| The organization id to use for the request. | 

### Return type

[**List[ApiKeyRespBody]**](ApiKeyRespBody.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | JSON body representing the api_key for the organization |  -  |
**400** | Service error relating to creating api_key for the organization |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_organization_usage**
> OrganizationUsageCount get_organization_usage(tr_organization, organization_id)

Get Organization Usage

Fetch the current usage specification of an organization by its id. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.organization_usage_count import OrganizationUsageCount
from trieve_py_client.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to https://api.trieve.ai
# See configuration.py for a list of all supported configuration parameters.
configuration = trieve_py_client.Configuration(
    host = "https://api.trieve.ai"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: ApiKey
configuration.api_key['ApiKey'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['ApiKey'] = 'Bearer'

# Enter a context with an instance of the API client
with trieve_py_client.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = trieve_py_client.OrganizationApi(api_client)
    tr_organization = 'tr_organization_example' # str | The organization id to use for the request
    organization_id = 'organization_id_example' # str | The id of the organization you want to fetch the usage of.

    try:
        # Get Organization Usage
        api_response = api_instance.get_organization_usage(tr_organization, organization_id)
        print("The response of OrganizationApi->get_organization_usage:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling OrganizationApi->get_organization_usage: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_organization** | **str**| The organization id to use for the request | 
 **organization_id** | **str**| The id of the organization you want to fetch the usage of. | 

### Return type

[**OrganizationUsageCount**](OrganizationUsageCount.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | The current usage of the specified organization |  -  |
**400** | Service error relating to finding the organization&#39;s usage by id |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_organization_users**
> List[SlimUser] get_organization_users(tr_organization, organization_id)

Get Organization Users

Fetch the users of an organization by its id. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.slim_user import SlimUser
from trieve_py_client.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to https://api.trieve.ai
# See configuration.py for a list of all supported configuration parameters.
configuration = trieve_py_client.Configuration(
    host = "https://api.trieve.ai"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: ApiKey
configuration.api_key['ApiKey'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['ApiKey'] = 'Bearer'

# Enter a context with an instance of the API client
with trieve_py_client.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = trieve_py_client.OrganizationApi(api_client)
    tr_organization = 'tr_organization_example' # str | The organization id to use for the request
    organization_id = 'organization_id_example' # str | The id of the organization you want to fetch the users of.

    try:
        # Get Organization Users
        api_response = api_instance.get_organization_users(tr_organization, organization_id)
        print("The response of OrganizationApi->get_organization_users:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling OrganizationApi->get_organization_users: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_organization** | **str**| The organization id to use for the request | 
 **organization_id** | **str**| The id of the organization you want to fetch the users of. | 

### Return type

[**List[SlimUser]**](SlimUser.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Array of users who belong to the specified by organization |  -  |
**400** | Service error relating to finding the organization&#39;s users by id |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **update_all_org_dataset_configs**
> update_all_org_dataset_configs(tr_organization, update_all_org_dataset_configs_req_payload)

Update All Dataset Configurations

Update the configurations for all datasets in an organization. Only the specified keys in the configuration object will be changed per dataset such that you can preserve dataset unique values. Auth'ed user or api key must have an owner role for the specified organization.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.update_all_org_dataset_configs_req_payload import UpdateAllOrgDatasetConfigsReqPayload
from trieve_py_client.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to https://api.trieve.ai
# See configuration.py for a list of all supported configuration parameters.
configuration = trieve_py_client.Configuration(
    host = "https://api.trieve.ai"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: ApiKey
configuration.api_key['ApiKey'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['ApiKey'] = 'Bearer'

# Enter a context with an instance of the API client
with trieve_py_client.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = trieve_py_client.OrganizationApi(api_client)
    tr_organization = 'tr_organization_example' # str | The organization id to use for the request
    update_all_org_dataset_configs_req_payload = trieve_py_client.UpdateAllOrgDatasetConfigsReqPayload() # UpdateAllOrgDatasetConfigsReqPayload | The organization data that you want to create

    try:
        # Update All Dataset Configurations
        api_instance.update_all_org_dataset_configs(tr_organization, update_all_org_dataset_configs_req_payload)
    except Exception as e:
        print("Exception when calling OrganizationApi->update_all_org_dataset_configs: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_organization** | **str**| The organization id to use for the request | 
 **update_all_org_dataset_configs_req_payload** | [**UpdateAllOrgDatasetConfigsReqPayload**](UpdateAllOrgDatasetConfigsReqPayload.md)| The organization data that you want to create | 

### Return type

void (empty response body)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**204** | Confirmation that the dataset ServerConfigurations were updated successfully |  -  |
**400** | Service error relating to updating the dataset ServerConfigurations |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **update_organization**
> Organization update_organization(tr_organization, update_organization_req_payload)

Update Organization

Update an organization. Only the owner of the organization can update it.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.organization import Organization
from trieve_py_client.models.update_organization_req_payload import UpdateOrganizationReqPayload
from trieve_py_client.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to https://api.trieve.ai
# See configuration.py for a list of all supported configuration parameters.
configuration = trieve_py_client.Configuration(
    host = "https://api.trieve.ai"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: ApiKey
configuration.api_key['ApiKey'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['ApiKey'] = 'Bearer'

# Enter a context with an instance of the API client
with trieve_py_client.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = trieve_py_client.OrganizationApi(api_client)
    tr_organization = 'tr_organization_example' # str | The organization id to use for the request
    update_organization_req_payload = trieve_py_client.UpdateOrganizationReqPayload() # UpdateOrganizationReqPayload | The organization data that you want to update

    try:
        # Update Organization
        api_response = api_instance.update_organization(tr_organization, update_organization_req_payload)
        print("The response of OrganizationApi->update_organization:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling OrganizationApi->update_organization: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_organization** | **str**| The organization id to use for the request | 
 **update_organization_req_payload** | [**UpdateOrganizationReqPayload**](UpdateOrganizationReqPayload.md)| The organization data that you want to update | 

### Return type

[**Organization**](Organization.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Updated organization object |  -  |
**400** | Service error relating to updating the organization |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

