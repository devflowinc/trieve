# GetAllTagsResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**tags** | [**List[TagsWithCount]**](TagsWithCount.md) | List of tags with the number of chunks in the dataset with that tag. | 
**total** | **int** | Total number of unique tags in the dataset. | 

## Example

```python
from trieve_py_client.models.get_all_tags_response import GetAllTagsResponse

# TODO update the JSON string below
json = "{}"
# create an instance of GetAllTagsResponse from a JSON string
get_all_tags_response_instance = GetAllTagsResponse.from_json(json)
# print the JSON string representation of the object
print(GetAllTagsResponse.to_json())

# convert the object into a dict
get_all_tags_response_dict = get_all_tags_response_instance.to_dict()
# create an instance of GetAllTagsResponse from a dict
get_all_tags_response_form_dict = get_all_tags_response.from_dict(get_all_tags_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


