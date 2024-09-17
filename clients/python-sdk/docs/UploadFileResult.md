# UploadFileResult


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**file_metadata** | [**File**](File.md) |  | 

## Example

```python
from trieve_py_client.models.upload_file_result import UploadFileResult

# TODO update the JSON string below
json = "{}"
# create an instance of UploadFileResult from a JSON string
upload_file_result_instance = UploadFileResult.from_json(json)
# print the JSON string representation of the object
print(UploadFileResult.to_json())

# convert the object into a dict
upload_file_result_dict = upload_file_result_instance.to_dict()
# create an instance of UploadFileResult from a dict
upload_file_result_form_dict = upload_file_result.from_dict(upload_file_result_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


