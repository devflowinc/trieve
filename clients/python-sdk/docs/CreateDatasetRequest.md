# CreateDatasetRequest


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**crawl_options** | [**CrawlOptions**](CrawlOptions.md) |  | [optional] 
**dataset_name** | **str** | Name of the dataset. | 
**organization_id** | **str** | Organization ID that the dataset will belong to. | 
**server_configuration** | [**DatasetConfigurationDTO**](DatasetConfigurationDTO.md) |  | [optional] 
**tracking_id** | **str** | Optional tracking ID for the dataset. Can be used to track the dataset in external systems. Must be unique within the organization. Strongly recommended to not use a valid uuid value as that will not work with the TR-Dataset header. | [optional] 

## Example

```python
from trieve_py_client.models.create_dataset_request import CreateDatasetRequest

# TODO update the JSON string below
json = "{}"
# create an instance of CreateDatasetRequest from a JSON string
create_dataset_request_instance = CreateDatasetRequest.from_json(json)
# print the JSON string representation of the object
print(CreateDatasetRequest.to_json())

# convert the object into a dict
create_dataset_request_dict = create_dataset_request_instance.to_dict()
# create an instance of CreateDatasetRequest from a dict
create_dataset_request_form_dict = create_dataset_request.from_dict(create_dataset_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


