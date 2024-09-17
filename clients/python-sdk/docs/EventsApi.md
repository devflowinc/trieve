# trieve_py_client.EventsApi

All URIs are relative to *https://api.trieve.ai*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_events**](EventsApi.md#get_events) | **POST** /api/events | Get events for the dataset


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
    api_instance = trieve_py_client.EventsApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    get_events_data = trieve_py_client.GetEventsData() # GetEventsData | JSON request payload to get events for a dataset

    try:
        # Get events for the dataset
        api_response = api_instance.get_events(tr_dataset, get_events_data)
        print("The response of EventsApi->get_events:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling EventsApi->get_events: %s\n" % e)
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

