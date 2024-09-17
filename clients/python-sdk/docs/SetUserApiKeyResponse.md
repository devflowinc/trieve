# SetUserApiKeyResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**api_key** | **str** | The api key which was created. This is the value which should be used in the Authorization header. | 

## Example

```python
from trieve_py_client.models.set_user_api_key_response import SetUserApiKeyResponse

# TODO update the JSON string below
json = "{}"
# create an instance of SetUserApiKeyResponse from a JSON string
set_user_api_key_response_instance = SetUserApiKeyResponse.from_json(json)
# print the JSON string representation of the object
print(SetUserApiKeyResponse.to_json())

# convert the object into a dict
set_user_api_key_response_dict = set_user_api_key_response_instance.to_dict()
# create an instance of SetUserApiKeyResponse from a dict
set_user_api_key_response_form_dict = set_user_api_key_response.from_dict(set_user_api_key_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


