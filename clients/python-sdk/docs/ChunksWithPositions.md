# ChunksWithPositions


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**chunk_id** | **str** |  | 
**position** | **int** |  | 

## Example

```python
from trieve_py_client.models.chunks_with_positions import ChunksWithPositions

# TODO update the JSON string below
json = "{}"
# create an instance of ChunksWithPositions from a JSON string
chunks_with_positions_instance = ChunksWithPositions.from_json(json)
# print the JSON string representation of the object
print(ChunksWithPositions.to_json())

# convert the object into a dict
chunks_with_positions_dict = chunks_with_positions_instance.to_dict()
# create an instance of ChunksWithPositions from a dict
chunks_with_positions_form_dict = chunks_with_positions.from_dict(chunks_with_positions_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


