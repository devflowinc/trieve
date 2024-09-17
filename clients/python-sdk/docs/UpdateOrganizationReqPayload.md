# UpdateOrganizationReqPayload


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | **str** | The new name of the organization. If not provided, the name will not be updated. | [optional] 
**organization_id** | **str** | The id of the organization to update. | 

## Example

```python
from trieve_py_client.models.update_organization_req_payload import UpdateOrganizationReqPayload

# TODO update the JSON string below
json = "{}"
# create an instance of UpdateOrganizationReqPayload from a JSON string
update_organization_req_payload_instance = UpdateOrganizationReqPayload.from_json(json)
# print the JSON string representation of the object
print(UpdateOrganizationReqPayload.to_json())

# convert the object into a dict
update_organization_req_payload_dict = update_organization_req_payload_instance.to_dict()
# create an instance of UpdateOrganizationReqPayload from a dict
update_organization_req_payload_form_dict = update_organization_req_payload.from_dict(update_organization_req_payload_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


