# ChunkWithPosition


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**chunk_id** | **str** |  | 
**position** | **int** |  | 

## Example

```python
from trieve_py_client.models.chunk_with_position import ChunkWithPosition

# TODO update the JSON string below
json = "{}"
# create an instance of ChunkWithPosition from a JSON string
chunk_with_position_instance = ChunkWithPosition.from_json(json)
# print the JSON string representation of the object
print(ChunkWithPosition.to_json())

# convert the object into a dict
chunk_with_position_dict = chunk_with_position_instance.to_dict()
# create an instance of ChunkWithPosition from a dict
chunk_with_position_form_dict = chunk_with_position.from_dict(chunk_with_position_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


