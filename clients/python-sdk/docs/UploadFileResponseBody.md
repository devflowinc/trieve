# UploadFileResponseBody


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**file_metadata** | [**File**](File.md) |  | 

## Example

```python
from trieve_py_client.models.upload_file_response_body import UploadFileResponseBody

# TODO update the JSON string below
json = "{}"
# create an instance of UploadFileResponseBody from a JSON string
upload_file_response_body_instance = UploadFileResponseBody.from_json(json)
# print the JSON string representation of the object
print(UploadFileResponseBody.to_json())

# convert the object into a dict
upload_file_response_body_dict = upload_file_response_body_instance.to_dict()
# create an instance of UploadFileResponseBody from a dict
upload_file_response_body_form_dict = upload_file_response_body.from_dict(upload_file_response_body_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


