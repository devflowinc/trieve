# Recommendation


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**event_type** | **str** |  | 
**negative_ids** | **List[str]** | Negative ids used for the recommendation | [optional] 
**negative_tracking_ids** | **List[str]** | Negative tracking ids used for the recommendation | [optional] 
**positive_ids** | **List[str]** | Positive ids used for the recommendation | [optional] 
**positive_tracking_ids** | **List[str]** | Positive tracking ids used for the recommendation | [optional] 
**recommendation_type** | [**ClickhouseRecommendationTypes**](ClickhouseRecommendationTypes.md) |  | [optional] 
**request_params** | **object** | The request params of the recommendation | [optional] 
**results** | **List[object]** | The results of the Recommendation event | [optional] 
**top_score** | **float** | Top score of the recommendation | [optional] 
**user_id** | **str** | The user id of the user who made the recommendation | [optional] 

## Example

```python
from trieve_py_client.models.recommendation import Recommendation

# TODO update the JSON string below
json = "{}"
# create an instance of Recommendation from a JSON string
recommendation_instance = Recommendation.from_json(json)
# print the JSON string representation of the object
print(Recommendation.to_json())

# convert the object into a dict
recommendation_dict = recommendation_instance.to_dict()
# create an instance of Recommendation from a dict
recommendation_form_dict = recommendation.from_dict(recommendation_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


