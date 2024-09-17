# GetDatasetsPagination


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**limit** | **int** |  | [optional] 
**offset** | **int** |  | [optional] 

## Example

```python
from trieve_py_client.models.get_datasets_pagination import GetDatasetsPagination

# TODO update the JSON string below
json = "{}"
# create an instance of GetDatasetsPagination from a JSON string
get_datasets_pagination_instance = GetDatasetsPagination.from_json(json)
# print the JSON string representation of the object
print(GetDatasetsPagination.to_json())

# convert the object into a dict
get_datasets_pagination_dict = get_datasets_pagination_instance.to_dict()
# create an instance of GetDatasetsPagination from a dict
get_datasets_pagination_form_dict = get_datasets_pagination.from_dict(get_datasets_pagination_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


