# UpdateDatasetRequest


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**crawl_options** | [**CrawlOptions**](CrawlOptions.md) |  | [optional] 
**dataset_id** | **str** | The id of the dataset you want to update. | [optional] 
**dataset_name** | **str** | The new name of the dataset. Must be unique within the organization. If not provided, the name will not be updated. | [optional] 
**new_tracking_id** | **str** | Optional new tracking ID for the dataset. Can be used to track the dataset in external systems. Must be unique within the organization. If not provided, the tracking ID will not be updated. Strongly recommended to not use a valid uuid value as that will not work with the TR-Dataset header. | [optional] 
**server_configuration** | [**DatasetConfigurationDTO**](DatasetConfigurationDTO.md) |  | [optional] 
**tracking_id** | **str** | The tracking ID of the dataset you want to update. | [optional] 

## Example

```python
from trieve_py_client.models.update_dataset_request import UpdateDatasetRequest

# TODO update the JSON string below
json = "{}"
# create an instance of UpdateDatasetRequest from a JSON string
update_dataset_request_instance = UpdateDatasetRequest.from_json(json)
# print the JSON string representation of the object
print(UpdateDatasetRequest.to_json())

# convert the object into a dict
update_dataset_request_dict = update_dataset_request_instance.to_dict()
# create an instance of UpdateDatasetRequest from a dict
update_dataset_request_form_dict = update_dataset_request.from_dict(update_dataset_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


