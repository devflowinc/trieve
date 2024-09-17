# RecommendationCTRMetrics


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**avg_position_of_click** | **float** |  | 
**percent_recommendations_with_clicks** | **float** |  | 
**percent_recommendations_without_clicks** | **float** |  | 
**recommendations_with_clicks** | **int** |  | 

## Example

```python
from trieve_py_client.models.recommendation_ctr_metrics import RecommendationCTRMetrics

# TODO update the JSON string below
json = "{}"
# create an instance of RecommendationCTRMetrics from a JSON string
recommendation_ctr_metrics_instance = RecommendationCTRMetrics.from_json(json)
# print the JSON string representation of the object
print(RecommendationCTRMetrics.to_json())

# convert the object into a dict
recommendation_ctr_metrics_dict = recommendation_ctr_metrics_instance.to_dict()
# create an instance of RecommendationCTRMetrics from a dict
recommendation_ctr_metrics_form_dict = recommendation_ctr_metrics.from_dict(recommendation_ctr_metrics_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


