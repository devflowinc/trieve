# ScoreChunk


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**chunk** | [**NewChunkMetadataTypes**](NewChunkMetadataTypes.md) |  | 
**highlights** | **List[str]** |  | [optional] 
**score** | **float** |  | 

## Example

```python
from trieve_py_client.models.score_chunk import ScoreChunk

# TODO update the JSON string below
json = "{}"
# create an instance of ScoreChunk from a JSON string
score_chunk_instance = ScoreChunk.from_json(json)
# print the JSON string representation of the object
print(ScoreChunk.to_json())

# convert the object into a dict
score_chunk_dict = score_chunk_instance.to_dict()
# create an instance of ScoreChunk from a dict
score_chunk_form_dict = score_chunk.from_dict(score_chunk_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


