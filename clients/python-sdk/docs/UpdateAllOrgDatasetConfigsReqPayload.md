# UpdateAllOrgDatasetConfigsReqPayload


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**dataset_config** | **object** | The new configuration for all datasets in the organization. Only the specified keys in the configuration object will be changed per dataset such that you can preserve dataset unique values. | 
**organization_id** | **str** | The id of the organization to update the dataset configurations for. | 

## Example

```python
from trieve_py_client.models.update_all_org_dataset_configs_req_payload import UpdateAllOrgDatasetConfigsReqPayload

# TODO update the JSON string below
json = "{}"
# create an instance of UpdateAllOrgDatasetConfigsReqPayload from a JSON string
update_all_org_dataset_configs_req_payload_instance = UpdateAllOrgDatasetConfigsReqPayload.from_json(json)
# print the JSON string representation of the object
print(UpdateAllOrgDatasetConfigsReqPayload.to_json())

# convert the object into a dict
update_all_org_dataset_configs_req_payload_dict = update_all_org_dataset_configs_req_payload_instance.to_dict()
# create an instance of UpdateAllOrgDatasetConfigsReqPayload from a dict
update_all_org_dataset_configs_req_payload_form_dict = update_all_org_dataset_configs_req_payload.from_dict(update_all_org_dataset_configs_req_payload_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


