# CreateApiKeyResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**api_key** | **str** | The api key which was created. This is the value which should be used in the Authorization header. | 

## Example

```python
from trieve_py_client.models.create_api_key_response import CreateApiKeyResponse

# TODO update the JSON string below
json = "{}"
# create an instance of CreateApiKeyResponse from a JSON string
create_api_key_response_instance = CreateApiKeyResponse.from_json(json)
# print the JSON string representation of the object
print(CreateApiKeyResponse.to_json())

# convert the object into a dict
create_api_key_response_dict = create_api_key_response_instance.to_dict()
# create an instance of CreateApiKeyResponse from a dict
create_api_key_response_form_dict = create_api_key_response.from_dict(create_api_key_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


