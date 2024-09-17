# AuthQuery


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**inv_code** | **str** | Code sent via email as a result of successful call to send_invitation | [optional] 
**organization_id** | **str** | ID of organization to authenticate into | [optional] 
**redirect_uri** | **str** | URL to redirect to after successful login | [optional] 

## Example

```python
from trieve_py_client.models.auth_query import AuthQuery

# TODO update the JSON string below
json = "{}"
# create an instance of AuthQuery from a JSON string
auth_query_instance = AuthQuery.from_json(json)
# print the JSON string representation of the object
print(AuthQuery.to_json())

# convert the object into a dict
auth_query_dict = auth_query_instance.to_dict()
# create an instance of AuthQuery from a dict
auth_query_form_dict = auth_query.from_dict(auth_query_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


