# PublicDatasetOptions


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**enabled** | **bool** |  | 
**extra_params** | [**PublicPageParameters**](PublicPageParameters.md) |  | [optional] 

## Example

```python
from trieve_py_client.models.public_dataset_options import PublicDatasetOptions

# TODO update the JSON string below
json = "{}"
# create an instance of PublicDatasetOptions from a JSON string
public_dataset_options_instance = PublicDatasetOptions.from_json(json)
# print the JSON string representation of the object
print(PublicDatasetOptions.to_json())

# convert the object into a dict
public_dataset_options_dict = public_dataset_options_instance.to_dict()
# create an instance of PublicDatasetOptions from a dict
public_dataset_options_form_dict = public_dataset_options.from_dict(public_dataset_options_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


