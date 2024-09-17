# ScoreChunkDTO


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**highlights** | **List[str]** |  | [optional] 
**metadata** | [**List[ChunkMetadataTypes]**](ChunkMetadataTypes.md) |  | 
**score** | **float** |  | 

## Example

```python
from trieve_py_client.models.score_chunk_dto import ScoreChunkDTO

# TODO update the JSON string below
json = "{}"
# create an instance of ScoreChunkDTO from a JSON string
score_chunk_dto_instance = ScoreChunkDTO.from_json(json)
# print the JSON string representation of the object
print(ScoreChunkDTO.to_json())

# convert the object into a dict
score_chunk_dto_dict = score_chunk_dto_instance.to_dict()
# create an instance of ScoreChunkDTO from a dict
score_chunk_dto_form_dict = score_chunk_dto.from_dict(score_chunk_dto_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


