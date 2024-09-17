# RecommendationsWithClicks


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**filter** | [**RecommendationAnalyticsFilter**](RecommendationAnalyticsFilter.md) |  | [optional] 
**page** | **int** |  | [optional] 
**type** | **str** |  | 

## Example

```python
from trieve_py_client.models.recommendations_with_clicks import RecommendationsWithClicks

# TODO update the JSON string below
json = "{}"
# create an instance of RecommendationsWithClicks from a JSON string
recommendations_with_clicks_instance = RecommendationsWithClicks.from_json(json)
# print the JSON string representation of the object
print(RecommendationsWithClicks.to_json())

# convert the object into a dict
recommendations_with_clicks_dict = recommendations_with_clicks_instance.to_dict()
# create an instance of RecommendationsWithClicks from a dict
recommendations_with_clicks_form_dict = recommendations_with_clicks.from_dict(recommendations_with_clicks_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


