# TagsWithCount


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**count** | **int** |  | 
**tag** | **str** |  | 

## Example

```python
from trieve_py_client.models.tags_with_count import TagsWithCount

# TODO update the JSON string below
json = "{}"
# create an instance of TagsWithCount from a JSON string
tags_with_count_instance = TagsWithCount.from_json(json)
# print the JSON string representation of the object
print(TagsWithCount.to_json())

# convert the object into a dict
tags_with_count_dict = tags_with_count_instance.to_dict()
# create an instance of TagsWithCount from a dict
tags_with_count_form_dict = tags_with_count.from_dict(tags_with_count_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


