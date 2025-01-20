# trieve_py_client.ChunkGroupApi

All URIs are relative to *https://api.trieve.ai*

Method | HTTP request | Description
------------- | ------------- | -------------
[**add_chunk_to_group**](ChunkGroupApi.md#add_chunk_to_group) | **POST** /api/chunk_group/chunk/{group_id} | Add Chunk to Group
[**add_chunk_to_group_by_tracking_id**](ChunkGroupApi.md#add_chunk_to_group_by_tracking_id) | **POST** /api/chunk_group/tracking_id/{tracking_id} | Add Chunk to Group by Tracking ID
[**count_group_chunks**](ChunkGroupApi.md#count_group_chunks) | **POST** /api/chunk_group/count | Count Chunks in a Group
[**create_chunk_group**](ChunkGroupApi.md#create_chunk_group) | **POST** /api/chunk_group | Create or Upsert Group or Groups
[**delete_chunk_group**](ChunkGroupApi.md#delete_chunk_group) | **DELETE** /api/chunk_group/{group_id} | Delete Group
[**delete_group_by_tracking_id**](ChunkGroupApi.md#delete_group_by_tracking_id) | **DELETE** /api/chunk_group/tracking_id/{tracking_id} | Delete Group by Tracking ID
[**get_chunk_group**](ChunkGroupApi.md#get_chunk_group) | **GET** /api/chunk_group/{group_id} | Get Group
[**get_chunks_in_group**](ChunkGroupApi.md#get_chunks_in_group) | **GET** /api/chunk_group/{group_id}/{page} | Get Chunks in Group
[**get_chunks_in_group_by_tracking_id**](ChunkGroupApi.md#get_chunks_in_group_by_tracking_id) | **GET** /api/chunk_group/tracking_id/{group_tracking_id}/{page} | Get Chunks in Group by Tracking ID
[**get_group_by_tracking_id**](ChunkGroupApi.md#get_group_by_tracking_id) | **GET** /api/chunk_group/tracking_id/{tracking_id} | Get Group by Tracking ID
[**get_groups_for_chunks**](ChunkGroupApi.md#get_groups_for_chunks) | **POST** /api/chunk_group/chunks | Get Groups for Chunks
[**get_groups_for_dataset**](ChunkGroupApi.md#get_groups_for_dataset) | **GET** /api/dataset/groups/{dataset_id}/{page} | Get Groups for Dataset
[**get_recommended_groups**](ChunkGroupApi.md#get_recommended_groups) | **POST** /api/chunk_group/recommend | Get Recommended Groups
[**remove_chunk_from_group**](ChunkGroupApi.md#remove_chunk_from_group) | **DELETE** /api/chunk_group/chunk/{group_id} | Remove Chunk from Group
[**search_over_groups**](ChunkGroupApi.md#search_over_groups) | **POST** /api/chunk_group/group_oriented_search | Search Over Groups
[**search_within_group**](ChunkGroupApi.md#search_within_group) | **POST** /api/chunk_group/search | Search Within Group
[**update_chunk_group**](ChunkGroupApi.md#update_chunk_group) | **PUT** /api/chunk_group | Update Group


# **add_chunk_to_group**
> add_chunk_to_group(tr_dataset, group_id, add_chunk_to_group_req_payload)

Add Chunk to Group

Route to add a chunk to a group. One of chunk_id or chunk_tracking_id must be provided. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.add_chunk_to_group_req_payload import AddChunkToGroupReqPayload
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
    api_instance = trieve_py_client.ChunkGroupApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    group_id = 'group_id_example' # str | Id of the group to add the chunk to as a bookmark
    add_chunk_to_group_req_payload = trieve_py_client.AddChunkToGroupReqPayload() # AddChunkToGroupReqPayload | JSON request payload to add a chunk to a group (bookmark it)

    try:
        # Add Chunk to Group
        api_instance.add_chunk_to_group(tr_dataset, group_id, add_chunk_to_group_req_payload)
    except Exception as e:
        print("Exception when calling ChunkGroupApi->add_chunk_to_group: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **group_id** | **str**| Id of the group to add the chunk to as a bookmark | 
 **add_chunk_to_group_req_payload** | [**AddChunkToGroupReqPayload**](AddChunkToGroupReqPayload.md)| JSON request payload to add a chunk to a group (bookmark it) | 

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
**204** | Confirmation that the chunk was added to the group (bookmark&#39;ed). |  -  |
**400** | Service error relating to getting the groups that the chunk is in. |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **add_chunk_to_group_by_tracking_id**
> add_chunk_to_group_by_tracking_id(tr_dataset, tracking_id, add_chunk_to_group_req_payload)

Add Chunk to Group by Tracking ID

Route to add a chunk to a group by tracking id. One of chunk_id or chunk_tracking_id must be provided. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.add_chunk_to_group_req_payload import AddChunkToGroupReqPayload
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
    api_instance = trieve_py_client.ChunkGroupApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    tracking_id = 'tracking_id_example' # str | Tracking id of the group to add the chunk to as a bookmark
    add_chunk_to_group_req_payload = trieve_py_client.AddChunkToGroupReqPayload() # AddChunkToGroupReqPayload | JSON request payload to add a chunk to a group via tracking_id

    try:
        # Add Chunk to Group by Tracking ID
        api_instance.add_chunk_to_group_by_tracking_id(tr_dataset, tracking_id, add_chunk_to_group_req_payload)
    except Exception as e:
        print("Exception when calling ChunkGroupApi->add_chunk_to_group_by_tracking_id: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **tracking_id** | **str**| Tracking id of the group to add the chunk to as a bookmark | 
 **add_chunk_to_group_req_payload** | [**AddChunkToGroupReqPayload**](AddChunkToGroupReqPayload.md)| JSON request payload to add a chunk to a group via tracking_id | 

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
**204** | Confirmation that the chunk was added to the group |  -  |
**400** | Service error related to adding the chunk group by tracking_id |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **count_group_chunks**
> GetChunkGroupCountResponse count_group_chunks(tr_dataset, get_chunk_group_count_request)

Count Chunks in a Group

Route to get the number of chunks that is in a group

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.get_chunk_group_count_request import GetChunkGroupCountRequest
from trieve_py_client.models.get_chunk_group_count_response import GetChunkGroupCountResponse
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
    api_instance = trieve_py_client.ChunkGroupApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    get_chunk_group_count_request = trieve_py_client.GetChunkGroupCountRequest() # GetChunkGroupCountRequest | JSON request payload to add a chunk to a group (bookmark it)

    try:
        # Count Chunks in a Group
        api_response = api_instance.count_group_chunks(tr_dataset, get_chunk_group_count_request)
        print("The response of ChunkGroupApi->count_group_chunks:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling ChunkGroupApi->count_group_chunks: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **get_chunk_group_count_request** | [**GetChunkGroupCountRequest**](GetChunkGroupCountRequest.md)| JSON request payload to add a chunk to a group (bookmark it) | 

### Return type

[**GetChunkGroupCountResponse**](GetChunkGroupCountResponse.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | JSON body representing the group with the count |  -  |
**400** | Service error relating to getting the group with the given tracking id |  -  |
**404** | Group not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **create_chunk_group**
> CreateChunkGroupResponseEnum create_chunk_group(tr_dataset, create_chunk_group_req_payload_enum)

Create or Upsert Group or Groups

Create new chunk_group(s). This is a way to group chunks together. If you try to create a chunk_group with the same tracking_id as an existing chunk_group, this operation will fail. Only 1000 chunk groups can be created at a time. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.create_chunk_group_req_payload_enum import CreateChunkGroupReqPayloadEnum
from trieve_py_client.models.create_chunk_group_response_enum import CreateChunkGroupResponseEnum
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
    api_instance = trieve_py_client.ChunkGroupApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    create_chunk_group_req_payload_enum = trieve_py_client.CreateChunkGroupReqPayloadEnum() # CreateChunkGroupReqPayloadEnum | JSON request payload to cretea a chunk_group(s)

    try:
        # Create or Upsert Group or Groups
        api_response = api_instance.create_chunk_group(tr_dataset, create_chunk_group_req_payload_enum)
        print("The response of ChunkGroupApi->create_chunk_group:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling ChunkGroupApi->create_chunk_group: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **create_chunk_group_req_payload_enum** | [**CreateChunkGroupReqPayloadEnum**](CreateChunkGroupReqPayloadEnum.md)| JSON request payload to cretea a chunk_group(s) | 

### Return type

[**CreateChunkGroupResponseEnum**](CreateChunkGroupResponseEnum.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Returns the created chunk_group if a single chunk_group was specified or an array of all chunk_groups which were created |  -  |
**400** | Service error relating to creating the chunk_group(s) |  -  |
**413** | Service error indicating more 1000 chunk groups are trying to be created at once |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **delete_chunk_group**
> delete_chunk_group(tr_dataset, group_id, delete_chunks)

Delete Group

This will delete a chunk_group. If you set delete_chunks to true, it will also delete the chunks within the group. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.

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
    api_instance = trieve_py_client.ChunkGroupApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    group_id = 'group_id_example' # str | Id of the group you want to fetch.
    delete_chunks = True # bool | Delete the chunks within the group

    try:
        # Delete Group
        api_instance.delete_chunk_group(tr_dataset, group_id, delete_chunks)
    except Exception as e:
        print("Exception when calling ChunkGroupApi->delete_chunk_group: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **group_id** | **str**| Id of the group you want to fetch. | 
 **delete_chunks** | **bool**| Delete the chunks within the group | 

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
**204** | Confirmation that the chunkGroup was deleted |  -  |
**400** | Service error relating to deleting the chunkGroup |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **delete_group_by_tracking_id**
> delete_group_by_tracking_id(tr_dataset, tracking_id, delete_chunks)

Delete Group by Tracking ID

Delete a chunk_group with the given tracking id. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.

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
    api_instance = trieve_py_client.ChunkGroupApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    tracking_id = 'tracking_id_example' # str | Tracking id of the chunk_group to delete
    delete_chunks = True # bool | Delete the chunks within the group

    try:
        # Delete Group by Tracking ID
        api_instance.delete_group_by_tracking_id(tr_dataset, tracking_id, delete_chunks)
    except Exception as e:
        print("Exception when calling ChunkGroupApi->delete_group_by_tracking_id: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **tracking_id** | **str**| Tracking id of the chunk_group to delete | 
 **delete_chunks** | **bool**| Delete the chunks within the group | 

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
**204** | Confirmation that the chunkGroup was deleted |  -  |
**400** | Service error relating to deleting the chunkGroup |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_chunk_group**
> ChunkGroupAndFileId get_chunk_group(tr_dataset, group_id)

Get Group

Fetch the group with the given id.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.chunk_group_and_file_id import ChunkGroupAndFileId
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
    api_instance = trieve_py_client.ChunkGroupApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    group_id = 'group_id_example' # str | Id of the group you want to fetch.

    try:
        # Get Group
        api_response = api_instance.get_chunk_group(tr_dataset, group_id)
        print("The response of ChunkGroupApi->get_chunk_group:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling ChunkGroupApi->get_chunk_group: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **group_id** | **str**| Id of the group you want to fetch. | 

### Return type

[**ChunkGroupAndFileId**](ChunkGroupAndFileId.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | JSON body representing the group with the given tracking id |  -  |
**400** | Service error relating to getting the group with the given tracking id |  -  |
**404** | Group not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_chunks_in_group**
> GetChunksInGroupResponse get_chunks_in_group(tr_dataset, group_id, page, x_api_version=x_api_version)

Get Chunks in Group

Route to get all chunks for a group. The response is paginated, with each page containing 10 chunks. Page is 1-indexed.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.get_chunks_in_group_response import GetChunksInGroupResponse
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
    api_instance = trieve_py_client.ChunkGroupApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    group_id = 'group_id_example' # str | Id of the group you want to fetch.
    page = 56 # int | The page of chunks to get from the group
    x_api_version = trieve_py_client.APIVersion() # APIVersion | The version of the API to use for the request (optional)

    try:
        # Get Chunks in Group
        api_response = api_instance.get_chunks_in_group(tr_dataset, group_id, page, x_api_version=x_api_version)
        print("The response of ChunkGroupApi->get_chunks_in_group:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling ChunkGroupApi->get_chunks_in_group: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **group_id** | **str**| Id of the group you want to fetch. | 
 **page** | **int**| The page of chunks to get from the group | 
 **x_api_version** | [**APIVersion**](.md)| The version of the API to use for the request | [optional] 

### Return type

[**GetChunksInGroupResponse**](GetChunksInGroupResponse.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Chunks present within the specified group |  -  |
**400** | Service error relating to getting the groups that the chunk is in |  -  |
**404** | Group not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_chunks_in_group_by_tracking_id**
> GetChunksInGroupResponse get_chunks_in_group_by_tracking_id(tr_dataset, group_tracking_id, page, x_api_version=x_api_version)

Get Chunks in Group by Tracking ID

Route to get all chunks for a group. The response is paginated, with each page containing 10 chunks. Support for custom page size is coming soon. Page is 1-indexed.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.get_chunks_in_group_response import GetChunksInGroupResponse
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
    api_instance = trieve_py_client.ChunkGroupApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    group_tracking_id = 'group_tracking_id_example' # str | The id of the group to get the chunks from
    page = 56 # int | The page of chunks to get from the group
    x_api_version = trieve_py_client.APIVersion() # APIVersion | The version of the API to use for the request (optional)

    try:
        # Get Chunks in Group by Tracking ID
        api_response = api_instance.get_chunks_in_group_by_tracking_id(tr_dataset, group_tracking_id, page, x_api_version=x_api_version)
        print("The response of ChunkGroupApi->get_chunks_in_group_by_tracking_id:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling ChunkGroupApi->get_chunks_in_group_by_tracking_id: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **group_tracking_id** | **str**| The id of the group to get the chunks from | 
 **page** | **int**| The page of chunks to get from the group | 
 **x_api_version** | [**APIVersion**](.md)| The version of the API to use for the request | [optional] 

### Return type

[**GetChunksInGroupResponse**](GetChunksInGroupResponse.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Chunks present within the specified group |  -  |
**400** | Service error relating to getting the groups that the chunk is in |  -  |
**404** | Group not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_group_by_tracking_id**
> ChunkGroupAndFileId get_group_by_tracking_id(tr_dataset, tracking_id)

Get Group by Tracking ID

Fetch the group with the given tracking id. get_group_by_tracking_id

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.chunk_group_and_file_id import ChunkGroupAndFileId
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
    api_instance = trieve_py_client.ChunkGroupApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    tracking_id = 'tracking_id_example' # str | The tracking id of the group to fetch.

    try:
        # Get Group by Tracking ID
        api_response = api_instance.get_group_by_tracking_id(tr_dataset, tracking_id)
        print("The response of ChunkGroupApi->get_group_by_tracking_id:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling ChunkGroupApi->get_group_by_tracking_id: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **tracking_id** | **str**| The tracking id of the group to fetch. | 

### Return type

[**ChunkGroupAndFileId**](ChunkGroupAndFileId.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | JSON body representing the group with the given tracking id |  -  |
**400** | Service error relating to getting the group with the given tracking id |  -  |
**404** | Group not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_groups_for_chunks**
> List[GroupsForChunk] get_groups_for_chunks(tr_dataset, get_groups_for_chunks_req_payload)

Get Groups for Chunks

Route to get the groups that a chunk is in.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.get_groups_for_chunks_req_payload import GetGroupsForChunksReqPayload
from trieve_py_client.models.groups_for_chunk import GroupsForChunk
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
    api_instance = trieve_py_client.ChunkGroupApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    get_groups_for_chunks_req_payload = trieve_py_client.GetGroupsForChunksReqPayload() # GetGroupsForChunksReqPayload | JSON request payload to get the groups that a chunk is in

    try:
        # Get Groups for Chunks
        api_response = api_instance.get_groups_for_chunks(tr_dataset, get_groups_for_chunks_req_payload)
        print("The response of ChunkGroupApi->get_groups_for_chunks:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling ChunkGroupApi->get_groups_for_chunks: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **get_groups_for_chunks_req_payload** | [**GetGroupsForChunksReqPayload**](GetGroupsForChunksReqPayload.md)| JSON request payload to get the groups that a chunk is in | 

### Return type

[**List[GroupsForChunk]**](GroupsForChunk.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | JSON body representing the groups that the chunk is in |  -  |
**400** | Service error relating to getting the groups that the chunk is in |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_groups_for_dataset**
> GroupData get_groups_for_dataset(tr_dataset, dataset_id, page)

Get Groups for Dataset

Fetch the groups which belong to a dataset specified by its id.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.group_data import GroupData
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
    api_instance = trieve_py_client.ChunkGroupApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    dataset_id = 'dataset_id_example' # str | The id of the dataset to fetch groups for.
    page = 56 # int | The page of groups to fetch. Page is 1-indexed.

    try:
        # Get Groups for Dataset
        api_response = api_instance.get_groups_for_dataset(tr_dataset, dataset_id, page)
        print("The response of ChunkGroupApi->get_groups_for_dataset:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling ChunkGroupApi->get_groups_for_dataset: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **dataset_id** | **str**| The id of the dataset to fetch groups for. | 
 **page** | **int**| The page of groups to fetch. Page is 1-indexed. | 

### Return type

[**GroupData**](GroupData.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | JSON body representing the groups created by the given dataset |  -  |
**400** | Service error relating to getting the groups created by the given dataset |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_recommended_groups**
> RecommendGroupsResponse get_recommended_groups(tr_dataset, recommend_groups_req_payload, x_api_version=x_api_version)

Get Recommended Groups

Route to get recommended groups. This route will return groups which are similar to the groups in the request body. You must provide at least one positive group id or group tracking id.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.recommend_groups_req_payload import RecommendGroupsReqPayload
from trieve_py_client.models.recommend_groups_response import RecommendGroupsResponse
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
    api_instance = trieve_py_client.ChunkGroupApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    recommend_groups_req_payload = trieve_py_client.RecommendGroupsReqPayload() # RecommendGroupsReqPayload | JSON request payload to get recommendations of chunks similar to the chunks in the request
    x_api_version = trieve_py_client.APIVersion() # APIVersion | The API version to use for this request. Defaults to V2 for orgs created after July 12, 2024 and V1 otherwise. (optional)

    try:
        # Get Recommended Groups
        api_response = api_instance.get_recommended_groups(tr_dataset, recommend_groups_req_payload, x_api_version=x_api_version)
        print("The response of ChunkGroupApi->get_recommended_groups:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling ChunkGroupApi->get_recommended_groups: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **recommend_groups_req_payload** | [**RecommendGroupsReqPayload**](RecommendGroupsReqPayload.md)| JSON request payload to get recommendations of chunks similar to the chunks in the request | 
 **x_api_version** | [**APIVersion**](.md)| The API version to use for this request. Defaults to V2 for orgs created after July 12, 2024 and V1 otherwise. | [optional] 

### Return type

[**RecommendGroupsResponse**](RecommendGroupsResponse.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | JSON body representing the groups which are similar to the positive groups and dissimilar to the negative ones |  -  |
**400** | Service error relating to to getting similar chunks |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **remove_chunk_from_group**
> remove_chunk_from_group(tr_dataset, group_id, chunk_id=chunk_id, remove_chunk_from_group_req_payload=remove_chunk_from_group_req_payload)

Remove Chunk from Group

Route to remove a chunk from a group. Auth'ed user or api key must be an admin or owner of the dataset's organization to remove a chunk from a group.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.remove_chunk_from_group_req_payload import RemoveChunkFromGroupReqPayload
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
    api_instance = trieve_py_client.ChunkGroupApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    group_id = 'group_id_example' # str | Id of the group you want to remove the chunk from.
    chunk_id = 'chunk_id_example' # str | Id of the chunk you want to remove from the group (optional)
    remove_chunk_from_group_req_payload = trieve_py_client.RemoveChunkFromGroupReqPayload() # RemoveChunkFromGroupReqPayload | JSON request payload to remove a chunk from a group (optional)

    try:
        # Remove Chunk from Group
        api_instance.remove_chunk_from_group(tr_dataset, group_id, chunk_id=chunk_id, remove_chunk_from_group_req_payload=remove_chunk_from_group_req_payload)
    except Exception as e:
        print("Exception when calling ChunkGroupApi->remove_chunk_from_group: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **group_id** | **str**| Id of the group you want to remove the chunk from. | 
 **chunk_id** | **str**| Id of the chunk you want to remove from the group | [optional] 
 **remove_chunk_from_group_req_payload** | [**RemoveChunkFromGroupReqPayload**](RemoveChunkFromGroupReqPayload.md)| JSON request payload to remove a chunk from a group | [optional] 

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
**204** | Confirmation that the chunk was removed to the group |  -  |
**400** | Service error relating to removing the chunk from the group |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **search_over_groups**
> SearchOverGroupsResponseTypes search_over_groups(tr_dataset, search_over_groups_req_payload, x_api_version=x_api_version)

Search Over Groups

This route allows you to get groups as results instead of chunks. Each group returned will have the matching chunks sorted by similarity within the group. This is useful for when you want to get groups of chunks which are similar to the search query. If choosing hybrid search, the top chunk of each group will be re-ranked using scores from a cross encoder model. Compatible with semantic, fulltext, or hybrid search modes.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.search_over_groups_req_payload import SearchOverGroupsReqPayload
from trieve_py_client.models.search_over_groups_response_types import SearchOverGroupsResponseTypes
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
    api_instance = trieve_py_client.ChunkGroupApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    search_over_groups_req_payload = trieve_py_client.SearchOverGroupsReqPayload() # SearchOverGroupsReqPayload | JSON request payload to semantically search over groups
    x_api_version = trieve_py_client.APIVersion() # APIVersion | The API version to use for this request. Defaults to V2 for orgs created after July 12, 2024 and V1 otherwise. (optional)

    try:
        # Search Over Groups
        api_response = api_instance.search_over_groups(tr_dataset, search_over_groups_req_payload, x_api_version=x_api_version)
        print("The response of ChunkGroupApi->search_over_groups:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling ChunkGroupApi->search_over_groups: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **search_over_groups_req_payload** | [**SearchOverGroupsReqPayload**](SearchOverGroupsReqPayload.md)| JSON request payload to semantically search over groups | 
 **x_api_version** | [**APIVersion**](.md)| The API version to use for this request. Defaults to V2 for orgs created after July 12, 2024 and V1 otherwise. | [optional] 

### Return type

[**SearchOverGroupsResponseTypes**](SearchOverGroupsResponseTypes.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Group chunks which are similar to the embedding vector of the search query |  -  |
**400** | Service error relating to searching over groups |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **search_within_group**
> SearchGroupResponseTypes search_within_group(tr_dataset, search_within_group_req_payload, x_api_version=x_api_version)

Search Within Group

This route allows you to search only within a group. This is useful for when you only want search results to contain chunks which are members of a specific group. If choosing hybrid search, the results will be re-ranked using scores from a cross encoder model.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.search_group_response_types import SearchGroupResponseTypes
from trieve_py_client.models.search_within_group_req_payload import SearchWithinGroupReqPayload
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
    api_instance = trieve_py_client.ChunkGroupApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    search_within_group_req_payload = trieve_py_client.SearchWithinGroupReqPayload() # SearchWithinGroupReqPayload | JSON request payload to semantically search a group
    x_api_version = trieve_py_client.APIVersion() # APIVersion | The API version to use for this request. Defaults to V2 for orgs created after July 12, 2024 and V1 otherwise. (optional)

    try:
        # Search Within Group
        api_response = api_instance.search_within_group(tr_dataset, search_within_group_req_payload, x_api_version=x_api_version)
        print("The response of ChunkGroupApi->search_within_group:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling ChunkGroupApi->search_within_group: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **search_within_group_req_payload** | [**SearchWithinGroupReqPayload**](SearchWithinGroupReqPayload.md)| JSON request payload to semantically search a group | 
 **x_api_version** | [**APIVersion**](.md)| The API version to use for this request. Defaults to V2 for orgs created after July 12, 2024 and V1 otherwise. | [optional] 

### Return type

[**SearchGroupResponseTypes**](SearchGroupResponseTypes.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Group chunks which are similar to the embedding vector of the search query |  -  |
**400** | Service error relating to getting the groups that the chunk is in |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **update_chunk_group**
> update_chunk_group(tr_dataset, update_chunk_group_req_payload)

Update Group

Update a chunk_group. One of group_id or tracking_id must be provided. If you try to change the tracking_id to one that already exists, this operation will fail. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.update_chunk_group_req_payload import UpdateChunkGroupReqPayload
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
    api_instance = trieve_py_client.ChunkGroupApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    update_chunk_group_req_payload = trieve_py_client.UpdateChunkGroupReqPayload() # UpdateChunkGroupReqPayload | JSON request payload to update a chunkGroup

    try:
        # Update Group
        api_instance.update_chunk_group(tr_dataset, update_chunk_group_req_payload)
    except Exception as e:
        print("Exception when calling ChunkGroupApi->update_chunk_group: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **update_chunk_group_req_payload** | [**UpdateChunkGroupReqPayload**](UpdateChunkGroupReqPayload.md)| JSON request payload to update a chunkGroup | 

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
**204** | Confirmation that the chunkGroup was updated |  -  |
**400** | Service error relating to updating the chunkGroup |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

