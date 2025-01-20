# trieve_py_client.ChunkApi

All URIs are relative to *https://api.trieve.ai*

Method | HTTP request | Description
------------- | ------------- | -------------
[**autocomplete**](ChunkApi.md#autocomplete) | **POST** /api/chunk/autocomplete | Autocomplete
[**bulk_delete_chunk**](ChunkApi.md#bulk_delete_chunk) | **DELETE** /api/chunk | Bulk Delete Chunks
[**count_chunks**](ChunkApi.md#count_chunks) | **POST** /api/chunk/count | Count chunks above threshold
[**create_chunk**](ChunkApi.md#create_chunk) | **POST** /api/chunk | Create or Upsert Chunk or Chunks
[**delete_chunk**](ChunkApi.md#delete_chunk) | **DELETE** /api/chunk/{chunk_id} | Delete Chunk
[**delete_chunk_by_tracking_id**](ChunkApi.md#delete_chunk_by_tracking_id) | **DELETE** /api/chunk/tracking_id/{tracking_id} | Delete Chunk By Tracking Id
[**generate_off_chunks**](ChunkApi.md#generate_off_chunks) | **POST** /api/chunk/generate | RAG on Specified Chunks
[**get_chunk_by_id**](ChunkApi.md#get_chunk_by_id) | **GET** /api/chunk/{chunk_id} | Get Chunk By Id
[**get_chunk_by_tracking_id**](ChunkApi.md#get_chunk_by_tracking_id) | **GET** /api/chunk/tracking_id/{tracking_id} | Get Chunk By Tracking Id
[**get_chunks_by_ids**](ChunkApi.md#get_chunks_by_ids) | **POST** /api/chunks | Get Chunks By Ids
[**get_chunks_by_tracking_ids**](ChunkApi.md#get_chunks_by_tracking_ids) | **POST** /api/chunks/tracking | Get Chunks By Tracking Ids
[**get_recommended_chunks**](ChunkApi.md#get_recommended_chunks) | **POST** /api/chunk/recommend | Get Recommended Chunks
[**get_suggested_queries**](ChunkApi.md#get_suggested_queries) | **POST** /api/chunk/suggestions | Generate suggested queries
[**scroll_dataset_chunks**](ChunkApi.md#scroll_dataset_chunks) | **POST** /api/chunks/scroll | Scroll Chunks
[**search_chunks**](ChunkApi.md#search_chunks) | **POST** /api/chunk/search | Search
[**split_html_content**](ChunkApi.md#split_html_content) | **POST** /api/chunk/split | Split HTML Content into Chunks
[**update_chunk**](ChunkApi.md#update_chunk) | **PUT** /api/chunk | Update Chunk
[**update_chunk_by_tracking_id**](ChunkApi.md#update_chunk_by_tracking_id) | **PUT** /api/chunk/tracking_id/update | Update Chunk By Tracking Id


# **autocomplete**
> SearchResponseTypes autocomplete(tr_dataset, autocomplete_req_payload, x_api_version=x_api_version)

Autocomplete

This route provides the primary autocomplete functionality for the API. This prioritize prefix matching with semantic or full-text search.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.autocomplete_req_payload import AutocompleteReqPayload
from trieve_py_client.models.search_response_types import SearchResponseTypes
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
    api_instance = trieve_py_client.ChunkApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    autocomplete_req_payload = trieve_py_client.AutocompleteReqPayload() # AutocompleteReqPayload | JSON request payload to semantically search for chunks (chunks)
    x_api_version = trieve_py_client.APIVersion() # APIVersion | The API version to use for this request. Defaults to V2 for orgs created after July 12, 2024 and V1 otherwise. (optional)

    try:
        # Autocomplete
        api_response = api_instance.autocomplete(tr_dataset, autocomplete_req_payload, x_api_version=x_api_version)
        print("The response of ChunkApi->autocomplete:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling ChunkApi->autocomplete: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **autocomplete_req_payload** | [**AutocompleteReqPayload**](AutocompleteReqPayload.md)| JSON request payload to semantically search for chunks (chunks) | 
 **x_api_version** | [**APIVersion**](.md)| The API version to use for this request. Defaults to V2 for orgs created after July 12, 2024 and V1 otherwise. | [optional] 

### Return type

[**SearchResponseTypes**](SearchResponseTypes.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Chunks with embedding vectors which are similar to those in the request body |  -  |
**400** | Service error relating to searching |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **bulk_delete_chunk**
> bulk_delete_chunk(tr_dataset, bulk_delete_chunk_payload)

Bulk Delete Chunks

Delete multiple chunks using a filter. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.bulk_delete_chunk_payload import BulkDeleteChunkPayload
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
    api_instance = trieve_py_client.ChunkApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    bulk_delete_chunk_payload = trieve_py_client.BulkDeleteChunkPayload() # BulkDeleteChunkPayload | JSON request payload to speicy a filter to bulk delete chunks

    try:
        # Bulk Delete Chunks
        api_instance.bulk_delete_chunk(tr_dataset, bulk_delete_chunk_payload)
    except Exception as e:
        print("Exception when calling ChunkApi->bulk_delete_chunk: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **bulk_delete_chunk_payload** | [**BulkDeleteChunkPayload**](BulkDeleteChunkPayload.md)| JSON request payload to speicy a filter to bulk delete chunks | 

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
**204** | Confirmation that the chunk with the id specified was deleted |  -  |
**400** | Service error relating to finding a chunk by tracking_id |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **count_chunks**
> CountChunkQueryResponseBody count_chunks(tr_dataset, count_chunks_req_payload)

Count chunks above threshold

This route can be used to determine the number of chunk results that match a search query including score threshold and filters. It may be high latency for large limits. There is a dataset configuration imposed restriction on the maximum limit value (default 10,000) which is used to prevent DDOS attacks. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.count_chunk_query_response_body import CountChunkQueryResponseBody
from trieve_py_client.models.count_chunks_req_payload import CountChunksReqPayload
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
    api_instance = trieve_py_client.ChunkApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    count_chunks_req_payload = trieve_py_client.CountChunksReqPayload() # CountChunksReqPayload | JSON request payload to count chunks for a search query

    try:
        # Count chunks above threshold
        api_response = api_instance.count_chunks(tr_dataset, count_chunks_req_payload)
        print("The response of ChunkApi->count_chunks:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling ChunkApi->count_chunks: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **count_chunks_req_payload** | [**CountChunksReqPayload**](CountChunksReqPayload.md)| JSON request payload to count chunks for a search query | 

### Return type

[**CountChunkQueryResponseBody**](CountChunkQueryResponseBody.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Number of chunks satisfying the query |  -  |
**404** | Failed to count chunks |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **create_chunk**
> ReturnQueuedChunk create_chunk(tr_dataset, create_chunk_req_payload_enum)

Create or Upsert Chunk or Chunks

Create new chunk(s). If the chunk has the same tracking_id as an existing chunk, the request will fail. Once a chunk is created, it can be searched for using the search endpoint. If uploading in bulk, the maximum amount of chunks that can be uploaded at once is 120 chunks. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.create_chunk_req_payload_enum import CreateChunkReqPayloadEnum
from trieve_py_client.models.return_queued_chunk import ReturnQueuedChunk
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
    api_instance = trieve_py_client.ChunkApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    create_chunk_req_payload_enum = trieve_py_client.CreateChunkReqPayloadEnum() # CreateChunkReqPayloadEnum | JSON request payload to create a new chunk (chunk)

    try:
        # Create or Upsert Chunk or Chunks
        api_response = api_instance.create_chunk(tr_dataset, create_chunk_req_payload_enum)
        print("The response of ChunkApi->create_chunk:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling ChunkApi->create_chunk: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **create_chunk_req_payload_enum** | [**CreateChunkReqPayloadEnum**](CreateChunkReqPayloadEnum.md)| JSON request payload to create a new chunk (chunk) | 

### Return type

[**ReturnQueuedChunk**](ReturnQueuedChunk.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | JSON response payload containing the created chunk |  -  |
**400** | Error typically due to deserialization issues |  -  |
**413** | Error when more than 120 chunks are provided in bulk |  -  |
**426** | Error when upgrade is needed to process more chunks |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **delete_chunk**
> delete_chunk(tr_dataset, chunk_id)

Delete Chunk

Delete a chunk by its id. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.

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
    api_instance = trieve_py_client.ChunkApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    chunk_id = 'chunk_id_example' # str | Id of the chunk you want to fetch.

    try:
        # Delete Chunk
        api_instance.delete_chunk(tr_dataset, chunk_id)
    except Exception as e:
        print("Exception when calling ChunkApi->delete_chunk: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **chunk_id** | **str**| Id of the chunk you want to fetch. | 

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
**204** | Confirmation that the chunk with the id specified was deleted |  -  |
**400** | Service error relating to finding a chunk by tracking_id |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **delete_chunk_by_tracking_id**
> delete_chunk_by_tracking_id(tr_dataset, tracking_id)

Delete Chunk By Tracking Id

Delete a chunk by tracking_id. This is useful for when you are coordinating with an external system and want to use the tracking_id to identify the chunk. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.

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
    api_instance = trieve_py_client.ChunkApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    tracking_id = 'tracking_id_example' # str | tracking_id of the chunk you want to delete

    try:
        # Delete Chunk By Tracking Id
        api_instance.delete_chunk_by_tracking_id(tr_dataset, tracking_id)
    except Exception as e:
        print("Exception when calling ChunkApi->delete_chunk_by_tracking_id: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **tracking_id** | **str**| tracking_id of the chunk you want to delete | 

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
**204** | Confirmation that the chunk with the tracking_id specified was deleted |  -  |
**400** | Service error relating to finding a chunk by tracking_id |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **generate_off_chunks**
> str generate_off_chunks(tr_dataset, generate_off_chunks_req_payload)

RAG on Specified Chunks

This endpoint exists as an alternative to the topic+message resource pattern where our Trieve handles chat memory. With this endpoint, the user is responsible for providing the context window and the prompt and the conversation is ephemeral.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.generate_off_chunks_req_payload import GenerateOffChunksReqPayload
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
    api_instance = trieve_py_client.ChunkApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    generate_off_chunks_req_payload = trieve_py_client.GenerateOffChunksReqPayload() # GenerateOffChunksReqPayload | JSON request payload to perform RAG on some chunks (chunks)

    try:
        # RAG on Specified Chunks
        api_response = api_instance.generate_off_chunks(tr_dataset, generate_off_chunks_req_payload)
        print("The response of ChunkApi->generate_off_chunks:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling ChunkApi->generate_off_chunks: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **generate_off_chunks_req_payload** | [**GenerateOffChunksReqPayload**](GenerateOffChunksReqPayload.md)| JSON request payload to perform RAG on some chunks (chunks) | 

### Return type

**str**

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: text/plain, application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | This will be a JSON response of a string containing the LLM&#39;s generated inference. Response if not streaming. |  * TR-QueryID - Query ID that is used for tracking analytics <br>  |
**400** | Service error relating to to updating chunk, likely due to conflicting tracking_id |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_chunk_by_id**
> ChunkReturnTypes get_chunk_by_id(tr_dataset, chunk_id, x_api_version=x_api_version)

Get Chunk By Id

Get a singular chunk by id.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.chunk_return_types import ChunkReturnTypes
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
    api_instance = trieve_py_client.ChunkApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    chunk_id = 'chunk_id_example' # str | Id of the chunk you want to fetch.
    x_api_version = trieve_py_client.APIVersion() # APIVersion | The API version to use for this request. Defaults to V2 for orgs created after July 12, 2024 and V1 otherwise. (optional)

    try:
        # Get Chunk By Id
        api_response = api_instance.get_chunk_by_id(tr_dataset, chunk_id, x_api_version=x_api_version)
        print("The response of ChunkApi->get_chunk_by_id:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling ChunkApi->get_chunk_by_id: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **chunk_id** | **str**| Id of the chunk you want to fetch. | 
 **x_api_version** | [**APIVersion**](.md)| The API version to use for this request. Defaults to V2 for orgs created after July 12, 2024 and V1 otherwise. | [optional] 

### Return type

[**ChunkReturnTypes**](ChunkReturnTypes.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | chunk with the id that you were searching for |  -  |
**400** | Service error relating to fidning a chunk by tracking_id |  -  |
**404** | Chunk not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_chunk_by_tracking_id**
> ChunkReturnTypes get_chunk_by_tracking_id(tr_dataset, tracking_id, x_api_version=x_api_version)

Get Chunk By Tracking Id

Get a singular chunk by tracking_id. This is useful for when you are coordinating with an external system and want to use your own id as the primary reference for a chunk.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.chunk_return_types import ChunkReturnTypes
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
    api_instance = trieve_py_client.ChunkApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    tracking_id = 'tracking_id_example' # str | tracking_id of the chunk you want to fetch
    x_api_version = trieve_py_client.APIVersion() # APIVersion | The API version to use for this request. Defaults to V2 for orgs created after July 12, 2024 and V1 otherwise. (optional)

    try:
        # Get Chunk By Tracking Id
        api_response = api_instance.get_chunk_by_tracking_id(tr_dataset, tracking_id, x_api_version=x_api_version)
        print("The response of ChunkApi->get_chunk_by_tracking_id:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling ChunkApi->get_chunk_by_tracking_id: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **tracking_id** | **str**| tracking_id of the chunk you want to fetch | 
 **x_api_version** | [**APIVersion**](.md)| The API version to use for this request. Defaults to V2 for orgs created after July 12, 2024 and V1 otherwise. | [optional] 

### Return type

[**ChunkReturnTypes**](ChunkReturnTypes.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | chunk with the tracking_id that you were searching for |  -  |
**400** | Service error relating to fidning a chunk by tracking_id |  -  |
**404** | Chunk not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_chunks_by_ids**
> List[ChunkReturnTypes] get_chunks_by_ids(tr_dataset, get_chunks_data, x_api_version=x_api_version)

Get Chunks By Ids

Get multiple chunks by multiple ids.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.chunk_return_types import ChunkReturnTypes
from trieve_py_client.models.get_chunks_data import GetChunksData
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
    api_instance = trieve_py_client.ChunkApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    get_chunks_data = trieve_py_client.GetChunksData() # GetChunksData | JSON request payload to get the chunks in the request
    x_api_version = trieve_py_client.APIVersion() # APIVersion | The API version to use for this request. Defaults to V2 for orgs created after July 12, 2024 and V1 otherwise. (optional)

    try:
        # Get Chunks By Ids
        api_response = api_instance.get_chunks_by_ids(tr_dataset, get_chunks_data, x_api_version=x_api_version)
        print("The response of ChunkApi->get_chunks_by_ids:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling ChunkApi->get_chunks_by_ids: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **get_chunks_data** | [**GetChunksData**](GetChunksData.md)| JSON request payload to get the chunks in the request | 
 **x_api_version** | [**APIVersion**](.md)| The API version to use for this request. Defaults to V2 for orgs created after July 12, 2024 and V1 otherwise. | [optional] 

### Return type

[**List[ChunkReturnTypes]**](ChunkReturnTypes.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | chunks with the id that you were searching for |  -  |
**400** | Service error relating to fidning a chunk by tracking_id |  -  |
**404** | Any one of the specified chunks not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_chunks_by_tracking_ids**
> List[ChunkReturnTypes] get_chunks_by_tracking_ids(tr_dataset, get_tracking_chunks_data, x_api_version=x_api_version)

Get Chunks By Tracking Ids

Get multiple chunks by ids.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.chunk_return_types import ChunkReturnTypes
from trieve_py_client.models.get_tracking_chunks_data import GetTrackingChunksData
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
    api_instance = trieve_py_client.ChunkApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    get_tracking_chunks_data = trieve_py_client.GetTrackingChunksData() # GetTrackingChunksData | JSON request payload to get the chunks in the request
    x_api_version = trieve_py_client.APIVersion() # APIVersion | The API version to use for this request. Defaults to V2 for orgs created after July 12, 2024 and V1 otherwise. (optional)

    try:
        # Get Chunks By Tracking Ids
        api_response = api_instance.get_chunks_by_tracking_ids(tr_dataset, get_tracking_chunks_data, x_api_version=x_api_version)
        print("The response of ChunkApi->get_chunks_by_tracking_ids:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling ChunkApi->get_chunks_by_tracking_ids: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **get_tracking_chunks_data** | [**GetTrackingChunksData**](GetTrackingChunksData.md)| JSON request payload to get the chunks in the request | 
 **x_api_version** | [**APIVersion**](.md)| The API version to use for this request. Defaults to V2 for orgs created after July 12, 2024 and V1 otherwise. | [optional] 

### Return type

[**List[ChunkReturnTypes]**](ChunkReturnTypes.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Chunks with one the ids which were specified |  -  |
**400** | Service error relating to finding a chunk by tracking_id |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_recommended_chunks**
> RecommendResponseTypes get_recommended_chunks(tr_dataset, recommend_chunks_request, x_api_version=x_api_version)

Get Recommended Chunks

Get recommendations of chunks similar to the positive samples in the request and dissimilar to the negative.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.recommend_chunks_request import RecommendChunksRequest
from trieve_py_client.models.recommend_response_types import RecommendResponseTypes
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
    api_instance = trieve_py_client.ChunkApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    recommend_chunks_request = trieve_py_client.RecommendChunksRequest() # RecommendChunksRequest | JSON request payload to get recommendations of chunks similar to the chunks in the request
    x_api_version = trieve_py_client.APIVersion() # APIVersion | The API version to use for this request. Defaults to V2 for orgs created after July 12, 2024 and V1 otherwise. (optional)

    try:
        # Get Recommended Chunks
        api_response = api_instance.get_recommended_chunks(tr_dataset, recommend_chunks_request, x_api_version=x_api_version)
        print("The response of ChunkApi->get_recommended_chunks:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling ChunkApi->get_recommended_chunks: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **recommend_chunks_request** | [**RecommendChunksRequest**](RecommendChunksRequest.md)| JSON request payload to get recommendations of chunks similar to the chunks in the request | 
 **x_api_version** | [**APIVersion**](.md)| The API version to use for this request. Defaults to V2 for orgs created after July 12, 2024 and V1 otherwise. | [optional] 

### Return type

[**RecommendResponseTypes**](RecommendResponseTypes.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Chunks with embedding vectors which are similar to positives and dissimilar to negatives |  -  |
**400** | Service error relating to to getting similar chunks |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_suggested_queries**
> SuggestedQueriesResponse get_suggested_queries(tr_dataset, suggested_queries_req_payload)

Generate suggested queries

This endpoint will generate 3 suggested queries based off a hybrid search using RAG with the query provided in the request body and return them as a JSON object.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.suggested_queries_req_payload import SuggestedQueriesReqPayload
from trieve_py_client.models.suggested_queries_response import SuggestedQueriesResponse
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
    api_instance = trieve_py_client.ChunkApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    suggested_queries_req_payload = trieve_py_client.SuggestedQueriesReqPayload() # SuggestedQueriesReqPayload | JSON request payload to get alternative suggested queries

    try:
        # Generate suggested queries
        api_response = api_instance.get_suggested_queries(tr_dataset, suggested_queries_req_payload)
        print("The response of ChunkApi->get_suggested_queries:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling ChunkApi->get_suggested_queries: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **suggested_queries_req_payload** | [**SuggestedQueriesReqPayload**](SuggestedQueriesReqPayload.md)| JSON request payload to get alternative suggested queries | 

### Return type

[**SuggestedQueriesResponse**](SuggestedQueriesResponse.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | A JSON object containing a list of alternative suggested queries |  -  |
**400** | Service error relating to to updating chunk, likely due to conflicting tracking_id |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **scroll_dataset_chunks**
> ScrollChunksResponseBody scroll_dataset_chunks(tr_dataset, scroll_chunks_req_payload)

Scroll Chunks

Get paginated chunks from your dataset with filters and custom sorting. If sort by is not specified, the results will sort by the id's of the chunks in ascending order. Sort by and offset_chunk_id cannot be used together; if you want to scroll with a sort by then you need to use a must_not filter with the ids you have already seen. There is a limit of 1000 id's in a must_not filter at a time.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.scroll_chunks_req_payload import ScrollChunksReqPayload
from trieve_py_client.models.scroll_chunks_response_body import ScrollChunksResponseBody
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
    api_instance = trieve_py_client.ChunkApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    scroll_chunks_req_payload = trieve_py_client.ScrollChunksReqPayload() # ScrollChunksReqPayload | JSON request payload to scroll through chunks (chunks)

    try:
        # Scroll Chunks
        api_response = api_instance.scroll_dataset_chunks(tr_dataset, scroll_chunks_req_payload)
        print("The response of ChunkApi->scroll_dataset_chunks:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling ChunkApi->scroll_dataset_chunks: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **scroll_chunks_req_payload** | [**ScrollChunksReqPayload**](ScrollChunksReqPayload.md)| JSON request payload to scroll through chunks (chunks) | 

### Return type

[**ScrollChunksResponseBody**](ScrollChunksResponseBody.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Number of chunks equivalent to page_size starting from offset_chunk_id |  -  |
**400** | Service error relating to scrolling chunks |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **search_chunks**
> SearchResponseTypes search_chunks(tr_dataset, search_chunks_req_payload, x_api_version=x_api_version)

Search

This route provides the primary search functionality for the API. It can be used to search for chunks by semantic similarity, full-text similarity, or a combination of both. Results' `chunk_html` values will be modified with `<mark><b>` or custom specified tags for sub-sentence highlighting.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.search_chunks_req_payload import SearchChunksReqPayload
from trieve_py_client.models.search_response_types import SearchResponseTypes
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
    api_instance = trieve_py_client.ChunkApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    search_chunks_req_payload = trieve_py_client.SearchChunksReqPayload() # SearchChunksReqPayload | JSON request payload to semantically search for chunks (chunks)
    x_api_version = trieve_py_client.APIVersion() # APIVersion | The API version to use for this request. Defaults to V2 for orgs created after July 12, 2024 and V1 otherwise. (optional)

    try:
        # Search
        api_response = api_instance.search_chunks(tr_dataset, search_chunks_req_payload, x_api_version=x_api_version)
        print("The response of ChunkApi->search_chunks:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling ChunkApi->search_chunks: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **search_chunks_req_payload** | [**SearchChunksReqPayload**](SearchChunksReqPayload.md)| JSON request payload to semantically search for chunks (chunks) | 
 **x_api_version** | [**APIVersion**](.md)| The API version to use for this request. Defaults to V2 for orgs created after July 12, 2024 and V1 otherwise. | [optional] 

### Return type

[**SearchResponseTypes**](SearchResponseTypes.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Chunks with embedding vectors which are similar to those in the request body |  -  |
**400** | Service error relating to searching |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **split_html_content**
> SplitHtmlResponse split_html_content(chunk_html_content_req_payload)

Split HTML Content into Chunks

This endpoint receives a single html string and splits it into chunks based on the headings and body content. The headings are split based on heading html tags. chunk_html has a maximum size of 256Kb.

### Example


```python
import trieve_py_client
from trieve_py_client.models.chunk_html_content_req_payload import ChunkHtmlContentReqPayload
from trieve_py_client.models.split_html_response import SplitHtmlResponse
from trieve_py_client.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to https://api.trieve.ai
# See configuration.py for a list of all supported configuration parameters.
configuration = trieve_py_client.Configuration(
    host = "https://api.trieve.ai"
)


# Enter a context with an instance of the API client
with trieve_py_client.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = trieve_py_client.ChunkApi(api_client)
    chunk_html_content_req_payload = trieve_py_client.ChunkHtmlContentReqPayload() # ChunkHtmlContentReqPayload | JSON request payload to perform RAG on some chunks (chunks)

    try:
        # Split HTML Content into Chunks
        api_response = api_instance.split_html_content(chunk_html_content_req_payload)
        print("The response of ChunkApi->split_html_content:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling ChunkApi->split_html_content: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **chunk_html_content_req_payload** | [**ChunkHtmlContentReqPayload**](ChunkHtmlContentReqPayload.md)| JSON request payload to perform RAG on some chunks (chunks) | 

### Return type

[**SplitHtmlResponse**](SplitHtmlResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | This will be a JSON response of the chunks split from the HTML content with the headings and body |  -  |
**413** | Payload too large, if the HTML contnet is greater than 256Kb |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **update_chunk**
> update_chunk(tr_dataset, update_chunk_req_payload)

Update Chunk

Update a chunk. If you try to change the tracking_id of the chunk to have the same tracking_id as an existing chunk, the request will fail. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.update_chunk_req_payload import UpdateChunkReqPayload
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
    api_instance = trieve_py_client.ChunkApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    update_chunk_req_payload = trieve_py_client.UpdateChunkReqPayload() # UpdateChunkReqPayload | JSON request payload to update a chunk (chunk)

    try:
        # Update Chunk
        api_instance.update_chunk(tr_dataset, update_chunk_req_payload)
    except Exception as e:
        print("Exception when calling ChunkApi->update_chunk: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **update_chunk_req_payload** | [**UpdateChunkReqPayload**](UpdateChunkReqPayload.md)| JSON request payload to update a chunk (chunk) | 

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
**204** | No content Ok response indicating the chunk was updated as requested |  -  |
**400** | Service error relating to to updating chunk, likely due to conflicting tracking_id |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **update_chunk_by_tracking_id**
> update_chunk_by_tracking_id(tr_dataset, update_chunk_by_tracking_id_data)

Update Chunk By Tracking Id

Update a chunk by tracking_id. This is useful for when you are coordinating with an external system and want to use the tracking_id to identify the chunk. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.update_chunk_by_tracking_id_data import UpdateChunkByTrackingIdData
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
    api_instance = trieve_py_client.ChunkApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    update_chunk_by_tracking_id_data = trieve_py_client.UpdateChunkByTrackingIdData() # UpdateChunkByTrackingIdData | JSON request payload to update a chunk by tracking_id (chunks)

    try:
        # Update Chunk By Tracking Id
        api_instance.update_chunk_by_tracking_id(tr_dataset, update_chunk_by_tracking_id_data)
    except Exception as e:
        print("Exception when calling ChunkApi->update_chunk_by_tracking_id: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **update_chunk_by_tracking_id_data** | [**UpdateChunkByTrackingIdData**](UpdateChunkByTrackingIdData.md)| JSON request payload to update a chunk by tracking_id (chunks) | 

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
**204** | Confirmation that the chunk has been updated as per your request |  -  |
**400** | Service error relating to to updating chunk |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

