# CreateBatchDataset


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**dataset_name** | **str** | Name of the dataset. | 
**server_configuration** | [**DatasetConfigurationDTO**](DatasetConfigurationDTO.md) |  | [optional] 
**tracking_id** | **str** | Optional tracking ID for the dataset. Can be used to track the dataset in external systems. Must be unique within the organization. Strongly recommended to not use a valid uuid value as that will not work with the TR-Dataset header. | [optional] 

## Example

```python
from trieve_py_client.models.create_batch_dataset import CreateBatchDataset

# TODO update the JSON string below
json = "{}"
# create an instance of CreateBatchDataset from a JSON string
create_batch_dataset_instance = CreateBatchDataset.from_json(json)
# print the JSON string representation of the object
print(CreateBatchDataset.to_json())

# convert the object into a dict
create_batch_dataset_dict = create_batch_dataset_instance.to_dict()
# create an instance of CreateBatchDataset from a dict
create_batch_dataset_form_dict = create_batch_dataset.from_dict(create_batch_dataset_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


