# File


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_at** | **datetime** |  | 
**dataset_id** | **str** |  | 
**file_name** | **str** |  | 
**id** | **str** |  | 
**link** | **str** |  | [optional] 
**metadata** | **object** |  | [optional] 
**size** | **int** |  | 
**tag_set** | **List[Optional[str]]** |  | [optional] 
**time_stamp** | **datetime** |  | [optional] 
**updated_at** | **datetime** |  | 

## Example

```python
from trieve_py_client.models.file import File

# TODO update the JSON string below
json = "{}"
# create an instance of File from a JSON string
file_instance = File.from_json(json)
# print the JSON string representation of the object
print(File.to_json())

# convert the object into a dict
file_dict = file_instance.to_dict()
# create an instance of File from a dict
file_form_dict = file.from_dict(file_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


