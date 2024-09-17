# CreateChunkGroupResponseEnum


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
from trieve_py_client.models.create_chunk_group_response_enum import CreateChunkGroupResponseEnum

# TODO update the JSON string below
json = "{}"
# create an instance of CreateChunkGroupResponseEnum from a JSON string
create_chunk_group_response_enum_instance = CreateChunkGroupResponseEnum.from_json(json)
# print the JSON string representation of the object
print(CreateChunkGroupResponseEnum.to_json())

# convert the object into a dict
create_chunk_group_response_enum_dict = create_chunk_group_response_enum_instance.to_dict()
# create an instance of CreateChunkGroupResponseEnum from a dict
create_chunk_group_response_enum_form_dict = create_chunk_group_response_enum.from_dict(create_chunk_group_response_enum_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


