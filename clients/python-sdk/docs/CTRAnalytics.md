# CTRAnalytics


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**filter** | [**RecommendationAnalyticsFilter**](RecommendationAnalyticsFilter.md) |  | [optional] 
**type** | **str** |  | 
**page** | **int** |  | [optional] 

## Example

```python
from trieve_py_client.models.ctr_analytics import CTRAnalytics

# TODO update the JSON string below
json = "{}"
# create an instance of CTRAnalytics from a JSON string
ctr_analytics_instance = CTRAnalytics.from_json(json)
# print the JSON string representation of the object
print(CTRAnalytics.to_json())

# convert the object into a dict
ctr_analytics_dict = ctr_analytics_instance.to_dict()
# create an instance of CTRAnalytics from a dict
ctr_analytics_form_dict = ctr_analytics.from_dict(ctr_analytics_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


