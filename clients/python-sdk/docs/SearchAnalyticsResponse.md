# SearchAnalyticsResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**latency_points** | [**List[SearchLatencyGraph]**](SearchLatencyGraph.md) |  | 
**usage_points** | [**List[UsageGraphPoint]**](UsageGraphPoint.md) |  | 
**avg_latency** | **float** |  | 
**p50** | **float** |  | 
**p95** | **float** |  | 
**p99** | **float** |  | 
**search_rps** | **float** |  | 
**total_queries** | [**List[SearchTypeCount]**](SearchTypeCount.md) |  | 
**queries** | [**List[SearchQueryEvent]**](SearchQueryEvent.md) |  | 
**created_at** | **str** |  | 
**dataset_id** | **str** |  | 
**id** | **str** |  | 
**latency** | **float** |  | 
**query** | **str** |  | 
**query_rating** | **str** |  | 
**request_params** | **object** |  | 
**results** | **List[object]** |  | 
**search_type** | **str** |  | 
**top_score** | **float** |  | 
**user_id** | **str** |  | 
**popular_filters** | [**List[PopularFilters]**](PopularFilters.md) |  | 

## Example

```python
from trieve_py_client.models.search_analytics_response import SearchAnalyticsResponse

# TODO update the JSON string below
json = "{}"
# create an instance of SearchAnalyticsResponse from a JSON string
search_analytics_response_instance = SearchAnalyticsResponse.from_json(json)
# print the JSON string representation of the object
print(SearchAnalyticsResponse.to_json())

# convert the object into a dict
search_analytics_response_dict = search_analytics_response_instance.to_dict()
# create an instance of SearchAnalyticsResponse from a dict
search_analytics_response_form_dict = search_analytics_response.from_dict(search_analytics_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


