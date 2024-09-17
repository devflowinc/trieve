# SearchTypeCount


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**search_count** | **int** |  | 
**search_method** | **str** |  | 
**search_type** | **str** |  | 

## Example

```python
from trieve_py_client.models.search_type_count import SearchTypeCount

# TODO update the JSON string below
json = "{}"
# create an instance of SearchTypeCount from a JSON string
search_type_count_instance = SearchTypeCount.from_json(json)
# print the JSON string representation of the object
print(SearchTypeCount.to_json())

# convert the object into a dict
search_type_count_dict = search_type_count_instance.to_dict()
# create an instance of SearchTypeCount from a dict
search_type_count_form_dict = search_type_count.from_dict(search_type_count_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


