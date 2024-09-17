# RecommendationsEventResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**queries** | [**List[RecommendationEvent]**](RecommendationEvent.md) |  | 

## Example

```python
from trieve_py_client.models.recommendations_event_response import RecommendationsEventResponse

# TODO update the JSON string below
json = "{}"
# create an instance of RecommendationsEventResponse from a JSON string
recommendations_event_response_instance = RecommendationsEventResponse.from_json(json)
# print the JSON string representation of the object
print(RecommendationsEventResponse.to_json())

# convert the object into a dict
recommendations_event_response_dict = recommendations_event_response_instance.to_dict()
# create an instance of RecommendationsEventResponse from a dict
recommendations_event_response_form_dict = recommendations_event_response.from_dict(recommendations_event_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


