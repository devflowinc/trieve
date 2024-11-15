# CreateDatasetReqPayload


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**crawl_options** | [**CrawlOptions**](CrawlOptions.md) |  | [optional] 
**dataset_name** | **str** | Name of the dataset. | 
**server_configuration** | [**DatasetConfigurationDTO**](DatasetConfigurationDTO.md) |  | [optional] 
**tracking_id** | **str** | Optional tracking ID for the dataset. Can be used to track the dataset in external systems. Must be unique within the organization. Strongly recommended to not use a valid uuid value as that will not work with the TR-Dataset header. | [optional] 

## Example

```python
from trieve_py_client.models.create_dataset_req_payload import CreateDatasetReqPayload

# TODO update the JSON string below
json = "{}"
# create an instance of CreateDatasetReqPayload from a JSON string
create_dataset_req_payload_instance = CreateDatasetReqPayload.from_json(json)
# print the JSON string representation of the object
print(CreateDatasetReqPayload.to_json())

# convert the object into a dict
create_dataset_req_payload_dict = create_dataset_req_payload_instance.to_dict()
# create an instance of CreateDatasetReqPayload from a dict
create_dataset_req_payload_form_dict = create_dataset_req_payload.from_dict(create_dataset_req_payload_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


