# SearchModalitiesOneOf


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**image_url** | **str** |  | 
**llm_prompt** | **str** |  | [optional] 

## Example

```python
from trieve_py_client.models.search_modalities_one_of import SearchModalitiesOneOf

# TODO update the JSON string below
json = "{}"
# create an instance of SearchModalitiesOneOf from a JSON string
search_modalities_one_of_instance = SearchModalitiesOneOf.from_json(json)
# print the JSON string representation of the object
print(SearchModalitiesOneOf.to_json())

# convert the object into a dict
search_modalities_one_of_dict = search_modalities_one_of_instance.to_dict()
# create an instance of SearchModalitiesOneOf from a dict
search_modalities_one_of_form_dict = search_modalities_one_of.from_dict(search_modalities_one_of_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


