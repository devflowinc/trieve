# CreatePresignedUrlForCsvJsonResponseBody


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**file_metadata** | [**File**](File.md) |  | 
**presigned_put_url** | **str** | Signed URL to upload the file to. | 

## Example

```python
from trieve_py_client.models.create_presigned_url_for_csv_json_response_body import CreatePresignedUrlForCsvJsonResponseBody

# TODO update the JSON string below
json = "{}"
# create an instance of CreatePresignedUrlForCsvJsonResponseBody from a JSON string
create_presigned_url_for_csv_json_response_body_instance = CreatePresignedUrlForCsvJsonResponseBody.from_json(json)
# print the JSON string representation of the object
print(CreatePresignedUrlForCsvJsonResponseBody.to_json())

# convert the object into a dict
create_presigned_url_for_csv_json_response_body_dict = create_presigned_url_for_csv_json_response_body_instance.to_dict()
# create an instance of CreatePresignedUrlForCsvJsonResponseBody from a dict
create_presigned_url_for_csv_json_response_body_form_dict = create_presigned_url_for_csv_json_response_body.from_dict(create_presigned_url_for_csv_json_response_body_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


