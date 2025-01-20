# SearchModalities


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**image_url** | **str** |  | 
**llm_prompt** | **str** |  | [optional] 
**audio_base64** | **str** |  | 

## Example

```python
from trieve_py_client.models.search_modalities import SearchModalities

# TODO update the JSON string below
json = "{}"
# create an instance of SearchModalities from a JSON string
search_modalities_instance = SearchModalities.from_json(json)
# print the JSON string representation of the object
print(SearchModalities.to_json())

# convert the object into a dict
search_modalities_dict = search_modalities_instance.to_dict()
# create an instance of SearchModalities from a dict
search_modalities_form_dict = search_modalities.from_dict(search_modalities_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


