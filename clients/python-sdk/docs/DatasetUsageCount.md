# DatasetUsageCount


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**chunk_count** | **int** |  | 
**dataset_id** | **str** |  | 
**id** | **str** |  | 

## Example

```python
from trieve_py_client.models.dataset_usage_count import DatasetUsageCount

# TODO update the JSON string below
json = "{}"
# create an instance of DatasetUsageCount from a JSON string
dataset_usage_count_instance = DatasetUsageCount.from_json(json)
# print the JSON string representation of the object
print(DatasetUsageCount.to_json())

# convert the object into a dict
dataset_usage_count_dict = dataset_usage_count_instance.to_dict()
# create an instance of DatasetUsageCount from a dict
dataset_usage_count_form_dict = dataset_usage_count.from_dict(dataset_usage_count_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


