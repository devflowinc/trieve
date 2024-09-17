# RecommendationAnalyticsFilter


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**date_range** | [**DateRange**](DateRange.md) |  | [optional] 
**recommendation_type** | [**RecommendationType**](RecommendationType.md) |  | [optional] 

## Example

```python
from trieve_py_client.models.recommendation_analytics_filter import RecommendationAnalyticsFilter

# TODO update the JSON string below
json = "{}"
# create an instance of RecommendationAnalyticsFilter from a JSON string
recommendation_analytics_filter_instance = RecommendationAnalyticsFilter.from_json(json)
# print the JSON string representation of the object
print(RecommendationAnalyticsFilter.to_json())

# convert the object into a dict
recommendation_analytics_filter_dict = recommendation_analytics_filter_instance.to_dict()
# create an instance of RecommendationAnalyticsFilter from a dict
recommendation_analytics_filter_form_dict = recommendation_analytics_filter.from_dict(recommendation_analytics_filter_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


