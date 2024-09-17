# CreateOrganizationReqPayload


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | **str** | The arbitrary name which will be used to identify the organization. This name must be unique. | 

## Example

```python
from trieve_py_client.models.create_organization_req_payload import CreateOrganizationReqPayload

# TODO update the JSON string below
json = "{}"
# create an instance of CreateOrganizationReqPayload from a JSON string
create_organization_req_payload_instance = CreateOrganizationReqPayload.from_json(json)
# print the JSON string representation of the object
print(CreateOrganizationReqPayload.to_json())

# convert the object into a dict
create_organization_req_payload_dict = create_organization_req_payload_instance.to_dict()
# create an instance of CreateOrganizationReqPayload from a dict
create_organization_req_payload_form_dict = create_organization_req_payload.from_dict(create_organization_req_payload_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


