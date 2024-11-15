# trieve_py_client.AnalyticsApi

All URIs are relative to *https://api.trieve.ai*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_all_events**](AnalyticsApi.md#get_all_events) | **POST** /api/analytics/events/all | Get All User Events
[**get_cluster_analytics**](AnalyticsApi.md#get_cluster_analytics) | **POST** /api/analytics/search/cluster | Get Cluster Analytics
[**get_ctr_analytics**](AnalyticsApi.md#get_ctr_analytics) | **POST** /api/analytics/events/ctr | Get CTR Analytics
[**get_event_by_id**](AnalyticsApi.md#get_event_by_id) | **GET** /api/analytics/events/{event_id} | Get User Event By ID
[**get_rag_analytics**](AnalyticsApi.md#get_rag_analytics) | **POST** /api/analytics/rag | Get RAG Analytics
[**get_recommendation_analytics**](AnalyticsApi.md#get_recommendation_analytics) | **POST** /api/analytics/recommendations | Get Recommendation Analytics
[**get_search_analytics**](AnalyticsApi.md#get_search_analytics) | **POST** /api/analytics/search | Get Search Analytics
[**get_top_datasets**](AnalyticsApi.md#get_top_datasets) | **POST** /api/analytics/top | Get Top Datasets
[**send_ctr_data**](AnalyticsApi.md#send_ctr_data) | **PUT** /api/analytics/ctr | Send CTR Data
[**send_event_data**](AnalyticsApi.md#send_event_data) | **PUT** /api/analytics/events | Send User Event Data
[**set_rag_query_rating**](AnalyticsApi.md#set_rag_query_rating) | **PUT** /api/analytics/rag | Rate RAG
[**set_search_query_rating**](AnalyticsApi.md#set_search_query_rating) | **PUT** /api/analytics/search | Rate Search


# **get_all_events**
> GetEventsResponseBody get_all_events(get_events_request_body)

Get All User Events

This route allows you to view all user events.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.get_events_request_body import GetEventsRequestBody
from trieve_py_client.models.get_events_response_body import GetEventsResponseBody
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
    api_instance = trieve_py_client.AnalyticsApi(api_client)
    get_events_request_body = trieve_py_client.GetEventsRequestBody() # GetEventsRequestBody | JSON request payload to filter the events

    try:
        # Get All User Events
        api_response = api_instance.get_all_events(get_events_request_body)
        print("The response of AnalyticsApi->get_all_events:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AnalyticsApi->get_all_events: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **get_events_request_body** | [**GetEventsRequestBody**](GetEventsRequestBody.md)| JSON request payload to filter the events | 

### Return type

[**GetEventsResponseBody**](GetEventsResponseBody.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | The events for the request |  -  |
**400** | Service error relating to getting events |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_cluster_analytics**
> ClusterAnalyticsResponse get_cluster_analytics(tr_dataset, cluster_analytics)

Get Cluster Analytics

This route allows you to view the cluster analytics for a dataset.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.cluster_analytics import ClusterAnalytics
from trieve_py_client.models.cluster_analytics_response import ClusterAnalyticsResponse
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
    api_instance = trieve_py_client.AnalyticsApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    cluster_analytics = trieve_py_client.ClusterAnalytics() # ClusterAnalytics | JSON request payload to filter the graph

    try:
        # Get Cluster Analytics
        api_response = api_instance.get_cluster_analytics(tr_dataset, cluster_analytics)
        print("The response of AnalyticsApi->get_cluster_analytics:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AnalyticsApi->get_cluster_analytics: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **cluster_analytics** | [**ClusterAnalytics**](ClusterAnalytics.md)| JSON request payload to filter the graph | 

### Return type

[**ClusterAnalyticsResponse**](ClusterAnalyticsResponse.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | The cluster analytics for the dataset |  -  |
**400** | Service error relating to getting cluster analytics |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_ctr_analytics**
> CTRAnalyticsResponse get_ctr_analytics(tr_dataset, ctr_analytics)

Get CTR Analytics

This route allows you to view the CTR analytics for a dataset.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.ctr_analytics import CTRAnalytics
from trieve_py_client.models.ctr_analytics_response import CTRAnalyticsResponse
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
    api_instance = trieve_py_client.AnalyticsApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    ctr_analytics = trieve_py_client.CTRAnalytics() # CTRAnalytics | JSON request payload to filter the graph

    try:
        # Get CTR Analytics
        api_response = api_instance.get_ctr_analytics(tr_dataset, ctr_analytics)
        print("The response of AnalyticsApi->get_ctr_analytics:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AnalyticsApi->get_ctr_analytics: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **ctr_analytics** | [**CTRAnalytics**](CTRAnalytics.md)| JSON request payload to filter the graph | 

### Return type

[**CTRAnalyticsResponse**](CTRAnalyticsResponse.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | The CTR analytics for the dataset |  -  |
**400** | Service error relating to getting CTR analytics |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_event_by_id**
> EventData get_event_by_id(tr_dataset, event_id)

Get User Event By ID

This route allows you to view an user event by its ID. You can pass in any type of event and get the details for that event.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.event_data import EventData
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
    api_instance = trieve_py_client.AnalyticsApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    event_id = 'event_id_example' # str | The event id to use for the request

    try:
        # Get User Event By ID
        api_response = api_instance.get_event_by_id(tr_dataset, event_id)
        print("The response of AnalyticsApi->get_event_by_id:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AnalyticsApi->get_event_by_id: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **event_id** | **str**| The event id to use for the request | 

### Return type

[**EventData**](EventData.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | The event for the request |  -  |
**400** | Service error relating to getting an event |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_rag_analytics**
> RAGAnalyticsResponse get_rag_analytics(tr_dataset, rag_analytics)

Get RAG Analytics

This route allows you to view the RAG analytics for a dataset.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.rag_analytics import RAGAnalytics
from trieve_py_client.models.rag_analytics_response import RAGAnalyticsResponse
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
    api_instance = trieve_py_client.AnalyticsApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    rag_analytics = trieve_py_client.RAGAnalytics() # RAGAnalytics | JSON request payload to filter the graph

    try:
        # Get RAG Analytics
        api_response = api_instance.get_rag_analytics(tr_dataset, rag_analytics)
        print("The response of AnalyticsApi->get_rag_analytics:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AnalyticsApi->get_rag_analytics: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **rag_analytics** | [**RAGAnalytics**](RAGAnalytics.md)| JSON request payload to filter the graph | 

### Return type

[**RAGAnalyticsResponse**](RAGAnalyticsResponse.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | The RAG analytics for the dataset |  -  |
**400** | Service error relating to getting RAG analytics |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_recommendation_analytics**
> RecommendationAnalyticsResponse get_recommendation_analytics(tr_dataset, recommendation_analytics)

Get Recommendation Analytics

This route allows you to view the recommendation analytics for a dataset.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.recommendation_analytics import RecommendationAnalytics
from trieve_py_client.models.recommendation_analytics_response import RecommendationAnalyticsResponse
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
    api_instance = trieve_py_client.AnalyticsApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    recommendation_analytics = trieve_py_client.RecommendationAnalytics() # RecommendationAnalytics | JSON request payload to filter the graph

    try:
        # Get Recommendation Analytics
        api_response = api_instance.get_recommendation_analytics(tr_dataset, recommendation_analytics)
        print("The response of AnalyticsApi->get_recommendation_analytics:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AnalyticsApi->get_recommendation_analytics: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **recommendation_analytics** | [**RecommendationAnalytics**](RecommendationAnalytics.md)| JSON request payload to filter the graph | 

### Return type

[**RecommendationAnalyticsResponse**](RecommendationAnalyticsResponse.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | The recommendation analytics for the dataset |  -  |
**400** | Service error relating to getting recommendation analytics |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_search_analytics**
> SearchAnalyticsResponse get_search_analytics(tr_dataset, search_analytics)

Get Search Analytics

This route allows you to view the search analytics for a dataset.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.search_analytics import SearchAnalytics
from trieve_py_client.models.search_analytics_response import SearchAnalyticsResponse
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
    api_instance = trieve_py_client.AnalyticsApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    search_analytics = trieve_py_client.SearchAnalytics() # SearchAnalytics | JSON request payload to filter the graph

    try:
        # Get Search Analytics
        api_response = api_instance.get_search_analytics(tr_dataset, search_analytics)
        print("The response of AnalyticsApi->get_search_analytics:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AnalyticsApi->get_search_analytics: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **search_analytics** | [**SearchAnalytics**](SearchAnalytics.md)| JSON request payload to filter the graph | 

### Return type

[**SearchAnalyticsResponse**](SearchAnalyticsResponse.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | The search analytics for the dataset |  -  |
**400** | Service error relating to getting search analytics |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_top_datasets**
> List[TopDatasetsResponse] get_top_datasets(tr_organization, get_top_datasets_request_body)

Get Top Datasets

This route allows you to view the top datasets for a given type.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.get_top_datasets_request_body import GetTopDatasetsRequestBody
from trieve_py_client.models.top_datasets_response import TopDatasetsResponse
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
    api_instance = trieve_py_client.AnalyticsApi(api_client)
    tr_organization = 'tr_organization_example' # str | The organization id to use for the request
    get_top_datasets_request_body = trieve_py_client.GetTopDatasetsRequestBody() # GetTopDatasetsRequestBody | JSON request payload to filter the top datasets

    try:
        # Get Top Datasets
        api_response = api_instance.get_top_datasets(tr_organization, get_top_datasets_request_body)
        print("The response of AnalyticsApi->get_top_datasets:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AnalyticsApi->get_top_datasets: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_organization** | **str**| The organization id to use for the request | 
 **get_top_datasets_request_body** | [**GetTopDatasetsRequestBody**](GetTopDatasetsRequestBody.md)| JSON request payload to filter the top datasets | 

### Return type

[**List[TopDatasetsResponse]**](TopDatasetsResponse.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | The top datasets for the request |  -  |
**400** | Service error relating to getting top datasets |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **send_ctr_data**
> send_ctr_data(tr_dataset, ctr_data_request_body)

Send CTR Data

This route allows you to send clickstream data to the system. Clickstream data is used to fine-tune the re-ranking of search results and recommendations.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.ctr_data_request_body import CTRDataRequestBody
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
    api_instance = trieve_py_client.AnalyticsApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    ctr_data_request_body = trieve_py_client.CTRDataRequestBody() # CTRDataRequestBody | JSON request payload to send CTR data

    try:
        # Send CTR Data
        api_instance.send_ctr_data(tr_dataset, ctr_data_request_body)
    except Exception as e:
        print("Exception when calling AnalyticsApi->send_ctr_data: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **ctr_data_request_body** | [**CTRDataRequestBody**](CTRDataRequestBody.md)| JSON request payload to send CTR data | 

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
**204** | The CTR data was successfully sent |  -  |
**400** | Service error relating to sending CTR data |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **send_event_data**
> send_event_data(tr_dataset, event_types)

Send User Event Data

This route allows you to send user event data to the system.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.event_types import EventTypes
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
    api_instance = trieve_py_client.AnalyticsApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    event_types = trieve_py_client.EventTypes() # EventTypes | JSON request payload to send event data

    try:
        # Send User Event Data
        api_instance.send_event_data(tr_dataset, event_types)
    except Exception as e:
        print("Exception when calling AnalyticsApi->send_event_data: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **event_types** | [**EventTypes**](EventTypes.md)| JSON request payload to send event data | 

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
**204** | The event data was successfully sent |  -  |
**400** | Service error relating to sending event data |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **set_rag_query_rating**
> set_rag_query_rating(tr_dataset, rate_query_request)

Rate RAG

This route allows you to Rate a RAG query.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.rate_query_request import RateQueryRequest
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
    api_instance = trieve_py_client.AnalyticsApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    rate_query_request = trieve_py_client.RateQueryRequest() # RateQueryRequest | JSON request payload to rate a RAG query

    try:
        # Rate RAG
        api_instance.set_rag_query_rating(tr_dataset, rate_query_request)
    except Exception as e:
        print("Exception when calling AnalyticsApi->set_rag_query_rating: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **rate_query_request** | [**RateQueryRequest**](RateQueryRequest.md)| JSON request payload to rate a RAG query | 

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
**204** | The RAG query was successfully rated |  -  |
**400** | Service error relating to rating a RAG query |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **set_search_query_rating**
> set_search_query_rating(tr_dataset, rate_query_request)

Rate Search

This route allows you to Rate a search query.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.rate_query_request import RateQueryRequest
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
    api_instance = trieve_py_client.AnalyticsApi(api_client)
    tr_dataset = 'tr_dataset_example' # str | The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid.
    rate_query_request = trieve_py_client.RateQueryRequest() # RateQueryRequest | JSON request payload to rate a search query

    try:
        # Rate Search
        api_instance.set_search_query_rating(tr_dataset, rate_query_request)
    except Exception as e:
        print("Exception when calling AnalyticsApi->set_search_query_rating: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_dataset** | **str**| The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid. | 
 **rate_query_request** | [**RateQueryRequest**](RateQueryRequest.md)| JSON request payload to rate a search query | 

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
**204** | The search query was successfully rated |  -  |
**400** | Service error relating to rating a search query |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

