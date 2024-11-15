# RAGAnalytics


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**filter** | [**RAGAnalyticsFilter**](RAGAnalyticsFilter.md) |  | [optional] 
**page** | **int** |  | [optional] 
**sort_by** | [**RAGSortBy**](RAGSortBy.md) |  | [optional] 
**sort_order** | [**SortOrder**](SortOrder.md) |  | [optional] 
**type** | **str** |  | 
**granularity** | [**Granularity**](Granularity.md) |  | [optional] 
**request_id** | **str** |  | 

## Example

```python
from trieve_py_client.models.rag_analytics import RAGAnalytics

# TODO update the JSON string below
json = "{}"
# create an instance of RAGAnalytics from a JSON string
rag_analytics_instance = RAGAnalytics.from_json(json)
# print the JSON string representation of the object
print(RAGAnalytics.to_json())

# convert the object into a dict
rag_analytics_dict = rag_analytics_instance.to_dict()
# create an instance of RAGAnalytics from a dict
rag_analytics_form_dict = rag_analytics.from_dict(rag_analytics_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


