# RecommendationAnalyticsResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**queries** | [**List[RecommendationEvent]**](RecommendationEvent.md) |  | 
**created_at** | **str** |  | 
**dataset_id** | **str** |  | 
**id** | **str** |  | 
**negative_ids** | **List[str]** |  | 
**negative_tracking_ids** | **List[str]** |  | 
**positive_ids** | **List[str]** |  | 
**positive_tracking_ids** | **List[str]** |  | 
**recommendation_type** | [**ClickhouseRecommendationTypes**](ClickhouseRecommendationTypes.md) |  | 
**request_params** | **object** |  | 
**results** | **List[object]** |  | 
**top_score** | **float** |  | 
**user_id** | **str** |  | 

## Example

```python
from trieve_py_client.models.recommendation_analytics_response import RecommendationAnalyticsResponse

# TODO update the JSON string below
json = "{}"
# create an instance of RecommendationAnalyticsResponse from a JSON string
recommendation_analytics_response_instance = RecommendationAnalyticsResponse.from_json(json)
# print the JSON string representation of the object
print(RecommendationAnalyticsResponse.to_json())

# convert the object into a dict
recommendation_analytics_response_dict = recommendation_analytics_response_instance.to_dict()
# create an instance of RecommendationAnalyticsResponse from a dict
recommendation_analytics_response_form_dict = recommendation_analytics_response.from_dict(recommendation_analytics_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


