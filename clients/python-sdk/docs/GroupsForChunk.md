# GroupsForChunk


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**chunk_uuid** | **str** |  | 
**slim_groups** | [**List[ChunkGroupAndFileId]**](ChunkGroupAndFileId.md) |  | 

## Example

```python
from trieve_py_client.models.groups_for_chunk import GroupsForChunk

# TODO update the JSON string below
json = "{}"
# create an instance of GroupsForChunk from a JSON string
groups_for_chunk_instance = GroupsForChunk.from_json(json)
# print the JSON string representation of the object
print(GroupsForChunk.to_json())

# convert the object into a dict
groups_for_chunk_dict = groups_for_chunk_instance.to_dict()
# create an instance of GroupsForChunk from a dict
groups_for_chunk_form_dict = groups_for_chunk.from_dict(groups_for_chunk_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


