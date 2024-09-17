# GroupData


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**groups** | [**List[ChunkGroupAndFileId]**](ChunkGroupAndFileId.md) |  | 
**total_pages** | **int** |  | 

## Example

```python
from trieve_py_client.models.group_data import GroupData

# TODO update the JSON string below
json = "{}"
# create an instance of GroupData from a JSON string
group_data_instance = GroupData.from_json(json)
# print the JSON string representation of the object
print(GroupData.to_json())

# convert the object into a dict
group_data_dict = group_data_instance.to_dict()
# create an instance of GroupData from a dict
group_data_form_dict = group_data.from_dict(group_data_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


