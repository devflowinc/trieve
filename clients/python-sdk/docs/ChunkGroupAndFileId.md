# ChunkGroupAndFileId


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_at** | **datetime** |  | 
**dataset_id** | **str** |  | 
**description** | **str** |  | 
**file_id** | **str** |  | [optional] 
**id** | **str** |  | 
**metadata** | **object** |  | [optional] 
**name** | **str** |  | 
**tag_set** | **List[Optional[str]]** |  | [optional] 
**tracking_id** | **str** |  | [optional] 
**updated_at** | **datetime** |  | 

## Example

```python
from trieve_py_client.models.chunk_group_and_file_id import ChunkGroupAndFileId

# TODO update the JSON string below
json = "{}"
# create an instance of ChunkGroupAndFileId from a JSON string
chunk_group_and_file_id_instance = ChunkGroupAndFileId.from_json(json)
# print the JSON string representation of the object
print(ChunkGroupAndFileId.to_json())

# convert the object into a dict
chunk_group_and_file_id_dict = chunk_group_and_file_id_instance.to_dict()
# create an instance of ChunkGroupAndFileId from a dict
chunk_group_and_file_id_form_dict = chunk_group_and_file_id.from_dict(chunk_group_and_file_id_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


