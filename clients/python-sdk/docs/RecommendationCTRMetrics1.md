# RecommendationCTRMetrics1


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**filter** | [**RecommendationAnalyticsFilter**](RecommendationAnalyticsFilter.md) |  | [optional] 
**type** | **str** |  | 

## Example

```python
from trieve_py_client.models.recommendation_ctr_metrics1 import RecommendationCTRMetrics1

# TODO update the JSON string below
json = "{}"
# create an instance of RecommendationCTRMetrics1 from a JSON string
recommendation_ctr_metrics1_instance = RecommendationCTRMetrics1.from_json(json)
# print the JSON string representation of the object
print(RecommendationCTRMetrics1.to_json())

# convert the object into a dict
recommendation_ctr_metrics1_dict = recommendation_ctr_metrics1_instance.to_dict()
# create an instance of RecommendationCTRMetrics1 from a dict
recommendation_ctr_metrics1_form_dict = recommendation_ctr_metrics1.from_dict(recommendation_ctr_metrics1_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


