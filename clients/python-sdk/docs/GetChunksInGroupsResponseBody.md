# GetChunksInGroupsResponseBody


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**chunks** | [**List[ChunkMetadata]**](ChunkMetadata.md) |  | 
**group** | [**ChunkGroupAndFileId**](ChunkGroupAndFileId.md) |  | 
**total_pages** | **int** |  | 

## Example

```python
from trieve_py_client.models.get_chunks_in_groups_response_body import GetChunksInGroupsResponseBody

# TODO update the JSON string below
json = "{}"
# create an instance of GetChunksInGroupsResponseBody from a JSON string
get_chunks_in_groups_response_body_instance = GetChunksInGroupsResponseBody.from_json(json)
# print the JSON string representation of the object
print(GetChunksInGroupsResponseBody.to_json())

# convert the object into a dict
get_chunks_in_groups_response_body_dict = get_chunks_in_groups_response_body_instance.to_dict()
# create an instance of GetChunksInGroupsResponseBody from a dict
get_chunks_in_groups_response_body_form_dict = get_chunks_in_groups_response_body.from_dict(get_chunks_in_groups_response_body_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


