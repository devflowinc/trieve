# InvitationData


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**app_url** | **str** | The url of the app that the user will be directed to in order to set their password. Usually admin.trieve.ai, but may differ for local dev or self-hosted setups. | 
**email** | **str** | The email of the user to invite. Must be a valid email as they will be sent an email to register. | 
**organization_id** | **str** | The id of the organization to invite the user to. | 
**redirect_uri** | **str** | The url that the user will be redirected to after setting their password. | 
**user_role** | **int** | The role the user will have in the organization. 0 &#x3D; User, 1 &#x3D; Admin, 2 &#x3D; Owner. | 

## Example

```python
from trieve_py_client.models.invitation_data import InvitationData

# TODO update the JSON string below
json = "{}"
# create an instance of InvitationData from a JSON string
invitation_data_instance = InvitationData.from_json(json)
# print the JSON string representation of the object
print(InvitationData.to_json())

# convert the object into a dict
invitation_data_dict = invitation_data_instance.to_dict()
# create an instance of InvitationData from a dict
invitation_data_form_dict = invitation_data.from_dict(invitation_data_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


