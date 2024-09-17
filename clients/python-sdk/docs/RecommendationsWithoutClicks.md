# RecommendationsWithoutClicks


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**filter** | [**RecommendationAnalyticsFilter**](RecommendationAnalyticsFilter.md) |  | [optional] 
**page** | **int** |  | [optional] 
**type** | **str** |  | 

## Example

```python
from trieve_py_client.models.recommendations_without_clicks import RecommendationsWithoutClicks

# TODO update the JSON string below
json = "{}"
# create an instance of RecommendationsWithoutClicks from a JSON string
recommendations_without_clicks_instance = RecommendationsWithoutClicks.from_json(json)
# print the JSON string representation of the object
print(RecommendationsWithoutClicks.to_json())

# convert the object into a dict
recommendations_without_clicks_dict = recommendations_without_clicks_instance.to_dict()
# create an instance of RecommendationsWithoutClicks from a dict
recommendations_without_clicks_form_dict = recommendations_without_clicks.from_dict(recommendations_without_clicks_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


