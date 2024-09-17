# RecommendationsWithClicksCTRResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**clicked_chunks** | [**List[ChunkMetadata]**](ChunkMetadata.md) |  | 
**created_at** | **str** |  | 
**negative_ids** | **List[str]** |  | [optional] 
**negative_tracking_ids** | **List[str]** |  | [optional] 
**positions** | **List[int]** |  | 
**positive_ids** | **List[str]** |  | [optional] 
**positive_tracking_ids** | **List[str]** |  | [optional] 

## Example

```python
from trieve_py_client.models.recommendations_with_clicks_ctr_response import RecommendationsWithClicksCTRResponse

# TODO update the JSON string below
json = "{}"
# create an instance of RecommendationsWithClicksCTRResponse from a JSON string
recommendations_with_clicks_ctr_response_instance = RecommendationsWithClicksCTRResponse.from_json(json)
# print the JSON string representation of the object
print(RecommendationsWithClicksCTRResponse.to_json())

# convert the object into a dict
recommendations_with_clicks_ctr_response_dict = recommendations_with_clicks_ctr_response_instance.to_dict()
# create an instance of RecommendationsWithClicksCTRResponse from a dict
recommendations_with_clicks_ctr_response_form_dict = recommendations_with_clicks_ctr_response.from_dict(recommendations_with_clicks_ctr_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


