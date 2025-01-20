# ApiKeyRespBody


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_at** | **datetime** |  | 
**dataset_ids** | **List[str]** |  | [optional] 
**id** | **str** |  | 
**name** | **str** |  | 
**organization_id** | **str** |  | 
**organization_ids** | **List[str]** |  | [optional] 
**role** | **int** |  | 
**updated_at** | **datetime** |  | 

## Example

```python
from trieve_py_client.models.api_key_resp_body import ApiKeyRespBody

# TODO update the JSON string below
json = "{}"
# create an instance of ApiKeyRespBody from a JSON string
api_key_resp_body_instance = ApiKeyRespBody.from_json(json)
# print the JSON string representation of the object
print(ApiKeyRespBody.to_json())

# convert the object into a dict
api_key_resp_body_dict = api_key_resp_body_instance.to_dict()
# create an instance of ApiKeyRespBody from a dict
api_key_resp_body_form_dict = api_key_resp_body.from_dict(api_key_resp_body_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


