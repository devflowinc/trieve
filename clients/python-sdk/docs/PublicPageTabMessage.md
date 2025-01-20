# PublicPageTabMessage


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**show_component_code** | **bool** |  | 
**tab_inner_html** | **str** |  | 
**title** | **str** |  | 

## Example

```python
from trieve_py_client.models.public_page_tab_message import PublicPageTabMessage

# TODO update the JSON string below
json = "{}"
# create an instance of PublicPageTabMessage from a JSON string
public_page_tab_message_instance = PublicPageTabMessage.from_json(json)
# print the JSON string representation of the object
print(PublicPageTabMessage.to_json())

# convert the object into a dict
public_page_tab_message_dict = public_page_tab_message_instance.to_dict()
# create an instance of PublicPageTabMessage from a dict
public_page_tab_message_form_dict = public_page_tab_message.from_dict(public_page_tab_message_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


