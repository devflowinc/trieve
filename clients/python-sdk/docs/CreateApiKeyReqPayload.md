# CreateApiKeyReqPayload


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**dataset_ids** | **List[str]** | The dataset ids which the api key will have access to. If not provided or empty, the api key will have access to all datasets in the dataset. | [optional] 
**default_params** | [**ApiKeyRequestParams**](ApiKeyRequestParams.md) |  | [optional] 
**expires_at** | **str** | The expiration date of the api key. If not provided, the api key will not expire. This should be provided in UTC time. | [optional] 
**name** | **str** | The name which will be assigned to the new api key. | 
**role** | **int** | The role which will be assigned to the new api key. Either 0 (read), 1 (Admin) or 2 (Owner). The auth&#39;ed user must have a role greater than or equal to the role being assigned. | 
**scopes** | **List[str]** | The routes which the api key will have access to. If not provided or empty, the api key will have access to all routes. Specify the routes as a list of strings. For example, [\&quot;GET /api/dataset\&quot;, \&quot;POST /api/dataset\&quot;]. | [optional] 

## Example

```python
from trieve_py_client.models.create_api_key_req_payload import CreateApiKeyReqPayload

# TODO update the JSON string below
json = "{}"
# create an instance of CreateApiKeyReqPayload from a JSON string
create_api_key_req_payload_instance = CreateApiKeyReqPayload.from_json(json)
# print the JSON string representation of the object
print(CreateApiKeyReqPayload.to_json())

# convert the object into a dict
create_api_key_req_payload_dict = create_api_key_req_payload_instance.to_dict()
# create an instance of CreateApiKeyReqPayload from a dict
create_api_key_req_payload_form_dict = create_api_key_req_payload.from_dict(create_api_key_req_payload_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


