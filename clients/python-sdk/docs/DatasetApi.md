# trieve_py_client.DatasetApi

All URIs are relative to *https://api.trieve.ai*

Method | HTTP request | Description
------------- | ------------- | -------------
[**batch_create_datasets**](DatasetApi.md#batch_create_datasets) | **POST** /api/dataset/batch_create_datasets | Batch Create Datasets
[**clear_dataset**](DatasetApi.md#clear_dataset) | **PUT** /api/dataset/clear/{dataset_id} | Clear Dataset
[**create_dataset**](DatasetApi.md#create_dataset) | **POST** /api/dataset | Create Dataset
[**create_etl_job**](DatasetApi.md#create_etl_job) | **POST** /api/etl/create_job | Create ETL Job
[**create_pagefind_index_for_dataset**](DatasetApi.md#create_pagefind_index_for_dataset) | **PUT** /api/dataset/pagefind | Create Pagefind Index for Dataset
[**delete_dataset**](DatasetApi.md#delete_dataset) | **DELETE** /api/dataset/{dataset_id} | Delete Dataset
[**delete_dataset_by_tracking_id**](DatasetApi.md#delete_dataset_by_tracking_id) | **DELETE** /api/dataset/tracking_id/{tracking_id} | Delete Dataset by Tracking ID
[**get_all_tags**](DatasetApi.md#get_all_tags) | **POST** /api/dataset/get_all_tags | Get All Tags
[**get_dataset**](DatasetApi.md#get_dataset) | **GET** /api/dataset/{dataset_id} | Get Dataset By ID
[**get_dataset_by_tracking_id**](DatasetApi.md#get_dataset_by_tracking_id) | **GET** /api/dataset/tracking_id/{tracking_id} | Get Dataset by Tracking ID
[**get_dataset_crawl_options**](DatasetApi.md#get_dataset_crawl_options) | **GET** /api/dataset/crawl_options/{dataset_id} | Get Dataset Crawl Options
[**get_datasets_from_organization**](DatasetApi.md#get_datasets_from_organization) | **GET** /api/dataset/organization/{organization_id} | Get Datasets from Organization
[**get_events**](DatasetApi.md#get_events) | **POST** /api/dataset/events | Get events for the dataset
[**get_pagefind_index_for_dataset**](DatasetApi.md#get_pagefind_index_for_dataset) | **GET** /api/dataset/pagefind | Get Pagefind Index Url for Dataset
[**get_usage_by_dataset_id**](DatasetApi.md#get_usage_by_dataset_id) | **GET** /api/dataset/usage/{dataset_id} | Get Usage By Dataset ID
[**update_dataset**](DatasetApi.md#update_dataset) | **PUT** /api/dataset | Update Dataset by ID or Tracking ID


# **batch_create_datasets**
> List[Dataset] batch_create_datasets(tr_organization, create_dataset_batch_req_payload)

Batch Create Datasets

Datasets will be created in the org specified via the TR-Organization header. Auth'ed user must be an owner of the organization to create datasets. If a tracking_id is ignored due to it already existing on the org, the response will not contain a dataset with that tracking_id and it can be assumed that a dataset with the missing tracking_id already exists.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.create_dataset_batch_req_payload import CreateDatasetBatchReqPayload
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
    create_dataset_batch_req_payload = trieve_py_client.CreateDatasetBatchReqPayload() # CreateDatasetBatchReqPayload | JSON request payload to bulk create datasets

    try:
        # Batch Create Datasets
        api_response = api_instance.batch_create_datasets(tr_organization, create_dataset_batch_req_payload)
        print("The response of DatasetApi->batch_create_datasets:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling DatasetApi->batch_create_datasets: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_organization** | **str**| The organization id to use for the request | 
 **create_dataset_batch_req_payload** | [**CreateDatasetBatchReqPayload**](CreateDatasetBatchReqPayload.md)| JSON request payload to bulk create datasets | 

### Return type

[**List[Dataset]**](Dataset.md)

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
> Dataset create_dataset(tr_organization, create_dataset_req_payload)

Create Dataset

Dataset will be created in the org specified via the TR-Organization header. Auth'ed user must be an owner of the organization to create a dataset.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.create_dataset_req_payload import CreateDatasetReqPayload
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
    create_dataset_req_payload = trieve_py_client.CreateDatasetReqPayload() # CreateDatasetReqPayload | JSON request payload to create a new dataset

    try:
        # Create Dataset
        api_response = api_instance.create_dataset(tr_organization, create_dataset_req_payload)
        print("The response of DatasetApi->create_dataset:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling DatasetApi->create_dataset: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_organization** | **str**| The organization id to use for the request | 
 **create_dataset_req_payload** | [**CreateDatasetReqPayload**](CreateDatasetReqPayload.md)| JSON request payload to create a new dataset | 

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

# **create_etl_job**
> create_etl_job(tr_dataset, create_schema_req_payload)

Create ETL Job

This endpoint is used to create a new ETL job for a dataset.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.create_schema_req_payload import CreateSchemaReqPayload
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
    tr_dataset = 'tr_dataset_example' # str | The dataset id to use for the request
    create_schema_req_payload = trieve_py_client.CreateSchemaReqPayload() # CreateSchemaReqPayload | JSON request payload to create a new ETL Job

    try:
        # Create ETL Job
        api_instance.create_etl_job(tr_dataset, create_schema_req_payload)
    except Exception as e:
        print("Exception when calling DatasetApi->create_etl_job: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id to use for the request | 
 **create_schema_req_payload** | [**CreateSchemaReqPayload**](CreateSchemaReqPayload.md)| JSON request payload to create a new ETL Job | 

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
**204** | ETL Job created successfully |  -  |
**400** | Service error relating to creating the dataset |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **create_pagefind_index_for_dataset**
> create_pagefind_index_for_dataset(tr_dataset)

Create Pagefind Index for Dataset

Uses pagefind to index the dataset and store the result into a CDN for retrieval. The auth'ed user must be an admin of the organization to create a pagefind index for a dataset.

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

    try:
        # Create Pagefind Index for Dataset
        api_instance.create_pagefind_index_for_dataset(tr_dataset)
    except Exception as e:
        print("Exception when calling DatasetApi->create_pagefind_index_for_dataset: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 

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
**204** | Dataset indexed successfully |  -  |
**400** | Service error relating to creating the index |  -  |

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

# **get_dataset_by_tracking_id**
> Dataset get_dataset_by_tracking_id(tr_organization, tracking_id)

Get Dataset by Tracking ID

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
    tr_organization = 'tr_organization_example' # str | The organization id to use for the request
    tracking_id = 'tracking_id_example' # str | The tracking id of the dataset you want to retrieve.

    try:
        # Get Dataset by Tracking ID
        api_response = api_instance.get_dataset_by_tracking_id(tr_organization, tracking_id)
        print("The response of DatasetApi->get_dataset_by_tracking_id:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling DatasetApi->get_dataset_by_tracking_id: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_organization** | **str**| The organization id to use for the request | 
 **tracking_id** | **str**| The tracking id of the dataset you want to retrieve. | 

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

# **get_dataset_crawl_options**
> GetCrawlOptionsResponse get_dataset_crawl_options(tr_dataset, dataset_id)

Get Dataset Crawl Options

Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.get_crawl_options_response import GetCrawlOptionsResponse
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
        # Get Dataset Crawl Options
        api_response = api_instance.get_dataset_crawl_options(tr_dataset, dataset_id)
        print("The response of DatasetApi->get_dataset_crawl_options:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling DatasetApi->get_dataset_crawl_options: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **dataset_id** | **str**| The id of the dataset you want to retrieve. | 

### Return type

[**GetCrawlOptionsResponse**](GetCrawlOptionsResponse.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Crawl options retrieved successfully |  -  |
**400** | Service error relating to retrieving the crawl options |  -  |
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

# **get_events**
> EventReturn get_events(tr_dataset, get_events_data)

Get events for the dataset

Get events for the dataset specified by the TR-Dataset header.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.event_return import EventReturn
from trieve_py_client.models.get_events_data import GetEventsData
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
    get_events_data = trieve_py_client.GetEventsData() # GetEventsData | JSON request payload to get events for a dataset

    try:
        # Get events for the dataset
        api_response = api_instance.get_events(tr_dataset, get_events_data)
        print("The response of DatasetApi->get_events:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling DatasetApi->get_events: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **get_events_data** | [**GetEventsData**](GetEventsData.md)| JSON request payload to get events for a dataset | 

### Return type

[**EventReturn**](EventReturn.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Events for the dataset |  -  |
**400** | Service error relating to getting events for the dataset |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_pagefind_index_for_dataset**
> GetPagefindIndexResponse get_pagefind_index_for_dataset(tr_dataset)

Get Pagefind Index Url for Dataset

Returns the root URL for your pagefind index, will error if pagefind is not enabled

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.get_pagefind_index_response import GetPagefindIndexResponse
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

    try:
        # Get Pagefind Index Url for Dataset
        api_response = api_instance.get_pagefind_index_for_dataset(tr_dataset)
        print("The response of DatasetApi->get_pagefind_index_for_dataset:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling DatasetApi->get_pagefind_index_for_dataset: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 

### Return type

[**GetPagefindIndexResponse**](GetPagefindIndexResponse.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Dataset indexed successfully |  -  |
**400** | Service error relating to creating the index |  -  |

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
> Dataset update_dataset(tr_organization, update_dataset_req_payload)

Update Dataset by ID or Tracking ID

One of id or tracking_id must be provided. The auth'ed user must be an owner of the organization to update a dataset.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.dataset import Dataset
from trieve_py_client.models.update_dataset_req_payload import UpdateDatasetReqPayload
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
    update_dataset_req_payload = trieve_py_client.UpdateDatasetReqPayload() # UpdateDatasetReqPayload | JSON request payload to update a dataset

    try:
        # Update Dataset by ID or Tracking ID
        api_response = api_instance.update_dataset(tr_organization, update_dataset_req_payload)
        print("The response of DatasetApi->update_dataset:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling DatasetApi->update_dataset: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_organization** | **str**| The organization id to use for the request | 
 **update_dataset_req_payload** | [**UpdateDatasetReqPayload**](UpdateDatasetReqPayload.md)| JSON request payload to update a dataset | 

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

