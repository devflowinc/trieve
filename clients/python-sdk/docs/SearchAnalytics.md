# SearchAnalytics


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**filter** | [**SearchAnalyticsFilter**](SearchAnalyticsFilter.md) |  | [optional] 
**granularity** | [**Granularity**](Granularity.md) |  | [optional] 
**type** | **str** |  | 
**page** | **int** |  | [optional] 
**threshold** | **float** |  | [optional] 
**sort_by** | [**SearchSortBy**](SearchSortBy.md) |  | [optional] 
**sort_order** | [**SortOrder**](SortOrder.md) |  | [optional] 
**search_id** | **str** |  | 

## Example

```python
from trieve_py_client.models.search_analytics import SearchAnalytics

# TODO update the JSON string below
json = "{}"
# create an instance of SearchAnalytics from a JSON string
search_analytics_instance = SearchAnalytics.from_json(json)
# print the JSON string representation of the object
print(SearchAnalytics.to_json())

# convert the object into a dict
search_analytics_dict = search_analytics_instance.to_dict()
# create an instance of SearchAnalytics from a dict
search_analytics_form_dict = search_analytics.from_dict(search_analytics_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


