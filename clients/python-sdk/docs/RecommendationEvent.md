# RecommendationEvent


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
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
from trieve_py_client.models.recommendation_event import RecommendationEvent

# TODO update the JSON string below
json = "{}"
# create an instance of RecommendationEvent from a JSON string
recommendation_event_instance = RecommendationEvent.from_json(json)
# print the JSON string representation of the object
print(RecommendationEvent.to_json())

# convert the object into a dict
recommendation_event_dict = recommendation_event_instance.to_dict()
# create an instance of RecommendationEvent from a dict
recommendation_event_form_dict = recommendation_event.from_dict(recommendation_event_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


