# FileDTO


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_at** | **datetime** |  | 
**file_name** | **str** |  | 
**id** | **str** |  | 
**link** | **str** |  | [optional] 
**metadata** | **object** |  | [optional] 
**s3_url** | **str** |  | 
**size** | **int** |  | 
**updated_at** | **datetime** |  | 

## Example

```python
from trieve_py_client.models.file_dto import FileDTO

# TODO update the JSON string below
json = "{}"
# create an instance of FileDTO from a JSON string
file_dto_instance = FileDTO.from_json(json)
# print the JSON string representation of the object
print(FileDTO.to_json())

# convert the object into a dict
file_dto_dict = file_dto_instance.to_dict()
# create an instance of FileDTO from a dict
file_dto_form_dict = file_dto.from_dict(file_dto_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


