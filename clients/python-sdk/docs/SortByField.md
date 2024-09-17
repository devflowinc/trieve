# SortByField


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**direction** | [**SortOrder**](SortOrder.md) |  | [optional] 
**field** | **str** | Field to sort by. This has to be a numeric field with a Qdrant &#x60;Range&#x60; index on it. i.e. num_value and timestamp | 
**prefetch_amount** | **int** | How many results to pull in before the sort | [optional] 

## Example

```python
from trieve_py_client.models.sort_by_field import SortByField

# TODO update the JSON string below
json = "{}"
# create an instance of SortByField from a JSON string
sort_by_field_instance = SortByField.from_json(json)
# print the JSON string representation of the object
print(SortByField.to_json())

# convert the object into a dict
sort_by_field_dict = sort_by_field_instance.to_dict()
# create an instance of SortByField from a dict
sort_by_field_form_dict = sort_by_field.from_dict(sort_by_field_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


