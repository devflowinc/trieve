# RecommendationQueries


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**filter** | [**RecommendationAnalyticsFilter**](RecommendationAnalyticsFilter.md) |  | [optional] 
**page** | **int** |  | [optional] 
**sort_by** | [**SearchSortBy**](SearchSortBy.md) |  | [optional] 
**sort_order** | [**SortOrder**](SortOrder.md) |  | [optional] 
**type** | **str** |  | 

## Example

```python
from trieve_py_client.models.recommendation_queries import RecommendationQueries

# TODO update the JSON string below
json = "{}"
# create an instance of RecommendationQueries from a JSON string
recommendation_queries_instance = RecommendationQueries.from_json(json)
# print the JSON string representation of the object
print(RecommendationQueries.to_json())

# convert the object into a dict
recommendation_queries_dict = recommendation_queries_instance.to_dict()
# create an instance of RecommendationQueries from a dict
recommendation_queries_form_dict = recommendation_queries.from_dict(recommendation_queries_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


