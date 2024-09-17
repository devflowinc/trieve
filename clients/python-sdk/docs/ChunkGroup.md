# ChunkGroup


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_at** | **datetime** |  | 
**dataset_id** | **str** |  | 
**description** | **str** |  | 
**id** | **str** |  | 
**metadata** | **object** |  | [optional] 
**name** | **str** |  | 
**tag_set** | **List[Optional[str]]** |  | [optional] 
**tracking_id** | **str** |  | [optional] 
**updated_at** | **datetime** |  | 

## Example

```python
from trieve_py_client.models.chunk_group import ChunkGroup

# TODO update the JSON string below
json = "{}"
# create an instance of ChunkGroup from a JSON string
chunk_group_instance = ChunkGroup.from_json(json)
# print the JSON string representation of the object
print(ChunkGroup.to_json())

# convert the object into a dict
chunk_group_dict = chunk_group_instance.to_dict()
# create an instance of ChunkGroup from a dict
chunk_group_form_dict = chunk_group.from_dict(chunk_group_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


