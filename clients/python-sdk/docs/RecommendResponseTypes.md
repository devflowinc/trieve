# RecommendResponseTypes


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**chunks** | [**List[ScoreChunk]**](ScoreChunk.md) |  | 
**id** | **str** |  | 

## Example

```python
from trieve_py_client.models.recommend_response_types import RecommendResponseTypes

# TODO update the JSON string below
json = "{}"
# create an instance of RecommendResponseTypes from a JSON string
recommend_response_types_instance = RecommendResponseTypes.from_json(json)
# print the JSON string representation of the object
print(RecommendResponseTypes.to_json())

# convert the object into a dict
recommend_response_types_dict = recommend_response_types_instance.to_dict()
# create an instance of RecommendResponseTypes from a dict
recommend_response_types_form_dict = recommend_response_types.from_dict(recommend_response_types_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


