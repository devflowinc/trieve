# Dataset


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_at** | **datetime** |  | 
**deleted** | **int** |  | 
**id** | **str** |  | 
**name** | **str** |  | 
**organization_id** | **str** |  | 
**server_configuration** | **object** |  | 
**tracking_id** | **str** |  | [optional] 
**updated_at** | **datetime** |  | 

## Example

```python
from trieve_py_client.models.dataset import Dataset

# TODO update the JSON string below
json = "{}"
# create an instance of Dataset from a JSON string
dataset_instance = Dataset.from_json(json)
# print the JSON string representation of the object
print(Dataset.to_json())

# convert the object into a dict
dataset_dict = dataset_instance.to_dict()
# create an instance of Dataset from a dict
dataset_form_dict = dataset.from_dict(dataset_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


