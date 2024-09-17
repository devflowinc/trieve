# DatasetAndUsage


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**dataset** | [**DatasetDTO**](DatasetDTO.md) |  | 
**dataset_usage** | [**DatasetUsageCount**](DatasetUsageCount.md) |  | 

## Example

```python
from trieve_py_client.models.dataset_and_usage import DatasetAndUsage

# TODO update the JSON string below
json = "{}"
# create an instance of DatasetAndUsage from a JSON string
dataset_and_usage_instance = DatasetAndUsage.from_json(json)
# print the JSON string representation of the object
print(DatasetAndUsage.to_json())

# convert the object into a dict
dataset_and_usage_dict = dataset_and_usage_instance.to_dict()
# create an instance of DatasetAndUsage from a dict
dataset_and_usage_form_dict = dataset_and_usage.from_dict(dataset_and_usage_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


