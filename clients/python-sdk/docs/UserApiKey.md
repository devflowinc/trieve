# UserApiKey


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**api_key_hash** | **str** |  | [optional] 
**blake3_hash** | **str** |  | [optional] 
**created_at** | **datetime** |  | 
**dataset_ids** | **List[Optional[str]]** |  | [optional] 
**expires_at** | **datetime** |  | [optional] 
**id** | **str** |  | 
**name** | **str** |  | 
**organization_ids** | **List[Optional[str]]** |  | [optional] 
**params** | **object** |  | [optional] 
**role** | **int** |  | 
**scopes** | **List[Optional[str]]** |  | [optional] 
**updated_at** | **datetime** |  | 
**user_id** | **str** |  | 

## Example

```python
from trieve_py_client.models.user_api_key import UserApiKey

# TODO update the JSON string below
json = "{}"
# create an instance of UserApiKey from a JSON string
user_api_key_instance = UserApiKey.from_json(json)
# print the JSON string representation of the object
print(UserApiKey.to_json())

# convert the object into a dict
user_api_key_dict = user_api_key_instance.to_dict()
# create an instance of UserApiKey from a dict
user_api_key_form_dict = user_api_key.from_dict(user_api_key_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


