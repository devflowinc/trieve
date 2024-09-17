# RecommendationsWithoutClicksCTRResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_at** | **str** |  | 
**negative_ids** | **List[str]** |  | [optional] 
**negative_tracking_ids** | **List[str]** |  | [optional] 
**positive_ids** | **List[str]** |  | [optional] 
**positive_tracking_ids** | **List[str]** |  | [optional] 

## Example

```python
from trieve_py_client.models.recommendations_without_clicks_ctr_response import RecommendationsWithoutClicksCTRResponse

# TODO update the JSON string below
json = "{}"
# create an instance of RecommendationsWithoutClicksCTRResponse from a JSON string
recommendations_without_clicks_ctr_response_instance = RecommendationsWithoutClicksCTRResponse.from_json(json)
# print the JSON string representation of the object
print(RecommendationsWithoutClicksCTRResponse.to_json())

# convert the object into a dict
recommendations_without_clicks_ctr_response_dict = recommendations_without_clicks_ctr_response_instance.to_dict()
# create an instance of RecommendationsWithoutClicksCTRResponse from a dict
recommendations_without_clicks_ctr_response_form_dict = recommendations_without_clicks_ctr_response.from_dict(recommendations_without_clicks_ctr_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


