# PopularFilters


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**clause** | **str** |  | 
**common_values** | **Dict[str, int]** |  | 
**count** | **int** |  | 
**field** | **str** |  | 
**filter_type** | **str** |  | 

## Example

```python
from trieve_py_client.models.popular_filters import PopularFilters

# TODO update the JSON string below
json = "{}"
# create an instance of PopularFilters from a JSON string
popular_filters_instance = PopularFilters.from_json(json)
# print the JSON string representation of the object
print(PopularFilters.to_json())

# convert the object into a dict
popular_filters_dict = popular_filters_instance.to_dict()
# create an instance of PopularFilters from a dict
popular_filters_form_dict = popular_filters.from_dict(popular_filters_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


