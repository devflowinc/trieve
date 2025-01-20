# PublicPageTag


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**icon_class_name** | **str** |  | [optional] 
**label** | **str** |  | [optional] 
**selected** | **bool** |  | [optional] 
**tag** | **str** |  | 

## Example

```python
from trieve_py_client.models.public_page_tag import PublicPageTag

# TODO update the JSON string below
json = "{}"
# create an instance of PublicPageTag from a JSON string
public_page_tag_instance = PublicPageTag.from_json(json)
# print the JSON string representation of the object
print(PublicPageTag.to_json())

# convert the object into a dict
public_page_tag_dict = public_page_tag_instance.to_dict()
# create an instance of PublicPageTag from a dict
public_page_tag_form_dict = public_page_tag.from_dict(public_page_tag_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


