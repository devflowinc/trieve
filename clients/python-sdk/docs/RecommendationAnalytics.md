# RecommendationAnalytics


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**filter** | [**RecommendationAnalyticsFilter**](RecommendationAnalyticsFilter.md) |  | [optional] 
**page** | **int** |  | [optional] 
**threshold** | **float** |  | [optional] 
**type** | **str** |  | 
**sort_by** | [**SearchSortBy**](SearchSortBy.md) |  | [optional] 
**sort_order** | [**SortOrder**](SortOrder.md) |  | [optional] 
**request_id** | **str** |  | 

## Example

```python
from trieve_py_client.models.recommendation_analytics import RecommendationAnalytics

# TODO update the JSON string below
json = "{}"
# create an instance of RecommendationAnalytics from a JSON string
recommendation_analytics_instance = RecommendationAnalytics.from_json(json)
# print the JSON string representation of the object
print(RecommendationAnalytics.to_json())

# convert the object into a dict
recommendation_analytics_dict = recommendation_analytics_instance.to_dict()
# create an instance of RecommendationAnalytics from a dict
recommendation_analytics_form_dict = recommendation_analytics.from_dict(recommendation_analytics_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


