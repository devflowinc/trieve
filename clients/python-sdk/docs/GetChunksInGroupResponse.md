# GetChunksInGroupResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**chunks** | [**List[ChunkMetadataStringTagSet]**](ChunkMetadataStringTagSet.md) |  | 
**group** | [**ChunkGroupAndFileId**](ChunkGroupAndFileId.md) |  | 
**total_pages** | **int** |  | 

## Example

```python
from trieve_py_client.models.get_chunks_in_group_response import GetChunksInGroupResponse

# TODO update the JSON string below
json = "{}"
# create an instance of GetChunksInGroupResponse from a JSON string
get_chunks_in_group_response_instance = GetChunksInGroupResponse.from_json(json)
# print the JSON string representation of the object
print(GetChunksInGroupResponse.to_json())

# convert the object into a dict
get_chunks_in_group_response_dict = get_chunks_in_group_response_instance.to_dict()
# create an instance of GetChunksInGroupResponse from a dict
get_chunks_in_group_response_form_dict = get_chunks_in_group_response.from_dict(get_chunks_in_group_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


