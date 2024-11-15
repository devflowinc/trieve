# ChunkMetadataWithPosition


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**chunk** | [**ChunkMetadata**](ChunkMetadata.md) |  | 
**position** | **int** |  | 

## Example

```python
from trieve_py_client.models.chunk_metadata_with_position import ChunkMetadataWithPosition

# TODO update the JSON string below
json = "{}"
# create an instance of ChunkMetadataWithPosition from a JSON string
chunk_metadata_with_position_instance = ChunkMetadataWithPosition.from_json(json)
# print the JSON string representation of the object
print(ChunkMetadataWithPosition.to_json())

# convert the object into a dict
chunk_metadata_with_position_dict = chunk_metadata_with_position_instance.to_dict()
# create an instance of ChunkMetadataWithPosition from a dict
chunk_metadata_with_position_form_dict = chunk_metadata_with_position.from_dict(chunk_metadata_with_position_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


