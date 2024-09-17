# DeleteUserApiKeyRequest


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**api_key_id** | **str** | The id of the api key to delete. | 

## Example

```python
from trieve_py_client.models.delete_user_api_key_request import DeleteUserApiKeyRequest

# TODO update the JSON string below
json = "{}"
# create an instance of DeleteUserApiKeyRequest from a JSON string
delete_user_api_key_request_instance = DeleteUserApiKeyRequest.from_json(json)
# print the JSON string representation of the object
print(DeleteUserApiKeyRequest.to_json())

# convert the object into a dict
delete_user_api_key_request_dict = delete_user_api_key_request_instance.to_dict()
# create an instance of DeleteUserApiKeyRequest from a dict
delete_user_api_key_request_form_dict = delete_user_api_key_request.from_dict(delete_user_api_key_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


