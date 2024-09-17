# trieve_py_client.DatasetApi

All URIs are relative to *https://api.trieve.ai*

Method | HTTP request | Description
------------- | ------------- | -------------
[**clear_dataset**](DatasetApi.md#clear_dataset) | **PUT** /api/dataset/clear/{dataset_id} | Clear Dataset
[**create_dataset**](DatasetApi.md#create_dataset) | **POST** /api/dataset | Create Dataset
[**delete_dataset**](DatasetApi.md#delete_dataset) | **DELETE** /api/dataset/{dataset_id} | Delete Dataset
[**delete_dataset_by_tracking_id**](DatasetApi.md#delete_dataset_by_tracking_id) | **DELETE** /api/dataset/tracking_id/{tracking_id} | Delete Dataset by Tracking ID
[**get_all_tags**](DatasetApi.md#get_all_tags) | **POST** /api/dataset/get_all_tags | Get All Tags
[**get_dataset**](DatasetApi.md#get_dataset) | **GET** /api/dataset/{dataset_id} | Get Dataset By ID
[**get_datasets_from_organization**](DatasetApi.md#get_datasets_from_organization) | **GET** /api/dataset/organization/{organization_id} | Get Datasets from Organization
[**get_usage_by_dataset_id**](DatasetApi.md#get_usage_by_dataset_id) | **GET** /api/dataset/usage/{dataset_id} | Get Usage By Dataset ID
[**update_dataset**](DatasetApi.md#update_dataset) | **PUT** /api/dataset | Update Dataset by ID or Tracking ID


# **clear_dataset**
> clear_dataset(tr_dataset, dataset_id)

Clear Dataset

Removes all chunks, files, and groups from the dataset while retaining the analytics and dataset itself. The auth'ed user must be an owner of the organization to clear a dataset.

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
    api_instance = trieve_py_client.DatasetApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    dataset_id = 'dataset_id_example' # str | The id of the dataset you want to clear.

    try:
        # Clear Dataset
        api_instance.clear_dataset(tr_dataset, dataset_id)
    except Exception as e:
        print("Exception when calling DatasetApi->clear_dataset: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **dataset_id** | **str**| The id of the dataset you want to clear. | 

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
**204** | Dataset cleared successfully |  -  |
**400** | Service error relating to deleting the dataset |  -  |
**404** | Dataset not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **create_dataset**
> Dataset create_dataset(tr_organization, create_dataset_request)

Create Dataset

Auth'ed user must be an owner of the organization to create a dataset.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.create_dataset_request import CreateDatasetRequest
from trieve_py_client.models.dataset import Dataset
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
    api_instance = trieve_py_client.DatasetApi(api_client)
    tr_organization = 'tr_organization_example' # str | The organization id to use for the request
    create_dataset_request = trieve_py_client.CreateDatasetRequest() # CreateDatasetRequest | JSON request payload to create a new dataset

    try:
        # Create Dataset
        api_response = api_instance.create_dataset(tr_organization, create_dataset_request)
        print("The response of DatasetApi->create_dataset:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling DatasetApi->create_dataset: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_organization** | **str**| The organization id to use for the request | 
 **create_dataset_request** | [**CreateDatasetRequest**](CreateDatasetRequest.md)| JSON request payload to create a new dataset | 

### Return type

[**Dataset**](Dataset.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Dataset created successfully |  -  |
**400** | Service error relating to creating the dataset |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **delete_dataset**
> delete_dataset(tr_dataset, dataset_id)

Delete Dataset

Auth'ed user must be an owner of the organization to delete a dataset.

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
    api_instance = trieve_py_client.DatasetApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    dataset_id = 'dataset_id_example' # str | The id of the dataset you want to delete.

    try:
        # Delete Dataset
        api_instance.delete_dataset(tr_dataset, dataset_id)
    except Exception as e:
        print("Exception when calling DatasetApi->delete_dataset: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **dataset_id** | **str**| The id of the dataset you want to delete. | 

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
**204** | Dataset deleted successfully |  -  |
**400** | Service error relating to deleting the dataset |  -  |
**404** | Dataset not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **delete_dataset_by_tracking_id**
> delete_dataset_by_tracking_id(tr_dataset, tracking_id)

Delete Dataset by Tracking ID

Auth'ed user must be an owner of the organization to delete a dataset.

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
    api_instance = trieve_py_client.DatasetApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    tracking_id = 'tracking_id_example' # str | The tracking id of the dataset you want to delete.

    try:
        # Delete Dataset by Tracking ID
        api_instance.delete_dataset_by_tracking_id(tr_dataset, tracking_id)
    except Exception as e:
        print("Exception when calling DatasetApi->delete_dataset_by_tracking_id: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **tracking_id** | **str**| The tracking id of the dataset you want to delete. | 

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
**204** | Dataset deleted successfully |  -  |
**400** | Service error relating to deleting the dataset |  -  |
**404** | Dataset not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_all_tags**
> GetAllTagsResponse get_all_tags(tr_dataset, get_all_tags_req_payload)

Get All Tags

Scroll through all tags in the dataset and get the number of chunks in the dataset with that tag plus the total number of unique tags for the whole datset.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.get_all_tags_req_payload import GetAllTagsReqPayload
from trieve_py_client.models.get_all_tags_response import GetAllTagsResponse
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
    api_instance = trieve_py_client.DatasetApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    get_all_tags_req_payload = trieve_py_client.GetAllTagsReqPayload() # GetAllTagsReqPayload | JSON request payload to get items with the tag in the request

    try:
        # Get All Tags
        api_response = api_instance.get_all_tags(tr_dataset, get_all_tags_req_payload)
        print("The response of DatasetApi->get_all_tags:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling DatasetApi->get_all_tags: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **get_all_tags_req_payload** | [**GetAllTagsReqPayload**](GetAllTagsReqPayload.md)| JSON request payload to get items with the tag in the request | 

### Return type

[**GetAllTagsResponse**](GetAllTagsResponse.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Page of tags requested with all tags and the number of chunks in the dataset with that tag plus the total number of unique tags for the whole datset |  -  |
**400** | Service error relating to finding items by tag |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_dataset**
> Dataset get_dataset(tr_dataset, dataset_id)

Get Dataset By ID

Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.dataset import Dataset
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
    api_instance = trieve_py_client.DatasetApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    dataset_id = 'dataset_id_example' # str | The id of the dataset you want to retrieve.

    try:
        # Get Dataset By ID
        api_response = api_instance.get_dataset(tr_dataset, dataset_id)
        print("The response of DatasetApi->get_dataset:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling DatasetApi->get_dataset: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **dataset_id** | **str**| The id of the dataset you want to retrieve. | 

### Return type

[**Dataset**](Dataset.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Dataset retrieved successfully |  -  |
**400** | Service error relating to retrieving the dataset |  -  |
**404** | Dataset not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_datasets_from_organization**
> List[DatasetAndUsage] get_datasets_from_organization(tr_organization, organization_id, limit=limit, offset=offset)

Get Datasets from Organization

Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.dataset_and_usage import DatasetAndUsage
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
    api_instance = trieve_py_client.DatasetApi(api_client)
    tr_organization = 'tr_organization_example' # str | The organization id to use for the request
    organization_id = 'organization_id_example' # str | id of the organization you want to retrieve datasets for
    limit = 56 # int | The number of records to return (optional)
    offset = 56 # int | The number of records to skip (optional)

    try:
        # Get Datasets from Organization
        api_response = api_instance.get_datasets_from_organization(tr_organization, organization_id, limit=limit, offset=offset)
        print("The response of DatasetApi->get_datasets_from_organization:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling DatasetApi->get_datasets_from_organization: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_organization** | **str**| The organization id to use for the request | 
 **organization_id** | **str**| id of the organization you want to retrieve datasets for | 
 **limit** | **int**| The number of records to return | [optional] 
 **offset** | **int**| The number of records to skip | [optional] 

### Return type

[**List[DatasetAndUsage]**](DatasetAndUsage.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Datasets retrieved successfully |  -  |
**400** | Service error relating to retrieving the dataset |  -  |
**404** | Could not find organization |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_usage_by_dataset_id**
> DatasetUsageCount get_usage_by_dataset_id(tr_dataset, dataset_id)

Get Usage By Dataset ID

Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.dataset_usage_count import DatasetUsageCount
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
    api_instance = trieve_py_client.DatasetApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    dataset_id = 'dataset_id_example' # str | The id of the dataset you want to retrieve usage for.

    try:
        # Get Usage By Dataset ID
        api_response = api_instance.get_usage_by_dataset_id(tr_dataset, dataset_id)
        print("The response of DatasetApi->get_usage_by_dataset_id:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling DatasetApi->get_usage_by_dataset_id: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **dataset_id** | **str**| The id of the dataset you want to retrieve usage for. | 

### Return type

[**DatasetUsageCount**](DatasetUsageCount.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Dataset usage retrieved successfully |  -  |
**400** | Service error relating to retrieving the dataset usage |  -  |
**404** | Dataset not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **update_dataset**
> Dataset update_dataset(tr_organization, update_dataset_request)

Update Dataset by ID or Tracking ID

One of id or tracking_id must be provided. The auth'ed user must be an owner of the organization to update a dataset.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.dataset import Dataset
from trieve_py_client.models.update_dataset_request import UpdateDatasetRequest
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
    api_instance = trieve_py_client.DatasetApi(api_client)
    tr_organization = 'tr_organization_example' # str | The organization id to use for the request
    update_dataset_request = trieve_py_client.UpdateDatasetRequest() # UpdateDatasetRequest | JSON request payload to update a dataset

    try:
        # Update Dataset by ID or Tracking ID
        api_response = api_instance.update_dataset(tr_organization, update_dataset_request)
        print("The response of DatasetApi->update_dataset:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling DatasetApi->update_dataset: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_organization** | **str**| The organization id to use for the request | 
 **update_dataset_request** | [**UpdateDatasetRequest**](UpdateDatasetRequest.md)| JSON request payload to update a dataset | 

### Return type

[**Dataset**](Dataset.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Dataset updated successfully |  -  |
**400** | Service error relating to updating the dataset |  -  |
**404** | Dataset not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

