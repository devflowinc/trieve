# UserOrganization


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_at** | **datetime** |  | 
**id** | **str** |  | 
**organization_id** | **str** |  | 
**role** | **int** |  | 
**updated_at** | **datetime** |  | 
**user_id** | **str** |  | 

## Example

```python
from trieve_py_client.models.user_organization import UserOrganization

# TODO update the JSON string below
json = "{}"
# create an instance of UserOrganization from a JSON string
user_organization_instance = UserOrganization.from_json(json)
# print the JSON string representation of the object
print(UserOrganization.to_json())

# convert the object into a dict
user_organization_dict = user_organization_instance.to_dict()
# create an instance of UserOrganization from a dict
user_organization_form_dict = user_organization.from_dict(user_organization_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


