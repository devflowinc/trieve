# RecommendChunksResponseBody


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**chunks** | [**List[ScoreChunk]**](ScoreChunk.md) |  | 
**id** | **str** |  | 

## Example

```python
from trieve_py_client.models.recommend_chunks_response_body import RecommendChunksResponseBody

# TODO update the JSON string below
json = "{}"
# create an instance of RecommendChunksResponseBody from a JSON string
recommend_chunks_response_body_instance = RecommendChunksResponseBody.from_json(json)
# print the JSON string representation of the object
print(RecommendChunksResponseBody.to_json())

# convert the object into a dict
recommend_chunks_response_body_dict = recommend_chunks_response_body_instance.to_dict()
# create an instance of RecommendChunksResponseBody from a dict
recommend_chunks_response_body_form_dict = recommend_chunks_response_body.from_dict(recommend_chunks_response_body_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


