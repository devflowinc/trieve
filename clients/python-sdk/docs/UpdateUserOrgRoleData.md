# UpdateUserOrgRoleData


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**organization_id** | **str** | The id of the organization to update the user for. | 
**role** | **int** | Either 0 (user), 1 (admin), or 2 (owner). If not provided, the current role will be used. The auth&#39;ed user must have a role greater than or equal to the role being assigned. | 
**user_id** | **str** | The id of the user to update, if not provided, the auth&#39;ed user will be updated. If provided, the role of the auth&#39;ed user or api key must be an admin (1) or owner (2) of the organization. | [optional] 

## Example

```python
from trieve_py_client.models.update_user_org_role_data import UpdateUserOrgRoleData

# TODO update the JSON string below
json = "{}"
# create an instance of UpdateUserOrgRoleData from a JSON string
update_user_org_role_data_instance = UpdateUserOrgRoleData.from_json(json)
# print the JSON string representation of the object
print(UpdateUserOrgRoleData.to_json())

# convert the object into a dict
update_user_org_role_data_dict = update_user_org_role_data_instance.to_dict()
# create an instance of UpdateUserOrgRoleData from a dict
update_user_org_role_data_form_dict = update_user_org_role_data.from_dict(update_user_org_role_data_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


