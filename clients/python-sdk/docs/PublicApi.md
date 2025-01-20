# trieve_py_client.PublicApi

All URIs are relative to *https://api.trieve.ai*

Method | HTTP request | Description
------------- | ------------- | -------------
[**public_page**](PublicApi.md#public_page) | **GET** /api/public_page/{dataset_id} | 


# **public_page**
> public_page(dataset_id)



### Example


```python
import trieve_py_client
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
    api_instance = trieve_py_client.PublicApi(api_client)
    dataset_id = 'dataset_id_example' # str | The id or tracking_id of the dataset you want to get the demo page for.

    try:
        api_instance.public_page(dataset_id)
    except Exception as e:
        print("Exception when calling PublicApi->public_page: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **dataset_id** | **str**| The id or tracking_id of the dataset you want to get the demo page for. | 

### Return type

void (empty response body)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Public Page associated to the dataset |  -  |
**400** | Service error relating to loading the public page |  -  |
**404** | Dataset not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

