# LowConfidenceRecommendations


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**filter** | [**RecommendationAnalyticsFilter**](RecommendationAnalyticsFilter.md) |  | [optional] 
**page** | **int** |  | [optional] 
**threshold** | **float** |  | [optional] 
**type** | **str** |  | 

## Example

```python
from trieve_py_client.models.low_confidence_recommendations import LowConfidenceRecommendations

# TODO update the JSON string below
json = "{}"
# create an instance of LowConfidenceRecommendations from a JSON string
low_confidence_recommendations_instance = LowConfidenceRecommendations.from_json(json)
# print the JSON string representation of the object
print(LowConfidenceRecommendations.to_json())

# convert the object into a dict
low_confidence_recommendations_dict = low_confidence_recommendations_instance.to_dict()
# create an instance of LowConfidenceRecommendations from a dict
low_confidence_recommendations_form_dict = low_confidence_recommendations.from_dict(low_confidence_recommendations_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


