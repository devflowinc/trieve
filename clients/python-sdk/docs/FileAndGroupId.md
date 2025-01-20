# FileAndGroupId


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**file** | [**File**](File.md) |  | 
**group_id** | **str** |  | [optional] 

## Example

```python
from trieve_py_client.models.file_and_group_id import FileAndGroupId

# TODO update the JSON string below
json = "{}"
# create an instance of FileAndGroupId from a JSON string
file_and_group_id_instance = FileAndGroupId.from_json(json)
# print the JSON string representation of the object
print(FileAndGroupId.to_json())

# convert the object into a dict
file_and_group_id_dict = file_and_group_id_instance.to_dict()
# create an instance of FileAndGroupId from a dict
file_and_group_id_form_dict = file_and_group_id.from_dict(file_and_group_id_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


