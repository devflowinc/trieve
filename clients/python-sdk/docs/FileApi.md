# trieve_py_client.FileApi

All URIs are relative to *https://api.trieve.ai*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_presigned_url_for_csv_jsonl**](FileApi.md#create_presigned_url_for_csv_jsonl) | **POST** /api/file/csv_or_jsonl | Create Presigned CSV/JSONL S3 PUT URL
[**delete_file_handler**](FileApi.md#delete_file_handler) | **DELETE** /api/file/{file_id} | Delete File
[**get_dataset_files_handler**](FileApi.md#get_dataset_files_handler) | **GET** /api/dataset/files/{dataset_id}/{page} | Get Files for Dataset
[**get_file_handler**](FileApi.md#get_file_handler) | **GET** /api/file/{file_id} | Get File Signed URL
[**upload_file_handler**](FileApi.md#upload_file_handler) | **POST** /api/file | Upload File
[**upload_html_page**](FileApi.md#upload_html_page) | **POST** /api/file/html_page | Upload HTML Page


# **create_presigned_url_for_csv_jsonl**
> CreatePresignedUrlForCsvJsonResponseBody create_presigned_url_for_csv_jsonl(tr_dataset, create_presigned_url_for_csv_jsonl_req_payload)

Create Presigned CSV/JSONL S3 PUT URL

This route is useful for uploading very large CSV or JSONL files. Once you have completed the upload, chunks will be automatically created from the file for each line in the CSV or JSONL file. The chunks will be indexed and searchable. Auth'ed user must be an admin or owner of the dataset's organization to upload a file.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.create_presigned_url_for_csv_json_response_body import CreatePresignedUrlForCsvJsonResponseBody
from trieve_py_client.models.create_presigned_url_for_csv_jsonl_req_payload import CreatePresignedUrlForCsvJsonlReqPayload
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
    api_instance = trieve_py_client.FileApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    create_presigned_url_for_csv_jsonl_req_payload = trieve_py_client.CreatePresignedUrlForCsvJsonlReqPayload() # CreatePresignedUrlForCsvJsonlReqPayload | JSON request payload to upload a CSV or JSONL file

    try:
        # Create Presigned CSV/JSONL S3 PUT URL
        api_response = api_instance.create_presigned_url_for_csv_jsonl(tr_dataset, create_presigned_url_for_csv_jsonl_req_payload)
        print("The response of FileApi->create_presigned_url_for_csv_jsonl:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling FileApi->create_presigned_url_for_csv_jsonl: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **create_presigned_url_for_csv_jsonl_req_payload** | [**CreatePresignedUrlForCsvJsonlReqPayload**](CreatePresignedUrlForCsvJsonlReqPayload.md)| JSON request payload to upload a CSV or JSONL file | 

### Return type

[**CreatePresignedUrlForCsvJsonResponseBody**](CreatePresignedUrlForCsvJsonResponseBody.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | File object information and signed put URL |  -  |
**400** | Service error relating to uploading the file |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **delete_file_handler**
> delete_file_handler(tr_dataset, file_id, delete_chunks)

Delete File

Delete a file from S3 attached to the server based on its id. This will disassociate chunks from the file, but only delete them all together if you specify delete_chunks to be true. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.

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
    api_instance = trieve_py_client.FileApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    file_id = 'file_id_example' # str | The id of the file to delete
    delete_chunks = True # bool | Delete the chunks within the group

    try:
        # Delete File
        api_instance.delete_file_handler(tr_dataset, file_id, delete_chunks)
    except Exception as e:
        print("Exception when calling FileApi->delete_file_handler: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **file_id** | **str**| The id of the file to delete | 
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
**204** | Confirmation that the file has been deleted |  -  |
**400** | Service error relating to finding or deleting the file |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_dataset_files_handler**
> FileData get_dataset_files_handler(tr_dataset, dataset_id, page)

Get Files for Dataset

Get all files which belong to a given dataset specified by the dataset_id parameter. 10 files are returned per page.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.file_data import FileData
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
    api_instance = trieve_py_client.FileApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    dataset_id = 'dataset_id_example' # str | The id of the dataset to fetch files for.
    page = 56 # int | The page number of files you wish to fetch. Each page contains at most 10 files.

    try:
        # Get Files for Dataset
        api_response = api_instance.get_dataset_files_handler(tr_dataset, dataset_id, page)
        print("The response of FileApi->get_dataset_files_handler:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling FileApi->get_dataset_files_handler: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **dataset_id** | **str**| The id of the dataset to fetch files for. | 
 **page** | **int**| The page number of files you wish to fetch. Each page contains at most 10 files. | 

### Return type

[**FileData**](FileData.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | JSON body representing the files in the current dataset |  -  |
**400** | Service error relating to getting the files in the current datase |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_file_handler**
> FileDTO get_file_handler(tr_dataset, file_id, content_type=content_type)

Get File Signed URL

Get a signed s3 url corresponding to the file_id requested such that you can download the file.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.file_dto import FileDTO
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
    api_instance = trieve_py_client.FileApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    file_id = 'file_id_example' # str | The id of the file to fetch
    content_type = 'content_type_example' # str | Optional field to override the presigned url's Content-Type header (optional)

    try:
        # Get File Signed URL
        api_response = api_instance.get_file_handler(tr_dataset, file_id, content_type=content_type)
        print("The response of FileApi->get_file_handler:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling FileApi->get_file_handler: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **file_id** | **str**| The id of the file to fetch | 
 **content_type** | **str**| Optional field to override the presigned url&#39;s Content-Type header | [optional] 

### Return type

[**FileDTO**](FileDTO.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | The file&#39;s information and s3_url where the original file can be downloaded |  -  |
**400** | Service error relating to finding the file |  -  |
**404** | File not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **upload_file_handler**
> UploadFileResponseBody upload_file_handler(tr_dataset, upload_file_req_payload)

Upload File

Upload a file to S3 bucket attached to your dataset. You can select between a naive chunking strategy where the text is extracted with Apache Tika and split into segments with a target number of segments per chunk OR you can use a vision LLM to convert the file to markdown and create chunks per page. Auth'ed user must be an admin or owner of the dataset's organization to upload a file.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.upload_file_req_payload import UploadFileReqPayload
from trieve_py_client.models.upload_file_response_body import UploadFileResponseBody
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
    api_instance = trieve_py_client.FileApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    upload_file_req_payload = trieve_py_client.UploadFileReqPayload() # UploadFileReqPayload | JSON request payload to upload a file

    try:
        # Upload File
        api_response = api_instance.upload_file_handler(tr_dataset, upload_file_req_payload)
        print("The response of FileApi->upload_file_handler:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling FileApi->upload_file_handler: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **upload_file_req_payload** | [**UploadFileReqPayload**](UploadFileReqPayload.md)| JSON request payload to upload a file | 

### Return type

[**UploadFileResponseBody**](UploadFileResponseBody.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Confirmation that the file is uploading |  -  |
**400** | Service error relating to uploading the file |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **upload_html_page**
> upload_html_page(upload_html_page_req_payload)

Upload HTML Page

Chunk HTML by headings and queue for indexing into the specified dataset.

### Example


```python
import trieve_py_client
from trieve_py_client.models.upload_html_page_req_payload import UploadHtmlPageReqPayload
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
    api_instance = trieve_py_client.FileApi(api_client)
    upload_html_page_req_payload = trieve_py_client.UploadHtmlPageReqPayload() # UploadHtmlPageReqPayload | JSON request payload to upload a file

    try:
        # Upload HTML Page
        api_instance.upload_html_page(upload_html_page_req_payload)
    except Exception as e:
        print("Exception when calling FileApi->upload_html_page: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **upload_html_page_req_payload** | [**UploadHtmlPageReqPayload**](UploadHtmlPageReqPayload.md)| JSON request payload to upload a file | 

### Return type

void (empty response body)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**204** | Confirmation that html is being processed |  -  |
**400** | Service error relating to processing the file |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

